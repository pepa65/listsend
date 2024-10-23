use clap::Parser;
use dotenv::from_path;
use lettre::message::header::{ContentTransferEncoding, ContentType};
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::Serialize;
use std::{error::Error, fs, thread::sleep, time};

const SMTP_ENV: &str = "./smtp.env";
const EMAIL_ENV: &str = "./email.env";
const LIST_CSV: &str = "./list.csv";
const EMAIL_TPL: &str = "./email.tpl";
const SMTP_PORT: &str = "465";

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(help_template(
	"\
{before-help}{name} {version} - {about}
{usage-heading} {usage}
{all-args}{after-help}
"
))]
struct Cli {
	/// CSV file (name,email,data)
	#[arg(short, long, default_value = LIST_CSV)]
	csv: std::path::PathBuf,

	/// Email template file
	#[arg(short, long, default_value = EMAIL_TPL)]
	template: std::path::PathBuf,

	/// SMTP config file
	#[arg(short, long, default_value = SMTP_ENV)]
	smtp: std::path::PathBuf,

	/// Email config file
	#[arg(short, long, default_value = EMAIL_ENV)]
	email: std::path::PathBuf,

	/// Output readme file from repo
	#[arg(short, long)]
	readme: bool,
}

#[derive(Serialize)]
struct Csv {
	name: String,
	email: String,
	data: String,
}

#[rustfmt::skip]
#[derive(Debug)]
struct EnvConfig {
	smtp_host: String,
	smtp_port: u16, // Default 465
	smtp_user: String,
	smtp_pass: String,
	smtp_from: String,
	reply_to: String,
	subject: String,
	html: String, // Default unset
	attachment: String, // Default unset
	delay: u64, // Default 1
}

impl EnvConfig {
	pub fn check_or_default() -> Self {
		Self {
			smtp_host: std::env::var("SENDLIST_HOST").expect("SENDLIST_HOST must be set"),
			smtp_port: std::env::var("SENDLIST_PORT").unwrap_or(SMTP_PORT.to_string()).parse::<u16>().unwrap(),
			smtp_user: std::env::var("SENDLIST_USER").expect("SENDLIST_USER must be set"),
			smtp_pass: std::env::var("SENDLIST_PASSWORD").expect("SENDLIST_PASSWORD must be set"),
			smtp_from: std::env::var("SENDLIST_FROM").expect("SENDLIST_FROM must be set"),
			reply_to: std::env::var("SENDLIST_REPLY_TO").unwrap_or("".to_string()),
			subject: std::env::var("SENDLIST_SUBJECT").expect("SENDLIST_SUBJECT must be set"),
			html: std::env::var("SENDLIST_HTML").unwrap_or("".to_string()),
			attachment: std::env::var("SENDLIST_ATTACHMENT").unwrap_or("".to_string()),
			delay: std::env::var("SENDLIST_DELAY").unwrap_or("1".to_string()).parse::<u64>().unwrap(),
		}
	}
}

fn read_csv_file(content: String) -> Result<Vec<Csv>, Box<dyn Error>> {
	let mut csv = Vec::new();
	let mut reader = csv::ReaderBuilder::new().flexible(true).comment(Some(b'#')).from_reader(content.as_bytes());
	for result in reader.records() {
		let record = result?;
		csv.push(Csv { name: record.get(0).unwrap().into(), email: record.get(1).unwrap().into(), data: record.get(2).unwrap().into() })
	}
	Ok(csv)
}

fn create_mailer(env: &EnvConfig) -> SmtpTransport {
	let creds = Credentials::new(env.smtp_user.clone(), env.smtp_pass.clone());
	let mailer = SmtpTransport::relay(&env.smtp_host).unwrap().port(env.smtp_port).credentials(creds).build();
	mailer
}

fn main() {
	let args = Cli::parse();
	if args.readme {
		print!("{}", include_str!("../README.md"));
		return;
	}
	from_path(&args.smtp).ok();
	from_path(&args.email).ok();
	let env = EnvConfig::check_or_default();
	let html = env.html != "" && env.html != "no" && env.html != "unset" && env.html != "0" && env.html != "false";
	let template = fs::read_to_string(&args.template).expect("failed to read template file");
	let csv_content = fs::read_to_string(&args.csv).expect("failed to read csv file");
	let csv = read_csv_file(csv_content.clone()).expect("Failed to parse csv file");
	let mut hbs = handlebars::Handlebars::new();
	hbs.register_template_string("body", template.clone()).expect("Failed to read template file content");
	hbs.register_template_string("subject", env.subject.clone()).expect("Subject must be set");
	let mailer = create_mailer(&env);
	let mut n = 0;
	let mut err = 0;
	if env.attachment.to_owned().len() > 0 {
		let binary = ContentType::parse("application/octet-stream").unwrap();
		let attachment = fs::read(env.attachment.clone()).expect("failed to read attachment");
		let attname = env.attachment.split("/").last().unwrap().to_string();
		let att = SinglePart::builder()
			.header(ContentTransferEncoding::Base64)
			.header(binary.clone())
			.header(lettre::message::header::ContentDisposition::attachment(&attname))
			.body(attachment.clone());
		for line in &csv {
			let to = format!("\"{}\" <{}>", &line.name, &line.email);
			print!("--- Sending to: {to} ");
			let mut email = Message::builder()
				.subject(hbs.render("subject", &line).unwrap())
				.from(env.smtp_from.parse().unwrap())
				.to(to.parse().unwrap());
			if env.reply_to.len() > 0 {
				email = email.reply_to(env.reply_to.parse().unwrap());
			};
			match mailer.send(&email
				.multipart(
					MultiPart::mixed()
						.singlepart(
							SinglePart::builder()
								.header(if html { ContentType::TEXT_HTML } else { ContentType::TEXT_PLAIN })
								.body(hbs.render("body", &line).unwrap()),
						)
						.singlepart(att.clone()),
				)
				.unwrap()
			) {
				Ok(_) => println!(""),
				Err(e) => {
					println!("### Failed: {:?}", e);
					err += 1;
				}
			};
			n += 1;
			sleep(time::Duration::from_secs(env.delay));
		}
	} else {
		for line in &csv {
			let to = format!("\"{}\" <{}>", &line.name, &line.email);
			print!("--- Sending to: {to} ");
			let mut email = Message::builder()
				.subject(hbs.render("subject", &line).unwrap())
				.from(env.smtp_from.parse().unwrap())
				.to(to.parse().unwrap());
			if env.reply_to.len() > 0 {
				email = email.reply_to(env.reply_to.parse().unwrap());
			};
			match mailer.send(&email
				.header(if html {ContentType::TEXT_HTML} else {ContentType::TEXT_PLAIN})
				.body(hbs.render("body", &line).unwrap())
				.unwrap()
			) {
				Ok(_) => println!(""),
				Err(e) => {
					println!("### Failed: {:?}", e);
					err += 1;
				}
			};
			n += 1;
			sleep(time::Duration::from_secs(env.delay));
		}
	};
	if err > 0 {
		println!("### Failed to send: {err}");
	};
	println!("=== Processed {n} mails");
}

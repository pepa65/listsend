use clap::Parser;
use dotenv::from_path;
use lettre::message::header::{ContentTransferEncoding, ContentType};
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::Serialize;
use std::{error::Error, fs, path::PathBuf, thread::sleep, time};
use tap::Pipe;

const SMTP_ENV: &str = "./smtp.env";
const EMAIL_ENV: &str = "./email.env";
const EMAIL_TPL: &str = "./email.tpl";
const LIST_CSV: &str = "./list.csv";
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

#[derive(Debug)]
struct EnvConfig {
	smtp_host: String,  // Mandatory
	smtp_port: u16,     // Default 465
	smtp_user: String,  // Mandatory
	smtp_pass: String,  // Mandatory
	smtp_from: String,  // Mandatory
	reply_to: String,   // Default unset
	cc: String,         // Default unset
	bcc: String,        // Default unset
	subject: String,    // Default unset
	html: String,       // Default unset
	attachment: String, // Default unset
	delay: u64,         // Default 1
}

fn get_env(var: &str, val: &str) -> String {
	if std::env::var(var).unwrap_or_default().is_empty() {
		val.to_string()
	} else {
		std::env::var(var).unwrap()
	}
}

impl EnvConfig {
	pub fn check_or_default(smtp: PathBuf, email: PathBuf) -> Self {
		from_path(smtp).ok();
		from_path(email).ok();
		Self {
			smtp_host: std::env::var("SENDLIST_HOST").expect("SENDLIST_HOST must be set"),
			smtp_port: get_env("SENDLIST_PORT", SMTP_PORT).parse::<u16>().unwrap(),
			smtp_user: std::env::var("SENDLIST_USER").expect("SENDLIST_USER must be set"),
			smtp_pass: std::env::var("SENDLIST_PASSWORD").expect("SENDLIST_PASSWORD must be set"),
			smtp_from: std::env::var("SENDLIST_FROM").expect("SENDLIST_FROM must be set"),
			reply_to: get_env("SENDLIST_REPLY_TO", ""),
			cc: get_env("SENDLIST_CC", ""),
			bcc: get_env("SENDLIST_BCC", ""),
			subject: get_env("SENDLIST_SUBJECT", ""),
			html: get_env("SENDLIST_HTML", ""),
			attachment: get_env("SENDLIST_ATTACHMENT", ""),
			delay: get_env("SENDLIST_DELAY", "1").parse::<u64>().unwrap(),
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
	let env = EnvConfig::check_or_default(args.smtp, args.email);
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
	if !env.attachment.to_owned().is_empty() {
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
			let email = Message::builder()
				.from(env.smtp_from.parse().unwrap())
				.to(to.parse().unwrap())
				.pipe(|o| if env.reply_to.is_empty() { o } else { o.reply_to(env.reply_to.parse().unwrap()) })
				.pipe(|o| if env.cc.is_empty() { o } else { o.cc(env.cc.parse().unwrap()) })
				.pipe(|o| if env.bcc.is_empty() { o } else { o.bcc(env.bcc.parse().unwrap()) })
				.subject(hbs.render("subject", &line).unwrap())
				.multipart(
					MultiPart::mixed()
						.singlepart(
							SinglePart::builder()
								.header(if html { ContentType::TEXT_HTML } else { ContentType::TEXT_PLAIN })
								.body(hbs.render("body", &line).unwrap()),
						)
						.singlepart(att.clone()),
				)
				.unwrap();
			match mailer.send(&email) {
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
			let email = Message::builder()
				.from(env.smtp_from.parse().unwrap())
				.to(to.parse().unwrap())
				.pipe(|o| if env.reply_to.is_empty() { o } else { o.reply_to(env.reply_to.parse().unwrap()) })
				.pipe(|o| if env.cc.is_empty() { o } else { o.cc(env.cc.parse().unwrap()) })
				.pipe(|o| if env.bcc.is_empty() { o } else { o.bcc(env.bcc.parse().unwrap()) })
				.header(if html { ContentType::TEXT_HTML } else { ContentType::TEXT_PLAIN })
				.subject(hbs.render("subject", &line).unwrap())
				.body(hbs.render("body", &line).unwrap())
				.unwrap();
			match mailer.send(&email) {
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

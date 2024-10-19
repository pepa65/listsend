use std::{fs, error::Error, thread::sleep, time};
use dotenv::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use lettre::message::header::ContentType;
use lettre::{Message, SmtpTransport, Transport};
use clap::Parser;
use serde::Serialize;

#[derive(Serialize)]
struct Csv {
    name: String, 
    email: String,
    data: String 
}
#[derive(Debug)]
struct EnvConfig {
    smtp_host: String,
    smtp_port: u16,
    smtp_pass: String,
    smtp_user: String,
    smtp_from: String,
}

impl EnvConfig {
    pub fn new() -> Self {
        Self {
            smtp_host: std::env::var("SMTP_HOST").expect("SMTP_HOST must be set in .env"),
            smtp_port: std::env::var("SMTP_PORT").expect("SMTP_PORT must be set in .env").parse::<u16>().unwrap(),
            smtp_pass: std::env::var("SMTP_PASS").expect("SMTP_PASS must be set in .env"),
            smtp_user: std::env::var("SMTP_USER").expect("SMTP_USER must be set in .env"),
            smtp_from: std::env::var("SMTP_FROM").expect("SMTP_FROM must be set in .env"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(help_template("\
{before-help}{name} {version} - {about}
{usage-heading} {usage}
{all-args}{after-help}
"))]
struct Cli {
    /// CSV file (name,email,data)
    #[arg(short, long, default_value = "list.csv")]
    csv: std::path::PathBuf,

    /// Email template file
    #[arg(short, long, default_value = "email.tpl")]
    template: std::path::PathBuf,

    /// Subject of the email
    #[arg(short, long)]
    subject: String,

    /// Email template is html [default: plain text]
    #[arg(short = 'H', long)]
    html: bool,

    /// Delay between mails in seconds
    #[arg(short, long, default_value_t = 1)]
    delay: u64,
}

fn read_csv_file(content: String) -> Result<Vec<Csv>, Box<dyn Error>>{
    let mut csv = Vec::new();
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    for result in reader.records() {
        let record = result?;
        csv.push(Csv {
            name: record.get(0).unwrap().into(),
            email: record.get(1).unwrap().into(),
            data: record.get(2).unwrap().into(),
        })
    }
    Ok(csv)
}

fn create_mailer(env: &EnvConfig) -> SmtpTransport {
    let creds = Credentials::new(env.smtp_user.clone(), env.smtp_pass.clone());
    let mailer = SmtpTransport::relay(&env.smtp_host)
        .unwrap()
        .port(env.smtp_port)
        .credentials(creds)
        .build();
    mailer
}

fn main() {
    let mut err = 0;
    let args = Cli::parse();
    dotenv().ok();
    let env = EnvConfig::new();
    let csv_content = fs::read_to_string(&args.csv).expect("could not read csv file");
    let template_content = fs::read_to_string(&args.template).expect("could not read template file");
    let csv = read_csv_file(csv_content).expect("Failed to read csv file with header: name,email,data");
    let mut hbs = handlebars::Handlebars::new();
    hbs.register_template_string("mail", template_content.clone()).expect("Failed to read template file content");
    let mailer = create_mailer(&env);
    for line in &csv {
        let csv_str = format!("\"{}\" <{}>", &line.name, &line.email);
        let email = Message::builder()
            .from(env.smtp_from.parse().unwrap())
            .to(csv_str.parse().unwrap())
            .subject(&args.subject)
            .header(if args.html {ContentType::TEXT_HTML} else {ContentType::TEXT_PLAIN})
            .body(hbs.render("mail", &line).unwrap())
            .unwrap();
        print!("--- Sending to: {} ", csv_str);
        match mailer.send(&email) {
            Ok(_) => println!(""),
            Err(e) => {
                println!("### Failed: {:?}", e);
                err += 1;
            },
        };
        sleep(time::Duration::from_secs(args.delay));
    }
    if err > 0 {
        println!("### Failed to send: {err}");
    }
    println!("=== All emails processed");
}

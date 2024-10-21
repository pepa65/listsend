use std::{fs, error::Error, thread::sleep, time};
use clap::Parser;
use dotenv::from_path;
use lettre::transport::smtp::authentication::Credentials;
use lettre::message::header::ContentType;
use lettre::{Message, SmtpTransport, Transport};
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
    smtp_port: u16, // Default 465
    smtp_user: String,
    smtp_pass: String,
    smtp_from: String,
    subject: String,
    html: String, // Default unset
    delay: u64, // Default 1
}

impl EnvConfig {
    pub fn check_or_default() -> Self {
        Self {
            smtp_host: std::env::var("SENDLIST_HOST").expect("SENDLIST_HOST must be set"),
            smtp_port: std::env::var("SENDLIST_PORT").unwrap_or("465".to_string()).parse::<u16>().unwrap(),
            smtp_user: std::env::var("SENDLIST_USER").expect("SENDLIST_USER must be set"),
            smtp_pass: std::env::var("SENDLIST_PASSWORD").expect("SENDLIST_PASSWORD must be set"),
            smtp_from: std::env::var("SENDLIST_FROM").expect("SENDLIST_FROM must be set"),
            subject: std::env::var("SENDLIST_SUBJECT").expect("SENDLIST_SUBJECT must be set"),
            html: std::env::var("SENDLIST_HTML").unwrap_or("".to_string()),
            delay: std::env::var("SENDLIST_DELAY").unwrap_or("1".to_string()).parse::<u64>().unwrap(),
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
    #[arg(short, long, default_value = "./list.csv")]
    csv: std::path::PathBuf,

    /// Email template file
    #[arg(short, long, default_value = "./email.tpl")]
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

const SMTP_ENV: &str = "./smtp.env";
const EMAIL_ENV: &str = "./email.env";

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
    let args = Cli::parse();
    if args.readme {
        print!("{}", include_str!("../README.md"));
        return;
    }
    from_path(&args.smtp).ok();
    from_path(&args.email).ok();
    let env = EnvConfig::check_or_default();
    let html = env.html != "" && env.html != "no" && env.html != "unset" && env.html != "0" && env.html != "false";
    let csv_content = fs::read_to_string(&args.csv).expect("could not read csv file");
    let template_content = fs::read_to_string(&args.template).expect("could not read template file");
    let csv = read_csv_file(csv_content).expect("Failed to read csv file with header: name,email,data");
    let mut hbs = handlebars::Handlebars::new();
    hbs.register_template_string("mail", template_content.clone()).expect("Failed to read template file content");
    let mailer = create_mailer(&env);
    let mut err = 0;
    for line in &csv {
        if line.name.bytes().nth(0) == Some(b'#') { continue; }
        let csv_str = format!("\"{}\" <{}>", &line.name, &line.email);
        let email = Message::builder()
            .from(env.smtp_from.parse().unwrap())
            .to(csv_str.parse().unwrap())
            .subject(&env.subject)
            .header(if html {ContentType::TEXT_HTML} else {ContentType::TEXT_PLAIN})
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
        sleep(time::Duration::from_secs(env.delay));
    }
    if err > 0 {
        println!("### Failed to send: {err}");
    }
    println!("=== All emails processed");
}

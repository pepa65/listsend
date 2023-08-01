use std::{fs, string, error::Error, thread::sleep, time, fmt::format};
use dotenv::dotenv;
use lettre::{SmtpTransport, transport::smtp::authentication::Credentials, Transport, Message, message::header::ContentType};
use clap::Parser;
use serde::Serialize;

#[derive(Serialize)]
struct Receiver {
    name: String, 
    email: String,
    registration_url: String 
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
            smtp_host: std::env::var("SMTP_HOST").unwrap(),
            smtp_port: std::env::var("SMTP_PORT").unwrap().parse::<u16>().unwrap(),
            smtp_pass: std::env::var("SMTP_PASS").unwrap(),
            smtp_user: std::env::var("SMTP_USER").unwrap(),
            smtp_from: std::env::var("SMTP_FROM").unwrap(),
        }
    }
}

#[derive(Parser)]
struct Cli {
    // Receivers file path (csv)
    // Note that the file should have the format `name,email` comma delimited
    #[arg(short = 'r', long)]
    receivers: std::path::PathBuf,

    // The html file path
    #[arg(long)]
    html: std::path::PathBuf,

    #[arg(long, short, default_value_t = 1)]
    duration: u64 
}

fn read_receivers_file(content: String) -> Result<Vec<Receiver>, Box<dyn Error>>{
    let mut receivers = Vec::new();
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    for result in reader.records() {
        let record = result?;
        receivers.push(Receiver {
            name: record.get(0).unwrap().into(),
            email: record.get(1).unwrap().into(),
            registration_url: "#".into(),
        })
    }

    Ok(receivers)
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
    dotenv().ok();

    let env = EnvConfig::new();

    // Parse the files
    let receivers_content = fs::read_to_string(&args.receivers).expect("could not read receivers file");
    let html_content = fs::read_to_string(&args.html).expect("could not read html file");

    // Parse receivers content file
    let receivers = read_receivers_file(receivers_content).expect("Failed to read receivers content file. Please follow the csv format (name,email) comma separated.");

    // Parse html file
    let mut hbs = handlebars::Handlebars::new();
    hbs.register_template_string("mail", html_content).expect("Failed to read template file content");

    // Setup email
    let mailer = create_mailer(&env);

    for receiver in receivers {
        let receiver_str = format!("{} <{}>", &receiver.name, &receiver.email);
        let email = Message::builder()
            .from(env.smtp_from.parse().unwrap())
            .to(receiver_str.parse().unwrap())
            .subject("TeknumConf Invitaton")
            .header(ContentType::TEXT_HTML)
            .body(hbs.render("mail", &receiver).unwrap())
            .unwrap();

        println!("Sending message to {}", receiver_str);
        mailer.send(&email).expect("Failed to send email");
        sleep(time::Duration::from_secs(args.duration))
    }

    println!("All email sent!");
}

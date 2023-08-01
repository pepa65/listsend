use std::{fs, string, error::Error};

use clap::Parser;

struct Receiver {
    name: String, 
    email: String,
    registration_url: String 
}

#[derive(Parser)]
struct Cli {
    // Receivers file path (csv)
    // Note that the file should have the format `name,email` comma delimited
    #[arg(short = 'r', long)]
    receivers: std::path::PathBuf,

    // The html file path
    #[arg(long)]
    html: std::path::PathBuf
}

fn read_receivers_file(content: String) -> Result<Vec<Receiver>, Box<dyn Error>>{
    let mut receivers = Vec::new();
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    for result in reader.records() {
        let record = result?;
        println!("Name: {:?}, Email: {:?}", record.get(0), record.get(1));
        receivers.push(Receiver {
            name: record.get(0).unwrap().into(),
            email: record.get(1).unwrap().into(),
            registration_url: "#".into(),
        })
    }

    Ok(receivers)

}

fn main() {
    let args = Cli::parse();

    // Parse the files
    let receivers_content = fs::read_to_string(&args.receivers).expect("could not read receivers file");
    let html_content = fs::read_to_string(&args.html).expect("could not read html file");

    // Parse receivers content file
    println!("Receivers content: {}", receivers_content);
    let receivers = read_receivers_file(receivers_content).expect("Failed to read receivers content file. Please follow the csv format (name,email) comma separated.");


}

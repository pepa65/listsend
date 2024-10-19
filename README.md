[![Cargo build](https://github.com/pepa65/listsend/actions/workflows/rust.yml/badge.svg)](https://github.com/pepa65/listsend/actions/workflows/rust.yml)

# listsend 0.2.2
**Send emails to CSV list from template on CLI**

* Repo: https:/github.com/pepa65/listsend
* Author: github.com/pepa65 <pepa65@passchier.net>
* License: MIT/Apache-2.0

<!--
## Install static single-binary
```
wget https://github.com/pepa65/listsend/releases/download/0.2.2/listsend
sudo mv listsend /usr/local/bin
sudo chown root:root /usr/local/bin/listsend
sudo chmod +x /usr/local/bin/listsend
```
-->
## Install with cargo
If not installed yet, install a **Rust toolchain**, see https://www.rust-lang.org/tools/install
<!--
### Direct from crates.io
```
cargo add openssl-sys
cargo install listsend
```
-->
### Direct from repo
```
cargo add openssl-sys
cargo install --git https://github.com/pepa65/listsend
```

### Static build (avoiding GLIBC incompatibilities)
```
git clone https://github.com/pepa65/listsend
cd listsend
rustup target add x86_64-unknown-linux-musl
cargo add openssl-sys
cargo rel  # Alias defined in .cargo/config.toml
```

The binary will be at `target/x86_64-unknown-linux-musl/release/listsend`

## Usage
```
listsend 0.3.0 - Send emails to CSV list from template on CLI
Usage: listsend [OPTIONS]
Options:
  -c, --csv <CSV>            CSV file (name,email,data) [default: list.csv]
  -t, --template <TEMPLATE>  Email template file [default: email.tpl]
  -s, --smtp <SMTP>          SMTP config file [default: ./smtp.env]
  -e, --email <EMAIL>        Email config file [default: ./email.env]
  -h, --help                 Print help
  -V, --version              Print version
```

The whole configuration goes through environment variables that can be set
independently or in the files `smtp.env` and `email.env`, and through the
email template `email.tpl` and the recipients list `list.csv`.

Copy the files in `example` to your working directory:
* Edit the `smtp.env` file with the data and credentials for the SMTP relay:
  `SENDLIST_HOST`, `SENDLIST_PORT` (default: 465), `SENDLIST_USER`,
  `SENDLIST_PASSWORD`, `SENDLIST_FROM`.
* Edit `email.env` with the data for the email: `SENDLIST_SUBJECT`,
  `SENDLIST_HTML` (default: no), `SENDLIST_DELAY` (default: 1 second).
* All fields above will be overridden by any corresponding environment variable
  that can be set like: `export SENDLIST_DELAY=0`.
* Edit `email.tpl` to the desired content, the fields `{{name}}`. `{{email}}`
  and `{{data}}` can be used in the template file.
  The template can be plain text (default) or html: set `HTML` to something.
* Edit `list.csv` for the recipient's data, where the column header is
  `name,email,data` and following lines specify each recipient.
  The fields `name` and `email` are mandatory, `data` is optional.
* Any of the above files can be set on the commandline as well to override the
  default name & path.


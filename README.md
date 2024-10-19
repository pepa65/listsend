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
listsend 0.2.2 - Send emails to CSV list from template on CLI
Usage: listsend [OPTIONS] --subject <SUBJECT>
Options:
  -c, --csv <CSV>            CSV file (name,email,data) [default: list.csv]
  -t, --template <TEMPLATE>  Email template file [default: email.tpl]
  -s, --subject <SUBJECT>    Subject of the email
  -H, --html                 Email template is html [default: plain text]
  -d, --delay <DELAY>        Delay between mails in seconds [default: 1]
  -h, --help                 Print help
  -V, --version              Print version
```

Copy the files in `example` to your working directory, and edit the `.env` file
with the data and credentials for the SMTP relay, edit `email.tpl` to the
desired content (the fields `{{name}}`. `{{email}}` and `{{data}}` can be used
in the template file), and edit `list.csv` for the recipient's data, where the
column header is `name,email,data` and following lines specify each recipient.

The template can be plain text (default) or html (in which case, supply the
`--html` flag on the commandline). The only mandatory argument is `--subject`,
the rest all have defaults (or no argument).

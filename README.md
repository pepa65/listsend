[![version](https://img.shields.io/crates/v/listsend.svg)](https://crates.io/crates/listsend)
[![build](https://github.com/pepa65/listsend/actions/workflows/rust.yml/badge.svg)](https://github.com/pepa65/listsend/actions/workflows/rust.yml)
[![dependencies](https://deps.rs/repo/github/pepa65/listsend/status.svg)](https://deps.rs/repo/github/pepa65/listsend)
[![docs](https://img.shields.io/badge/docs-listsend-blue.svg)](https://docs.rs/crate/listsend/latest)
[![license](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/pepa65/listsend/blob/main/LICENSE)
[![downloads](https://img.shields.io/crates/d/listsend.svg)](https://crates.io/crates/listsend)

# listsend 0.4.80
**Send emails to CSV list from template on CLI**

* License: MIT/Apache-2.0
* Authors: github.com/pepa65 <pepa65@passchier.net>, Ahmad Saugi <saugi.dev@gmail.com>
* Repo: https:/github.com/pepa65/listsend
* After: https://github.com/zuramai/emsend

## Install static single-binary
```
wget https://github.com/pepa65/listsend/releases/download/0.4.80/listsend
sudo mv listsend /usr/local/bin
sudo chown root:root /usr/local/bin/listsend
sudo chmod +x /usr/local/bin/listsend
```

## Install with cargo
If not installed yet, install a **Rust toolchain**, see https://www.rust-lang.org/tools/install

### Direct from crates.io
```
cargo install listsend
```

### Direct from repo
```
cargo install --git https://github.com/pepa65/listsend
```

### Static build (avoiding GLIBC incompatibilities)
```
git clone https://github.com/pepa65/listsend
cd listsend
rustup target add x86_64-unknown-linux-musl
cargo rel  # Alias defined in .cargo/config.toml
```

The binary will be at `target/x86_64-unknown-linux-musl/release/listsend`

## Install with cargo-binstall
Even without a full Rust toolchain, rust binaries can be installed with the static binary `cargo-binstall`:

```
# Install cargo-binstall for Linux x86_64
# (Other versions are available at https://crates.io/crates/cargo-binstall)
wget github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
tar xf cargo-binstall-x86_64-unknown-linux-musl.tgz
sudo chown root:root cargo-binstall
sudo mv cargo-binstall /usr/local/bin/
```

Only a linux-x86_64 (musl) binary available: `cargo-binstall listsend`

It will be installed in `~/.cargo/bin/` which will need to be added to `PATH`!

## Usage
```
listsend 0.4.80 - Send emails to CSV list from template on CLI
Usage: listsend [OPTIONS]
Options:
  -c, --csv <CSV>            CSV file (name,email,data) [default: ./list.csv]
  -t, --template <TEMPLATE>  Email template file [default: ./email.tpl]
  -s, --smtp <SMTP>          SMTP config file [default: ./smtp.env]
  -e, --email <EMAIL>        Email config file [default: ./email.env]
  -r, --readme               Output readme file from repo
  -h, --help                 Print help
  -V, --version              Print version
```

The whole configuration goes through environment variables that can be set
independently or in the files `smtp.env` and `email.env`, and through the
email template `email.tpl` and the recipients list `list.csv`.

Copy the files in `example` to your working directory:
* Edit the `smtp.env` file with the data and credentials for the SMTP relay:
  `SENDLIST_HOST`, `SENDLIST_PORT` (default: 465), `SENDLIST_USER`,
  `SENDLIST_PASSWORD` and `SENDLIST_FROM`, all are mandatory.
* Edit `email.env` with the data for the email: `SENDLIST_REPLY_TO` (default: none),
  `SENDLIST_CC` (default: none), `SENDLIST_BCC` (default: none),
  `SENDLIST_SUBJECT` (mandatory), `SENDLIST_HTML` (default: plain text),
  `SENDLIST_ATTACHMENT` (default: none), `SENDLIST_DELAY` (default: 1 second).
* All fields above will be overridden by any corresponding environment variable
  that can be set like: `export SENDLIST_DELAY=0`.
* Edit `email.tpl` to the desired content, the fields `{{name}}`, `{{email}}`
  and `{{data}}` from `list.csv` can be used in the template file and
  in `SENDLIST_SUBJECT` (the Subject line). The template can be
  plain text (default) or html: set `SENDLIST_HTML` to something other than
  `no`, `unset`, `false`, `0` or empty.
* Edit `list.csv` for the recipient's data, where the column header is
  `name,email,data` and following lines specify each recipient.
  The fields `name` and `email` are mandatory, `data` is optional.
  If `#` is used as the first character of a line, it gets ignored.
* Any field/variable that takes an email address can do so as a single email,
  or in the mailbox format: 'Some Name <email@address.to>', but only one!
* Any of the above files can be set on the commandline as well to override the
  default name & path.

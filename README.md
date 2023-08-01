# emsend

Email Sender created with Rust

## Usage

```bash
emsend --receivers PATH_TO_RECEIVERS_FILE --html PATH_TO_HTML_FILE --delay DELAY_IN_SECS
```

Example: 
```bash
emsend --receivers example/receivers.csv --html example/template.html --delay 1
```

## Receiver CSV Format

name,email

## Usage

```terminal
$ howto --help

Usage: howto <ACTION>

Arguments:
  <ACTION>  The high-level action you would like to get a CLI command for

Options:
  -h, --help  Print help
```

## Installation

```terminal
cargo build --release

cp target/release/howto ~/.local/bin/

chmod +x ~/.local/bin/howto
```

## Credentials

Chuck an API key in `~/.howto-cli/credentials`

OR

Set the `HOWTO_CLI_OPENAI_API_KEY` environment variable

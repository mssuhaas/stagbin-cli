
# Stagbin CLI Tool


## Installation

**Note:** This is a basic example, you may need to adjust depending on your environment.

You'll need Rust installed to use `stagbin`. Once Rust is set up:

1. Clone this repository:

```bash
git clone https://github.com/mssuhaas/stagbin-cli.git
```

2. Navigate to the project directory:

```bash
cd stagbin-cli
```

3. Build the release version for optimized performance:

```bash
cargo build --release
```

This will create an executable file named `stagbin` (or `stagbin.exe` on Windows) in the `target/release` directory. You can then run the tool from that directory or move the executable to a location in your system path for easier access.


## Usage

This program allows you to send or retrieve data from Stagbin.

To send data, use the `-s` or `--send` flag. You can provide the data directly using the `-d` or `--data` flag, or from a file using the `-f` or `--file` flag. An optional `-i` or `--id` flag can be used to specify a custom identifier for the data.

To retrieve data, use the `-r` or `--retrieve` flag. You must provide the ID of the data you want to download using the `-i` or `--id` flag. An optional `-o` or `--output` flag can be specified to provide the filename where the retrieved data will be saved. If no output filename is provided, a default filename based on the ID will be used.

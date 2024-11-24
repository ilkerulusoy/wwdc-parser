# WWDC Video Parser

A command-line tool to convert WWDC video pages to markdown format.

## Installation

### From Source

Clone the repository
```bash
git clone https://github.com/username/wwdc-parser
cd wwdc-parser  
```

Build and install
```bash
cargo install --path .
```

### From Cargo  

```bash
cargo install wwdc-parser
```

## Usage

```bash
wwdc-parser <video-url>
```

Example:
```bash
wwdc-parser https://developer.apple.com/videos/play/wwdc2023/10087/
```

This will generate a markdown file with the video content in your current directory.

## Features

- Converts WWDC video pages to markdown format
- Extracts video title, description, and content
- Generates clean, readable markdown files
- Simple command-line interface
- Automatic file naming based on video title

## Requirements

- Rust 1.70 or higher
- Internet connection to fetch video data

## Building from Source

1. Ensure you have Rust installed
2. Clone the repository
3. Run `cargo build --release`
4. The binary will be available in `target/release/`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with Rust
- Uses [reqwest](https://github.com/seanmonstar/reqwest) for HTTP requests
- Inspired by WWDC video content

## Author

[Ilker Ulusoy](https://github.com/ilkerulusoy)

## Support

If you encounter any problems, please file an issue along with a detailed description.
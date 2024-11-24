# WWDC Video & Documentation Parser

A command-line tool to convert WWDC video pages and Apple Developer documentation to markdown format.

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

### For WWDC Videos
```bash
wwdc-parser --content-type video <video-url>
```

Example:
```bash
wwdc-parser --content-type video https://developer.apple.com/videos/play/wwdc2024/10091/
```

### For Documentation Pages
```bash
wwdc-parser --content-type document <documentation-url>
```

Example:
```bash
wwdc-parser --content-type document https://developer.apple.com/documentation/groupactivities/
```

This will generate a markdown file with the content in your current directory.

## Features

- Converts WWDC video pages to markdown format
- Converts Apple Developer documentation to markdown format
- Extracts titles, descriptions, and content
- Generates clean, readable markdown files
- Simple command-line interface
- Automatic file naming based on content title

## Requirements

- Rust 1.70 or higher
- Internet connection to fetch content

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
- Uses [scraper](https://github.com/causal-agent/scraper) for HTML parsing

## Author

[Ilker Ulusoy](https://github.com/ilkerulusoy)

## Support

If you encounter any problems, please file an issue along with a detailed description.
# Inspector Gadget

Inspector Gadget is a command-line tool that inspects all links on a given documentation site, recursively discovering and analyzing child pages. It outputs the results in JSON, YAML, or a clipboard-friendly format.

## Installation

To install Inspector Gadget, you need to have Rust and Cargo installed on your system. Then, you can clone this repository and build the project:

```bash
git clone https://github.com/yourusername/inspector-gadget.git
cd inspector-gadget
cargo build --release
```

The binary will be available in `target/release/inspector-gadget`.

## Usage

```bash
inspector-gadget <URL> [OPTIONS]
```

Options:
- `--output-format <FORMAT>`: Choose between json, yaml, or clipboard (default: json)
- `--log-level <LEVEL>`: Adjust the verbosity of logs (e.g., info, debug, error) (default: info)
- `--help`: Displays help information
- `--version`: Shows version information

Example:
```bash
inspector-gadget https://docs.dagger.io/
```

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request.

## License

Inspector Gadget is licensed under the [MIT License](LICENSE).
# Inspector Gadget

Inspector Gadget is a command-line tool that inspects all links on a given documentation site, recursively discovering and analyzing child pages. It outputs the results in JSON, YAML, TXT, or a clipboard-friendly format.

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

| Option | Description |
|--------|-------------|
| `--output-format <FORMAT>` | Choose between json, yaml, txt, or clipboard (default: json) |
| `--output-file <FILE>` | Specify the output file name (default: inspect-result-<domain>.<format>) |
| `--log-level <LEVEL>` | Adjust the verbosity of logs (e.g., info, debug, error) (default: info) |
| `--help` | Displays help information |
| `--version` | Shows version information |
| `--show-links` | Show links in the terminal |
| `--detailed` | Show detailed information including ignored links |
| `--strict` | Only scan links that are under or children of the passed URL |

Example:
```bash
inspector-gadget https://docs.dagger.io --show-links --strict --output-format=txt --output-file=dagger-doc-links
```

## Configuration

The inspector-gadget tool uses a YAML configuration file named `.inspector-config.yml` in the same directory as the executable. This file allows you to customize various aspects of the link inspection process.

Here's a description of the configuration options:

| Field | Type | Description |
|-------|------|-------------|
| `ignore` | Object | Contains settings for ignoring certain URLs |
| `ignore.domains` | Array of Strings | List of domain suffixes to ignore |
| `ignore.regex` | Array of Strings | List of regex patterns to ignore URLs |
| `forbidden_domains` | Array of Strings | List of domain suffixes that are forbidden to scan |
| `ignored_childs` | Array of Strings | List of URL path prefixes to ignore |
| `timeout` | Integer | Timeout in seconds for each HTTP request |
| `default_output` | String | Default output format if not specified in CLI arguments |

### Example Configuration

```yaml
ignore:
  domains:
    - "example.com"
    - "test.org"
  regex:
    - "^https?://localhost"
    - "^https?://127\\.0\\.0\\.1"
forbidden_domains:
  - "forbidden.com"
  - "restricted.org"
ignored_childs:
  - "/api/"
  - "/internal/"
timeout: 30
default_output: "json"
```

In this example:
- URLs from domains ending with "example.com" or "test.org" will be ignored.
- URLs matching the regex patterns (localhost or 127.0.0.1) will be ignored.
- URLs from domains ending with "forbidden.com" or "restricted.org" are forbidden to scan.
- URLs with paths starting with "/api/" or "/internal/" will be ignored.
- Each HTTP request will timeout after 30 seconds.
- The default output format is set to JSON if not specified in the CLI arguments.

You can customize these settings to fit your specific link inspection needs.

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request.

## License

Inspector Gadget is licensed under the [MIT License](LICENSE).
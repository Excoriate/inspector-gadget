# Inspector CLI

A CLI tool for inspecting and analyzing web links.

## Installation

You can install Inspector CLI using Cargo:

```
cargo install inspector-cli
```

## Usage

```bash
inspector-cli <URL> [OPTIONS]
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
| `--config <FILE>` | Sets a custom config file |
| `--ignore-domains <DOMAINS>` | Comma-separated list of domains to ignore |
| `--ignore-regex <REGEX>` | Comma-separated list of regex patterns to ignore URLs |
| `--forbidden-domains <DOMAINS>` | Comma-separated list of forbidden domains |
| `--ignored-childs <PATHS>` | Comma-separated list of child paths to ignore |
| `--timeout <SECONDS>` | Timeout in seconds for each HTTP request |

Example:
```bash
inspector-cli https://docs.dagger.io --show-links --strict --output-format=txt --output-file=dagger-doc-links
```

## Configuration

The inspector-cli tool uses a YAML configuration file named `.inspector-config.yml` in the user's home directory. This file allows you to customize various aspects of the link inspection process.

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

Inspector CLI is licensed under the [MIT License](LICENSE).
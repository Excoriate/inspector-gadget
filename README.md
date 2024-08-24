# Inspector Gadget CLI

A powerful CLI tool for inspecting and analyzing web links. Handy when it comes to feeding your LLM with the right context.

## Installation

You can install Inspector CLI using Cargo:

```
cargo install inspector
```

## Usage

```bash
inspector <URL> [OPTIONS]
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
| `--config <FILE>` | Sets a custom config file |
| `--ignore-domains <DOMAINS>` | Comma-separated list of domains to ignore |
| `--ignore-regex <REGEX>` | Comma-separated list of regex patterns to ignore URLs |
| `--forbidden-domains <DOMAINS>` | Comma-separated list of forbidden domains |
| `--ignored-childs <PATHS>` | Comma-separated list of child paths to ignore |
| `--timeout <SECONDS>` | Timeout in seconds for each HTTP request |

Example:
```bash
inspector https://docs.dagger.io --show-links --output-format=txt --output-file=dagger-doc-links
```

## Configuration

The inspector tool uses a YAML configuration file named `.inspector-config.yml` in the user's home directory. This file allows you to customize various aspects of the link inspection process.

Here's a description of the configuration options:

| Field | Type | Description |
|-------|------|-------------|
| `url` | String | The base URL to start the inspection from (required) |
| `ignore` | Object | Contains settings for ignoring certain URLs |
| `ignore.domains` | Array of Strings | List of domain suffixes to ignore |
| `ignore.regex` | Array of Strings | List of regex patterns to ignore URLs |
| `forbidden_domains` | Array of Strings | List of domain suffixes that are forbidden to scan |
| `ignored_childs` | Array of Strings | List of URL path prefixes to ignore |
| `timeout` | Integer | Timeout in seconds for each HTTP request |
| `default_output` | String | Default output format if not specified in CLI arguments |

### Example Configuration

```yaml
url: https://docs.dagger.io
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

You can use a custom configuration file by specifying its path:

```bash
inspector https://example.com --config /path/to/custom-config.yml
```

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request.

## License

Inspector CLI is licensed under the [MIT License](LICENSE).

## Examples

### Terragrunt Documentation

To fetch all the Terragrunt documentation, you can use the following commands:

```bash
# Locally
just run https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry/ --show-links --output-format=txt --output-file=terragrunt-docs-links
# or using the inspector cli
inspector https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry/ --show-links --output-format=txt -o terragrunt-docs-links
```

Alternatively, you can use a configuration file:

```bash
# Locally
just run https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry/ --config docs/examples/terragrunt-docs/terragrunt-inspector-config.yml
# or using the inspector cli
inspector https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry/ --config docs/examples/terragrunt-docs/terragrunt-inspector-config.yml
```

The `terragrunt-inspector-config.yml` file contains the following configuration:

```yaml
url: https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry/
ignore:
  domains:
    - "github.com"
    - "twitter.com"
    - "linkedin.com"
  regex:
    - "^https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry/#.*$"
forbidden_domains:
  - "example.com"
ignored_childs:
  - "/community/"
  - "/plugins/"
timeout: 30
default_output: "json"
```

### Terraform Documentation

To fetch all the Terraform documentation, you can use the following commands:

```bash
# Locally
just run https://terraform-docs.io/user-guide/introduction/ --show-links --output-format=txt --output-file=terraform-docs-links
# or using the inspector cli
inspector https://terraform-docs.io/user-guide/introduction/ --show-links --output-format=txt -o terraform-docs-links
```

Alternatively, you can use a configuration file:

```bash
# Locally
just run https://terraform-docs.io/user-guide/introduction/ --config docs/examples/terraform-docs/terraform-docs-inspector-config.yml
# or using the inspector cli
inspector https://terraform-docs.io/user-guide/introduction/ --config docs/examples/terraform-docs/terraform-docs-inspector-config.yml
```

The `terraform-docs-inspector-config.yml` file contains the following configuration:

```yaml
url: https://terraform-docs.io/user-guide/introduction/
ignore:
  domains:
    - "github.com"
    - "twitter.com"
    - "linkedin.com"
  regex:
    - "^https://terraform-docs.io/user-guide/.*#.*$"
forbidden_domains:
  - "example.com"
ignored_childs:
  - "/community/"
  - "/plugins/"
timeout: 30
default_output: "json"
```
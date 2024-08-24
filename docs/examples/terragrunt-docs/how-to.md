## How to fetch all the Terragrunt documentation
### From the command line
```bash
# locally
just run https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry --show-links --output-format=txt --output-file=terragrunt-docs-links
# or using the inspector cli
inspector https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry --show-links --output-format=txt -o terragrunt-docs-links
```

### With a configuration file
```bash
# Locally
just run https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry --config docs/examples/terragrunt-docs/terragrunt-inspector-config.yml
# or using the inspector cli
inspector https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry --config docs/examples/terragrunt-docs/terragrunt-inspector-config.yml
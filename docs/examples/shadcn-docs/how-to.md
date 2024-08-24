## How to fetch all the components documentation
### From the command line.
```bash
# locally
just run https://ui.shadcn.com/docs/components --show-links --output-format=txt -output-file=shadcn-components-links
# or using the inspector cli
inspector https://ui.shadcn.com/docs/components --show-links --output-format=txt -o shadcn-components-links
```

### With a configuration file
```bash
# Locally
just run https://ui.shadcn.com/docs/components --config docs/examples/shadcn-docs/shadcn-components-config.yml
# or using the inspector cli
inspector https://ui.shadcn.com/docs/components --config docs/examples/shadcn-docs/shadcn-components-config.yml
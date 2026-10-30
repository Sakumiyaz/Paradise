# Security

## Reporting

Report a vulnerability privately to the repository owner. Do not open a public issue that includes a token, a private host, or a working exploit.

## Release binaries

- Install only after `SHA256SUMS` matches the downloaded asset.
- v0.4.0 predates checksums and compiled in an operator HTTP endpoint. Treat that binary as superseded.
- The installer refuses a platform it does not publish instead of exiting from `pipefail` before the error text.

## Data

API keys and provider config stay in the Paradise state directory unless you configure a provider that receives prompts. Do not put a bearer token in a shell history you share. The binary must not contain a default model-server URL or token.

## Supported platforms

Linux aarch64 is the only published target. Other platforms are unsupported until a binary and checksum exist for them.

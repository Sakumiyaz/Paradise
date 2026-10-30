# Security

## Reporting

Report a vulnerability privately. Do not open a public issue that includes a token, a private host, or a working exploit.

- GitHub private advisory: https://github.com/Sakumiyaz/Paradise/security/advisories/new
- Email: starsoftheend@gmail.com

## Release binaries

- Install only after `SHA256SUMS` matches the downloaded asset.
- v0.4.0 compiled in an operator HTTP endpoint. It has been withdrawn and the installer refuses that tag.
- Checksums published next to a release do not by themselves prove the release account was not compromised. A detached signature from a key that is not stored in this GitHub account is not published yet. Until that key exists, treat a changed `SHA256SUMS` on an already announced release as a signal to stop and report it.
- The installer refuses a platform it does not publish instead of exiting from `pipefail` before the error text.

## Data

Provider configuration and API keys are stored under `~/.config/paradise` (or `$XDG_CONFIG_HOME/paradise`). The directory is created mode 0700 and files written by the CLI are mode 0600. Do not keep an API key in a repository checkout or in `/tmp`.

A chat request sends the prompt only to a provider you configured. The binary does not contain a default model-server URL.

## Supported platforms

Linux aarch64 is the only published target. Other platforms are unsupported until a binary and checksum exist for them.

# Changelog

## v0.4.2

- Installer reads GitHub release JSON with Python without consuming stdin in a heredoc.
- Asset names are matched exactly, so a `.sig` file cannot be selected by substring.
- Unknown installer arguments fail instead of installing.
- v0.4.0 cannot be installed. That release is withdrawn.
- Default state directory is `~/.config/paradise` (directory mode 0700, files 0600), not the working tree and not `/tmp`.
- Release binary no longer embeds the build machine path, SSH operator commands, or `cargo run` instructions. Local evidence filenames, including Megatron report names, are still present because the runtime looks for those files.

## v0.4.1

- Removed the compiled-in operator HTTP endpoint.
- Installer verifies `SHA256SUMS` and installs mode 0755 as root when needed.
- Unsupported platforms print an error instead of exiting before the message.
- Python release parsing in this installer was broken; use v0.4.2.

## v0.4.0

Withdrawn. The binary embedded an operator HTTP endpoint and was published without a checksum. Do not install it.

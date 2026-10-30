# Changelog

## v0.4.2

- Installer reads GitHub release JSON with Python without consuming stdin in a heredoc.
- Asset names are matched exactly, so a `.sig` file cannot be selected by substring.
- Unknown installer arguments fail instead of installing.
- v0.4.0 cannot be installed. That release is withdrawn.
- Default state directory is `~/.config/paradise` (directory mode 0700, files 0600), not the working tree and not `/tmp`.
- Release binary no longer embeds the build machine path or `cargo run` instructions. `agent model query`, `server`, and `monitor` still contain SSH operator command text. Local evidence filenames, including Megatron report names, are still present because the runtime looks for those files.

## v0.4.1

Withdrawn. The installer published with this release could not read a real GitHub release document without Python, and the binary still contains SSH operator command text. Use v0.4.2.

## v0.4.0

Withdrawn. The binary embedded an operator HTTP endpoint and was published without a checksum. Do not install it.

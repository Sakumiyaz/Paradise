# Paradise — Self-Evolving Code Intelligence

Local coding agent runtime. It can remember patterns on this machine and route work through the EDEN-ARBOR modules that ship in the binary. It is not a hosted service, and this repository does not include the source.

## What this repository is

This public repository publishes the installer and release binaries only. The implementation is private. Cloning this repository will not give you a crate to compile.

## Supported systems

| System | Status |
| --- | --- |
| Linux aarch64 | Published binary |
| Linux x86_64 | Not published. The installer exits with an error instead of failing silently. |
| macOS | Not published. The installer exits with an error. |

Requirements: `curl`, `install`, and `sha256sum` (from coreutils). Python 3 is used when present to parse the GitHub release JSON; without it the installer uses a narrower fallback parser.

## Install

Read the script before running it. Do not pipe it into a shell.

```bash
curl -fsSL -o install.sh https://raw.githubusercontent.com/Sakumiyaz/Paradise/main/install.sh
# Read install.sh. Compare it with the copy in this repository if you have one.
bash install.sh
```

Pin a release:

```bash
PARADISE_VERSION=v0.4.2 bash install.sh
```

The installer downloads one GitHub release object, checks `SHA256SUMS` before copying the binary, and installs it as root:root mode 0755 when `/usr/local/bin` is not writable. It will not install if the checksum file is missing, unless you set `PARADISE_ALLOW_UNVERIFIED=1`.

Uninstall:

```bash
bash install.sh --uninstall
```

Install v0.4.2 or later. v0.4.0 and v0.4.1 are withdrawn.

## First run

```bash
paradise
paradise --help
```

With no arguments in a terminal, the binary opens the TUI. Otherwise it runs the CLI.

## Configuration and data

Operator state defaults to `~/.config/paradise`, or `$XDG_CONFIG_HOME/paradise` when that variable is set. Override it with `PARADISE_STATE_DIR` or `--state-dir`. The directory is mode 0700 and files written by the CLI are mode 0600. Do not use the working tree or `/tmp`: a checkout can be committed, and `/tmp` is shared.

Provider settings, including an API key if you add one, stay in that directory (`paradise_providers.json`). They are not uploaded by the installer. A chat request sends the prompt only to the provider you configured. If you do not configure a provider, do not assume a prompt left the machine.

There is no compiled-in model server. A remote Eden module server is used only when you set:

| Variable | Meaning |
| --- | --- |
| `EDEN_70B_MODEL_SERVER_URL` | `https://` URL of your model server. Required for that backend. |
| `EDEN_70B_MODEL_SERVER_TOKEN` | Optional bearer token. Refused on non-loopback HTTP. |
| `EDEN_70B_ALLOW_INSECURE_HTTP` | Set to `1` only on a trusted network if you must use HTTP. |
| `PARADISE_GPU_HOST` | Operator GPU host. Required for remote GPU commands. No default. |
| `PARADISE_GPU_USER` | SSH user for that host. |
| `PARADISE_GPU_SSH_KEY` | Optional SSH key path. |
| `PARADISE_STATE_DIR` | State directory. |

## What the comparison table used to claim

Earlier text compared Paradise with Copilot and Cursor as if those products never keep session memory. That is not a fair claim: Cursor can retain project memory, and Copilot has its own history features. Paradise's local distinction is that its state files stay on the machine you choose, and that model output is not treated as permission to act. Test counts are not listed here until a public CI run publishes them.

## License

Proprietary. All rights reserved. See `LICENSE`. Binary releases are provided for installation and use. Source code is not distributed from this repository.

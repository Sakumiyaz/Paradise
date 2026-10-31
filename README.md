# Paradise — Self-Evolving Code Intelligence

Local coding agent runtime. It can remember patterns on this machine and route work through the EDEN-ARBOR modules that ship in the binary. It is not a hosted service, and this repository does not include the source.

## What this repository is

This public repository publishes the installer and release binaries only. The implementation is private. Cloning this repository will not give you a crate to compile.

## Supported systems

| System | Asset |
| --- | --- |
| Linux aarch64 | `paradise-aarch64-linux` |
| Linux x86_64 | `paradise-x86_64-linux` |
| macOS Apple Silicon | `paradise-aarch64-darwin` |
| macOS Intel | Not published. ONNX Runtime has no prebuilt for that target. |

Requirements: `curl`, `install`, `sha256sum`, and `minisign`. Without `minisign` the installer stops, unless you set `PARADISE_ALLOW_UNSIGNED=1`. Linux binaries need OpenSSL 3, so Ubuntu 20.04 and Debian 11 are not supported. Python 3 is used when present to parse the GitHub release JSON; without it the installer uses a narrower fallback parser.

## Install

Read the script before running it. Do not pipe it into a shell.

```bash
curl -fsSL -o install.sh https://raw.githubusercontent.com/Sakumiyaz/Paradise/main/install.sh
# Read install.sh. Compare it with the copy in this repository if you have one.
bash install.sh
```

Pin a release:

```bash
PARADISE_VERSION=v0.4.13 bash install.sh
```

The installer downloads one GitHub release object, checks `SHA256SUMS`, and verifies `SHA256SUMS.minisig` with the embedded public key. Set `PARADISE_MINISIGN_PUB` to a key file you saved outside GitHub if you do not want to trust the copy in this repository. `PARADISE_ALLOW_UNSIGNED=1` skips that check. It installs mode 0755, using sudo only when the install directory is not writable by you. It will not install if the checksum file is missing, unless you set `PARADISE_ALLOW_UNVERIFIED=1`.

Uninstall:

```bash
bash install.sh --uninstall
```

Install v0.4.13 or later. v0.4.3, v0.4.4, and v0.4.5 are withdrawn.

## First run

```bash
paradise
paradise --help
```

With no arguments in a terminal, the binary opens the TUI. Otherwise it runs the CLI.

## Configuration and data

Operator state defaults to `~/.config/paradise`, or `$XDG_CONFIG_HOME/paradise` when that variable is set. Override it with `PARADISE_STATE_DIR` or `--state-dir`. The directory is mode 0700 and files written by the CLI are mode 0600. Do not use the working tree or `/tmp`: a checkout can be committed, and `/tmp` is shared.

Provider settings stay in that directory (`paradise_providers.json`, mode 0600). A new install uses local Ollama at `http://127.0.0.1:11434` and does not send prompts anywhere until that local server is running. A remote provider is used only after you select it with `paradise config set provider`. `paradise config set api_key` does not print the key.

There is no compiled-in model server. A remote Eden module server is used only when you set:

| Variable | Meaning |
| --- | --- |
| `EDEN_70B_MODEL_SERVER_URL` | `https://` URL of your model server. Required for that backend. |
| `EDEN_70B_MODEL_SERVER_TOKEN` | Optional bearer token. Refused on non-loopback HTTP. |
| `EDEN_70B_ALLOW_INSECURE_HTTP` | Set to `1` only on a trusted network if you must use HTTP. |
| `PARADISE_STATE_DIR` | State directory. |

## What the comparison table used to claim

Earlier text compared Paradise with Copilot and Cursor as if those products never keep session memory. That is not a fair claim: Cursor can retain project memory, and Copilot has its own history features. Paradise's local distinction is that its state files stay on the machine you choose, and that model output is not treated as permission to act. Test counts are not listed here until a public CI run publishes them.

Writes, shell commands, and git commands proposed by the model wait for a `y` confirmation. `paradise chat --yes` skips that prompt. `RUN`, `GIT`, `CHECK`, and `TEST` need [bubblewrap](https://github.com/containers/bubblewrap) and can write only in the current directory. macOS does not include bubblewrap, so those commands are refused there. A file edit is not treated as success unless the requested name is defined, for example `def multiply`. A filename, or words such as `define` and `returns`, are not required definitions. A comment, string, or filename does not count. If the request asks for a test and the change is Python, that test is run and must pass. An edit request that changes nothing exits non-zero. `paradise chat --api-key` applies only to that invocation and is not saved. Chat history stays with that project.

## License

Proprietary. All rights reserved. See `LICENSE`. Binary releases are provided for installation and use. Source code is not distributed from this repository.

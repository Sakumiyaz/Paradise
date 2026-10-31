# Changelog

## v0.4.15

- The name check follows the function the task asks for, not the rest of the sentence. `Add multiply please, cheers` and `Add multiply, use print() to debug` require `multiply` only. `quiero multiply` and `por favor multiply` recognize `multiply`.
- `Add multiply and divide` still requires both. A function mentioned only as a comparison is still not required.

## v0.4.14

- The name check asks only for the function the task introduces. `Add multiply(a, b) like add(a, b)` does not require `def add`. Words such as `and`, `keep`, `existing`, or `función` are not required definitions.
- A requested Python test must return and pass. `os._exit(0)` does not count. A test that does not finish in 5 seconds is killed and the task fails.

## v0.4.13

- The names required for a successful edit come from `def`, `fn`, or `name(...)`. A filename, or words such as `define` and `returns`, are not required definitions.
- When the request asks for a test and the change is Python, exit 0 requires the `test_*` functions in the edited files to run and pass. A definition that returns the wrong value is rejected.
- A test that does not check the result can still pass. Non-Python tests are still checked as definitions only.

## v0.4.12

- Writes through a symlink into `.git`, and paths spelled `.GIT`, are refused.
- A requested name counts only as a definition such as `def multiply`, not as a comment, string, or filename. If the request also asks for a test, that test definition is required.
- An edit request that changes nothing, or only part of what was asked, exits non-zero.
- `TEST:` and `CHECK:` accept a command again. The command runs in the sandbox.

## v0.4.11

- `git diff` and `git status` used after an edit no longer honor `core.fsmonitor`, hooks, or a global git config. Writes inside `.git/` are refused, and shell commands cannot modify `.git/`.
- `--api-key` is not written to `paradise_providers.json`.
- A lone `DONE` is rejected when the request asked for a named change and that name is not in the added code. A comment or a filename does not count.
- Actions inside markdown fences are parsed. A unified diff is applied, and `EDIT_FILE` tolerates indentation differences.
- `WRITE_FILE` ends the file with a newline. `config get` of an unknown key exits non-zero. Python projects can use `TEST` via pytest or unittest.

## v0.4.10

- `GIT`, `CHECK`, and `TEST` run in the same bubblewrap sandbox as `RUN`. A git hook or alias cannot write outside the project.
- Without bubblewrap those commands are refused. macOS does not include bubblewrap, so they are refused there.
- A file change is not success unless the requested name appears in the change. `DONE` on a question that did not try to edit a file is allowed. An empty response is not.
- Repeated `READ`, `RUN`, and `GIT` actions are skipped, not run again.
- `--api-key` is used for that invocation only. It is not written to disk. It is listed in `--help`.

## v0.4.9

- `DONE` is rejected unless a file was actually written or edited. A failed or missing edit exits non-zero.
- The model is no longer told invented action statistics, and a read-only response is not marked `NOT APPLIED`.
- The same edit is not applied again in the same turn.
- Chat history is stored per project and only the recent messages are sent.
- `--api-key` is a flag, not part of the prompt. A remote provider without a key is not called.
- `chat --help` shows help instead of sending that text to the model.
- `--yes` no longer claims that confirmation will be requested. Shell commands run in a sandbox that cannot write outside the project.
- `WRITE_FILE` creates missing directories inside the project.

## v0.4.8

- Model output is not permission to act. Writes, shell commands, and git commands wait for confirmation. `--yes` is the explicit override.
- File actions stay inside the current project. `../` and `/etc/passwd` are refused.
- `EDIT_FILE: path | OLD: ... | NEW: ...` works with spaces, and multi-line edits work. An edit that changes nothing is a failure, not `SUCCESS`.
- A read and an edit in the same response no longer edits before the file contents are returned.
- `agent run` and `agent workflow` use the configured model. They do not compile `eden_agent_system`.
- `paradise chat` exits non-zero when the provider or model fails.
- Skill files are written under the state directory, not in the project tree.

## v0.4.7

- The installer test changes `SHA256SUMS` and leaves `SHA256SUMS.minisig` untouched. That install is refused.
- Internal version text in the binaries is v0.4.7. The stale `v0.4.0` label is gone.
- v0.4.5 is withdrawn. It still created state files as mode 0644.

## v0.4.6

- State files created by `status` and `init` are mode 0600, not the user umask.
- An invalid signature is refused even if `PARADISE_ALLOW_UNSIGNED=1`.
- v0.4.4 is withdrawn. It still selected the remote ollama-cloud provider.
- The default local provider URL is `http://127.0.0.1:11434`.
- Release builds remap runner paths and no longer embed `/tmp/eden_garm`.

## v0.4.5

- A new install uses local Ollama. It does not select a remote provider.
- `paradise_providers.json` is written mode 0600. `config set api_key` does not print the key.
- `PARADISE_MINISIGN_PUB` must name an existing file. A missing path is an error, not a silent fallback.
- v0.4.3 is withdrawn. Its commit had a failing installer check.

## v0.4.4

- The installer refuses to continue when `minisign` is missing, unless `PARADISE_ALLOW_UNSIGNED=1`.
- Pinning an older unsigned release no longer skips the signature check.
- Creating `~/.local` for a user install no longer calls sudo.
- `paradise --version` reports 0.4.4 and does not claim a test count.
- `paradise status` creates the state directory mode 0700.
- Public help no longer lists GPU SSH commands or `PARADISE_GPU_*`.

## v0.4.3

- Linux x86_64 and macOS Apple Silicon binaries are published. macOS Intel is not, because ONNX Runtime has no prebuilt for that target.
- GPU SSH commands are compiled only with `--features operator-gpu`. Public binaries do not enable that feature.
- `SHA256SUMS` is signed with minisign. The secret key is not in the GitHub account. The public key is `minisign.pub` and is embedded in `install.sh`.

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

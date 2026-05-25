# Security Policy

Paradise is an experimental local-first runtime. Treat every command surface
as local operator infrastructure unless a future release explicitly adds
authentication, authorization and deployment guidance.

## Supported Versions

| Version | Supported |
| --- | --- |
| `master` | Security fixes accepted before public release. |
| Tagged public drafts | Best-effort fixes only. |

## Reporting A Vulnerability

Do not open a public issue for vulnerabilities, secrets, credentials, unsafe
remote execution paths or command-injection findings.

Use a private GitHub security advisory or contact the repository maintainer
through the private repository channels. Include:

- affected commit;
- reproduction steps;
- expected impact;
- whether any credential, token, `.env` file or private artifact was exposed.

## Runtime Boundary

The GARM API binds to `127.0.0.1` by default. Do not expose it directly to a
public network. Mutation-capable routes are intentionally narrow:

- `/api/command?cmd=...`
- `/api/command_sync?cmd=...`
- `POST /api/command`

Read-only routes and dry-run routes must remain non-mutating. `dry-run` must
not enqueue or execute commands.

## Secret Hygiene

The repository must not track:

- `.env` files;
- private keys or certificates;
- access tokens;
- generated local reports;
- model weights unless explicitly reviewed;
- runtime state directories.

Run a secret scan before any public release candidate.

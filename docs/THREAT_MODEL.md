# Paradise Threat Model

Scope: local GARM runtime, HTTP API, SDK, validation artifacts and public
repository operation.

## Assets

- Source code and architecture records.
- Runtime state under the configured `--state-dir`.
- Local validation artifacts and readiness packages.
- Operator command history and action evidence.
- Any local credentials or `.env` files, which must not be committed.

## Trust Boundaries

| Boundary | Current stance |
| --- | --- |
| Git repository | Must not contain secrets or generated private runtime state. |
| HTTP API | Localhost-only operator surface. |
| Read APIs | Intended to be non-mutating. |
| Command APIs | Mutation-capable and routed through GEWC command handling. |
| Dry-run API | Must classify only; it must not enqueue or execute. |
| External validation | Local harness only unless explicitly stated otherwise. |

## Primary Risks

1. Accidental secret commit through `.env`, keys, tokens or generated local
   state.
2. Public exposure of `127.0.0.1` command APIs through a proxy or tunnel.
3. Treating local architecture evidence as external AGI validation.
4. Regression where a read-only route mutates state.
5. Regression where dry-run starts queueing or executing commands.
6. Generated reports leaking private paths, local notes or operator data.

## Mitigations

- `.gitignore` blocks common secret, runtime and generated artifact patterns.
- `make eden-api-conformance` checks live read-only surfaces and dry-run
  non-execution.
- `make eden-api-contracts` verifies reproducible artifacts and independent
  package validation.
- `SECURITY.md` directs private vulnerability reporting.
- `docs/CLAIMS_AND_LIMITATIONS.md` defines what cannot be claimed publicly.

## Public Deployment Warning

Do not expose GARM directly to the internet. Before any remote deployment,
design and implement:

- authentication;
- authorization;
- rate limits;
- audit retention policy;
- network isolation;
- command allowlists;
- secret management;
- incident response procedure.

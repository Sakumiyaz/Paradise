# Contributing

Paradise is currently prepared as a public-ready repository. Do not
assume that local validation evidence is an AGI claim.

The project engineering standard lives in
[`docs/EDEN_ENGINEERING_PRACTICES.md`](docs/EDEN_ENGINEERING_PRACTICES.md).
Use that guide as the source of truth for review scope, evidence expectations,
GEWC authority boundaries and claim discipline.

## Development Rules

- Keep changes local-first and reproducible.
- Preserve `claim_allowed=false` and `agi_claim=false` in validation artifacts.
- Do not commit `.env`, credentials, private keys, generated reports or runtime
  state directories.
- Use Rust and Bash for the current runtime. If JavaScript tooling is ever
  introduced, use pnpm only.
- Keep mutation surfaces explicit and governed through GEWC command routing.
- Keep each change focused on one primary purpose. Split broad work by runtime
  layer, contract, API family, validation gate or documentation surface.
- Separate behavior changes from large refactors.

## Local Checks

Run focused checks before opening a PR:

```bash
make fmt
make check
make eden-api-conformance
```

For larger runtime, contract or GEWC changes:

```bash
make test
make eden-api-contracts
make operational-blackbox
```

## Pull Request Expectations

Every PR should include:

- a concise summary;
- validation commands run;
- security or privacy impact;
- documentation updates when public APIs or operational behavior change.
- a clear note when runtime behavior, public contracts or GEWC authority paths
  changed.

## Documentation

Write or update an ADR in `docs/decisions/` when a change creates a durable
architecture, runtime, API or safety decision.

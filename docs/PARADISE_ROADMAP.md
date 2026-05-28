# Paradise Roadmap

Paradise is tracked as a governed runtime first. GPU training remains a separate
lane and cannot turn into a production claim without checkpoint admission.

## Now

- Keep the public runtime installable, testable and no-claim gated.
- Keep GEWC as the authority over models, memory, tools and actions.
- Keep contracts, schemas, OpenAPI files and readiness reports executable.
- Keep dataset provenance explicit and free of private data.
- Keep checkpoint metadata separated from checkpoint weights.
- Keep checkpoint review and inference readiness visible through native
  Paradise CLI commands while admission remains blocked.
- Keep strong non-GPU evaluation and external validation packages reproducible
  without GPU access.

## Next

- Extend checkpoint admission from blocked review to accepted registration only
  after real checkpoint evidence exists.
- Convert more operator-console panels from static docs into local runtime API
  views.
- Add model adapter probes for real checkpoints once GPU runs produce accepted
  evidence.
- Attach schema reports, API conformance, strong eval and external validation
  manifests to future public releases.

## Later

- Run module-specific continued pretraining and SFT for the 70B modular target.
- Replace non-GPU family evidence with held-out learned-model evaluations for
  memory, planning, CWM, VLA, verifier and runtime integration.
- Add reproducible inference adapters for accepted checkpoints only.
- Add external validation results from independent reviewers without sharing
  private data or weights.

## Non-Goals

- No claim that Paradise is completed AGI.
- No committed checkpoint weights.
- No direct model authority over memory, tools, objectives or permissions.
- No hidden dependency on external proprietary models for the Eden training
  path.

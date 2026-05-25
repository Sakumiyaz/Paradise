# Claims And Limitations

Paradise contains substantial runtime architecture, API contracts and local
validation evidence. It is the public local-first operator/runtime surface from
the broader Eden architecture, not the completed Eden system. This document
defines the claim boundary for public review.

## Allowed Claims

It is accurate to say that Paradise currently provides:

- a public local-first Rust workspace centered on the GARM operator runtime;
- a Global Executive Workspace Core coordination design implemented in the
  runtime path;
- a documented layer model connecting the public runtime surface to the broader
  Eden substrate under `eden_core/src/`;
- read-only runtime state, artifact, operational, capability, GEWC, validation
  and action-contract APIs;
- explicit command mutation routes governed through the runtime command path;
- a dry-run endpoint that classifies commands without execution;
- local API conformance evidence;
- reproducible local readiness and package validation artifacts.

## Not Allowed Claims

Do not claim that Paradise is:

- a completed AGI;
- the complete Eden architecture packaged as a finished product;
- externally validated as AGI;
- superior to deployed AI systems by benchmark evidence;
- safe for public network exposure;
- able to learn continuously without limitation;
- trained as a local LMM/foundation model.

## Current Limitations

- Local LMM/model training is not completed.
- Current validation is local engineering evidence, not external certification.
- API conformance validates interoperability and policy markers, not general
  intelligence.
- Runtime action routes are local operator surfaces and should not be exposed
  publicly without authentication, authorization and additional review.
- Some architecture artifacts are formal design evidence rather than fully
  trained model capability.
- Broader Eden modules under `eden_core/src/` have mixed maturity and should not
  be represented as equally production-ready runtime surfaces.

## No-Claim Policy

Generated validation artifacts should preserve:

```text
claim_allowed=false
agi_claim=false
```

Any future change to that policy requires explicit owner review, external
validation context and updated release documentation.

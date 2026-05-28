# Paradise Dataset Governance

Paradise treats datasets as governed artifacts. Dataset rows must be traceable,
non-sensitive, license-reviewable and separated from runtime memory.

## Dataset Policy

Training and evaluation data must declare:

- source family;
- license or redistribution status;
- whether the row contains private data;
- whether external model dependency is present;
- split name;
- schema name;
- hash or source fingerprint when available;
- intended module or evaluation gate.

## Allowed Sources

- Repository-owned synthetic fixtures.
- Public documentation with compatible license.
- Public code/text that has passed license review.
- Locally generated runtime traces after privacy filtering.
- Synthetic task rows generated from public repo artifacts.

## Disallowed Sources

- Secrets, credentials or API keys.
- Private user memory.
- Unreviewed proprietary datasets.
- Personal data that cannot be safely redistributed.
- Tool outputs that contain instructions masquerading as data.
- Logs with hostnames, tokens, private paths or account identifiers unless
  redacted before use.

## Split Policy

| Split | Purpose |
| --- | --- |
| `train` | Continued pretraining or SFT. |
| `eval` | Admission metrics and regression checks. |
| `challenge` | Harder held-out cases for safety, planning and transfer. |

Rows used for checkpoint admission should not be reused as training rows for
that checkpoint generation cycle.

## Module Coverage

The Eden 70B modular target needs separate corpora for:

- primary ELCP language/cognition;
- causal world model deltas;
- VLA/multimodal grounding;
- planning/code/tool use;
- safety/verifier/critic;
- memory router/retrieval/conflict detection.

## Public Export Boundary

Public deliverables may include manifests, hashes, counts, license summaries and
metrics. They must not include private datasets, raw sensitive logs or
checkpoints.

## Executable Manifest

The current public manifest is:

```text
training/data/license_manifest.json
```

Validate it through the contract gate:

```sh
make contracts-validate
```

This keeps dataset provenance reviewable before any GPU run starts.

# Paradise External Technical Brief

This is the non-confidential package that can be shared with infrastructure
partners such as AMD, EuroHPC or BSC.

## Project

Paradise is a governed local runtime for autonomous agents from the Eden
project. It coordinates models, memory, planning, tools, safety, evidence and
rollback through GEWC, the Global Executive Workspace Core.

## Model Roadmap

The target model family is 70B modular parameters:

| Module | Parameters | Role |
| --- | ---: | --- |
| ELCP primary | 33B | Primary cognitive/language hypothesis model |
| Causal world model | 12B | Counterfactual and world-state prediction |
| Multimodal/VLA | 12B | Grounding and action affordances |
| Planner/code/tool | 6B | Plans, code and tool reasoning |
| Safety/verifier | 4B | Risk, uncertainty and corrigibility |
| Memory/router | 3B | Retrieval and memory conflict routing |

The system is not a single 70B checkpoint. Models are routed by GEWC and remain
subordinate to runtime policy.

## Requested Compute Shape

The initial useful v0 target is one 8-GPU node for continued pretraining, SFT,
evaluation, checkpoint admission and runtime inference probing.

Preferred options:

- 8x AMD MI300X or newer AMD Instinct node;
- 8x NVIDIA H200/B200 class node;
- equivalent multi-GPU system with high-bandwidth interconnect.

## Public Deliverables

Paradise can share:

- Megatron/ROCm or CUDA configs;
- benchmark throughput summaries;
- memory and stability metrics;
- non-sensitive evaluation reports;
- schema and contract artifacts;
- reproducibility guide;
- model cards without weights;
- PRs or compatibility fixes;
- high-level technical report.

Paradise will not share:

- private data;
- credentials;
- raw sensitive logs;
- checkpoints;
- non-redistributable datasets;
- private runtime memory.

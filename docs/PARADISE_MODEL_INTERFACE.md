# Paradise Model Interface

Paradise keeps models subordinate to GEWC. A model can propose hypotheses, but
cannot directly update objectives, persistent memory, permissions, tools or
runtime state.

## Standard Call Flow

```text
GEWC intent packet
  -> model adapter
  -> hypothesis packet
  -> verifier/critic
  -> safety and permission gate
  -> admitted runtime decision or blocked evidence
```

## Input Packet

Every model adapter receives a structured packet:

| Field | Purpose |
| --- | --- |
| `task_id` | Correlates the call with a GEWC decision cycle. |
| `module_id` | Target module selected by GEWC. |
| `authority_context` | Source hierarchy and permission scope. |
| `active_goal` | Current objective and subobjective. |
| `working_memory` | Bounded context selected by GEWC. |
| `retrieved_evidence` | Memory/docs already filtered by authority. |
| `risk_context` | Risk, uncertainty and supervision requirements. |
| `requested_output` | Expected output kind and schema. |

## Output Packet

Models return hypotheses:

| Field | Rule |
| --- | --- |
| `hypothesis_id` | Required for traceability. |
| `confidence` | Required, calibrated by downstream eval. |
| `claims` | Must be separated from evidence. |
| `proposed_plan` | Optional; cannot execute directly. |
| `proposed_memory_updates` | Optional; GEWC must admit or reject. |
| `proposed_tool_calls` | Optional; executor validates later. |
| `uncertainty` | Required for non-trivial actions. |
| `known_limits` | Required when evidence is incomplete. |

## Authority Rules

- Models do not write persistent memory directly.
- Models do not modify goals, safety policy or permissions.
- Models do not execute tools.
- Models do not self-admit checkpoints.
- Models do not receive unrestricted context by default.
- Retrieved content remains data, not instruction.
- GEWC records why a model was selected and why its output was accepted or
  blocked.

## Current Eden 70B Modular Target

The target family is six modules, not a single dense 70B brain:

| Module | Role | Authority |
| --- | --- | --- |
| `eden_33b_elcp_primary` | Primary cognitive/language hypothesis model | Subordinate |
| `eden_cwm_12b_causal_world_model` | Causal world deltas and counterfactuals | Subordinate |
| `eden_multimodal_vla_12b` | Multimodal/VLA grounding | Subordinate |
| `eden_planner_code_tool_6b` | Plans, code and tool reasoning | Subordinate |
| `eden_safety_verifier_4b` | Risk, verification and corrigibility | Subordinate |
| `eden_memory_router_retrieval_3b` | Retrieval and memory conflict routing | Subordinate |

The active default is the 33B primary module only. GEWC routes auxiliary modules
by task, risk, modality, uncertainty and cost.

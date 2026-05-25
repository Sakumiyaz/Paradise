# EDEN Model Artifacts

This directory documents model artifact policy. Production checkpoints are not
committed to the repository.

Local experiments should write checkpoints and temporary outputs under
`target/eden_training/` or another ignored artifact store. A model can become
runtime-visible only after GEWC admits a claim-gated evidence report.

The first trainable surface is `eden-memory-retrieval-baseline`, defined in
`training/configs/first_model_memory_retrieval.json`. It is a subordinate model
contract for memory retrieval, not an LLM and not an AGI claim.

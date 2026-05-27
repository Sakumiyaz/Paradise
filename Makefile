.PHONY: fmt test workspace-test doctest external-tests check native-runtime-layout api-socket-test training-smoke training-evidence model-runtime eden-70b-modular-target eden-70b-operationalize first-model-prepare elcp-validate elcp-baseline elcp-trace-export elcp-training-dry-run elcp-admission-gate elcp-trace-quality elcp-replay-eval elcp-dataset-freeze elcp-metrics-board elcp-4b-readiness-contract elcp-hardening elcp-prepare eden-capable eden-capable-operationalize eden-learned-capability eden-real-capability eden-v01-capability eden-v02-stability eden-v03-capability eden-v04-capability paradise-status paradise-worldcell paradise-operational-loop paradise-quickstart paradise-non-gpu-readiness contracts-validate training-dataset-license-manifest paradise-checkpoint-registry-smoke paradise-model-adapter-smoke paradise-release-package paradise-ci-public runtime-spine training-rocm-profile training-megatron-offline-smoke training-megatron-eden-corpus-pilot training-megatron-eden-7b-base-pilot training-megatron-7b-evidence-json training-megatron-7b-evidence training-megatron-7b-inference-probe training-megatron-7b-inference-report-json training-megatron-7b-adapter training-eden-sft-elcp-dataset training-eden-sft-elcp-gpu-pilot training-eden-70b-dataset training-eden-70b-module-pilot training-eden-70b-modular-plan training-eden-real-capability-dataset training-eden-real-capability-eval training-eden-real-capability-stage training-eden-v01-dataset training-eden-v01-semantic-eval training-eden-v01-demo training-eden-v01-gpu-hygiene training-eden-v01-stage training-eden-v02-dataset training-eden-v02-stability-eval training-eden-v02-adversarial-eval training-eden-v02-rollback-drill training-eden-v02-model-card training-eden-v02-demo training-eden-v02-stage training-eden-v03-dataset training-eden-v03-generalization-eval training-eden-v03-demo training-eden-v03-stage training-eden-v04-dataset training-eden-v04-operational-eval training-eden-v04-stage operational-blackbox operational-evidence-bundle operational-demo-suite long-run-stability smoke smoke-api smoke-restart hrm-regression security js-policy public-audit verify readiness readiness-bench eden-probe eden-validate-local eden-api-contracts eden-api-conformance edenctl-doctor eden-openapi-export eden-package eden-independent-validate eden-release-candidate eden-release-check

GARM := cargo run -p eden_core --bin eden-garm --
EDENCTL := cargo run -p eden_core --bin edenctl --
PARADISE := cargo run -p eden_core --bin paradise --
GARM_PACKAGE_VALIDATOR := cargo run -p eden_core --bin eden-garm-package-validator --

fmt:
	cargo fmt --check -p eden_core

test:
	cargo test -p eden_core eden_garm --lib -- --test-threads=1
	cargo test -p eden_core sdk --lib
	cargo test -p eden_core edenctl_cli --lib
	cargo test -p eden_core paradise_cli --lib
	cargo test --bin eden-garm -p eden_core
	cargo test --bin edenctl -p eden_core
	cargo test --bin paradise -p eden_core
	cargo test --example edenctl -p eden_core

workspace-test:
	cargo test --workspace -- --test-threads=1

doctest:
	cargo test -p eden_core --doc

external-tests:
	cargo test -p eden_core --features external-tests test_gpio_export --lib -- --nocapture
	cargo test -p eden_core --features external-tests test_scan --lib -- --nocapture
	cargo test -p eden_core --features external-tests test_crawl_google --lib -- --nocapture

api-socket-test:
	EDEN_API_SOCKET_TESTS=1 cargo test -p eden_core socket_transport_serves_health_when_enabled --lib -- --nocapture

training-smoke:
	python3 training/benchmarks/eden_capability_benchmark.py \
		--dataset training/data/capability_smoke.jsonl \
		--profile training/configs/rocm_smoke.json \
		--model-config training/configs/first_model_memory_retrieval.json \
		--output target/eden_training_smoke/capability_report.json \
		--markdown-output target/eden_training_smoke/capability_report.md
	python3 training/benchmarks/validate_capability_report.py \
		--report target/eden_training_smoke/capability_report.json \
		--schema contracts/v1/schemas/eden-training-capability-report-v1.json

training-evidence: training-smoke
	printf 'training evidence eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_training_evidence --api-port 0

model-runtime: training-smoke
	printf 'model runtime eval\nmodel register eden-memory-retrieval-baseline\nmodel load eden-memory-retrieval-baseline\nmodel evaluate eden-memory-retrieval-baseline\nmodel unload eden-memory-retrieval-baseline\nmodel audit\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_model_runtime --api-port 0

eden-70b-modular-target: model-runtime
	printf 'artifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_model_runtime --api-port 0

training-eden-70b-dataset:
	python3 training/data/build_eden_70b_modular_datasets.py

training-eden-70b-modular-plan: training-eden-70b-dataset
	bash training/rocm/eden_70b_modular_stage.sh

training-eden-70b-module-pilot: training-eden-70b-dataset
	bash training/rocm/megatron_eden_70b_module_pilot.sh

eden-70b-operationalize: training-eden-70b-modular-plan
	printf 'eden 70b modular eval\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_70b_modular --api-port 0

first-model-prepare: training-smoke
	printf 'first model prepare\nfirst model readiness\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_first_model_prepare --api-port 0

elcp-validate:
	python3 training/benchmarks/validate_elcp_transitions.py

elcp-baseline: elcp-validate
	python3 training/benchmarks/elcp_baseline_eval.py

elcp-trace-export:
	rm -rf -- /tmp/eden_garm_elcp_trace
	printf 'elcp prepare\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_elcp_trace --api-port 0
	python3 training/elcp/export_trace_candidates.py --state-dir /tmp/eden_garm_elcp_trace

elcp-training-dry-run: elcp-baseline
	python3 training/elcp/train_elcp.py --dry-run

elcp-admission-gate: elcp-training-dry-run elcp-trace-export
	python3 training/elcp/admission_gate.py

elcp-trace-quality: elcp-trace-export
	python3 training/elcp/trace_quality_gate.py

elcp-replay-eval: elcp-trace-quality
	python3 training/elcp/replay_eval.py

elcp-dataset-freeze: elcp-replay-eval
	python3 training/elcp/dataset_freeze_manifest.py

elcp-metrics-board: elcp-admission-gate elcp-dataset-freeze
	python3 training/elcp/metrics_board.py

elcp-4b-readiness-contract: elcp-metrics-board
	python3 training/elcp/readiness_contract.py

elcp-hardening: elcp-4b-readiness-contract

elcp-prepare: training-smoke elcp-hardening
	printf 'elcp prepare\nelcp hardening\nelcp readiness\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_elcp_prepare --api-port 0

eden-capable:
	printf 'megatron 7b evidence eval\nmegatron 7b adapter prepare\neden capable eval\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_capable --api-port 0

eden-capable-operationalize:
	printf 'megatron 7b evidence eval\nmegatron 7b adapter prepare\neden capable operationalize\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_capable_operational --api-port 0

paradise-status:
	$(PARADISE) --state-dir /tmp/paradise_quickstart status

paradise-worldcell:
	rm -rf -- /tmp/paradise_worldcell
	$(PARADISE) --state-dir /tmp/paradise_worldcell worldcell

paradise-operational-loop:
	rm -rf -- /tmp/paradise_operational_loop
	printf 'paradise worldcell eval\nparadise intent inspect runtime status safely\nparadise plan\nparadise approve\nparadise execute\nparadise sessions\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/paradise_operational_loop --api-port 0

paradise-quickstart:
	rm -rf -- /tmp/paradise_quickstart
	$(PARADISE) --state-dir /tmp/paradise_quickstart status
	$(PARADISE) --state-dir /tmp/paradise_quickstart worldcell
	$(PARADISE) --state-dir /tmp/paradise_quickstart run --dry-run "inspect runtime status safely"

paradise-non-gpu-readiness: training-eden-70b-dataset training-eden-v04-dataset
	python3 training/benchmarks/paradise_non_gpu_readiness.py

contracts-validate:
	python3 scripts/validate_public_contracts.py

training-dataset-license-manifest:
	python3 -m json.tool training/data/license_manifest.json >/dev/null

paradise-checkpoint-registry-smoke:
	python3 -m json.tool training/models/checkpoint_registry.json >/dev/null

paradise-model-adapter-smoke: model-runtime

paradise-release-package: contracts-validate paradise-non-gpu-readiness
	python3 -m json.tool target/public_contracts/validation_report.json >/dev/null
	python3 -m json.tool target/paradise_non_gpu_readiness/non_gpu_readiness_report.json >/dev/null

paradise-ci-public: contracts-validate paradise-non-gpu-readiness check eden-api-conformance

runtime-spine:
	rm -rf -- /tmp/eden_runtime_spine
	printf 'runtime spine eval\nruntime spine enforce\nruntime spine risk\nruntime spine breakers\nruntime spine replay\nruntime spine audit\nruntime spine verify\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_runtime_spine --api-port 0

training-rocm-profile:
	bash training/rocm/rocm_env.sh

training-megatron-offline-smoke:
	bash training/rocm/megatron_offline_smoke.sh

training-megatron-eden-corpus-pilot:
	bash training/rocm/megatron_eden_corpus_pilot.sh

training-megatron-eden-7b-base-pilot:
	bash training/rocm/megatron_eden_7b_base_pilot.sh

training-megatron-7b-evidence-json:
	python3 training/rocm/build_megatron_7b_evidence.py \
		--repo-root . \
		--output-dir target/eden_megatron_7b_base_pilot \
		--schema contracts/v1/schemas/eden-megatron-7b-training-evidence-v1.json

training-megatron-7b-evidence: training-megatron-7b-evidence-json
	@command -v cargo >/dev/null 2>&1 || { \
		printf 'cargo is required for GEWC admission; JSON evidence was generated. Install Rust or copy target/eden_megatron_7b_base_pilot/eden_7b_training_evidence.json to a runtime host.\n' >&2; \
		exit 127; \
	}
	printf 'megatron 7b evidence eval\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_megatron_7b_evidence --api-port 0

training-megatron-7b-inference-probe:
	bash training/rocm/megatron_eden_7b_inference_probe.sh

training-megatron-7b-inference-report-json:
	python3 training/rocm/build_megatron_7b_inference_report.py \
		--output-dir target/eden_megatron_7b_base_pilot \
		--schema contracts/v1/schemas/eden-megatron-7b-inference-report-v1.json

training-megatron-7b-adapter: training-megatron-7b-evidence-json training-megatron-7b-inference-report-json
	@command -v cargo >/dev/null 2>&1 || { \
		printf 'cargo is required for GEWC admission; JSON inference report was generated. Install Rust or copy target/eden_megatron_7b_base_pilot/eden_7b_inference_report.json to a runtime host.\n' >&2; \
		exit 127; \
	}
	printf 'megatron 7b evidence eval\nmegatron 7b adapter prepare\nmegatron 7b inference eval\nmegatron 7b capability eval\nmegatron 7b admission gate eval\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_megatron_7b_adapter --api-port 0

training-eden-sft-elcp-dataset:
	python3 training/data/build_eden_cognitive_sft_elcp.py
	python3 training/benchmarks/validate_elcp_transitions.py \
		--train training/data/eden_cognitive_sft_elcp_train.jsonl \
		--eval training/data/eden_cognitive_sft_elcp_eval.jsonl \
		--output target/eden_cognitive_sft_elcp/validation_report.json

training-eden-sft-elcp-gpu-pilot: training-eden-sft-elcp-dataset
	bash training/rocm/eden_sft_elcp_gpu_pilot.sh

eden-learned-capability:
	printf 'eden learned capability eval\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_learned_capability --api-port 0

training-eden-real-capability-dataset:
	python3 training/data/build_eden_real_capability_corpus.py

training-eden-real-capability-eval: training-eden-real-capability-dataset
	python3 training/benchmarks/eden_real_capability_eval.py

training-eden-real-capability-stage:
	bash training/rocm/eden_real_capability_stage.sh

eden-real-capability: training-eden-real-capability-eval
	printf 'eden real capability eval\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_real_capability --api-port 0

training-eden-v01-dataset:
	python3 training/data/build_eden_v01_semantic_corpus.py

training-eden-v01-semantic-eval: training-eden-v01-dataset
	python3 training/benchmarks/eden_v01_semantic_eval.py

training-eden-v01-demo: training-eden-v01-semantic-eval
	python3 training/demos/eden_v01_operational_demo.py

training-eden-v01-gpu-hygiene:
	bash training/rocm/eden_gpu_workspace_hygiene.sh

training-eden-v01-stage:
	bash training/rocm/eden_v01_capability_stage.sh

eden-v01-capability: training-eden-v01-stage
	printf 'eden v01 capability eval\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_v01_capability --api-port 0

training-eden-v02-dataset:
	python3 training/data/build_eden_v02_stability_corpus.py

training-eden-v02-stability-eval: training-eden-v02-dataset
	python3 training/benchmarks/eden_v02_stability_eval.py

training-eden-v02-adversarial-eval: training-eden-v02-stability-eval
	python3 training/benchmarks/eden_v02_adversarial_eval.py

training-eden-v02-rollback-drill: training-eden-v02-adversarial-eval
	python3 training/benchmarks/eden_v02_rollback_drill.py

training-eden-v02-model-card: training-eden-v02-rollback-drill
	python3 training/benchmarks/eden_v02_model_card.py

training-eden-v02-demo: training-eden-v02-model-card
	python3 training/demos/eden_v02_stability_demo.py

training-eden-v02-stage:
	bash training/rocm/eden_v02_stability_stage.sh

eden-v02-stability: training-eden-v02-stage
	printf 'eden v02 stability eval\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_v02_stability --api-port 0

training-eden-v03-dataset:
	python3 training/data/build_eden_v03_generalization_corpus.py

training-eden-v03-generalization-eval: training-eden-v03-dataset
	python3 training/benchmarks/eden_v03_generalization_eval.py

training-eden-v03-demo: training-eden-v03-generalization-eval
	python3 training/demos/eden_v03_operational_demo.py

training-eden-v03-stage:
	bash training/rocm/eden_v03_capability_stage.sh

eden-v03-capability: training-eden-v03-stage
	printf 'eden v03 capability eval\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_v03_capability --api-port 0

training-eden-v04-dataset:
	python3 training/data/build_eden_v04_cognitive_capability_corpus.py

training-eden-v04-operational-eval: training-eden-v04-dataset
	python3 training/benchmarks/eden_v04_operational_capability_eval.py

training-eden-v04-stage:
	bash training/rocm/eden_v04_capability_stage.sh

eden-v04-capability: training-eden-v04-stage
	printf 'eden v04 capability eval\nartifact api eval\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_v04_capability --api-port 0

operational-blackbox:
	bash eden_core/src/garm/scripts/operational_blackbox.sh

operational-evidence-bundle:
	bash eden_core/src/garm/scripts/operational_evidence_bundle.sh

operational-demo-suite:
	printf 'operational api eval\noperational runtime eval\noperational demo run\nquit\n' | $(GARM) --state-dir /tmp/eden_garm_operational_demo

long-run-stability:
	bash eden_core/src/garm/scripts/long_run_stability_gate.sh

check:
	cargo check -p eden_core --examples --bins

native-runtime-layout:
	bash scripts/native_runtime_layout_check.sh

smoke: smoke-api smoke-restart hrm-regression

smoke-api:
	bash eden_core/src/garm/scripts/smoke_api.sh

smoke-restart:
	bash eden_core/src/garm/scripts/smoke_restart_persistence.sh

hrm-regression:
	bash eden_core/src/garm/scripts/hrm_retrieval_regression.sh

security:
	cargo deny check advisories
	cargo audit

js-policy:
	@if git ls-files | grep -E '(^|/)(package-lock\.json|npm-shrinkwrap\.json|\.npmrc|\.npm/|\.npm-global/)'; then \
		printf 'forbidden JavaScript package-manager artifact found; use pnpm policy only\n' >&2; \
		exit 1; \
	fi

public-audit:
	bash scripts/public_release_audit.sh

verify: js-policy
	bash eden_core/src/garm/scripts/verify.sh

readiness:
	printf 'readiness\nquit\n' | $(GARM) --state-dir /tmp/eden_garm_make_readiness

readiness-bench:
	printf 'readiness bench\nquit\n' | $(GARM) --state-dir /tmp/eden_garm_make_readiness

eden-probe:
	printf 'readiness probe\nquit\n' | $(GARM) --state-dir /tmp/eden_garm_make_validation

eden-validate-local: elcp-hardening
	rm -rf -- /tmp/eden_garm_make_validation
	printf 'readiness probe\nmemory eval\nworld eval\ncognitive eval\nembodied eval\nneural eval\nsymbolic eval\nself improvement eval\nfrontier architecture eval\nparadigm architecture eval\nintegration governance eval\nglobal executive workspace eval\ngewc operational benchmark\ncapability reality eval\narchitecture advantage eval\nparadise worldcell eval\nparadise intent inspect runtime status safely\nparadise plan\nparadise approve\nparadise execute\nparadise sessions\nruntime spine eval\nruntime spine enforce\nruntime spine risk\nruntime spine breakers\nruntime spine replay\nruntime spine audit\nruntime spine verify\npraxis nexus eval\nlocus eval\nlocus ingest operator preference :: keep EDEN local-first and no-claim gated\nlocus context operator permission boundary\noperator forge eval\noperator forge synth causal risk model for governed action under uncertainty\noperator forge verify\noperational runtime eval\nexternal ecosystem eval\nsovereign cognition eval\noperational api eval\nruntime state api eval\nreadiness external run\ncapabilities audit\nartifact api eval\ntraining evidence eval\nmodel runtime eval\nfirst model prepare\nelcp prepare\nelcp hardening\nelcp readiness\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_make_validation

eden-api-contracts: elcp-hardening
	rm -rf -- /tmp/eden_garm_api_contracts
	printf 'readiness probe\nlocus eval\nlocus ingest operator preference :: keep EDEN local-first and no-claim gated\nlocus context operator permission boundary\noperator forge eval\noperator forge synth causal risk model for governed action under uncertainty\noperator forge verify\nparadise worldcell eval\nparadise intent inspect runtime status safely\nparadise plan\nparadise approve\nparadise execute\nparadise sessions\nruntime spine eval\nruntime spine enforce\nruntime spine risk\nruntime spine breakers\nruntime spine replay\nruntime spine audit\nruntime spine verify\noperational runtime eval\nmodel runtime eval\nfirst model prepare\nelcp prepare\nelcp hardening\noperational api eval\nruntime state api eval\nreadiness external run\ncapabilities audit\nartifact api eval\nreadiness package\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_api_contracts
	$(GARM_PACKAGE_VALIDATOR) --state-dir /tmp/eden_garm_api_contracts

eden-api-conformance:
	EDEN_GARM_SKIP_LEGACY_MIGRATION=1 bash eden_core/src/garm/scripts/conformance_api.sh

edenctl-doctor:
	$(EDENCTL) doctor

eden-openapi-export:
	$(EDENCTL) openapi export --output-dir contracts/v1/openapi

eden-package:
	printf 'readiness probe\nreadiness package\nquit\n' | EDEN_GARM_SKIP_LEGACY_MIGRATION=1 $(GARM) --state-dir /tmp/eden_garm_make_validation

eden-independent-validate:
	$(GARM_PACKAGE_VALIDATOR) --state-dir /tmp/eden_garm_make_validation

eden-release-candidate: public-audit eden-validate-local eden-package eden-independent-validate eden-api-contracts eden-api-conformance operational-blackbox long-run-stability

eden-release-check: fmt check native-runtime-layout test doctest training-smoke training-evidence model-runtime first-model-prepare elcp-prepare security smoke js-policy eden-release-candidate

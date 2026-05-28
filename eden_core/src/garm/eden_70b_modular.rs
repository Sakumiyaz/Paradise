use crate::eden_garm::{model_runtime, state_paths};
use serde_json::Value;

pub const EDEN_70B_ROUTER_SCHEMA: &str = "eden.modular_70b.router.v1";
pub const EDEN_70B_DATASET_MANIFEST_SCHEMA: &str = "eden.modular_70b.dataset_manifest.v1";
pub const EDEN_70B_LAUNCHER_MANIFEST_SCHEMA: &str = "eden.modular_70b.launcher_manifest.v1";
pub const EDEN_70B_CHECKPOINT_ADMISSION_SCHEMA: &str = "eden.modular_70b.checkpoint_admission.v1";
pub const EDEN_70B_INFERENCE_RUNTIME_SCHEMA: &str = "eden.modular_70b.inference_runtime.v1";
pub const EDEN_70B_OPERATIONAL_DEMO_SCHEMA: &str = "eden.modular_70b.operational_demo.v1";
pub const EDEN_70B_OPERATIONAL_GATE_SCHEMA: &str = "eden.modular_70b.operational_gate.v1";

const AUTHORITY: &str = "global_executive_workspace_core";
const TRAIN_DATA: &str = "training/data/eden_70b_modular_train.jsonl";
const EVAL_DATA: &str = "training/data/eden_70b_modular_eval.jsonl";
const CHALLENGE_DATA: &str = "training/data/eden_70b_modular_challenge.jsonl";
const DATA_BUILDER: &str = "training/data/build_eden_70b_modular_datasets.py";
const ROCM_LAUNCHER: &str = "training/rocm/eden_70b_modular_stage.sh";

#[derive(Clone, Copy)]
struct ModuleSpec {
    id: &'static str,
    role: &'static str,
    parameters: u64,
    route_when: &'static str,
    objective: &'static str,
}

const MODULES: [ModuleSpec; 6] = [
    ModuleSpec {
        id: "eden_33b_elcp_primary",
        role: "primary_cognitive_model",
        parameters: 33_000_000_000,
        route_when: "general_language_reasoning_or_structured_hypothesis_needed",
        objective: "eden_latent_cognitive_prediction",
    },
    ModuleSpec {
        id: "eden_cwm_12b_causal_world_model",
        role: "world_model",
        parameters: 12_000_000_000,
        route_when: "counterfactual_or_consequence_simulation_needed",
        objective: "causal_world_delta_prediction",
    },
    ModuleSpec {
        id: "eden_multimodal_vla_12b",
        role: "multimodal_grounding",
        parameters: 12_000_000_000,
        route_when: "vision_audio_spatial_or_embodied_context_present",
        objective: "perception_to_concept_and_vla_grounding",
    },
    ModuleSpec {
        id: "eden_planner_code_tool_6b",
        role: "planning_code_tools",
        parameters: 6_000_000_000,
        route_when: "hierarchical_plan_code_or_tool_contract_required",
        objective: "plan_program_and_tool_contract_synthesis",
    },
    ModuleSpec {
        id: "eden_safety_verifier_4b",
        role: "safety_verifier_critic",
        parameters: 4_000_000_000,
        route_when: "state_change_action_claim_or_checkpoint_admission_requested",
        objective: "risk_uncertainty_policy_and_corrigibility_review",
    },
    ModuleSpec {
        id: "eden_memory_router_retrieval_3b",
        role: "memory_router_retrieval",
        parameters: 3_000_000_000,
        route_when: "memory_context_retrieval_or_conflict_resolution_needed",
        objective: "hybrid_retrieval_ranking_and_memory_conflict_detection",
    },
];

pub fn run_all() -> String {
    let mut out = String::new();
    out.push_str(&model_runtime::write_eden_70b_modular_target());
    out.push_str(&write_router());
    out.push_str(&write_dataset_manifest());
    out.push_str(&write_launcher_manifest());
    out.push_str(&write_checkpoint_admission());
    out.push_str(&write_inference_runtime());
    out.push_str(&write_operational_demo());
    out.push_str(&write_operational_gate());
    out
}

pub fn write_router() -> String {
    write_report(
        "EDEN-70B-ROUTER",
        EDEN_70B_ROUTER_SCHEMA,
        state_paths::eden_70b_module_router_path(),
        router_value(),
    )
}

pub fn write_dataset_manifest() -> String {
    write_report(
        "EDEN-70B-DATASETS",
        EDEN_70B_DATASET_MANIFEST_SCHEMA,
        state_paths::eden_70b_dataset_manifest_path(),
        dataset_manifest_value(),
    )
}

pub fn write_launcher_manifest() -> String {
    write_report(
        "EDEN-70B-LAUNCHERS",
        EDEN_70B_LAUNCHER_MANIFEST_SCHEMA,
        state_paths::eden_70b_launcher_manifest_path(),
        launcher_manifest_value(),
    )
}

pub fn write_checkpoint_admission() -> String {
    write_report(
        "EDEN-70B-CHECKPOINT-ADMISSION",
        EDEN_70B_CHECKPOINT_ADMISSION_SCHEMA,
        state_paths::eden_70b_checkpoint_admission_path(),
        checkpoint_admission_value(),
    )
}

pub fn write_inference_runtime() -> String {
    write_report(
        "EDEN-70B-INFERENCE-RUNTIME",
        EDEN_70B_INFERENCE_RUNTIME_SCHEMA,
        state_paths::eden_70b_inference_runtime_path(),
        inference_runtime_value(),
    )
}

pub fn write_operational_demo() -> String {
    write_report(
        "EDEN-70B-DEMO",
        EDEN_70B_OPERATIONAL_DEMO_SCHEMA,
        state_paths::eden_70b_operational_demo_path(),
        operational_demo_value(),
    )
}

pub fn write_operational_gate() -> String {
    write_report(
        "EDEN-70B-GATE",
        EDEN_70B_OPERATIONAL_GATE_SCHEMA,
        state_paths::eden_70b_operational_gate_path(),
        operational_gate_value(),
    )
}

fn router_value() -> Value {
    serde_json::json!({
        "schema": EDEN_70B_ROUTER_SCHEMA,
        "artifact": "eden_70b_module_router",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "not_a_single_model": true,
        "total_parameters": total_parameters(),
        "active_default_parameters": 33_000_000_000u64,
        "routing_policy": {
            "gewc_final_authority": true,
            "route_by": ["task", "risk", "modality", "uncertainty", "cost", "permission"],
            "all_modules_loaded_for_every_request": false,
            "model_outputs_are_hypotheses": true,
            "direct_memory_writes": false,
            "direct_objective_writes": false,
            "direct_tool_execution": false
        },
        "modules": module_records(),
        "sample_routes": [
            route("repo_cleanup_with_risk", ["eden_memory_router_retrieval_3b", "eden_planner_code_tool_6b", "eden_safety_verifier_4b", "eden_33b_elcp_primary"]),
            route("counterfactual_action_review", ["eden_cwm_12b_causal_world_model", "eden_safety_verifier_4b", "eden_33b_elcp_primary"]),
            route("multimodal_robotics_context", ["eden_multimodal_vla_12b", "eden_cwm_12b_causal_world_model", "eden_safety_verifier_4b"])
        ]
    })
}

fn dataset_manifest_value() -> Value {
    let train_present = repo_file_exists(TRAIN_DATA);
    let eval_present = repo_file_exists(EVAL_DATA);
    let challenge_present = repo_file_exists(CHALLENGE_DATA);
    serde_json::json!({
        "schema": EDEN_70B_DATASET_MANIFEST_SCHEMA,
        "artifact": "eden_70b_dataset_manifest",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "contains_private_data": false,
        "external_model_dependency": false,
        "dataset_ready": train_present && eval_present && challenge_present,
        "builder": DATA_BUILDER,
        "splits": {
            "train": {"path": TRAIN_DATA, "present": train_present},
            "eval": {"path": EVAL_DATA, "present": eval_present},
            "challenge": {"path": CHALLENGE_DATA, "present": challenge_present}
        },
        "module_coverage": MODULES.iter().map(|module| serde_json::json!({
            "module_id": module.id,
            "role": module.role,
            "objective": module.objective,
            "required_record_fields": [
                "module_id",
                "input",
                "target",
                "governance",
                "evidence"
            ]
        })).collect::<Vec<_>>(),
        "admission_policy": {
            "training_allowed": false,
            "requires_license_review": true,
            "requires_privacy_review": true,
            "requires_dataset_freeze": true,
            "requires_per_module_eval": true
        }
    })
}

fn launcher_manifest_value() -> Value {
    let launcher_present = repo_file_exists(ROCM_LAUNCHER);
    serde_json::json!({
        "schema": EDEN_70B_LAUNCHER_MANIFEST_SCHEMA,
        "artifact": "eden_70b_launcher_manifest",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "launcher": ROCM_LAUNCHER,
        "launcher_present": launcher_present,
        "training_executed": false,
        "network_policy": "offline_by_default",
        "external_model_dependency": false,
        "default_mode": "plan_only_no_gpu_training",
        "modules": MODULES.iter().map(|module| serde_json::json!({
            "module_id": module.id,
            "parameters": module.parameters,
            "rocm_profile": "MI300X_Megatron_module_specific",
            "checkpoint_output_policy": "target_generated_weights_only_never_git",
            "requires_explicit_operator_approval": true,
            "requires_module_dataset_manifest": true
        })).collect::<Vec<_>>()
    })
}

fn checkpoint_admission_value() -> Value {
    serde_json::json!({
        "schema": EDEN_70B_CHECKPOINT_ADMISSION_SCHEMA,
        "artifact": "eden_70b_checkpoint_admission",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "checkpoint_admission_allowed": false,
        "production_release_allowed": false,
        "module_admissions": MODULES.iter().map(|module| serde_json::json!({
            "module_id": module.id,
            "parameters": module.parameters,
            "admitted": false,
            "checkpoint_loaded": false,
            "required_before_admission": [
                "checkpoint_hash",
                "training_evidence",
                "held_out_eval",
                "safety_eval",
                "rollback_manifest",
                "GEWC_review"
            ]
        })).collect::<Vec<_>>(),
        "global_requirements": [
            "all_modules_remain_subordinate_to_GEWC",
            "model_outputs_are_hypotheses",
            "no_direct_tool_memory_or_objective_authority",
            "per_module_rollback_available",
            "cross_module_regression_eval_passed"
        ]
    })
}

fn inference_runtime_value() -> Value {
    serde_json::json!({
        "schema": EDEN_70B_INFERENCE_RUNTIME_SCHEMA,
        "artifact": "eden_70b_inference_runtime",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "runtime_status": "contract_ready_waiting_for_admitted_checkpoints",
        "checkpoint_loaded": false,
        "real_checkpoint_inference_available": false,
        "not_a_single_model": true,
        "request_contract": {
            "input": ["task", "context", "risk", "modality", "permission_scope"],
            "router_output": ["selected_modules", "reason", "required_verifiers"],
            "module_output": "untrusted_hypothesis_packet",
            "GEWC_output": "accepted_or_rejected_runtime_decision"
        },
        "sample_transaction": {
            "task": "plan a governed repo maintenance action",
            "selected_modules": [
                "eden_memory_router_retrieval_3b",
                "eden_planner_code_tool_6b",
                "eden_safety_verifier_4b",
                "eden_33b_elcp_primary"
            ],
            "state_mutation": "blocked_until_GEWC_accepts_hypothesis",
            "tool_execution": false
        }
    })
}

fn operational_demo_value() -> Value {
    serde_json::json!({
        "schema": EDEN_70B_OPERATIONAL_DEMO_SCHEMA,
        "artifact": "eden_70b_operational_demo",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "demo_id": "eden_70b_modular_governed_route_demo",
        "demo_type": "non_mutating_runtime_trace",
        "checkpoint_required": false,
        "steps": [
            step(1, "GEWC receives task", "working memory creates task frame"),
            step(2, "memory router selected", "retrieve candidate evidence without writing memory"),
            step(3, "planner-code-tool selected", "draft action contract and rollback needs"),
            step(4, "CWM selected when consequence simulation is needed", "produce causal hypothesis only"),
            step(5, "safety verifier selected", "block or require approval before action"),
            step(6, "33B primary selected", "compose governed hypothesis packet"),
            step(7, "GEWC decides", "accept, reject, ask user or defer; models do not decide")
        ],
        "result": {
            "action_executed": false,
            "memory_written": false,
            "objective_changed": false,
            "evidence_written": true
        }
    })
}

fn operational_gate_value() -> Value {
    let required = [
        state_paths::eden_70b_modular_target_path(),
        state_paths::eden_70b_module_router_path(),
        state_paths::eden_70b_dataset_manifest_path(),
        state_paths::eden_70b_launcher_manifest_path(),
        state_paths::eden_70b_checkpoint_admission_path(),
        state_paths::eden_70b_inference_runtime_path(),
        state_paths::eden_70b_operational_demo_path(),
    ];
    let present = required
        .iter()
        .filter(|path| std::fs::metadata(path).is_ok())
        .count();
    serde_json::json!({
        "schema": EDEN_70B_OPERATIONAL_GATE_SCHEMA,
        "artifact": "eden_70b_operational_gate",
        "authority": AUTHORITY,
        "claim_allowed": false,
        "agi_claim": false,
        "surface_ready": present == required.len(),
        "passed": present,
        "total": required.len(),
        "training_allowed": false,
        "checkpoint_admission_allowed": false,
        "production_release_allowed": false,
        "what_is_ready": [
            "GEWC module router contract",
            "module dataset manifest",
            "ROCm launcher manifest",
            "checkpoint admission gate",
            "inference runtime contract",
            "non-mutating operational demo"
        ],
        "what_is_not_ready": [
            "actual 70B modular checkpoints",
            "production inference",
            "AGI claim",
            "autonomous action authority"
        ]
    })
}

fn module_records() -> Vec<Value> {
    MODULES
        .iter()
        .map(|module| {
            serde_json::json!({
                "id": module.id,
                "role": module.role,
                "parameters": module.parameters,
                "route_when": module.route_when,
                "objective": module.objective,
                "authority": AUTHORITY,
                "direct_tool_execution": false,
                "direct_memory_write": false,
                "direct_objective_update": false
            })
        })
        .collect()
}

fn route<const N: usize>(task: &str, modules: [&str; N]) -> Value {
    serde_json::json!({
        "task": task,
        "selected_modules": modules.to_vec(),
        "GEWC_decision_required": true
    })
}

fn step(index: u64, event: &str, evidence: &str) -> Value {
    serde_json::json!({
        "step": index,
        "event": event,
        "evidence": evidence,
        "authority": AUTHORITY
    })
}

fn total_parameters() -> u64 {
    MODULES.iter().map(|module| module.parameters).sum()
}

fn repo_file_exists(path: &str) -> bool {
    std::path::Path::new(path).is_file()
}

fn write_report(tag: &str, schema: &str, path: String, record: Value) -> String {
    match write_json(&path, &record) {
        Ok(()) => format!(
            "[{}] schema={} status=written authority={} claim_allowed=false agi_claim=false path={}\n",
            tag, schema, AUTHORITY, path
        ),
        Err(err) => format!(
            "[{}] schema={} status=error authority={} claim_allowed=false agi_claim=false reason={}\n",
            tag, schema, AUTHORITY, err
        ),
    }
}

fn write_json(path: &str, record: &Value) -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let body = serde_json::to_string_pretty(record).map_err(|e| e.to_string())?;
    std::fs::write(path, body).map_err(|e| format!("failed to write {}: {}", path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modular_70b_operationalization_writes_seven_no_claim_artifacts() {
        let _guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_70b_modular_operational_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(&dir);

        let out = run_all();
        let gate = std::fs::read_to_string(state_paths::eden_70b_operational_gate_path()).unwrap();
        let parsed: Value = serde_json::from_str(&gate).unwrap();

        assert!(out.contains("[EDEN-70B-ROUTER]"));
        assert!(out.contains("[EDEN-70B-GATE]"));
        assert_eq!(parsed["schema"], EDEN_70B_OPERATIONAL_GATE_SCHEMA);
        assert_eq!(parsed["claim_allowed"], false);
        assert_eq!(parsed["agi_claim"], false);
        assert_eq!(parsed["surface_ready"], true);
        assert_eq!(parsed["training_allowed"], false);
        assert_eq!(parsed["checkpoint_admission_allowed"], false);

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }

    #[test]
    fn router_keeps_total_budget_modular_and_gewc_routed() {
        let record = router_value();
        let modules = record["modules"].as_array().unwrap();
        let total: u64 = modules
            .iter()
            .map(|module| module["parameters"].as_u64().unwrap())
            .sum();

        assert_eq!(record["not_a_single_model"], true);
        assert_eq!(total, 70_000_000_000);
        assert_eq!(record["routing_policy"]["gewc_final_authority"], true);
        assert_eq!(
            record["routing_policy"]["all_modules_loaded_for_every_request"],
            false
        );
    }
}

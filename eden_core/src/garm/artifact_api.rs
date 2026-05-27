use crate::eden_garm::{reproducible_package, state_paths};

pub fn run() -> String {
    let specs = reproducible_package::artifact_specs();
    let contracts = contracts_value(&specs);
    write_json(state_paths::artifact_api_contracts_path(), contracts);
    write_json(
        state_paths::artifact_api_catalog_path(),
        catalog_value(&specs),
    );
    write_json(
        state_paths::artifact_api_runtime_path(),
        serde_json::json!({"schema": "eden-artifact-api-runtime-v1"}),
    );

    let catalog = catalog_value(&specs);
    let runtime = runtime_value(&specs);
    let present = catalog
        .get("present")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let total = catalog
        .get("total")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    write_json(state_paths::artifact_api_catalog_path(), catalog);
    write_json(state_paths::artifact_api_runtime_path(), runtime);

    format!(
        "[ARTIFACT-API] artifacts={}/{} endpoints=3 claim_allowed=false path={}\n[ARTIFACT-API-CONTRACTS] path={}\n[ARTIFACT-API-RUNTIME] path={}\n",
        present,
        total,
        state_paths::artifact_api_catalog_path(),
        state_paths::artifact_api_contracts_path(),
        state_paths::artifact_api_runtime_path()
    )
}

pub fn catalog_json() -> String {
    std::fs::read_to_string(state_paths::artifact_api_catalog_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&catalog_value(&reproducible_package::artifact_specs()))
            .unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn runtime_json() -> String {
    std::fs::read_to_string(state_paths::artifact_api_runtime_path()).unwrap_or_else(|_| {
        serde_json::to_string_pretty(&runtime_value(&reproducible_package::artifact_specs()))
            .unwrap_or_else(|_| "{}".to_string())
    })
}

pub fn read_artifact(name: &str) -> Option<(String, &'static str)> {
    let spec = reproducible_package::artifact_specs()
        .into_iter()
        .find(|artifact| artifact.name == name)?;
    let body = std::fs::read_to_string(&spec.path).ok()?;
    Some((body, content_type_for(&spec.path)))
}

fn catalog_value(specs: &[reproducible_package::ReproducibleArtifactSpec]) -> serde_json::Value {
    let records: Vec<_> = specs.iter().map(record_for).collect();
    let present = records
        .iter()
        .filter(|record| record.get("present").and_then(serde_json::Value::as_bool) == Some(true))
        .count();
    serde_json::json!({
        "schema": "eden-artifact-api-catalog-v1",
        "artifact": "artifact_api_catalog",
        "claim_allowed": false,
        "agi_claim": false,
        "purpose": "Expose every release artifact through executable read, inspect, validate and generation contracts.",
        "base_routes": [
            "/api/artifact_catalog",
            "/api/artifact/catalog",
            "/api/artifact?name=<artifact_name>",
            "/api/artifact/runtime"
        ],
        "present": present,
        "total": records.len(),
        "records": records,
    })
}

fn contracts_value(specs: &[reproducible_package::ReproducibleArtifactSpec]) -> serde_json::Value {
    let contracts: Vec<_> = specs
        .iter()
        .map(|spec| {
            serde_json::json!({
                "artifact": spec.name,
                "api": {
                    "read": format!("/api/artifact?name={}", spec.name),
                    "inspect": "/api/artifact/catalog",
                    "validate": "make eden-independent-validate",
                    "generate": generator_command(spec.name),
                },
                "safety_contract": {
                    "read_only": true,
                    "path_whitelist": "reproducible_package_artifact_specs",
                    "claim_allowed": false,
                    "agi_claim": false
                },
                "domain": domain_for(spec.name),
            })
        })
        .collect();
    serde_json::json!({
        "schema": "eden-artifact-api-contracts-v1",
        "artifact": "artifact_api_contracts",
        "claim_allowed": false,
        "agi_claim": false,
        "contracts": contracts,
    })
}

fn runtime_value(specs: &[reproducible_package::ReproducibleArtifactSpec]) -> serde_json::Value {
    let mut present = 0usize;
    let mut missing = 0usize;
    let mut total_bytes = 0usize;
    for spec in specs {
        let bytes = std::fs::read(&spec.path).unwrap_or_default();
        if bytes.is_empty() {
            missing += 1;
        } else {
            present += 1;
            total_bytes += bytes.len();
        }
    }
    serde_json::json!({
        "schema": "eden-artifact-api-runtime-v1",
        "artifact": "artifact_api_runtime",
        "claim_allowed": false,
        "agi_claim": false,
        "present": present,
        "missing": missing,
        "total": specs.len(),
        "total_bytes": total_bytes,
        "routes": [
            {"method": "GET", "path": "/api/artifact/catalog", "effect": "read_catalog"},
            {"method": "GET", "path": "/api/artifact?name=<artifact_name>", "effect": "read_whitelisted_artifact"},
            {"method": "GET", "path": "/api/artifact/runtime", "effect": "read_runtime_status"}
        ],
        "write_policy": "artifact APIs are read-only over whitelisted state paths; generation remains via GEWC commands",
    })
}

fn record_for(spec: &reproducible_package::ReproducibleArtifactSpec) -> serde_json::Value {
    let bytes = std::fs::read(&spec.path).unwrap_or_default();
    serde_json::json!({
        "name": spec.name,
        "path": spec.path.as_str(),
        "present": !bytes.is_empty(),
        "bytes": bytes.len(),
        "fnv64": format!("{:016x}", fnv64(&bytes)),
        "domain": domain_for(spec.name),
        "content_type": content_type_for(&spec.path),
        "read_endpoint": format!("/api/artifact?name={}", spec.name),
        "generator_command": generator_command(spec.name),
    })
}

fn domain_for(name: &str) -> &'static str {
    if name.starts_with("runtime_") && !name.starts_with("runtime_state_api") {
        "runtime_spine"
    } else if name.starts_with("paradise_worldcell") {
        "paradise_worldcell"
    } else if name.contains("praxis") || name.contains("cognitive_contract") {
        "praxis_formal_substrate"
    } else if name == "locus_operator_bridge" {
        "operational_runtime"
    } else if name.starts_with("locus") || name == "eden_locus_layer" {
        "locus_context_authority"
    } else if name.starts_with("operator") || name == "eden_operator_forge" {
        "formal_synthesis"
    } else if name.contains("ecosystem") {
        "external_ecosystem"
    } else if name.contains("sovereign")
        || name.contains("fabric")
        || name.contains("symbolic_reasoning")
    {
        "sovereign_cognition"
    } else if name.contains("gewc") || name.contains("global_executive") {
        "global_executive_workspace"
    } else if name.starts_with("eden_sft_elcp_dataset_v2")
        || name.starts_with("eden_sft_elcp_gpu")
        || name.starts_with("eden_sft_elcp_prepost")
        || name.starts_with("eden_sft_elcp_repeated")
        || name.starts_with("eden_sft_elcp_checkpoint")
        || name.starts_with("eden_sft_elcp_operational")
        || name.starts_with("eden_external_tests")
        || name.starts_with("eden_learned_capability")
        || name.starts_with("eden_real_capability")
        || name.starts_with("eden_v01")
        || name.starts_with("eden_v02")
        || name.starts_with("eden_v03")
        || name.starts_with("eden_v04")
    {
        if name.starts_with("eden_v04") {
            "eden_v04_capability"
        } else if name.starts_with("eden_v03") {
            "eden_v03_capability"
        } else if name.starts_with("eden_v02") {
            "eden_v02_stability"
        } else if name.starts_with("eden_v01") {
            "eden_v01_capability"
        } else if name.starts_with("eden_real_capability") {
            "eden_real_capability"
        } else {
            "eden_learned_capability"
        }
    } else if name.starts_with("eden_capable")
        || name.starts_with("eden_capability")
        || name.starts_with("eden_cognitive")
        || name.starts_with("eden_native")
        || name.starts_with("eden_structured")
        || name.starts_with("eden_checkpoint")
        || name.starts_with("eden_live")
        || name.starts_with("eden_memory")
        || name.starts_with("eden_sft")
    {
        "eden_capable_runtime"
    } else if name.starts_with("megatron_7b") {
        "model_runtime_governance"
    } else if name.starts_with("model_")
        || name.starts_with("first_model")
        || name.starts_with("elcp")
        || name == "training_harness_report"
    {
        "model_runtime_governance"
    } else if name.contains("training") {
        "training_evidence"
    } else if name.contains("capability") || name.contains("lmm") {
        "capability_reality"
    } else if name.starts_with("operational_api")
        || name == "operational_action_contracts"
        || name == "operational_contract"
        || name == "operational_permissions"
        || name == "operational_permissions_audit"
        || name == "operational_permissions_diff"
        || name == "operational_permissions_history"
        || name == "operational_recovery_plan"
        || name == "operational_demo_suite"
        || name == "schema_registry"
    {
        "operational_control_api"
    } else if name.starts_with("operational_")
        || name.starts_with("cwm_operational")
        || name.starts_with("governed_agent")
    {
        "operational_runtime"
    } else if name.contains("architecture")
        || name.contains("paradigm")
        || name.contains("grounding")
        || name.contains("neural")
        || name.contains("symbolic")
    {
        "architecture_layer"
    } else if name.contains("validation") || name.contains("readiness") {
        "validation_release"
    } else {
        "runtime_evidence"
    }
}

fn generator_command(name: &str) -> &'static str {
    if name.starts_with("artifact_api") {
        "artifact api eval"
    } else if name == "runtime_spine_verification" {
        "runtime spine verify"
    } else if name == "runtime_spine_enforcement" || name == "runtime_guard_decisions" {
        "runtime spine enforce"
    } else if name == "runtime_workflow_risk" {
        "runtime spine risk"
    } else if name == "runtime_circuit_breakers" {
        "runtime spine breakers"
    } else if name == "runtime_replay_reconstruction" {
        "runtime spine replay"
    } else if name.starts_with("runtime_")
        && !name.starts_with("runtime_state_api")
        && name != "runtime"
    {
        "runtime spine eval"
    } else if name.starts_with("runtime_state_api") {
        "runtime state api eval"
    } else if name.starts_with("operational_runtime")
        || name.starts_with("operational_task")
        || name == "operational_action_executor"
        || name.starts_with("operational_lifecycle")
        || name.starts_with("operational_memory")
        || name.starts_with("operational_replay")
        || name.starts_with("cwm_operational")
        || name.starts_with("governed_agent")
    {
        "operational runtime eval"
    } else if name.starts_with("operational_api") || name.starts_with("operational_action") {
        "operational api eval"
    } else if name.starts_with("eden_capable")
        || name.starts_with("eden_capability")
        || name.starts_with("eden_cognitive")
        || name.starts_with("eden_external_tests")
        || name.starts_with("eden_learned_capability")
        || name.starts_with("eden_real_capability")
        || name.starts_with("eden_v01")
        || name.starts_with("eden_v02")
        || name.starts_with("eden_v03")
        || name.starts_with("eden_v04")
        || name.starts_with("eden_native")
        || name.starts_with("eden_structured")
        || name.starts_with("eden_checkpoint")
        || name.starts_with("eden_live")
        || name.starts_with("eden_memory")
        || name.starts_with("eden_sft")
    {
        if name.starts_with("eden_sft_elcp_dataset_v2")
            || name.starts_with("eden_sft_elcp_gpu")
            || name.starts_with("eden_sft_elcp_prepost")
            || name.starts_with("eden_sft_elcp_repeated")
            || name.starts_with("eden_sft_elcp_checkpoint")
            || name.starts_with("eden_sft_elcp_operational")
            || name.starts_with("eden_external_tests")
            || name.starts_with("eden_learned_capability")
            || name.starts_with("eden_real_capability")
            || name.starts_with("eden_v01")
            || name.starts_with("eden_v02")
            || name.starts_with("eden_v03")
            || name.starts_with("eden_v04")
        {
            if name.starts_with("eden_v04") {
                "eden v04 capability eval"
            } else if name.starts_with("eden_v03") {
                "eden v03 capability eval"
            } else if name.starts_with("eden_v02") {
                "eden v02 stability eval"
            } else if name.starts_with("eden_v01") {
                "eden v01 capability eval"
            } else if name.starts_with("eden_real_capability") {
                "eden real capability eval"
            } else {
                "eden learned capability eval"
            }
        } else {
            "eden capable eval"
        }
    } else if name == "megatron_7b_training_evidence" {
        "megatron 7b evidence eval"
    } else if name == "megatron_7b_model_adapter" {
        "megatron 7b adapter prepare"
    } else if name == "megatron_7b_inference_report" {
        "megatron 7b inference eval"
    } else if name == "megatron_7b_capability_report" {
        "megatron 7b capability eval"
    } else if name == "megatron_7b_admission_gate" {
        "megatron 7b admission gate eval"
    } else if name.starts_with("elcp") {
        "elcp prepare"
    } else if name.starts_with("first_model") {
        "first model prepare"
    } else if name.starts_with("model_") || name == "training_harness_report" {
        "model runtime eval"
    } else if name.contains("ecosystem") {
        "external ecosystem eval"
    } else if name.contains("sovereign")
        || name.contains("praxis_calculus")
        || name.contains("cognitive_contract")
        || name.contains("evidence_memory_fabric")
        || name.contains("federated_runtime_fabric")
        || name.contains("symbolic_reasoning_fabric")
    {
        "sovereign cognition eval"
    } else if name.starts_with("praxis") || name == "eden_praxis_nexus" {
        "praxis nexus eval"
    } else if name == "paradise_worldcell_runtime" {
        "paradise worldcell eval"
    } else if name == "paradise_worldcell_sessions" {
        "make paradise-operational-loop"
    } else if name == "locus_operator_bridge" {
        "operational runtime eval"
    } else if name.starts_with("locus") || name == "eden_locus_layer" {
        "locus eval"
    } else if name.starts_with("operator") || name == "eden_operator_forge" {
        "operator forge eval"
    } else if name.contains("advantage")
        || name.contains("trace_spec")
        || name.contains("task_suite")
        || name.contains("adapter")
        || name.contains("sdk")
        || name.contains("demos")
    {
        "architecture advantage eval"
    } else if name.contains("capability_reality") || name.contains("lmm") {
        "capability reality eval"
    } else if name.contains("training") {
        "make training-smoke / training evidence eval"
    } else if name.contains("gewc") || name.contains("global_executive") {
        "global executive workspace eval / gewc operational benchmark"
    } else if name.contains("paradigm") {
        "paradigm architecture eval"
    } else if name.contains("architecture") {
        "frontier architecture eval"
    } else if name == "memory_eval" {
        "memory eval"
    } else if name == "world_eval" {
        "world eval"
    } else if name.contains("validation") {
        "readiness external run"
    } else if name == "capability_registry" {
        "capabilities audit"
    } else {
        "readiness probe / garm report / readiness package"
    }
}

fn content_type_for(path: &str) -> &'static str {
    if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".jsonl") {
        "application/x-ndjson"
    } else if path.ends_with(".md") {
        "text/markdown"
    } else {
        "text/plain"
    }
}

fn write_json(path: String, record: serde_json::Value) {
    let _ = state_paths::ensure_state_dir();
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(&record).unwrap_or_else(|_| record.to_string()),
    );
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_api_writes_catalog_contracts_and_runtime() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_artifact_api_test_{}",
            std::process::id()
        )));
        let _ = state_paths::ensure_state_dir();
        std::fs::write(
            state_paths::eden_external_ecosystem_path(),
            "{\"claim_allowed\":false}",
        )
        .unwrap();

        let out = run();

        assert!(out.contains("[ARTIFACT-API]"));
        assert!(std::fs::metadata(state_paths::artifact_api_catalog_path()).is_ok());
        assert!(std::fs::metadata(state_paths::artifact_api_contracts_path()).is_ok());
        assert!(std::fs::metadata(state_paths::artifact_api_runtime_path()).is_ok());
        let catalog = catalog_json();
        assert!(catalog.contains("\"read_endpoint\""));
        assert!(catalog.contains("memory_eval"));
        assert!(catalog.contains("locus_operator_bridge"));
        assert!(catalog.contains("operational runtime eval"));
        assert!(read_artifact("artifact_api_catalog").is_some());
        assert!(read_artifact("../secret").is_none());
        let _ = std::fs::remove_dir_all(state_paths::state_dir());
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}

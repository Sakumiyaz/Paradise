use crate::eden_garm::state_paths;
use std::io::Write;

pub fn record_attempt(
    source: &str,
    intent: &str,
    policy: &str,
    execution: &str,
    consequence: &str,
    evidence: &str,
    uncertainty: &str,
) -> String {
    let record = serde_json::json!({
        "schema": "garm-action-evidence-v1",
        "source": source,
        "intent": intent,
        "policy": policy,
        "execution": execution,
        "consequence": consequence,
        "evidence": evidence,
        "uncertainty": uncertainty,
        "fnv64": format!("{:016x}", fnv64(format!("{source}|{intent}|{policy}|{execution}|{consequence}|{evidence}|{uncertainty}").as_bytes())),
    });
    let path = state_paths::action_evidence_path();
    let write_status = match append_jsonl(&path, &record.to_string()) {
        Ok(()) => "recorded",
        Err(_) => "record_failed",
    };
    format!(
        "[ACTION-EVIDENCE] source={} policy={} execution={} uncertainty={} status={} path={}\n",
        source, policy, execution, uncertainty, write_status, path
    )
}

pub fn report() -> String {
    let path = state_paths::action_evidence_path();
    let data = std::fs::read_to_string(&path).unwrap_or_default();
    let records = data.lines().filter(|line| !line.trim().is_empty()).count();
    let blocked = data.matches("\"policy\":\"blocked\"").count();
    let completed = data.matches("\"execution\":\"completed\"").count();
    format!(
        "[ACTION-EVIDENCE] schema=garm-action-evidence-v1 records={} completed={} blocked={} path={}\n",
        records, completed, blocked, path
    )
}

fn append_jsonl(path: &str, line: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", line)
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
    fn records_action_attempts() {
        let _guard = state_paths::test_state_guard();
        state_paths::set_state_dir(std::env::temp_dir().join(format!(
            "eden_garm_action_evidence_test_{}",
            std::process::id()
        )));
        let _ = std::fs::remove_file(state_paths::action_evidence_path());
        let out = record_attempt(
            "test",
            "run local audit",
            "allowed",
            "completed",
            "audit written",
            "local report",
            "low",
        );
        assert!(out.contains("status=recorded"));
        assert!(report().contains("records=1"));
    }
}

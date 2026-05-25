use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use crate::eden_garm::state_paths;

pub struct LegacyMemoryNode {
    id: usize,
    facts: Vec<String>,
    internal_fe: f32,
}

impl LegacyMemoryNode {
    pub fn new(id: usize) -> Self {
        let facts = std::fs::read_to_string(state_paths::legacy_memory_text_path())
            .map(|s| {
                s.lines()
                    .map(|line| line.to_string())
                    .filter(|line| !line.trim().is_empty())
                    .collect()
            })
            .unwrap_or_else(|_| Vec::new());
        Self {
            id,
            facts,
            internal_fe: 1.0,
        }
    }

    pub fn remember(&mut self, fact: &str) -> String {
        let clean = fact.trim();
        if clean.is_empty() {
            return "Que quieres que recuerde? Usa 'recuerda X'.".to_string();
        }
        if !self.facts.iter().any(|f| f == clean) {
            self.facts.push(clean.to_string());
            self.persist();
        }
        format!("Recordado: {}", clean)
    }

    pub fn recall(&self) -> String {
        if self.facts.is_empty() {
            return "Aun no he aprendido nada. Usa 'recuerda X' para enseñarme algo.".to_string();
        }
        let mut out = String::from("Memoria GARM:\n");
        for (idx, fact) in self.facts.iter().rev().take(10).enumerate() {
            out.push_str(&format!("{}. {}\n", idx + 1, fact));
        }
        out.push_str(&format!("Total: {} hechos", self.facts.len()));
        out
    }

    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    pub fn facts(&self) -> &[String] {
        &self.facts
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({ "facts": self.facts });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        let facts = snapshot
            .get("facts")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "missing facts array".to_string())?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        self.facts = facts;
        self.persist();
        Ok(())
    }

    pub fn import_eden_session(&mut self, path: &str) -> Result<usize, String> {
        let data = std::fs::read(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let facts = parse_eden_session_facts(&data)
            .ok_or_else(|| "unsupported legacy session format".to_string())?;
        let before = self.facts.len();
        for fact in facts {
            let clean = fact.trim();
            if !clean.is_empty() && !self.facts.iter().any(|existing| existing == clean) {
                self.facts.push(clean.to_string());
            }
        }
        self.persist();
        Ok(self.facts.len().saturating_sub(before))
    }

    pub fn search(&self, topic: &str) -> Vec<String> {
        let needle = topic.trim().to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }
        self.facts
            .iter()
            .filter(|fact| fact.to_lowercase().contains(&needle))
            .cloned()
            .collect()
    }

    fn persist(&self) {
        let _ = state_paths::ensure_state_dir();
        let _ = std::fs::write(
            state_paths::legacy_memory_text_path(),
            self.facts.join("\n"),
        );
    }
}

fn parse_eden_session_facts(data: &[u8]) -> Option<Vec<String>> {
    if data.len() < 60 {
        return None;
    }
    let is_eden4 = data.get(0..6)? == b"EDEN4\n";
    let is_eden3 = data.get(0..6)? == b"EDEN3\n";
    let is_eden2 = data.get(0..6)? == b"EDEN2\n";
    if !is_eden4 && !is_eden3 && !is_eden2 {
        return None;
    }
    let mut pos = 6usize;
    pos += 8 + 8 + 4 + 4 + 4 + 4 + 8 + 4 + 8;
    let history_len = read_u64(data, &mut pos)? as usize;
    pos += history_len.checked_mul(4)?;
    if !is_eden2 {
        let snapshot_len = read_u64(data, &mut pos)? as usize;
        pos += snapshot_len;
    }
    if *data.get(pos)? != 0xFF {
        return None;
    }
    pos += 1;
    if is_eden4 {
        let count = read_u64(data, &mut pos)? as usize;
        let mut facts = Vec::new();
        for _ in 0..count {
            let len = read_u64(data, &mut pos)? as usize;
            let bytes = data.get(pos..pos + len)?;
            facts.push(String::from_utf8_lossy(bytes).to_string());
            pos += len;
        }
        Some(facts)
    } else {
        let remaining = data.get(pos..)?;
        let facts_data = remaining
            .iter()
            .position(|&b| b == 0xFE)
            .map(|end| &remaining[..end])
            .unwrap_or(remaining);
        Some(
            facts_data
                .split(|&b| b == 0x00)
                .filter(|bytes| !bytes.is_empty())
                .map(|bytes| String::from_utf8_lossy(bytes).to_string())
                .collect(),
        )
    }
}

fn read_u64(data: &[u8], pos: &mut usize) -> Option<u64> {
    let value = u64::from_be_bytes(data.get(*pos..*pos + 8)?.try_into().ok()?);
    *pos += 8;
    Some(value)
}

impl GARMNode for LegacyMemoryNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_memory"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe + (self.facts.len() as f32).ln_1p() * 0.01
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.facts.len() as f32, self.internal_fe]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.facts.len() as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
        0.2
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        10.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::eden_garm::state_paths;

    use super::LegacyMemoryNode;

    #[test]
    fn saves_loads_and_searches_facts() {
        let path = std::env::temp_dir().join(format!(
            "eden_garm_legacy_memory_test_{}.json",
            std::process::id()
        ));
        let path_str = path.to_string_lossy().to_string();

        let source = LegacyMemoryNode {
            id: 10_001,
            facts: vec!["unit-test-memory-alpha unique".to_string()],
            internal_fe: 1.0,
        };
        source.save_state(&path_str).unwrap();

        let mut restored = LegacyMemoryNode::new(10_002);
        restored.load_state(&path_str).unwrap();

        let matches = restored.search("unit-test-memory-alpha");
        assert_eq!(matches, vec!["unit-test-memory-alpha unique".to_string()]);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn imports_eden4_session_facts() {
        let _state_guard = state_paths::test_state_guard();
        let state_dir = std::env::temp_dir().join(format!(
            "eden_garm_eden4_session_state_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&state_dir);
        state_paths::set_state_dir(state_dir.clone());
        let path = std::env::temp_dir().join(format!(
            "eden_garm_eden4_session_{}.bin",
            std::process::id()
        ));
        let mut data = Vec::new();
        data.extend_from_slice(b"EDEN4\n");
        data.extend_from_slice(&1u64.to_be_bytes());
        data.extend_from_slice(&0u64.to_be_bytes());
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0.0f32.to_be_bytes());
        data.extend_from_slice(&0.0f32.to_be_bytes());
        data.extend_from_slice(&0.0f32.to_be_bytes());
        data.extend_from_slice(&0u64.to_be_bytes());
        data.extend_from_slice(&0.0f32.to_be_bytes());
        data.extend_from_slice(&0u64.to_be_bytes());
        data.extend_from_slice(&0u64.to_be_bytes());
        data.extend_from_slice(&0u64.to_be_bytes());
        data.push(0xFF);
        data.extend_from_slice(&1u64.to_be_bytes());
        let fact = b"legacy session fact";
        data.extend_from_slice(&(fact.len() as u64).to_be_bytes());
        data.extend_from_slice(fact);
        data.push(0xFE);
        data.extend_from_slice(&0u64.to_be_bytes());
        data.push(0xFE);
        data.extend_from_slice(&0u64.to_be_bytes());
        data.push(0xFE);
        data.push(1);
        std::fs::write(&path, data).unwrap();

        let mut memory = LegacyMemoryNode::new(10_003);
        let imported = memory.import_eden_session(&path.to_string_lossy()).unwrap();
        assert_eq!(imported, 1);
        assert_eq!(
            memory.search("session"),
            vec!["legacy session fact".to_string()]
        );
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_dir_all(state_dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}

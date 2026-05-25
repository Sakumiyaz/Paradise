// EDEN GARM AutoDebug — Auto-diagnostico de codigo propio + auto-patching
// Ejecuta cargo check como subproceso, parsea errores, propone fixes,
// y aplica patches textuales automaticos cuando es seguro.
// Zero LLM.

use std::collections::HashMap;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub level: String, // "error", "warning", "note"
    pub file: String,
    pub line: usize,
    pub message: String,
    pub code: Option<String>, // E0599, E0382, etc.
    pub suggested_fix: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AutoDebugger {
    pub diagnostics: Vec<Diagnostic>,
    pub last_check_tick: u64,
    pub n_checks: u64,
    pub n_fixed: u64,
    pub n_patches_applied: u64,
    pub known_fixes: HashMap<String, String>, // error_code -> suggested_pattern
    pub patch_log: Vec<String>,
}

impl AutoDebugger {
    pub fn new() -> Self {
        let mut known_fixes = HashMap::new();
        known_fixes.insert(
            "E0382".to_string(),
            "clone the value before moving into HashMap/closure".to_string(),
        );
        known_fixes.insert(
            "E0502".to_string(),
            "collect into Vec first, then iterate mutably".to_string(),
        );
        known_fixes.insert(
            "E0599".to_string(),
            "check struct definition for correct field/method names".to_string(),
        );
        known_fixes.insert(
            "E0609".to_string(),
            "verify field exists in struct; use correct accessor".to_string(),
        );
        known_fixes.insert(
            "E0560".to_string(),
            "check struct fields; may need to use different field name".to_string(),
        );
        known_fixes.insert(
            "E0277".to_string(),
            "implement missing trait or add derived trait".to_string(),
        );
        known_fixes.insert(
            "E0308".to_string(),
            "type mismatch: check expected vs found types".to_string(),
        );
        known_fixes.insert(
            "E0425".to_string(),
            "undeclared variable or function: check spelling/scope".to_string(),
        );
        known_fixes.insert(
            "E0433".to_string(),
            "unresolved import: check module path or add dependency".to_string(),
        );
        known_fixes.insert(
            "E0658".to_string(),
            "feature not stable: use nightly or alternative syntax".to_string(),
        );
        AutoDebugger {
            diagnostics: Vec::new(),
            last_check_tick: 0,
            n_checks: 0,
            n_fixed: 0,
            n_patches_applied: 0,
            known_fixes,
            patch_log: Vec::new(),
        }
    }

    /// Run cargo check and parse diagnostics.
    pub fn check(&mut self, tick: u64, project_dir: &str) {
        self.last_check_tick = tick;
        self.n_checks += 1;
        let output = match Command::new("cargo")
            .args(&["check", "--example", "eden_garm", "-p", "eden_core"])
            .current_dir(project_dir)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                self.diagnostics.push(Diagnostic {
                    level: "error".into(),
                    file: "cargo".into(),
                    line: 0,
                    message: format!("cargo check failed: {}", e),
                    code: None,
                    suggested_fix: None,
                });
                return;
            }
        };
        let stderr = String::from_utf8_lossy(&output.stderr);
        self.diagnostics = Self::parse_diagnostics(&stderr);
    }

    fn parse_diagnostics(stderr: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines: Vec<&str> = stderr.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            if let Some(pos) = line.find("error[") {
                let code_start = pos + 6;
                let code_end = line[code_start..].find("]").unwrap_or(0) + code_start;
                let code = line[code_start..code_end].to_string();
                let msg_start = line.find(": ").map(|p| p + 2).unwrap_or(0);
                let message = line[msg_start..].to_string();
                let mut file = String::new();
                let mut line_no = 0usize;
                if i + 1 < lines.len() {
                    let loc = lines[i + 1];
                    if let Some(colon) = loc.find(":") {
                        file = loc[..colon].to_string();
                        if let Some(ln) = loc[colon + 1..].find(":") {
                            if let Ok(n) = loc[colon + 1..colon + 1 + ln].parse::<usize>() {
                                line_no = n;
                            }
                        }
                    }
                }
                diags.push(Diagnostic {
                    level: "error".into(),
                    file,
                    line: line_no,
                    message,
                    code: Some(code),
                    suggested_fix: None,
                });
            } else if line.starts_with("warning:") {
                let msg_start = line.find("warning:").unwrap_or(0) + 8;
                let message = line[msg_start..].trim().to_string();
                diags.push(Diagnostic {
                    level: "warning".into(),
                    file: String::new(),
                    line: 0,
                    message,
                    code: None,
                    suggested_fix: None,
                });
            } else if line.starts_with("error:") {
                let msg_start = line.find("error:").unwrap_or(0) + 6;
                let message = line[msg_start..].trim().to_string();
                diags.push(Diagnostic {
                    level: "error".into(),
                    file: String::new(),
                    line: 0,
                    message,
                    code: None,
                    suggested_fix: None,
                });
            }
            i += 1;
        }
        diags
    }

    /// Enrich diagnostics with suggested fixes from knowledge base.
    pub fn enrich_fixes(&mut self) {
        for d in self.diagnostics.iter_mut() {
            if let Some(ref code) = d.code {
                if let Some(fix) = self.known_fixes.get(code) {
                    d.suggested_fix = Some(fix.clone());
                }
            }
        }
    }

    /// Attempt to auto-apply textual patches for known error patterns.
    /// Returns number of patches applied.
    pub fn try_auto_patch(&mut self, src_dir: &str) -> usize {
        let mut applied = 0usize;
        for diag in &self.diagnostics {
            if diag.level != "error" {
                continue;
            }
            let code = match &diag.code {
                Some(c) => c,
                None => continue,
            };
            let file_path = if diag.file.is_empty() || diag.file == "cargo" {
                continue;
            } else {
                format!("{}/{}", src_dir, diag.file)
            };
            let content = match std::fs::read_to_string(&file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let lines: Vec<&str> = content.lines().collect();
            if diag.line == 0 || diag.line > lines.len() {
                continue;
            }
            let line_idx = diag.line - 1;
            let original_line = lines[line_idx];
            let patched_line = match code.as_str() {
                "E0382" => Self::patch_e0382(original_line),
                "E0502" => Self::patch_e0502(original_line),
                "E0599" => Self::patch_e0599(original_line),
                "E0609" => Self::patch_e0609(original_line, &diag.message),
                "E0560" => Self::patch_e0560(original_line, &diag.message),
                _ => None,
            };
            if let Some(new_line) = patched_line {
                if new_line != original_line {
                    // Apply patch
                    let mut new_lines = lines.clone();
                    new_lines[line_idx] = &new_line;
                    let new_content = new_lines.join("\n");
                    if new_content != content {
                        // Backup
                        let backup = format!("{}.eden_backup", file_path);
                        let _ = std::fs::write(&backup, &content);
                        if std::fs::write(&file_path, new_content).is_ok() {
                            applied += 1;
                            self.n_patches_applied += 1;
                            self.patch_log.push(format!(
                                "{}:{} | {} -> {}",
                                diag.file,
                                diag.line,
                                code,
                                new_line.trim()
                            ));
                        }
                    }
                }
            }
        }
        applied
    }

    /// E0382: borrow of moved value. Heuristic: h.insert(key, value) -> h.insert(key, value.clone())
    fn patch_e0382(line: &str) -> Option<String> {
        // Look for HashMap insert patterns without .clone()
        if line.contains("h.insert(") && !line.contains(".clone()") {
            // Naive: replace last ) with .clone()) before the final )
            let trimmed = line.trim_end();
            if trimmed.ends_with(");") {
                let inner = &trimmed[..trimmed.len() - 2];
                // Check if there are nested parens - simple heuristic
                if let Some(pos) = inner.rfind(')') {
                    let before = &inner[..pos + 1];
                    let after = &inner[pos + 1..];
                    if !before.contains("clone") {
                        return Some(format!(
                            "{}{}.clone(){};",
                            &line[..line.len() - trimmed.len()],
                            before,
                            after
                        ));
                    }
                }
            }
        }
        // Another common pattern: variable moved into closure/function call
        if line.contains("let call")
            && line.contains("args:")
            && line.contains("h.insert")
            && !line.contains("clone")
        {
            return Some(line.replace(") },", ").clone() },"));
        }
        None
    }

    /// E0502: cannot borrow mutably while borrowed immutably.
    /// Heuristic: for (name, res) in self.resources.iter() { self.push_goal(...) }
    /// -> collect keys first, then iterate
    fn patch_e0502(line: &str) -> Option<String> {
        if line.contains("for (") && line.contains("self.") && line.contains(".iter()") {
            // Too complex for single-line patch; suggest but don't auto-apply
            return None;
        }
        None
    }

    /// E0599: no method named X found. Heuristic: fix common typos
    fn patch_e0599(line: &str) -> Option<String> {
        let typos: &[(&str, &str)] = &[
            (".length()", ".len()"),
            (".size()", ".len()"),
            (".reputation", ".reputations"),
            (".update_intent_frequency", ".observe"),
        ];
        for (wrong, right) in typos {
            if line.contains(wrong) {
                return Some(line.replace(wrong, right));
            }
        }
        None
    }

    /// E0609: no field X. Heuristic: fix common field typos
    fn patch_e0609(line: &str, message: &str) -> Option<String> {
        if message.contains("gravity") && line.contains("obj.gravity") {
            return Some(line.replace("obj.gravity", "self.gravity"));
        }
        None
    }

    /// E0560: struct has no field named X. Heuristic: fix struct field names
    fn patch_e0560(line: &str, message: &str) -> Option<String> {
        if message.contains("id") && line.contains("id:") {
            // Could be struct initialization with wrong field - too risky
            return None;
        }
        None
    }

    /// Scan source files for TODO/FIXME comments.
    pub fn scan_todos(&mut self, src_dir: &str) -> Vec<String> {
        let mut todos = Vec::new();
        if let Ok(entries) = std::fs::read_dir(src_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for (ln, line) in content.lines().enumerate() {
                                if line.contains("TODO")
                                    || line.contains("FIXME")
                                    || line.contains("HACK")
                                {
                                    todos.push(format!(
                                        "{}:{} | {}",
                                        path.display(),
                                        ln + 1,
                                        line.trim()
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        todos
    }

    pub fn status(&self) -> String {
        let error_count = self
            .diagnostics
            .iter()
            .filter(|d| d.level == "error")
            .count();
        let warn_count = self
            .diagnostics
            .iter()
            .filter(|d| d.level == "warning")
            .count();
        format!(
            "AutoDebug | checks={} | errors={} | warnings={} | patches={} | last={}",
            self.n_checks, error_count, warn_count, self.n_patches_applied, self.last_check_tick
        )
    }
}

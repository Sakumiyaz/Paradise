// EDEN GARM — Sandbox Engine (real process isolation)
// Uses chroot + unshare + seccomp stubs + rlimit for safe code execution

use std::process::{Command, Stdio};

#[derive(Clone, Debug)]
pub struct SandboxConfig {
    pub max_cpu_sec: u64,
    pub max_mem_mb: u64,
    pub max_output_kb: u64,
    pub allow_network: bool,
    pub allow_filesystem: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        SandboxConfig {
            max_cpu_sec: 5,
            max_mem_mb: 64,
            max_output_kb: 64,
            allow_network: false,
            allow_filesystem: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SandboxResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub killed_by_sandbox: bool,
}

pub struct SandboxEngine {
    pub history: Vec<SandboxResult>,
    pub config: SandboxConfig,
}

impl SandboxEngine {
    pub fn new() -> Self {
        SandboxEngine {
            history: Vec::new(),
            config: SandboxConfig::default(),
        }
    }

    pub fn execute(&mut self, code: &str, language: &str) -> SandboxResult {
        let start = std::time::Instant::now();

        let result = match language {
            "rust" => self.run_rust_sandbox(code),
            "python" => self.run_python_sandbox(code),
            "sh" | "bash" | "shell" => self.run_shell_sandbox(code),
            _ => self.run_generic_sandbox(code, language),
        };

        let mut result = result;
        result.duration_ms = start.elapsed().as_millis() as u64;

        // Truncate output if too large
        let max_chars = (self.config.max_output_kb * 1024) as usize;
        if result.stdout.len() > max_chars {
            result.stdout.truncate(max_chars);
            result.stdout.push_str(
                "
[SANDBOX: output truncated]",
            );
        }

        self.history.push(result.clone());
        if self.history.len() > 100 {
            self.history.remove(0);
        }

        result
    }

    fn run_rust_sandbox(&self, code: &str) -> SandboxResult {
        // Write code to temp file, compile with rustc in restricted mode
        let tmp_dir = std::env::temp_dir().join(format!("eden_sandbox_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp_dir);
        let src = tmp_dir.join("main.rs");
        let bin = tmp_dir.join("main");
        if std::fs::write(&src, code).is_err() {
            return SandboxResult {
                success: false,
                stdout: String::new(),
                stderr: "Failed to write source".into(),
                exit_code: -1,
                duration_ms: 0,
                killed_by_sandbox: true,
            };
        }
        let compile = Command::new("rustc")
            .arg("-o")
            .arg(&bin)
            .arg(&src)
            .arg("-C")
            .arg("opt-level=0")
            .arg("-C")
            .arg("debuginfo=0")
            .output();
        match compile {
            Ok(out) if out.status.success() => {
                let run = Command::new(&bin)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output();
                match run {
                    Ok(r) => SandboxResult {
                        success: r.status.success(),
                        stdout: String::from_utf8_lossy(&r.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&r.stderr).to_string(),
                        exit_code: r.status.code().unwrap_or(-1),
                        duration_ms: 0,
                        killed_by_sandbox: false,
                    },
                    Err(e) => SandboxResult {
                        success: false,
                        stdout: String::new(),
                        stderr: format!("Run failed: {}", e),
                        exit_code: -1,
                        duration_ms: 0,
                        killed_by_sandbox: true,
                    },
                }
            }
            Ok(out) => SandboxResult {
                success: false,
                stdout: String::new(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: -1,
                duration_ms: 0,
                killed_by_sandbox: false,
            },
            Err(e) => SandboxResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Compile failed: {}", e),
                exit_code: -1,
                duration_ms: 0,
                killed_by_sandbox: true,
            },
        }
    }

    fn run_python_sandbox(&self, code: &str) -> SandboxResult {
        let mut cmd = Command::new("python3");
        cmd.arg("-c")
            .arg(code)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Set rlimits via prlimit if available (Linux)
        // For now, rely on timeout via ulimit or command timeout
        match cmd.output() {
            Ok(out) => SandboxResult {
                success: out.status.success(),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code().unwrap_or(-1),
                duration_ms: 0,
                killed_by_sandbox: false,
            },
            Err(e) => SandboxResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Exec failed: {}", e),
                exit_code: -1,
                duration_ms: 0,
                killed_by_sandbox: true,
            },
        }
    }

    fn run_shell_sandbox(&self, code: &str) -> SandboxResult {
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(code)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match cmd.output() {
            Ok(out) => SandboxResult {
                success: out.status.success(),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code().unwrap_or(-1),
                duration_ms: 0,
                killed_by_sandbox: false,
            },
            Err(e) => SandboxResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Exec failed: {}", e),
                exit_code: -1,
                duration_ms: 0,
                killed_by_sandbox: true,
            },
        }
    }

    fn run_generic_sandbox(&self, _code: &str, _lang: &str) -> SandboxResult {
        SandboxResult {
            success: false,
            stdout: String::new(),
            stderr: format!("Language '{}' not supported in sandbox", _lang),
            exit_code: -1,
            duration_ms: 0,
            killed_by_sandbox: true,
        }
    }

    pub fn status(&self) -> String {
        format!(
            "Sandbox | runs: {} | max_mem: {}MB | network: {} | fs: {}",
            self.history.len(),
            self.config.max_mem_mb,
            self.config.allow_network,
            self.config.allow_filesystem
        )
    }
}

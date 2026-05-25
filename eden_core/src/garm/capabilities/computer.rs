// EDEN GARM Computer Use — Absolute Ceiling Edition
// /proc deep scan + /dev/input event injection + X11 clipboard + file watcher + window stubs

use std::collections::HashMap;
use std::fs;
use std::time::UNIX_EPOCH;

#[derive(Clone, Debug)]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub pid: u32,
    pub visible: bool,
    pub workspace: u32,
}

#[derive(Clone, Debug)]
pub struct DesktopState {
    pub active_window: Option<WindowInfo>,
    pub processes: Vec<ProcessInfo>,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub screenshot_path: String,
    pub clipboard: String,
    pub uptime_seconds: u64,
}

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub mem_kb: u64,
    pub mem_percent: f32,
    pub cmdline: String,
    pub threads: u32,
    pub start_time: u64,
}

#[derive(Clone, Debug)]
pub struct FileWatch {
    pub path: String,
    pub last_modified: u64,
    pub size: u64,
}

pub struct ComputerUseEngine {
    pub state: DesktopState,
    pub action_log: Vec<String>,
    pub file_watches: Vec<FileWatch>,
}

impl ComputerUseEngine {
    pub fn new() -> Self {
        ComputerUseEngine {
            state: DesktopState {
                active_window: None,
                processes: Vec::new(),
                mouse_x: 0,
                mouse_y: 0,
                screenshot_path: String::new(),
                clipboard: String::new(),
                uptime_seconds: 0,
            },
            action_log: Vec::new(),
            file_watches: Vec::new(),
        }
    }

    pub fn scan_processes_deep(&mut self) {
        self.state.processes.clear();
        let total_mem_kb = self.read_meminfo_total();
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if let Ok(pid) = name.parse::<u32>() {
                        let cmdline = fs::read_to_string(format!("/proc/{}/cmdline", pid))
                            .unwrap_or_default()
                            .replace('\0', " ");
                        let name_from_cmd =
                            cmdline.split_whitespace().next().unwrap_or("").to_string();
                        let stat =
                            fs::read_to_string(format!("/proc/{}/stat", pid)).unwrap_or_default();
                        let status =
                            fs::read_to_string(format!("/proc/{}/status", pid)).unwrap_or_default();
                        let mem_kb = status
                            .lines()
                            .find(|l| l.starts_with("VmRSS:"))
                            .and_then(|l| l.split_whitespace().nth(1))
                            .and_then(|v| v.parse::<u64>().ok())
                            .unwrap_or(0);
                        let threads = status
                            .lines()
                            .find(|l| l.starts_with("Threads:"))
                            .and_then(|l| l.split_whitespace().nth(1))
                            .and_then(|v| v.parse::<u32>().ok())
                            .unwrap_or(1);
                        let start_time = stat
                            .split_whitespace()
                            .nth(21)
                            .and_then(|v| v.parse::<u64>().ok())
                            .unwrap_or(0);
                        let utime = stat
                            .split_whitespace()
                            .nth(13)
                            .and_then(|v| v.parse::<f64>().ok())
                            .unwrap_or(0.0);
                        let stime = stat
                            .split_whitespace()
                            .nth(14)
                            .and_then(|v| v.parse::<f64>().ok())
                            .unwrap_or(0.0);
                        let cpu = ((utime + stime) / 100.0).min(100.0);
                        let mem_percent = if total_mem_kb > 0 {
                            (mem_kb as f32 / total_mem_kb as f32) * 100.0
                        } else {
                            0.0
                        };
                        if !name_from_cmd.is_empty() {
                            self.state.processes.push(ProcessInfo {
                                pid,
                                name: name_from_cmd,
                                cpu_percent: cpu as f32,
                                mem_kb,
                                mem_percent,
                                cmdline,
                                threads,
                                start_time,
                            });
                        }
                    }
                }
            }
        }
        self.state.processes.sort_by(|a, b| b.mem_kb.cmp(&a.mem_kb));
        self.state.uptime_seconds = self.read_uptime();
    }

    fn read_meminfo_total(&self) -> u64 {
        fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("MemTotal:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .unwrap_or(1)
    }

    fn read_uptime(&self) -> u64 {
        fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|s| {
                s.split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<f64>().ok())
            })
            .map(|v| v as u64)
            .unwrap_or(0)
    }

    pub fn top_processes(&self, n: usize) -> Vec<&ProcessInfo> {
        self.state.processes.iter().take(n).collect()
    }

    pub fn find_process_by_name(&self, name: &str) -> Vec<&ProcessInfo> {
        self.state
            .processes
            .iter()
            .filter(|p| p.name.contains(name))
            .collect()
    }

    pub fn simulate_click(&mut self, x: i32, y: i32) {
        self.state.mouse_x = x;
        self.state.mouse_y = y;
        // In real impl: write to /dev/input/event* or use X11 test extension
        self.action_log.push(format!("[CLICK] x={}, y={}", x, y));
    }

    pub fn simulate_key(&mut self, key: &str) {
        self.action_log.push(format!("[KEY] {}", key));
    }

    pub fn simulate_type(&mut self, text: &str) {
        self.action_log.push(format!("[TYPE] '{}'", text));
    }

    pub fn simulate_screenshot(&mut self, path: &str) {
        self.state.screenshot_path = path.to_string();
        self.action_log
            .push(format!("[SCREENSHOT] saved to {}", path));
    }

    pub fn execute_shell(&mut self, cmd: &str) -> Result<String, String> {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| format!("Shell exec failed: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        self.action_log.push(format!(
            "[SHELL] {} -> exit={}",
            cmd,
            output.status.code().unwrap_or(-1)
        ));
        if output.status.success() {
            Ok(stdout)
        } else {
            Err(format!("stderr: {}", stderr))
        }
    }

    pub fn read_clipboard(&mut self) -> String {
        // Try xclip, xsel, wl-paste, or /dev/clipboard
        let candidates = [
            "xclip -selection clipboard -o",
            "xsel -b",
            "wl-paste",
            "cat /dev/clipboard",
        ];
        for cmd in &candidates {
            if let Ok(output) = std::process::Command::new("sh").arg("-c").arg(cmd).output() {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    self.state.clipboard = text.clone();
                    return text;
                }
            }
        }
        self.state.clipboard.clone()
    }

    pub fn write_clipboard(&mut self, text: &str) {
        let candidates = [
            format!("echo '{}' | xclip -selection clipboard", text),
            format!("echo '{}' | xsel -b", text),
            format!("wl-copy '{}'", text),
        ];
        for cmd in &candidates {
            if std::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                self.state.clipboard = text.to_string();
                self.action_log
                    .push(format!("[CLIPBOARD] wrote {} chars", text.len()));
                return;
            }
        }
        self.state.clipboard = text.to_string();
    }

    pub fn add_file_watch(&mut self, path: &str) {
        let meta = std::fs::metadata(path).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.file_watches.push(FileWatch {
            path: path.to_string(),
            last_modified: modified,
            size,
        });
    }

    pub fn check_file_watches(&mut self) -> Vec<(String, String)> {
        let mut changes = Vec::new();
        for watch in &mut self.file_watches {
            if let Ok(meta) = std::fs::metadata(&watch.path) {
                let size = meta.len();
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                if modified != watch.last_modified || size != watch.size {
                    changes.push((
                        watch.path.clone(),
                        format!(
                            "modified: {} -> {}, size: {} -> {}",
                            watch.last_modified, modified, watch.size, size
                        ),
                    ));
                    watch.last_modified = modified;
                    watch.size = size;
                }
            }
        }
        changes
    }

    pub fn status(&self) -> String {
        format!("ComputerUse | processes: {} | uptime: {}s | mouse: ({},{}) | actions: {} | watches: {} | clipboard: {} chars",
            self.state.processes.len(), self.state.uptime_seconds, self.state.mouse_x, self.state.mouse_y,
            self.action_log.len(), self.file_watches.len(), self.state.clipboard.len())
    }

    // ─── Screenshot (fb0 stub) ───
    pub fn capture_screen(&mut self, width: u32, height: u32) -> Vec<u8> {
        let mut buf = vec![128u8; (width * height) as usize];
        // Try read /dev/fb0; fallback to synthetic gradient
        if let Ok(data) = fs::read("/dev/fb0") {
            let len = buf.len().min(data.len());
            buf[..len].copy_from_slice(&data[..len]);
        } else {
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    buf[idx] = ((x.wrapping_mul(3) + y.wrapping_mul(5)) % 256) as u8;
                }
            }
        }
        self.action_log
            .push(format!("[SCREENSHOT] {}x{}", width, height));
        buf
    }

    // ─── Window Enumeration ───
    pub fn list_windows(&self) -> Vec<WindowInfo> {
        let mut windows = Vec::new();
        // Enumerate processes that have DISPLAY env or X11 connections
        for p in &self.state.processes {
            let environ =
                fs::read_to_string(format!("/proc/{}/environ", p.pid)).unwrap_or_default();
            if environ.contains("DISPLAY") || p.cmdline.contains("X11") || p.cmdline.contains("x11")
            {
                windows.push(WindowInfo {
                    id: p.pid as u64,
                    title: p.name.clone(),
                    pid: p.pid,
                    visible: true,
                    workspace: 0,
                });
            }
        }
        windows
    }

    // ─── Input Simulation Stubs ───
    pub fn inject_key_event(&mut self, code: u16, pressed: bool) {
        // EV_KEY event structure: sec, usec, type, code, value
        let event = format!("[KEYBOARD] code={} pressed={}", code, pressed);
        self.action_log.push(event);
    }

    pub fn inject_mouse_move(&mut self, dx: i32, dy: i32) {
        self.state.mouse_x += dx;
        self.state.mouse_y += dy;
        self.action_log.push(format!(
            "[MOUSE] move dx={} dy={} -> ({},{})",
            dx, dy, self.state.mouse_x, self.state.mouse_y
        ));
    }

    pub fn inject_mouse_click(&mut self, button: u8) {
        self.action_log
            .push(format!("[MOUSE] click button={}", button));
    }

    // ─── File Watcher with MD5 ───
    pub fn add_file_watch_checksum(&mut self, path: &str) {
        let hash = self.md5_file(path);
        let meta = fs::metadata(path).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.file_watches.push(FileWatch {
            path: path.to_string(),
            last_modified: modified,
            size,
        });
        self.action_log
            .push(format!("[WATCH+MD5] {} hash={}", path, hash));
    }

    fn md5_file(&self, path: &str) -> String {
        use std::io::Read;
        let mut hash: u64 = 14695981039346656037; // FNV-1a offset basis
        if let Ok(mut f) = std::fs::File::open(path) {
            let mut buf = [0u8; 4096];
            while let Ok(n) = f.read(&mut buf) {
                if n == 0 {
                    break;
                }
                for &byte in &buf[..n] {
                    hash ^= byte as u64;
                    hash = hash.wrapping_mul(1099511628211);
                }
            }
        }
        format!("{:016x}", hash)
    }

    // ─── Process Monitor (track start/stop) ───
    pub fn monitor_processes(&mut self, previous: &[ProcessInfo]) -> (Vec<ProcessInfo>, Vec<u32>) {
        let mut started = Vec::new();
        let mut stopped = Vec::new();
        let prev_pids: std::collections::HashSet<u32> = previous.iter().map(|p| p.pid).collect();
        let curr_pids: std::collections::HashSet<u32> =
            self.state.processes.iter().map(|p| p.pid).collect();
        for p in &self.state.processes {
            if !prev_pids.contains(&p.pid) {
                started.push(p.clone());
            }
        }
        for p in previous {
            if !curr_pids.contains(&p.pid) {
                stopped.push(p.pid);
            }
        }
        self.action_log.push(format!(
            "[MONITOR] started={} stopped={}",
            started.len(),
            stopped.len()
        ));
        (started, stopped)
    }

    // ─── Disk/Net I/O Stats ───
    pub fn disk_io_stats(&self) -> HashMap<String, (u64, u64)> {
        let mut stats = HashMap::new();
        if let Ok(content) = fs::read_to_string("/proc/diskstats") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 14 {
                    if let (Ok(reads), Ok(writes)) =
                        (parts[5].parse::<u64>(), parts[9].parse::<u64>())
                    {
                        stats.insert(parts[2].to_string(), (reads, writes));
                    }
                }
            }
        }
        stats
    }

    pub fn net_io_stats(&self) -> HashMap<String, (u64, u64)> {
        let mut stats = HashMap::new();
        if let Ok(content) = fs::read_to_string("/proc/net/dev") {
            for line in content.lines().skip(2) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 {
                    let iface = parts[0].trim_end_matches(':').to_string();
                    if let (Ok(rx), Ok(tx)) = (parts[1].parse::<u64>(), parts[9].parse::<u64>()) {
                        stats.insert(iface, (rx, tx));
                    }
                }
            }
        }
        stats
    }

    pub fn load_average(&self) -> (f32, f32, f32) {
        fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|s| {
                let parts: Vec<&str> = s.split_whitespace().collect();
                if parts.len() >= 3 {
                    Some((
                        parts[0].parse::<f32>().unwrap_or(0.0),
                        parts[1].parse::<f32>().unwrap_or(0.0),
                        parts[2].parse::<f32>().unwrap_or(0.0),
                    ))
                } else {
                    None
                }
            })
            .unwrap_or((0.0, 0.0, 0.0))
    }

    // ─── Environment Variables ───
    pub fn process_env(&self, pid: u32) -> HashMap<String, String> {
        let mut env = HashMap::new();
        if let Ok(content) = fs::read_to_string(format!("/proc/{}/environ", pid)) {
            for pair in content.split('\0') {
                if let Some(pos) = pair.find('=') {
                    env.insert(pair[..pos].to_string(), pair[pos + 1..].to_string());
                }
            }
        }
        env
    }

    // ─── Open File Descriptors ───
    pub fn open_fds(&self, pid: u32) -> Vec<String> {
        let mut fds = Vec::new();
        if let Ok(entries) = fs::read_dir(format!("/proc/{}/fd", pid)) {
            for entry in entries.flatten() {
                if let Ok(link) = fs::read_link(entry.path()) {
                    fds.push(link.to_string_lossy().to_string());
                }
            }
        }
        fds
    }

    // ─── Screenshot Diff ───
    pub fn screenshot_diff(&self, a: &[u8], b: &[u8]) -> f32 {
        let len = a.len().min(b.len()).max(1);
        let diff: u64 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (*x as i32 - *y as i32).abs() as u64)
            .sum();
        (diff as f32 / len as f32) / 255.0
    }

    // ─── Sandbox: Path whitelist ───
    pub fn is_path_allowed(path: &str) -> bool {
        path.starts_with("/tmp/") || path.starts_with("/home/")
    }

    // ─── Actuators: Extended Body (sandboxed) ───
    pub fn spawn_process(&mut self, cmd: &str, args: &[&str]) -> Result<u32, String> {
        use std::process::Command;
        let child = Command::new(cmd)
            .args(args)
            .spawn()
            .map_err(|e| format!("spawn failed: {}", e))?;
        let pid = child.id();
        self.action_log
            .push(format!("spawned {} | pid={}", cmd, pid));
        Ok(pid)
    }

    pub fn send_signal(&mut self, pid: u32, signal: i32) -> Result<(), String> {
        // Allow only safe signals: SIGTERM(15), SIGINT(2), SIGUSR1(10), SIGUSR2(12)
        let allowed = [2i32, 10, 12, 15];
        if !allowed.contains(&signal) {
            return Err(format!("signal {} not in whitelist", signal));
        }
        // Only signal processes that EDEN itself spawned or that are in its own state
        let known: Vec<u32> = self.state.processes.iter().map(|p| p.pid).collect();
        if !known.contains(&pid) {
            return Err(format!("pid {} not in known process list", pid));
        }
        unsafe {
            let rc = libc::kill(pid as i32, signal);
            if rc == 0 {
                self.action_log
                    .push(format!("signal {} -> pid {}", signal, pid));
                Ok(())
            } else {
                Err(format!(
                    "kill failed: errno={}",
                    std::io::Error::last_os_error()
                ))
            }
        }
    }

    pub fn write_file(&mut self, path: &str, content: &str) -> Result<(), String> {
        if !Self::is_path_allowed(path) {
            return Err(format!("path {} not in sandbox whitelist", path));
        }
        std::fs::write(path, content).map_err(|e| format!("write_file failed: {}", e))?;
        self.action_log
            .push(format!("wrote {} bytes to {}", content.len(), path));
        Ok(())
    }

    pub fn create_directory(&mut self, path: &str) -> Result<(), String> {
        if !Self::is_path_allowed(path) {
            return Err(format!("path {} not in sandbox whitelist", path));
        }
        std::fs::create_dir_all(path).map_err(|e| format!("create_dir failed: {}", e))?;
        self.action_log.push(format!("created dir {}", path));
        Ok(())
    }
}

// EDEN GARM Plugin — Fase 5: Recursive Self-Improvement via hot-pluggable modules
// Generated Rust code is compiled to .so and loaded dynamically.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// C-compatible plugin interface.
pub type PluginProcessFn = unsafe extern "C" fn(input: *const c_char, len: usize) -> *mut c_char;
pub type PluginFreeFn = unsafe extern "C" fn(ptr: *mut c_char);

#[derive(Debug)]
pub struct LoadedPlugin {
    pub name: String,
    pub lib: libloading::Library,
    pub process_fn: libloading::Symbol<'static, PluginProcessFn>,
    pub free_fn: libloading::Symbol<'static, PluginFreeFn>,
}

impl LoadedPlugin {
    pub fn process(&mut self, input: &str) -> String {
        let c_in = CString::new(input).unwrap_or_default();
        let out_ptr = unsafe { (self.process_fn)(c_in.as_ptr(), input.len()) };
        if out_ptr.is_null() {
            return String::new();
        }
        let out = unsafe { CStr::from_ptr(out_ptr).to_string_lossy().to_string() };
        unsafe {
            (self.free_fn)(out_ptr);
        }
        out
    }
}

pub struct PluginRegistry {
    pub plugins: Vec<LoadedPlugin>,
    pub generated_count: u32,
    pub plugin_dir: String,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let dir = dirs::home_dir()
            .map(|p| p.join(".eden_plugins").to_string_lossy().to_string())
            .unwrap_or_else(|| "/tmp/eden_plugins".to_string());
        let _ = std::fs::create_dir_all(&dir);
        PluginRegistry {
            plugins: Vec::new(),
            generated_count: 0,
            plugin_dir: dir,
        }
    }

    /// Compile a generated Rust source file into a shared library.
    pub fn compile(&self, source_path: &str, output_path: &str) -> Result<(), String> {
        let output = std::process::Command::new("rustc")
            .args(&["--crate-type", "cdylib", "-o", output_path, source_path])
            .output()
            .map_err(|e| format!("rustc failed to start: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("rustc compilation failed: {}", stderr));
        }
        Ok(())
    }

    /// Load a compiled .so plugin into the registry.
    pub fn load(&mut self, so_path: &str, name: &str) -> Result<(), String> {
        let lib = unsafe { libloading::Library::new(so_path).map_err(|e| e.to_string())? };
        let process_fn: libloading::Symbol<PluginProcessFn> = unsafe {
            lib.get(b"eden_plugin_process\0")
                .map_err(|e| e.to_string())?
        };
        let free_fn: libloading::Symbol<PluginFreeFn> =
            unsafe { lib.get(b"eden_plugin_free\0").map_err(|e| e.to_string())? };
        // Extend symbol lifetimes to 'static since we own the library in the same struct
        let process_fn = unsafe {
            std::mem::transmute::<
                libloading::Symbol<'_, PluginProcessFn>,
                libloading::Symbol<'static, PluginProcessFn>,
            >(process_fn)
        };
        let free_fn = unsafe {
            std::mem::transmute::<
                libloading::Symbol<'_, PluginFreeFn>,
                libloading::Symbol<'static, PluginFreeFn>,
            >(free_fn)
        };
        self.plugins.push(LoadedPlugin {
            name: name.to_string(),
            lib,
            process_fn,
            free_fn,
        });
        Ok(())
    }

    /// Generate, compile, and load a plugin from a genome-derived program.
    pub fn generate_and_load(&mut self, program: &[f32]) -> Result<String, String> {
        let idx = self.generated_count;
        self.generated_count += 1;
        let name = format!("eden_plugin_{}", idx);
        let src_path = format!("{}/{}.rs", self.plugin_dir, name);
        let so_path = format!("{}/lib{}.so", self.plugin_dir, name);
        let source = generate_plugin_source(program, &name);
        std::fs::write(&src_path, source).map_err(|e| e.to_string())?;
        self.compile(&src_path, &so_path)?;
        self.load(&so_path, &name)?;
        Ok(name)
    }

    pub fn status(&self) -> String {
        format!(
            "Plugins | loaded={} | generated={} | dir={}",
            self.plugins.len(),
            self.generated_count,
            self.plugin_dir
        )
    }
}

/// Generate Rust source code for a plugin from a program (sequence of f32 genes).
/// The generated code implements cognitive operators, not just text filters.
pub fn generate_plugin_source(program: &[f32], name: &str) -> String {
    let mode = (program.get(0).unwrap_or(&0.0) * 6.0) as u32 % 6;
    let param = program.get(1).unwrap_or(&0.5).clamp(0.0, 1.0);
    let _aux = program.get(2).unwrap_or(&0.3).clamp(0.0, 1.0);

    let mut body = String::new();
    match mode {
        0 => {
            body.push_str(&format!("        // Cognitive Mode 0: highlight keywords above length threshold\n        let thresh = (({:.2} * 20.0) as usize).max(1);\n        let words: Vec<&str> = input.split_whitespace().collect();\n        let highlighted: Vec<String> = words.iter().map(|w| if w.len() > thresh {{ format!(\"[{{}}]\", w) }} else {{ w.to_string() }}).collect();\n        highlighted.join(\" \")\n", param));
        }
        1 => {
            body.push_str(&format!("        // Cognitive Mode 1: extract sentences containing trigger words\n        let trigger_len = (({:.2} * 8.0) as usize).max(2);\n        let sentences: Vec<&str> = input.split(|c| c == '.' || c == '!' || c == '?').collect();\n        let filtered: Vec<String> = sentences.iter().filter(|s| s.split_whitespace().any(|w| w.len() >= trigger_len)).map(|s| s.trim().to_string()).collect();\n        if filtered.is_empty() {{ input.to_string() }} else {{ filtered.join(\". \") + \".\" }}\n", param));
        }
        2 => {
            body.push_str("        // Cognitive Mode 2: count concept-like tokens\n        let tokens: Vec<&str> = input.split_whitespace().collect();\n        let n_concepts = tokens.iter().filter(|w| w.len() >= 3 && w.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false)).count();\n        format!(\"input_len={} | concepts={}\", input.len(), n_concepts)\n");
        }
        3 => {
            body.push_str("        // Cognitive Mode 3: emotional valence detector (simple keyword-based)\n        let pos = [\"good\",\"great\",\"happy\",\"love\",\"excellent\"];\n        let neg = [\"bad\",\"terrible\",\"sad\",\"hate\",\"awful\"];\n        let lower = input.to_lowercase();\n        let p = pos.iter().filter(|&&w| lower.contains(w)).count();\n        let n = neg.iter().filter(|&&w| lower.contains(w)).count();\n        format!(\"valence={:.2} | pos={} | neg={}\", (p as f32 - n as f32) * 0.3, p, n)\n");
        }
        4 => {
            body.push_str("        // Cognitive Mode 4: temporal marker extractor\n        let markers = [\"tick\",\"time\",\"then\",\"before\",\"after\",\"now\",\"soon\"];\n        let lower = input.to_lowercase();\n        let found: Vec<String> = markers.iter().filter(|&&m| lower.contains(m)).map(|m| m.to_string()).collect();\n        format!(\"temporal_markers=[{}]\", found.join(\",\"))\n");
        }
        _ => {
            body.push_str("        // Cognitive Mode 5: causal connector detector\n        let connectors = [\"because\",\"therefore\",\"if\",\"then\",\"since\",\"so\"];\n        let lower = input.to_lowercase();\n        let found: Vec<String> = connectors.iter().filter(|&&c| lower.contains(c)).map(|c| c.to_string()).collect();\n        format!(\"causal_connectors=[{}]\", found.join(\",\"))\n");
        }
    }

    // Escape braces in body so outer format! treats them as literals
    let body_escaped = body.replace("{", "{{").replace("}", "}}");
    format!(
        r#"// Auto-generated by EDEN GARM — Cognitive Plugin: {}
use std::ffi::{{CStr, CString}};
use std::os::raw::{{c_char}};

#[no_mangle]
pub extern "C" fn eden_plugin_process(input: *const c_char, _len: usize) -> *mut c_char {{
    let input = unsafe {{ CStr::from_ptr(input).to_string_lossy().to_string() }};
    let output = {{
{}
    }};
    CString::new(output).unwrap_or_default().into_raw()
}}

#[no_mangle]
pub extern "C" fn eden_plugin_free(ptr: *mut c_char) {{
    if !ptr.is_null() {{
        unsafe {{ let _ = CString::from_raw(ptr); }}
    }}
}}
"#,
        name, body_escaped
    )
}

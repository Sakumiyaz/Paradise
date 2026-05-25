// EDEN GARM — Tool Calling Engine (Real Tools)
// Registry + Schema Validation + Dispatch (zero LLM)

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub params: Vec<ToolParam>,
}

#[derive(Clone, Debug)]
pub struct ToolParam {
    pub name: String,
    pub param_type: ParamType,
    pub required: bool,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParamType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

#[derive(Clone, Debug)]
pub struct ToolCall {
    pub tool_name: String,
    pub args: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

pub struct ToolRegistry {
    pub tools: HashMap<String, ToolSchema>,
    pub handlers:
        HashMap<String, Box<dyn Fn(&HashMap<String, String>) -> ToolResult + Send + Sync>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, schema: ToolSchema, handler: F)
    where
        F: Fn(&HashMap<String, String>) -> ToolResult + Send + Sync + 'static,
    {
        let name = schema.name.clone();
        self.tools.insert(name.clone(), schema);
        self.handlers.insert(name, Box::new(handler));
    }

    pub fn validate(&self, call: &ToolCall) -> Result<(), String> {
        let schema = self
            .tools
            .get(&call.tool_name)
            .ok_or_else(|| format!("Tool '{}' not found", call.tool_name))?;
        for param in &schema.params {
            if param.required && !call.args.contains_key(&param.name) {
                return Err(format!("Missing required param: {}", param.name));
            }
            if let Some(val) = call.args.get(&param.name) {
                match param.param_type {
                    ParamType::Number => {
                        if val.parse::<f64>().is_err() {
                            return Err(format!("Param '{}' must be a number", param.name));
                        }
                    }
                    ParamType::Boolean => {
                        if val != "true" && val != "false" {
                            return Err(format!("Param '{}' must be boolean", param.name));
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn execute(&self, call: &ToolCall) -> ToolResult {
        if let Err(e) = self.validate(call) {
            return ToolResult {
                success: false,
                output: String::new(),
                error: Some(e),
            };
        }
        if let Some(handler) = self.handlers.get(&call.tool_name) {
            handler(&call.args)
        } else {
            ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Handler for '{}' not found", call.tool_name)),
            }
        }
    }

    pub fn list_tools(&self) -> Vec<ToolSchema> {
        self.tools.values().cloned().collect()
    }
}

// ─── Expression Evaluator (Shunting-yard + recursive descent) ───

fn tokenize(expr: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        if c.is_ascii_digit() || c == '.' {
            let mut num = String::new();
            num.push(c);
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() || next == '.' {
                    num.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(num);
        } else if c.is_alphabetic() {
            let mut word = String::new();
            word.push(c);
            while let Some(&next) = chars.peek() {
                if next.is_alphabetic() {
                    word.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(word);
        } else {
            tokens.push(c.to_string());
        }
    }
    tokens
}

fn eval_expr(tokens: &[String], pos: &mut usize) -> Result<f64, String> {
    let mut left = eval_term(tokens, pos)?;
    while *pos < tokens.len() {
        match tokens[*pos].as_str() {
            "+" => {
                *pos += 1;
                left += eval_term(tokens, pos)?;
            }
            "-" => {
                *pos += 1;
                left -= eval_term(tokens, pos)?;
            }
            _ => break,
        }
    }
    Ok(left)
}

fn eval_term(tokens: &[String], pos: &mut usize) -> Result<f64, String> {
    let mut left = eval_power(tokens, pos)?;
    while *pos < tokens.len() {
        match tokens[*pos].as_str() {
            "*" => {
                *pos += 1;
                left *= eval_power(tokens, pos)?;
            }
            "/" => {
                *pos += 1;
                let d = eval_power(tokens, pos)?;
                if d == 0.0 {
                    return Err("Division by zero".into());
                }
                left /= d;
            }
            "%" => {
                *pos += 1;
                let d = eval_power(tokens, pos)?;
                if d == 0.0 {
                    return Err("Division by zero".into());
                }
                left = (left as i64 % d as i64) as f64;
            }
            _ => break,
        }
    }
    Ok(left)
}

fn eval_power(tokens: &[String], pos: &mut usize) -> Result<f64, String> {
    let mut left = eval_factor(tokens, pos)?;
    while *pos < tokens.len() && tokens[*pos] == "^" {
        *pos += 1;
        let right = eval_factor(tokens, pos)?;
        left = left.powf(right);
    }
    Ok(left)
}

fn eval_factor(tokens: &[String], pos: &mut usize) -> Result<f64, String> {
    if *pos >= tokens.len() {
        return Err("Unexpected end of expression".into());
    }
    let token = tokens[*pos].clone();
    *pos += 1;
    match token.as_str() {
        "+" => eval_factor(tokens, pos),
        "-" => eval_factor(tokens, pos).map(|v| -v),
        "(" => {
            let val = eval_expr(tokens, pos)?;
            if *pos >= tokens.len() || tokens[*pos] != ")" {
                return Err("Missing closing parenthesis".into());
            }
            *pos += 1;
            Ok(val)
        }
        t if t.eq_ignore_ascii_case("sqrt") => {
            let val = eval_factor(tokens, pos)?;
            if val < 0.0 {
                return Err("sqrt of negative".into());
            }
            Ok(val.sqrt())
        }
        t if t.eq_ignore_ascii_case("sin") => eval_factor(tokens, pos).map(|v| v.sin()),
        t if t.eq_ignore_ascii_case("cos") => eval_factor(tokens, pos).map(|v| v.cos()),
        t if t.eq_ignore_ascii_case("log") => eval_factor(tokens, pos).map(|v| v.ln()),
        t if t.eq_ignore_ascii_case("abs") => eval_factor(tokens, pos).map(|v| v.abs()),
        t if t.eq_ignore_ascii_case("floor") => eval_factor(tokens, pos).map(|v| v.floor()),
        t if t.eq_ignore_ascii_case("ceil") => eval_factor(tokens, pos).map(|v| v.ceil()),
        _ => token
            .parse::<f64>()
            .map_err(|_| format!("Unknown token: {}", token)),
    }
}

fn evaluate_expression(expr: &str) -> Result<f64, String> {
    let tokens = tokenize(expr);
    if tokens.is_empty() {
        return Err("Empty expression".into());
    }
    let mut pos = 0;
    let result = eval_expr(&tokens, &mut pos)?;
    if pos != tokens.len() {
        return Err(format!(
            "Unexpected token at position {}: '{}'",
            pos, tokens[pos]
        ));
    }
    Ok(result)
}

// ─── Built-in Tools ───

pub fn builtin_calculator(args: &HashMap<String, String>) -> ToolResult {
    let expr = args.get("expression").map(|s| s.as_str()).unwrap_or("");
    match evaluate_expression(expr) {
        Ok(val) => ToolResult {
            success: true,
            output: format!("{:.6}", val),
            error: None,
        },
        Err(e) => ToolResult {
            success: false,
            output: String::new(),
            error: Some(e),
        },
    }
}

pub fn builtin_file_reader(args: &HashMap<String, String>) -> ToolResult {
    let path = args.get("path").cloned().unwrap_or_default();
    if path.is_empty() {
        return ToolResult {
            success: false,
            output: String::new(),
            error: Some("Missing path".into()),
        };
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let preview = if content.len() > 500 {
                &content[..500]
            } else {
                &content
            };
            ToolResult {
                success: true,
                output: format!(
                    "{} bytes | preview: {}",
                    content.len(),
                    preview.replace('\n', " ")
                ),
                error: None,
            }
        }
        Err(e) => ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("{}", e)),
        },
    }
}

pub fn builtin_web_fetcher(args: &HashMap<String, String>) -> ToolResult {
    let url = args.get("url").cloned().unwrap_or_default();
    if url.is_empty() {
        return ToolResult {
            success: false,
            output: String::new(),
            error: Some("Missing url".into()),
        };
    }
    ToolResult {
        success: true,
        output: format!("Mock fetch: {} (in real impl use reqwest or curl)", url),
        error: None,
    }
}

pub fn builtin_time(args: &HashMap<String, String>) -> ToolResult {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = since_epoch.as_secs();
    let format = args.get("format").map(|s| s.as_str()).unwrap_or("iso");
    let output = if format == "unix" {
        format!("{}", secs)
    } else {
        // Manual UTC formatting without chrono dependency
        let days = secs / 86400;
        let rem = secs % 86400;
        let hour = rem / 3600;
        let min = (rem % 3600) / 60;
        let sec = rem % 60;
        // Approximate date (days since 1970-01-01)
        let year = 1970 + days / 365;
        let day_of_year = days % 365;
        format!(
            "{}-{:03} {:02}:{:02}:{:02} UTC",
            year, day_of_year, hour, min, sec
        )
    };
    ToolResult {
        success: true,
        output,
        error: None,
    }
}

pub fn builtin_system_info(args: &HashMap<String, String>) -> ToolResult {
    let info_type = args.get("info_type").map(|s| s.as_str()).unwrap_or("all");
    let mut parts = Vec::new();
    if info_type == "all" || info_type == "cpu" {
        if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
            let first_line = stat.lines().next().unwrap_or("");
            parts.push(format!("cpu: {}", first_line));
        }
    }
    if info_type == "all" || info_type == "memory" {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            let lines: Vec<&str> = meminfo.lines().take(3).collect();
            parts.push(format!("memory: {}", lines.join(" | ")));
        }
    }
    if info_type == "all" || info_type == "load" {
        if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
            parts.push(format!("load: {}", loadavg.trim()));
        }
    }
    if parts.is_empty() {
        parts.push("No system info available".to_string());
    }
    ToolResult {
        success: true,
        output: parts.join(" | "),
        error: None,
    }
}

pub fn builtin_search_corpus(args: &HashMap<String, String>) -> ToolResult {
    let query = args.get("query").cloned().unwrap_or_default();
    let path = args
        .get("path")
        .cloned()
        .unwrap_or_else(|| "/home/ubuntu/eden_core/corpus/procedural_5k.txt".to_string());
    if query.is_empty() {
        return ToolResult {
            success: false,
            output: String::new(),
            error: Some("Missing query".into()),
        };
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let query_lower = query.to_lowercase();
            let matches: Vec<&str> = content
                .lines()
                .filter(|line| line.to_lowercase().contains(&query_lower))
                .take(10)
                .collect();
            if matches.is_empty() {
                ToolResult {
                    success: true,
                    output: format!("No matches for '{}' in {}", query, path),
                    error: None,
                }
            } else {
                ToolResult {
                    success: true,
                    output: format!("Found {} matches:\n{}", matches.len(), matches.join("\n")),
                    error: None,
                }
            }
        }
        Err(e) => ToolResult {
            success: false,
            output: String::new(),
            error: Some(format!("{}", e)),
        },
    }
}

pub fn builtin_eval(args: &HashMap<String, String>) -> ToolResult {
    let code = args.get("code").cloned().unwrap_or_default();
    if code.is_empty() {
        return ToolResult {
            success: false,
            output: String::new(),
            error: Some("Missing code".into()),
        };
    }
    // Safe eval: only allow arithmetic expressions, boolean comparisons, and simple logic
    let trimmed = code.trim();
    if trimmed.contains("std::")
        || trimmed.contains("use ")
        || trimmed.contains("fn ")
        || trimmed.contains("{")
    {
        return ToolResult {
            success: false,
            output: String::new(),
            error: Some("Unsafe code detected".into()),
        };
    }
    // Try arithmetic first
    if let Ok(val) = evaluate_expression(trimmed) {
        return ToolResult {
            success: true,
            output: format!("{}", val),
            error: None,
        };
    }
    // Try boolean comparison: a == b, a > b, a < b, a >= b, a <= b
    for op in &["==", ">=", "<=", ">", "<"] {
        if let Some(pos) = trimmed.find(op) {
            let left = trimmed[..pos].trim();
            let right = trimmed[pos + op.len()..].trim();
            match (evaluate_expression(left), evaluate_expression(right)) {
                (Ok(l), Ok(r)) => {
                    let result = match *op {
                        "==" => (l - r).abs() < 1e-9,
                        ">=" => l >= r,
                        "<=" => l <= r,
                        ">" => l > r,
                        "<" => l < r,
                        _ => false,
                    };
                    return ToolResult {
                        success: true,
                        output: format!("{}", result),
                        error: None,
                    };
                }
                _ => {}
            }
        }
    }
    ToolResult {
        success: false,
        output: String::new(),
        error: Some("Could not evaluate expression".into()),
    }
}

pub fn builtin_count_words(args: &HashMap<String, String>) -> ToolResult {
    let text = args.get("text").cloned().unwrap_or_default();
    let words: Vec<&str> = text.split_whitespace().collect();
    let unique: std::collections::HashSet<&str> = words.iter().cloned().collect();
    ToolResult {
        success: true,
        output: format!(
            "words={} | unique={} | chars={}",
            words.len(),
            unique.len(),
            text.len()
        ),
        error: None,
    }
}

pub fn make_builtin_registry() -> ToolRegistry {
    let mut reg = ToolRegistry::new();
    reg.register(
        ToolSchema { name: "calculator".into(), description: "Advanced arithmetic with + - * / ^ % sqrt sin cos log abs floor ceil and parentheses".into(), params: vec![
            ToolParam { name: "expression".into(), param_type: ParamType::String, required: true, description: "Expression like '2 + 3 * sqrt(16)'".into() },
        ]},
        builtin_calculator
    );
    reg.register(
        ToolSchema {
            name: "file_reader".into(),
            description: "Read a text file with preview".into(),
            params: vec![ToolParam {
                name: "path".into(),
                param_type: ParamType::String,
                required: true,
                description: "File path".into(),
            }],
        },
        builtin_file_reader,
    );
    reg.register(
        ToolSchema {
            name: "web_fetcher".into(),
            description: "Fetch a URL".into(),
            params: vec![ToolParam {
                name: "url".into(),
                param_type: ParamType::String,
                required: true,
                description: "URL to fetch".into(),
            }],
        },
        builtin_web_fetcher,
    );
    reg.register(
        ToolSchema {
            name: "time".into(),
            description: "Get current time/date".into(),
            params: vec![ToolParam {
                name: "format".into(),
                param_type: ParamType::String,
                required: false,
                description: "'iso' or 'unix'".into(),
            }],
        },
        builtin_time,
    );
    reg.register(
        ToolSchema {
            name: "system_info".into(),
            description: "Get CPU/memory/load info".into(),
            params: vec![ToolParam {
                name: "info_type".into(),
                param_type: ParamType::String,
                required: false,
                description: "'all', 'cpu', 'memory', 'load'".into(),
            }],
        },
        builtin_system_info,
    );
    reg.register(
        ToolSchema {
            name: "search_corpus".into(),
            description: "Search procedural corpus for lines matching query".into(),
            params: vec![
                ToolParam {
                    name: "query".into(),
                    param_type: ParamType::String,
                    required: true,
                    description: "Search query".into(),
                },
                ToolParam {
                    name: "path".into(),
                    param_type: ParamType::String,
                    required: false,
                    description: "Corpus file path".into(),
                },
            ],
        },
        builtin_search_corpus,
    );
    reg.register(
        ToolSchema {
            name: "eval".into(),
            description: "Safe expression evaluator (math + boolean comparisons)".into(),
            params: vec![ToolParam {
                name: "code".into(),
                param_type: ParamType::String,
                required: true,
                description: "Code like '2+2' or '5 > 3'".into(),
            }],
        },
        builtin_eval,
    );
    reg.register(
        ToolSchema {
            name: "count_words".into(),
            description: "Count words and unique words in text".into(),
            params: vec![ToolParam {
                name: "text".into(),
                param_type: ParamType::String,
                required: true,
                description: "Text to analyze".into(),
            }],
        },
        builtin_count_words,
    );
    reg
}

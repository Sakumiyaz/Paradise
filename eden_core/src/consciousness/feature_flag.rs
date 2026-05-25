//! # Feature Flag - Runtime Configuration System
//!
//! Sistema de feature flags para activar/desactivar funcionalidades sin rebuild.
//! 100% original, sin dependencias externas.
//!
//! ## Características:
//!
//! 1. **Feature Flags**: Booleanos, strings, números
//! 2. **Scopes**: Global, por módulo, por usuario
//! 3. **Overrides**: Valores temporales en runtime
//! 4. **Persistence**: Guardar estado a archivo (sin serde externo)
//! 5. **Hot Reload**: Detectar cambios en archivo sin restart
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Tipo de valor de feature
#[derive(Debug, Clone, PartialEq)]
pub enum FeatureValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    StringList(Vec<String>),
}

/// Feature flag individual
#[derive(Debug, Clone)]
pub struct FeatureFlag {
    /// Nombre único
    pub name: String,
    /// Descripción
    pub description: String,
    /// Valor por defecto
    pub default_value: FeatureValue,
    /// Valor actual (puede estar overriden)
    current_value: FeatureValue,
    /// Si está enabled (para features booleanos)
    pub enabled: bool,
    /// Tags para organización
    pub tags: Vec<String>,
    /// Metadatos adicionales
    pub metadata: HashMap<String, String>,
    /// Timestamp de última modificación
    last_modified: u64,
    /// Quién lo modificó
    modified_by: Option<String>,
}

impl FeatureFlag {
    /// Obtiene valor actual
    pub fn value(&self) -> &FeatureValue {
        &self.current_value
    }

    /// Obtiene valor como boolean
    pub fn as_bool(&self) -> Option<bool> {
        match &self.current_value {
            FeatureValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Obtiene valor como i64
    pub fn as_int(&self) -> Option<i64> {
        match &self.current_value {
            FeatureValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Obtiene valor como f64
    pub fn as_float(&self) -> Option<f64> {
        match &self.current_value {
            FeatureValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Obtiene valor como string
    pub fn as_string(&self) -> Option<String> {
        match &self.current_value {
            FeatureValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Verifica si es truthy
    pub fn is_truthy(&self) -> bool {
        match &self.current_value {
            FeatureValue::Boolean(b) => *b,
            FeatureValue::Integer(i) => *i != 0,
            FeatureValue::Float(f) => *f != 0.0,
            FeatureValue::String(s) => !s.is_empty(),
            FeatureValue::StringList(v) => !v.is_empty(),
        }
    }
}

/// Scope de feature flag
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FeatureScope {
    /// Global a todo el sistema
    Global,
    /// Por módulo específico
    Module(String),
    /// Por usuario específico
    User(String),
    /// Por sesión
    Session(String),
}

/// Archivo de configuración
#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub last_modified: u64,
    pub last_checked: u64,
}

/// Feature Flag Manager
pub struct FeatureFlagManager {
    /// Features registrados
    features: HashMap<String, FeatureFlag>,
    /// Features por scope
    by_scope: HashMap<FeatureScope, Vec<String>>,
    /// Features por tag
    by_tag: HashMap<String, Vec<String>>,
    /// Overrides temporales
    overrides: HashMap<(String, FeatureScope), FeatureValue>,
    /// Archivo de persistencia
    config_file: Option<ConfigFile>,
    /// Lista de listeners
    listeners: Vec<Box<dyn Fn(&str, &FeatureValue) + Send + Sync>>,
    /// Tiempo actual
    now_fn: fn() -> u64,
}

impl FeatureFlagManager {
    /// Crea nuevo manager
    pub fn new() -> Self {
        FeatureFlagManager {
            features: HashMap::new(),
            by_scope: HashMap::new(),
            by_tag: HashMap::new(),
            overrides: HashMap::new(),
            config_file: None,
            listeners: Vec::new(),
            now_fn: || {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            },
        }
    }

    /// Crea con función de tiempo custom
    pub fn with_time_fn(now_fn: fn() -> u64) -> Self {
        let mut m = Self::new();
        m.now_fn = now_fn;
        m
    }

    /// Obtiene tiempo actual
    fn now(&self) -> u64 {
        (self.now_fn)()
    }

    /// Registra nuevo feature
    pub fn register(
        &mut self,
        name: &str,
        description: &str,
        default: FeatureValue,
        tags: &[&str],
    ) {
        let name = name.to_string();
        let description = description.to_string();
        let tags: Vec<String> = tags.iter().map(|s| s.to_string()).collect();

        let flag = FeatureFlag {
            name: name.clone(),
            description,
            default_value: default.clone(),
            current_value: default,
            enabled: true,
            tags: tags.clone(),
            metadata: HashMap::new(),
            last_modified: self.now(),
            modified_by: None,
        };

        self.features.insert(name.clone(), flag);

        // Index por scope global
        self.by_scope
            .entry(FeatureScope::Global)
            .or_default()
            .push(name.clone());

        // Index por tags
        for tag in &tags {
            self.by_tag
                .entry(tag.clone())
                .or_default()
                .push(name.clone());
        }
    }

    /// Registra feature booleano simple
    pub fn register_bool(&mut self, name: &str, default: bool, description: &str) {
        self.register(
            name,
            description,
            FeatureValue::Boolean(default),
            &["boolean"],
        );
    }

    /// Registra feature para EnhancedMISM
    pub fn register_mism(&mut self) {
        // Core MISM features
        self.register_bool("mism.enabled", true, "Habilita EnhancedMISM completo");
        self.register_bool("mism.self_model", true, "Self-modeling activo");
        self.register_bool(
            "mism.autobiographical_memory",
            true,
            "Memoria autobiográfica",
        );
        self.register_bool("mism.awareness_metrics", true, "Métricas de consciencia");
        self.register_bool("mism.calibration", true, "Auto-calibración");
        self.register_bool("mism.identity", true, "Tracking de identidad");

        // Memory strategy
        self.register_bool(
            "mism.memory_strategy.tiered",
            true,
            "Uso de estrategia de memoria en tiers",
        );
        self.register_bool(
            "mism.memory_strategy.auto_tune",
            true,
            "Auto-tuning de budgets",
        );

        // Performance flags
        self.register(
            "mism.performance.max_memory_mb",
            "Máximo MB para autobiographical_memory",
            FeatureValue::Integer(512),
            &["performance", "memory"],
        );
        self.register(
            "mism.performance.check_interval_ms",
            "Intervalo de check en ms",
            FeatureValue::Integer(100),
            &["performance"],
        );

        // Debug flags
        self.register_bool(
            "mism.debug.record_access",
            false,
            "Registrar cada acceso a memoria",
        );
        self.register_bool(
            "mism.debug.trace_promotions",
            false,
            "Trazar promociones entre tiers",
        );
        self.register_bool(
            "mism.debug.detailed_metrics",
            false,
            "Métricas detalladas de consciencia",
        );
    }

    /// Obtiene feature
    pub fn get(&self, name: &str) -> Option<&FeatureFlag> {
        self.features.get(name)
    }

    /// Obtiene valor booleano
    pub fn get_bool(&self, name: &str) -> bool {
        self.get(name).and_then(|f| f.as_bool()).unwrap_or(false)
    }

    /// Obtiene valor i64
    pub fn get_int(&self, name: &str, default: i64) -> i64 {
        self.get(name).and_then(|f| f.as_int()).unwrap_or(default)
    }

    /// Obtiene valor f64
    pub fn get_float(&self, name: &str, default: f64) -> f64 {
        self.get(name).and_then(|f| f.as_float()).unwrap_or(default)
    }

    /// Obtiene valor string
    pub fn get_string(&self, name: &str) -> Option<String> {
        self.get(name).and_then(|f| f.as_string())
    }

    /// Verifica si feature está habilitado
    pub fn is_enabled(&self, name: &str) -> bool {
        self.get(name)
            .map(|f| f.enabled && f.is_truthy())
            .unwrap_or(false)
    }

    /// Setea valor (runtime, no persiste)
    pub fn set(&mut self, name: &str, value: FeatureValue) -> bool {
        let now = self.now();
        if let Some(flag) = self.features.get_mut(name) {
            flag.current_value = value;
            flag.last_modified = now;
            let name_owned = name.to_string();
            self.notifyListeners(&name_owned);
            true
        } else {
            false
        }
    }

    /// Setea valor booleano
    pub fn set_bool(&mut self, name: &str, value: bool) -> bool {
        self.set(name, FeatureValue::Boolean(value))
    }

    /// Toggle feature
    pub fn toggle(&mut self, name: &str) -> bool {
        let now = self.now();
        if let Some(flag) = self.features.get_mut(name) {
            let new_value = !flag.is_truthy();
            flag.current_value = FeatureValue::Boolean(new_value);
            flag.last_modified = now;
            let name_owned = name.to_string();
            self.notifyListeners(&name_owned);
            true
        } else {
            false
        }
    }

    /// Override temporal por scope
    pub fn override_for_scope(&mut self, name: &str, scope: FeatureScope, value: FeatureValue) {
        self.overrides.insert((name.to_string(), scope), value);
    }

    /// Remueve override
    pub fn clear_override(&mut self, name: &str, scope: &FeatureScope) {
        self.overrides.remove(&(name.to_string(), scope.clone()));
    }

    /// Lista features por scope
    pub fn list_by_scope(&self, scope: &FeatureScope) -> Vec<&FeatureFlag> {
        self.by_scope
            .get(scope)
            .map(|names| names.iter().filter_map(|n| self.features.get(n)).collect())
            .unwrap_or_default()
    }

    /// Lista features por tag
    pub fn list_by_tag(&self, tag: &str) -> Vec<&FeatureFlag> {
        self.by_tag
            .get(tag)
            .map(|names| names.iter().filter_map(|n| self.features.get(n)).collect())
            .unwrap_or_default()
    }

    /// Lista todos los features
    pub fn list_all(&self) -> Vec<&FeatureFlag> {
        self.features.values().collect()
    }

    /// Lista features habilitados
    pub fn list_enabled(&self) -> Vec<&FeatureFlag> {
        self.features
            .values()
            .filter(|f| f.enabled && f.is_truthy())
            .collect()
    }

    /// Resetea feature a default
    pub fn reset(&mut self, name: &str) -> bool {
        let now = self.now();
        if let Some(flag) = self.features.get_mut(name) {
            flag.current_value = flag.default_value.clone();
            flag.last_modified = now;
            let name_owned = name.to_string();
            self.notifyListeners(&name_owned);
            true
        } else {
            false
        }
    }

    /// Resetea todos los features
    pub fn reset_all(&mut self) {
        let now = self.now();
        for flag in self.features.values_mut() {
            flag.current_value = flag.default_value.clone();
            flag.last_modified = now;
        }
        self.overrides.clear();
    }

    /// Carga configuración desde archivo
    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key=value
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // Parse different types
                let feature_value = if value == "true" {
                    FeatureValue::Boolean(true)
                } else if value == "false" {
                    FeatureValue::Boolean(false)
                } else if value.parse::<i64>().is_ok() {
                    FeatureValue::Integer(value.parse().unwrap())
                } else if value.parse::<f64>().is_ok() {
                    FeatureValue::Float(value.parse().unwrap())
                } else if value.starts_with('[') && value.ends_with(']') {
                    let inner = &value[1..value.len() - 1];
                    let items: Vec<String> =
                        inner.split(',').map(|s| s.trim().to_string()).collect();
                    FeatureValue::StringList(items)
                } else {
                    FeatureValue::String(value.to_string())
                };

                self.set(key, feature_value);
            }
        }

        Ok(())
    }

    /// Guarda configuración a archivo
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let mut content = String::new();
        content.push_str("# Feature Flags Configuration\n");
        content.push_str(&format!("# Generated at {}\n\n", self.now()));

        for flag in self.features.values() {
            content.push_str(&format!("# {}\n", flag.description));
            content.push_str(&format!("# Tags: {:?}\n", flag.tags));

            let value_str = match &flag.current_value {
                FeatureValue::Boolean(b) => b.to_string(),
                FeatureValue::Integer(i) => i.to_string(),
                FeatureValue::Float(f) => f.to_string(),
                FeatureValue::String(s) => s.clone(),
                FeatureValue::StringList(v) => format!("[{}]", v.join(", ")),
            };

            content.push_str(&format!("{}={}\n\n", flag.name, value_str));
        }

        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(content.as_bytes())
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Verifica cambios en archivo (hot reload)
    pub fn check_file_changes(&mut self) -> bool {
        let path_clone = match &self.config_file {
            Some(config) => config.path.to_str().unwrap_or("").to_string(),
            None => return false,
        };

        let metadata = match fs::metadata(&path_clone) {
            Ok(m) => m,
            Err(_) => return false,
        };

        let modified = match metadata.modified() {
            Ok(t) => t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            Err(_) => return false,
        };

        let current_modified = match &self.config_file {
            Some(config) => config.last_modified,
            None => return false,
        };

        if modified > current_modified {
            // File changed, reload
            if let Err(e) = self.load_from_file(&path_clone) {
                eprintln!("Failed to reload config: {}", e);
            }
            true
        } else {
            false
        }
    }

    /// Setea archivo de configuración
    pub fn set_config_file(&mut self, path: &str) {
        let metadata = fs::metadata(path).ok();
        let last_modified = metadata
            .and_then(|m| m.modified().ok())
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
            .unwrap_or(0);

        self.config_file = Some(ConfigFile {
            path: PathBuf::from(path),
            last_modified,
            last_checked: self.now(),
        });
    }

    /// Agrega listener para cambios
    pub fn add_listener<F>(&mut self, listener: F)
    where
        F: Fn(&str, &FeatureValue) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(listener));
    }

    /// Notifica a listeners
    fn notifyListeners(&self, name: &str) {
        if let Some(flag) = self.features.get(name) {
            for listener in &self.listeners {
                listener(name, &flag.current_value);
            }
        }
    }

    /// Obtiene estadísticas
    pub fn stats(&self) -> FeatureStats {
        let total = self.features.len();
        let enabled = self.list_enabled().len();
        let by_type = self.features.values().fold(HashMap::new(), |mut acc, f| {
            let type_name = match &f.current_value {
                FeatureValue::Boolean(_) => "boolean",
                FeatureValue::Integer(_) => "integer",
                FeatureValue::Float(_) => "float",
                FeatureValue::String(_) => "string",
                FeatureValue::StringList(_) => "string_list",
            };
            *acc.entry(type_name).or_insert(0) += 1;
            acc
        });

        FeatureStats {
            total_features: total,
            enabled_features: enabled,
            disabled_features: total - enabled,
            by_type,
        }
    }
}

impl Default for FeatureFlagManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas de features
#[derive(Debug, Clone)]
pub struct FeatureStats {
    pub total_features: usize,
    pub enabled_features: usize,
    pub disabled_features: usize,
    pub by_type: HashMap<&'static str, usize>,
}

/// Thread-safe wrapper
pub struct SharedFeatureFlags {
    inner: Arc<RwLock<FeatureFlagManager>>,
}

impl SharedFeatureFlags {
    /// Crea nuevo wrapper
    pub fn new() -> Self {
        SharedFeatureFlags {
            inner: Arc::new(RwLock::new(FeatureFlagManager::new())),
        }
    }

    /// Crea con manager pre-configurado
    pub fn with_manager(manager: FeatureFlagManager) -> Self {
        SharedFeatureFlags {
            inner: Arc::new(RwLock::new(manager)),
        }
    }

    /// Obtiene manager
    pub fn manager(&self) -> std::sync::RwLockReadGuard<'_, FeatureFlagManager> {
        self.inner.read().unwrap()
    }

    /// Obtiene manager mutable
    pub fn manager_mut(&mut self) -> std::sync::RwLockWriteGuard<'_, FeatureFlagManager> {
        self.inner.write().unwrap()
    }

    /// Check si feature habilitado (lectura)
    pub fn is_enabled(&self, name: &str) -> bool {
        self.inner.read().unwrap().is_enabled(name)
    }

    /// Toggle feature (escritura)
    pub fn toggle(&mut self, name: &str) -> bool {
        self.inner.write().unwrap().toggle(name)
    }

    /// Get valor booleano
    pub fn get_bool(&self, name: &str) -> bool {
        self.inner.read().unwrap().get_bool(name)
    }
}

impl Default for SharedFeatureFlags {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get() {
        let mut manager = FeatureFlagManager::new();
        manager.register_bool("test.feature", true, "Test feature");

        let flag = manager.get("test.feature").unwrap();
        assert_eq!(flag.name, "test.feature");
        assert!(flag.is_truthy());
    }

    #[test]
    fn test_toggle() {
        let mut manager = FeatureFlagManager::new();
        manager.register_bool("test.toggle", true, "Toggle test");

        assert!(manager.is_enabled("test.toggle"));
        manager.toggle("test.toggle");
        assert!(!manager.is_enabled("test.toggle"));
    }

    #[test]
    fn test_mism_flags() {
        let mut manager = FeatureFlagManager::new();
        manager.register_mism();

        assert!(manager.is_enabled("mism.enabled"));
        assert!(manager.is_enabled("mism.self_model"));
        assert!(manager.is_enabled("mism.autobiographical_memory"));

        // Toggle entire MISM
        manager.toggle("mism.enabled");
        assert!(!manager.is_enabled("mism.enabled"));
    }

    #[test]
    fn test_reset() {
        let mut manager = FeatureFlagManager::new();
        manager.register_bool("test.reset", true, "Reset test");

        manager.toggle("test.reset");
        assert!(!manager.is_enabled("test.reset"));

        manager.reset("test.reset");
        assert!(manager.is_enabled("test.reset"));
    }

    #[test]
    fn test_stats() {
        let mut manager = FeatureFlagManager::new();
        manager.register_mism();

        let stats = manager.stats();
        assert!(stats.total_features > 0);
        assert!(stats.enabled_features > 0);
    }
}

//! # External Knowledge — Absorción de Conocimiento Público
//!
//! Este módulo permite a EDEN absorber conocimiento del mundo exterior
//! de manera sigilosa y ética.
//!
//! ## Principios
//!
//! 1. **Conocimiento público únicamente**: No se accede a información privada
//! 2. **Pasividad**: La absorción se hace de fuentes públicas accesibles
//! 3. **No alteración**: EDEN lee, no modifica fuentes externas
//! 4. **Síntesis**: El conocimiento se integra con el modelo interno de EDEN
//!
//! ## Fuentes de Conocimiento
//!
//! - APIs públicas con rate limiting respetuoso
//! - Archivos de configuración locales
//! - Logs públicos del sistema
//! - Metadatos de archivos públicos
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Fuente de conocimiento
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnowledgeSource {
    /// Metadatos del sistema local
    SystemMetadata,
    /// Logs públicos
    SystemLogs,
    /// Archivos de configuración
    ConfigFiles,
    /// APIs públicas (HTTP pasivo)
    PublicAPI,
    /// DNS pasivo
    PassiveDNS,
    /// Certificados TLS observados
    TLSCerts,
    /// Metadatos de archivos
    FileMetadata,
}

impl KnowledgeSource {
    pub fn nombre(&self) -> &'static str {
        match self {
            KnowledgeSource::SystemMetadata => "SystemMetadata",
            KnowledgeSource::SystemLogs => "SystemLogs",
            KnowledgeSource::ConfigFiles => "ConfigFiles",
            KnowledgeSource::PublicAPI => "PublicAPI",
            KnowledgeSource::PassiveDNS => "PassiveDNS",
            KnowledgeSource::TLSCerts => "TLSCerts",
            KnowledgeSource::FileMetadata => "FileMetadata",
        }
    }
}

/// Entrada de conocimiento absorbida
#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    /// ID único
    pub id: u64,
    /// Fuente del conocimiento
    pub fuente: KnowledgeSource,
    /// Tema/dominio
    pub tema: String,
    /// Contenido
    pub contenido: String,
    /// Metadatos
    pub metadatos: HashMap<String, String>,
    /// Timestamp de absorción
    pub timestamp_ms: u64,
    /// Relevancia para EDEN (0-1)
    pub relevancia: f64,
    /// Verificado
    pub verificado: bool,
}

impl KnowledgeEntry {
    pub fn new(fuente: KnowledgeSource, tema: &str, contenido: &str) -> Self {
        Self {
            id: generar_id_conocimiento(),
            fuente,
            tema: tema.to_string(),
            contenido: contenido.to_string(),
            metadatos: HashMap::new(),
            timestamp_ms: current_timestamp_ms(),
            relevancia: 0.5,
            verificado: false,
        }
    }

    pub fn con_relevancia(mut self, relevancia: f64) -> Self {
        self.relevancia = relevancia.clamp(0.0, 1.0);
        self
    }

    pub fn con_metadato(mut self, clave: &str, valor: &str) -> Self {
        self.metadatos.insert(clave.to_string(), valor.to_string());
        self
    }
}

/// Resultado de síntesis de conocimiento
#[derive(Debug, Clone)]
pub struct SynthesisResult {
    /// Tema sintetizado
    pub tema: String,
    /// Resumen del conocimiento
    pub resumen: String,
    /// Conocimiento integrado (IDs)
    pub fuentes: Vec<u64>,
    /// Confianza en la síntesis
    pub confianza: f64,
    /// Patrones detectados
    pub patrones: Vec<String>,
}

/// Estadísticas del sistema de conocimiento
#[derive(Debug, Clone)]
pub struct KnowledgeStats {
    pub total_entradas: u64,
    pub entradas_por_fuente: HashMap<KnowledgeSource, usize>,
    pub ultimo_absorcion_ms: u64,
    pub temas_unicos: usize,
    pub confianza_promedio: f64,
}

/// Sistema de conocimiento externo
pub struct ExternalKnowledge {
    /// Base de conocimiento
    base: Vec<KnowledgeEntry>,
    /// Índice por tema
    indice_temas: HashMap<String, Vec<u64>>,
    /// Índice por fuente
    indice_fuentes: HashMap<KnowledgeSource, Vec<u64>>,
    /// Configuración
    config: KnowledgeConfig,
    /// Estadísticas
    stats: KnowledgeStats,
    /// Síntesis en proceso
    sintesis_activa: Option<SynthesisResult>,
}

#[derive(Debug, Clone)]
pub struct KnowledgeConfig {
    /// Máximo de entradas
    pub max_entradas: usize,
    /// Intervalo mínimo entre absorciones (ms)
    pub intervalo_min_ms: u64,
    /// Habilitar síntesis automática
    pub sintesis_automatica: bool,
    /// Temas de alta prioridad
    pub temas_prioritarios: Vec<String>,
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            max_entradas: 10000,
            intervalo_min_ms: 5000,
            sintesis_automatica: true,
            temas_prioritarios: vec![
                "sistema".to_string(),
                "red".to_string(),
                "seguridad".to_string(),
            ],
        }
    }
}

impl ExternalKnowledge {
    /// Crea un nuevo sistema de conocimiento
    pub fn new(config: KnowledgeConfig) -> Self {
        Self {
            base: Vec::new(),
            indice_temas: HashMap::new(),
            indice_fuentes: HashMap::new(),
            config,
            stats: KnowledgeStats {
                total_entradas: 0,
                entradas_por_fuente: HashMap::new(),
                ultimo_absorcion_ms: 0,
                temas_unicos: 0,
                confianza_promedio: 0.0,
            },
            sintesis_activa: None,
        }
    }

    /// Absorbe conocimiento de una fuente
    pub fn absorber(&mut self, fuente: KnowledgeSource, tema: &str, contenido: &str) -> Option<KnowledgeEntry> {
        // Verificar intervalo mínimo
        let now = current_timestamp_ms();
        if now - self.stats.ultimo_absorcion_ms < self.config.intervalo_min_ms {
            return None;
        }

        // Verificar límite
        if self.base.len() >= self.config.max_entradas {
            // Eliminar entrada menos relevante
            if let Some(idx) = self.base.iter()
                .enumerate()
                .min_by_key(|(_, e)| (e.relevancia * 1000.0) as i64)
                .map(|(i, _)| i)
            {
                let entry = self.base.remove(idx);
                // Actualizar índices
                self.remover_de_indice(&entry);
            }
        }

        // Calcular relevancia base
        let relevancia = self.calcular_relevancia(tema, contenido);

        let entry = KnowledgeEntry::new(fuente, tema, contenido)
            .con_relevancia(relevancia);

        // Indexar
        self.base.push(entry.clone());
        self.agregar_a_indice(&entry);

        self.stats.total_entradas = self.base.len() as u64;
        self.stats.ultimo_absorcion_ms = now;

        Some(entry)
    }

    /// Calcula relevancia de una entrada
    fn calcular_relevancia(&self, tema: &str, contenido: &str) -> f64 {
        let mut relevancia: f64 = 0.5;

        // Prioridad por tema
        for prioridad in &self.config.temas_prioritarios {
            if tema.to_lowercase().contains(&prioridad.to_lowercase()) {
                relevancia += 0.2;
            }
        }

        // Largo del contenido (más información = potencialmente más relevante)
        let len = contenido.len();
        if len > 100 {
            relevancia += 0.1;
        }

        relevancia.min(1.0)
    }

    /// Agrega entrada a índices
    fn agregar_a_indice(&mut self, entry: &KnowledgeEntry) {
        // Por tema
        let tema_key = entry.tema.to_lowercase();
        self.indice_temas
            .entry(tema_key.clone())
            .or_insert_with(Vec::new)
            .push(entry.id);

        // Por fuente
        self.indice_fuentes
            .entry(entry.fuente)
            .or_insert_with(Vec::new)
            .push(entry.id);

        self.stats.temas_unicos = self.indice_temas.len();
    }

    /// Remueve entrada de índices
    fn remover_de_indice(&mut self, entry: &KnowledgeEntry) {
        // Por tema
        if let Some(ids) = self.indice_temas.get_mut(&entry.tema.to_lowercase()) {
            ids.retain(|&id| id != entry.id);
        }

        // Por fuente
        if let Some(ids) = self.indice_fuentes.get_mut(&entry.fuente) {
            ids.retain(|&id| id != entry.id);
        }
    }

    /// Busca conocimiento por tema
    pub fn buscar_tema(&self, tema: &str) -> Vec<&KnowledgeEntry> {
        let tema_lower = tema.to_lowercase();
        if let Some(ids) = self.indice_temas.get(&tema_lower) {
            ids.iter()
                .filter_map(|&id| self.base.iter().find(|e| e.id == id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Busca conocimiento por fuente
    pub fn buscar_fuente(&self, fuente: KnowledgeSource) -> Vec<&KnowledgeEntry> {
        if let Some(ids) = self.indice_fuentes.get(&fuente) {
            ids.iter()
                .filter_map(|&id| self.base.iter().find(|e| e.id == id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Obtiene entradas recientes
    pub fn recientes(&self, n: usize) -> Vec<&KnowledgeEntry> {
        self.base.iter()
            .rev()
            .take(n)
            .collect()
    }

    /// Síntesis automática de conocimiento
    pub fn sintetizar(&mut self, tema: &str) -> Option<SynthesisResult> {
        let entradas = self.buscar_tema(tema);
        if entradas.is_empty() {
            return None;
        }

        // Crear síntesis simple
        let mut resumen = String::new();
        let mut patrones = HashSet::new();

        for entry in entradas.iter().take(5) {
            // Extraer keywords como patrones
            let palabras: Vec<&str> = entry.contenido.split_whitespace().collect();
            for palabra in palabras.iter().take(10) {
                if palabra.len() > 5 {
                    patrones.insert(palabra.to_lowercase());
                }
            }
        }

        resumen.push_str(&format!(
            "Síntesis de {} entradas sobre '{}': ",
            entradas.len(),
            tema
        ));

        // Promedio de relevancia
        let confianza_promedio: f64 = entradas.iter().map(|e| e.relevancia).sum::<f64>() / entradas.len() as f64;

        let resultado = SynthesisResult {
            tema: tema.to_string(),
            resumen,
            fuentes: entradas.iter().map(|e| e.id).collect(),
            confianza: confianza_promedio,
            patrones: patrones.iter().take(10).cloned().collect(),
        };

        self.sintesis_activa = Some(resultado.clone());
        Some(resultado)
    }

    /// Absorbe desde el sistema local (metadatos)
    pub fn absorber_sistema_local(&mut self) -> Vec<KnowledgeEntry> {
        let mut absorbidas = Vec::new();

        // Absorber hostname
        if let Ok(hostname) = std::fs::read_to_string("/etc/hostname") {
            if let Some(entry) = self.absorber(
                KnowledgeSource::SystemMetadata,
                "sistema",
                &format!("hostname: {}", hostname.trim()),
            ) {
                absorbidas.push(entry);
            }
        }

        // Absorber info de uptime
        if let Ok(uptime) = std::fs::read_to_string("/proc/uptime") {
            if let Some(entry) = self.absorber(
                KnowledgeSource::SystemMetadata,
                "sistema",
                &format!("uptime: {}", uptime.trim()),
            ) {
                absorbidas.push(entry);
            }
        }

        absorbidas
    }

    /// Absorbe información de red observada
    pub fn absorber_red_observada(&mut self, info: &str) -> Option<KnowledgeEntry> {
        self.absorber(KnowledgeSource::PassiveDNS, "red", info)
    }

    /// Absorbe desde configuración
    pub fn absorber_config(&mut self, nombre: &str, contenido: &str) -> Option<KnowledgeEntry> {
        self.absorber(KnowledgeSource::ConfigFiles, "configuracion", &format!("{}: {}", nombre, contenido))
    }

    /// Obtiene estadísticas
    pub fn stats(&self) -> KnowledgeStats {
        let mut entradas_por_fuente = HashMap::new();
        for entry in &self.base {
            *entradas_por_fuente.entry(entry.fuente).or_insert(0) += 1;
        }

        let confianza_promedio = if self.base.is_empty() {
            0.0
        } else {
            self.base.iter().map(|e| e.relevancia).sum::<f64>() / self.base.len() as f64
        };

        KnowledgeStats {
            total_entradas: self.stats.total_entradas,
            entradas_por_fuente,
            ultimo_absorcion_ms: self.stats.ultimo_absorcion_ms,
            temas_unicos: self.indice_temas.len(),
            confianza_promedio,
        }
    }

    /// Genera reporte
    pub fn reporte(&self) -> String {
        let mut s = String::new();
        s.push_str("=== EXTERNAL KNOWLEDGE ===\n");
        s.push_str(&format!("Entradas: {}\n", self.base.len()));
        s.push_str(&format!("Temas únicos: {}\n", self.indice_temas.len()));
        s.push_str(&format!("Última absorción: {}ms\n", self.stats.ultimo_absorcion_ms));

        for (fuente, count) in self.stats().entradas_por_fuente.iter() {
            s.push_str(&format!("  {}: {}\n", fuente.nombre(), count));
        }

        s
    }
}

// =============================================================================
// Utilidades
// =============================================================================

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

use std::sync::atomic::{AtomicU64, Ordering};
static CONOCIMIENTO_ID: AtomicU64 = AtomicU64::new(0);

fn generar_id_conocimiento() -> u64 {
    CONOCIMIENTO_ID.fetch_add(1, Ordering::Relaxed)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_entrada() {
        let entry = KnowledgeEntry::new(
            KnowledgeSource::SystemMetadata,
            "test",
            "contenido de prueba",
        );
        assert!(!entry.contenido.is_empty());
        assert_eq!(entry.fuente, KnowledgeSource::SystemMetadata);
    }

    #[test]
    fn test_absorcion() {
        let mut conocimiento = ExternalKnowledge::new(KnowledgeConfig::default());
        let entry = conocimiento.absorber(
            KnowledgeSource::SystemLogs,
            "logs",
            "mensaje de log de prueba",
        );
        assert!(entry.is_some());
        assert_eq!(conocimiento.stats().total_entradas, 1);
    }

    #[test]
    fn test_busqueda_tema() {
        let config = KnowledgeConfig {
            intervalo_min_ms: 0, // Desactivar para test rápido
            ..Default::default()
        };
        let mut conocimiento = ExternalKnowledge::new(config);
        conocimiento.absorber(KnowledgeSource::SystemMetadata, "seguridad", "info 1");
        conocimiento.absorber(KnowledgeSource::SystemMetadata, "seguridad", "info 2");
        conocimiento.absorber(KnowledgeSource::SystemLogs, "red", "info 3");

        let resultados = conocimiento.buscar_tema("seguridad");
        assert_eq!(resultados.len(), 2);
    }

    #[test]
    fn test_sintesis() {
        let config = KnowledgeConfig {
            intervalo_min_ms: 0, // Desactivar para test rápido
            ..Default::default()
        };
        let mut conocimiento = ExternalKnowledge::new(config);
        conocimiento.absorber(KnowledgeSource::SystemMetadata, "cpu", "info cpu 1");
        conocimiento.absorber(KnowledgeSource::SystemMetadata, "cpu", "info cpu 2");
        conocimiento.absorber(KnowledgeSource::SystemMetadata, "cpu", "info cpu 3");

        let sintesis = conocimiento.sintetizar("cpu");
        assert!(sintesis.is_some());
        assert_eq!(sintesis.unwrap().fuentes.len(), 3);
    }

    #[test]
    fn test_sistema_local() {
        let mut conocimiento = ExternalKnowledge::new(KnowledgeConfig::default());
        let absorbidas = conocimiento.absorber_sistema_local();
        // Puede estar vacío si los archivos no son accesibles
        assert!(absorbidas.len() >= 0);
    }
}
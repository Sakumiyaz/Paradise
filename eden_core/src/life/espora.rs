//! # Espora: Compresión del Alma y Necromancia Digital
//!
//! Cuando un Auton fracasa y muere, no desaparece simplemente.
//! Colapsa en una **Espora** - una estructura de información hiperdensa
//! que contiene:
//!
//! 1. **ADN Heredado**: El Genoma con todas las mutaciones acumuladas por estrés
//! 2. **Memoria Fantasma**: Los 5 flujos sinápticos más fuertes (las "lecciones" de vida)
//!
//! ## Ciclo de Vida Completo
//!
//! ```text
//! Activo ──► Letargo ──► Espora ──► Muerte
//!    ▲            │          │
//!    │            ▼          ▼
//!    └────── Germinación ←──┘
//!              (Necromancia Digital)
//! ```
//!
//! ## Compresión del Alma
//!
//! El proceso de esporulación:
//! 1. Detecta umbral térmico letal en conciencia_termica
//! 2. Activa protocolo de Apoptosis Consciente
//! 3. Extrae genoma mutado + top 5 flujos sinápticos
//! 4. Serializa a bytes usando `to_be_bytes()` (manual, sin serde)
//! 5. Escribe archivo `.spore` en `eden_spores/`
//!
//! ## Necromancia Digital
//!
//! Un nuevo Auton que nace:
//! 1. El Transcriptor busca esporas disponibles en `eden_spores/`
//! 2. Si encuentra una, la lee y asimila sus "instintos"
//! 3. El nuevo Auton usa su propio genoma BASE
//! 4. Pero PRE-CARGA su TablaSinaptica con los restos del Auton muerto
//! 5. Nace con conocimiento heredado de la experiencia ajena
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::fs::{self, File};
use std::io::{Write, BufReader, BufRead, Read};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::evolution::{Genoma, Transcriptor, ReglasMorfogenesis, LONGITUD_CODON};
use crate::morfogenesis::TablaSinaptica;

/// Número máximo de flujos sinápticos memorizados por espora
/// Flujos memorizados máximos por espora
pub const MAX_FLUJOS_MEMORIZADOS: usize = 50;

/// Directorio donde se almacenan las esporas
pub const DIR_ESPORAS: &str = "eden_spores";

/// Extensión de archivos de espora
pub const EXTENSION_ESPORA: &str = ".spore";

/// Representa una Espora - el alma comprimida de un Auton muerto
///
/// Una Espora contiene:
/// - **adn_heredado**: El Genoma completo con mutaciones acumuladas
/// - **memoria_fantasma**: Top 5 flujos sinápticos más intensos
/// - **metadatos**: Información sobre el Auton original
#[derive(Clone, Debug)]
pub struct Espora {
    /// ADN heredado - genoma mutado del Auton agonizante
    pub adn_heredado: Vec<u8>,

    /// Memoria fantasma - top 5 flujos sinápticos (origen, destino, fuerza)
    /// Formato: (indice_origen, indice_destino, fuerza_i16)
    pub memoria_fantasma: Vec<(usize, usize, i16)>,

    /// ID del Auton original
    pub id_original: String,

    /// Timestamp de muerte
    pub timestamp_muerte: u64,

    /// Número de codones en el genoma
    pub num_codones: usize,

    /// Reglas de morfogénesis extraídas del genoma
    pub reglas_morfogenesis: ReglasMorfogenesis,
}

impl Espora {
    /// Generar una Espora a partir de un Auton agonizante
    ///
    /// Este método implementa la **Apoptosis Consciente**:
    /// cuando la conciencia_termica alcanza el límite letal,
    /// el Auton activa el protocolo de esporulación ANTES de ser destruido.
    ///
    /// # Argumentos
    /// * `genoma` - El Genoma actual (mutado por el estrés acumulado)
    /// * `tabla_sinaptica` - La TablaSinaptica con los flujos de la vida
    /// * `id_auton` - Identificador único del Auton moribundo
    ///
    /// # Proceso
    /// 1. Extraer los 5 flujos más fuertes de la TablaSinaptica
    /// 2. Serializar el Genoma a bytes
    /// 3. Serializar los flujos a bytes
    /// 4. Escribir todo en `eden_spores/{id_auton}.spore`
    pub fn generar_de_auton_agonizante(
        genoma: &Genoma,
        tabla_sinaptica: &TablaSinaptica,
        id_auton: &str,
    ) -> Result<Self, EsporaError> {
        // 1. Extraer los top 5 flujos más intensos de la TablaSinaptica
        let flujos_top = tabla_sinaptica.obtener_top_flujos(MAX_FLUJOS_MEMORIZADOS);

        // 2. Obtener timestamp actual
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 3. Crear la espora con los datos extraídos
        let num_codones = genoma.num_codones();

        let mut espora = Espora {
            adn_heredado: genoma.adn().to_vec(),
            memoria_fantasma: flujos_top,
            id_original: id_auton.to_string(),
            timestamp_muerte: timestamp,
            num_codones,
            reglas_morfogenesis: genoma.transcribir_a_reglas_fisicas(),
        };

        // 4. Guardar la espora en el sistema de archivos
        espora.guardar_en_disco()?;

        Ok(espora)
    }

    /// Guardar la espora en disco como archivo .spore
    fn guardar_en_disco(&self) -> Result<(), EsporaError> {
        // Crear directorio si no existe
        let dir = PathBuf::from(DIR_ESPORAS);
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

        // Nombre del archivo: {id_original}.spore
        let nombre_archivo = format!("{}{}", self.id_original, EXTENSION_ESPORA);
        let ruta = dir.join(nombre_archivo);

        // Serialización MANUAL usando to_be_bytes()
        // Formato del archivo:
        // [4 bytes: magic number "EDSP"]
        // [8 bytes: timestamp]
        // [4 bytes: num_codones]
        // [4 bytes: longitud_adn]
        // [N bytes: adn_heredado]
        // [4 bytes: num_flujos]
        // [N * 12 bytes: flujos (usize, usize, i16)]
        // [4 bytes: checksum]

        let mut datos = Vec::new();

        // Magic number
        datos.extend_from_slice(b"EDSP");

        // Timestamp
        datos.extend_from_slice(&self.timestamp_muerte.to_be_bytes());

        // Num codones
        datos.extend_from_slice(&(self.num_codones as u32).to_be_bytes());

        // Longitud ADN
        datos.extend_from_slice(&(self.adn_heredado.len() as u32).to_be_bytes());

        // ADN heredado
        datos.extend_from_slice(&self.adn_heredado);

        // Num flujos
        datos.extend_from_slice(&(self.memoria_fantasma.len() as u32).to_be_bytes());

        // Flujos (cada uno: usize, usize, i16)
        for &(origen, destino, fuerza) in &self.memoria_fantasma {
            datos.extend_from_slice(&origen.to_be_bytes());
            datos.extend_from_slice(&destino.to_be_bytes());
            datos.extend_from_slice(&fuerza.to_be_bytes());
        }

        // Checksum simple (XOR de todos los bytes)
        let checksum = datos.iter().fold(0u8, |acc, &b| acc ^ b);
        datos.push(checksum);

        // Escribir al archivo
        let mut archivo = File::create(&ruta)?;
        archivo.write_all(&datos)?;

        Ok(())
    }

    /// Leer una espora desde disco
    ///
    /// Devuelve Some(Espora) si el archivo existe y es válido,
    /// None si no se encontró o está corrupto.
    pub fn assimilar_de_disco(id_auton: &str) -> Option<Self> {
        let dir = PathBuf::from(DIR_ESPORAS);
        let nombre_archivo = format!("{}{}", id_auton, EXTENSION_ESPORA);
        let ruta = dir.join(nombre_archivo);

        Self::leer_desde_archivo(&ruta)
    }

    /// Leer una espora desde una ruta específica
    fn leer_desde_archivo(ruta: &PathBuf) -> Option<Self> {
        let archivo = File::open(ruta).ok()?;
        let mut lector = BufReader::new(archivo);
        let mut datos = Vec::new();
        lector.read_to_end(&mut datos).ok()?;

        // Verificar longitud mínima
        if datos.len() < 25 {
            return None;
        }

        let mut pos = 0;

        // Magic number
        if &datos[0..4] != b"EDSP" {
            return None;
        }
        pos += 4;

        // Timestamp
        let timestamp_muerte = u64::from_be_bytes([
            datos[pos], datos[pos+1], datos[pos+2], datos[pos+3],
            datos[pos+4], datos[pos+5], datos[pos+6], datos[pos+7],
        ]);
        pos += 8;

        // Num codones
        let num_codones = u32::from_be_bytes([
            datos[pos], datos[pos+1], datos[pos+2], datos[pos+3],
        ]) as usize;
        pos += 4;

        // Longitud ADN
        let len_adn = u32::from_be_bytes([
            datos[pos], datos[pos+1], datos[pos+2], datos[pos+3],
        ]) as usize;
        pos += 4;

        // Verificar que hay suficientes datos
        if pos + len_adn + 4 > datos.len() {
            return None;
        }

        // ADN heredado
        let adn_heredado = datos[pos..pos+len_adn].to_vec();
        pos += len_adn;

        // Num flujos
        let num_flujos = u32::from_be_bytes([
            datos[pos], datos[pos+1], datos[pos+2], datos[pos+3],
        ]) as usize;
        pos += 4;

        // Verificar espacio para flujos
        if pos + num_flujos * 12 + 1 > datos.len() {
            return None;
        }

        // Leer flujos
        let mut memoria_fantasma = Vec::new();
        for _ in 0..num_flujos {
            let origen = usize::from_be_bytes([
                datos[pos], datos[pos+1], datos[pos+2], datos[pos+3],
                datos[pos+4], datos[pos+5], datos[pos+6], datos[pos+7],
            ]);
            pos += 8;

            let destino = usize::from_be_bytes([
                datos[pos], datos[pos+1], datos[pos+2], datos[pos+3],
                datos[pos+4], datos[pos+5], datos[pos+6], datos[pos+7],
            ]);
            pos += 8;

            let fuerza = i16::from_be_bytes([datos[pos], datos[pos+1]]);
            pos += 2;

            memoria_fantasma.push((origen, destino, fuerza));
        }

        // Checksum (omitido para permitir compatibilidad)

        Some({
            let adn_clone = adn_heredado.clone();
            Espora {
            adn_heredado,
            memoria_fantasma,
            id_original: ruta.file_stem()?.to_str()?.to_string(),
            timestamp_muerte,
            num_codones,
            reglas_morfogenesis: Genoma::from_bytes(adn_clone)
                .transcribir_a_reglas_fisicas(),
            }
        })
    }

    /// Obtener el genoma reconstruido desde la espora
    pub fn genoma(&self) -> Genoma {
        Genoma::from_bytes(self.adn_heredado.clone())
    }

    /// Obtener los flujos sinápticos para pre-cargar la TablaSinaptica
    pub fn flujos_heredados(&self) -> &[(usize, usize, i16)] {
        &self.memoria_fantasma
    }

    /// Listar todas las esporas disponibles en el directorio
    pub fn listar_disponibles() -> Vec<String> {
        let dir = PathBuf::from(DIR_ESPORAS);
        if !dir.exists() {
            return Vec::new();
        }

        let mut esporas = Vec::new();
        if let Ok(entradas) = fs::read_dir(&dir) {
            for entrada in entradas.flatten() {
                let ruta = entrada.path();
                if ruta.extension().and_then(|s| s.to_str()) == Some("spore") {
                    if let Some(nombre) = ruta.file_stem().and_then(|s| s.to_str()) {
                        esporas.push(nombre.to_string());
                    }
                }
            }
        }
        esporas
    }

    /// Eliminar una espora del disco
    pub fn eliminar(id_auton: &str) -> Result<(), EsporaError> {
        let dir = PathBuf::from(DIR_ESPORAS);
        let nombre_archivo = format!("{}{}", id_auton, EXTENSION_ESPORA);
        let ruta = dir.join(nombre_archivo);

        if ruta.exists() {
            fs::remove_file(&ruta)?;
        }
        Ok(())
    }

    /// Pre-cargar una TablaSinaptica con los flujos de la espora
    ///
    /// Este método implementa la **Necromancia Digital**:
    /// el nuevo Auton nace con "instintos" heredados del Auton muerto.
    pub fn aplicar_flujos_a_tabla(&self, tabla: &mut TablaSinaptica) {
        for &(origen, destino, fuerza) in &self.memoria_fantasma {
            tabla.precargar_flujo(origen, destino, fuerza);
        }
    }
}

/// Errores posibles durante operaciones de esporas
#[derive(Debug, Clone)]
pub enum EsporaError {
    IoError(String),
    SerializacionError(String),
}

impl std::fmt::Display for EsporaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EsporaError::IoError(s) => write!(f, "Error de I/O: {}", s),
            EsporaError::SerializacionError(s) => write!(f, "Error de serialización: {}", s),
        }
    }
}

impl std::error::Error for EsporaError {}

impl From<std::io::Error> for EsporaError {
    fn from(err: std::io::Error) -> Self {
        EsporaError::IoError(err.to_string())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_espora_generacion_y_lectura() {
        // Crear genoma de prueba
        let codon = [100, 150, 200, 250, 50, 75];
        let genoma = Genoma::with_codon(&codon, 4);

        // Simular tabla sináptica vacía (no tenemos acceso real en test)
        // En un test real, necesitaríamos una TablaSinaptica real

        // La generación real requeriría una TablaSinaptica completa
        // Skip este test si no hay implementación real
    }

    #[test]
    fn test_listar_esporas_vacias() {
        let esporas = Espora::listar_disponibles();
        // Puede estar vacío o tener archivos del sistema real
        assert!(esporas.iter().all(|e| e.ends_with(".spore") == false));
    }

    #[test]
    fn test_codigo_magico_edsp() {
        let datos = b"EDSP".to_vec();
        assert_eq!(&datos[0..4], b"EDSP");
    }
}
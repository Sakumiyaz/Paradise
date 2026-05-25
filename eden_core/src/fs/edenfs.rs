//! # EdenFS: Sistema de Archivos para Bifurcaciones Temporales
//!
//! Implementa persistencia de Auton usando el sistema de archivos del host,
//! con soporte para time forking via hard links (COW implícito del SO).
//!
//! ## Conceptos
//!
//! - **Bifurcación Temporal**: Cuando un Auton se divide, el hijo recibe un
//!   nuevo UUID y su archivo es un hard link del padre (copy-on-write del SO).
//! - **Herencia Lamarckiana**: Los archivos de Auton muertos se mueven a
//!   `meltrace/` donde pueden ser leídos por futuros Auton.
//! - **Persistencia Post-Mortem**: Análisis de grabaciones de Auton muertos.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Directorio base de Eden
const EDEN_DIR: &str = ".eden";

/// Subdirectorio de universos
const UNIVERSES_DIR: &str = "universes";

/// Subdirectorio de Autons vivos
const AUTONS_DIR: &str = "autons";

/// Subdirectorio de Meltrace (Autons muertos)
const MELTRACE_DIR: &str = "meltrace";

/// Extensión para archivos de estado
const EXT_ESTADO: &str = "bin";

/// Extensión para metadatos
const EXT_META: &str = "meta";

/// Causa de muerte de un Auton
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CausaMuerte {
    /// Energía agotada
    AgotamientoEnergia,
    /// Destruido por otro Auton
    Destruido,
    /// Campo Estructural colapsó
    ColapsoCampo,
    /// Muerte natural/programada
    Senescencia,
    /// Causa desconocida
    Desconocida,
}

impl CausaMuerte {
    pub fn to_str(&self) -> &'static str {
        match self {
            CausaMuerte::AgotamientoEnergia => "energy_exhaustion",
            CausaMuerte::Destruido => "destroyed",
            CausaMuerte::ColapsoCampo => "field_collapse",
            CausaMuerte::Senescencia => "senescence",
            CausaMuerte::Desconocida => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "energy_exhaustion" => CausaMuerte::AgotamientoEnergia,
            "destroyed" => CausaMuerte::Destruido,
            "field_collapse" => CausaMuerte::ColapsoCampo,
            "senescence" => CausaMuerte::Senescencia,
            _ => CausaMuerte::Desconocida,
        }
    }
}

/// Metadatos de un Auton
#[derive(Debug, Clone)]
pub struct MetadatosAuton {
    /// UUID del Auton
    pub uuid: u64,
    /// UUID del padre (0 si no tiene)
    pub uuid_padre: u64,
    /// Semilla del universo
    pub semilla_universo: u64,
    /// Tick en que nació
    pub tick_nacimiento: u64,
    /// Tick de última modificación
    pub tick_modificacion: u64,
    /// Tick de muerte (0 si está vivo)
    pub tick_muerte: u64,
    /// Causa de muerte
    pub causa_muerte: CausaMuerte,
    /// Energía actual
    pub energia: i64,
    /// Energía máxima histórica
    pub energia_max: i64,
    /// Número de generaciones desde el origen
    pub generacion: u32,
    /// Hash del estado final (para similitud)
    pub hash_estado: u64,
}

impl MetadatosAuton {
    pub fn nuevo(uuid: u64, semilla_universo: u64, tick: u64, uuid_padre: u64) -> Self {
        MetadatosAuton {
            uuid,
            uuid_padre,
            semilla_universo,
            tick_nacimiento: tick,
            tick_modificacion: tick,
            tick_muerte: 0,
            causa_muerte: CausaMuerte::Desconocida,
            energia: 0,
            energia_max: 0,
            generacion: 0,
            hash_estado: 0,
        }
    }

    /// Serializa a bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();

        v.extend_from_slice(&self.uuid.to_le_bytes());
        v.extend_from_slice(&self.uuid_padre.to_le_bytes());
        v.extend_from_slice(&self.semilla_universo.to_le_bytes());
        v.extend_from_slice(&self.tick_nacimiento.to_le_bytes());
        v.extend_from_slice(&self.tick_modificacion.to_le_bytes());
        v.extend_from_slice(&self.tick_muerte.to_le_bytes());

        let causa_bytes = self.causa_muerte.to_str();
        v.extend_from_slice(&(causa_bytes.len() as u32).to_le_bytes());
        v.extend_from_slice(causa_bytes.as_bytes());

        v.extend_from_slice(&self.energia.to_le_bytes());
        v.extend_from_slice(&self.energia_max.to_le_bytes());
        v.extend_from_slice(&self.generacion.to_le_bytes());
        v.extend_from_slice(&self.hash_estado.to_le_bytes());

        v
    }

    /// Deserializa desde bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut pos = 0;

        if bytes.len() < 8 {
            return None;
        }
        let uuid = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        if bytes.len() < pos + 8 {
            return None;
        }
        let uuid_padre = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        if bytes.len() < pos + 8 {
            return None;
        }
        let semilla_universo = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        if bytes.len() < pos + 8 {
            return None;
        }
        let tick_nacimiento = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        if bytes.len() < pos + 8 {
            return None;
        }
        let tick_modificacion = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        if bytes.len() < pos + 8 {
            return None;
        }
        let tick_muerte = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        if bytes.len() < pos + 4 {
            return None;
        }
        let causa_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;

        if bytes.len() < pos + causa_len {
            return None;
        }
        let causa_str = String::from_utf8_lossy(&bytes[pos..pos + causa_len]).to_string();
        pos += causa_len;

        let causa_muerte = CausaMuerte::from_str(&causa_str);

        if bytes.len() < pos + 8 {
            return None;
        }
        let energia = i64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        if bytes.len() < pos + 8 {
            return None;
        }
        let energia_max = i64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        if bytes.len() < pos + 4 {
            return None;
        }
        let generacion = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?);
        pos += 4;

        if bytes.len() < pos + 8 {
            return None;
        }
        let hash_estado = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);

        Some(MetadatosAuton {
            uuid,
            uuid_padre,
            semilla_universo,
            tick_nacimiento,
            tick_modificacion,
            tick_muerte,
            causa_muerte,
            energia,
            energia_max,
            generacion,
            hash_estado,
        })
    }
}

/// Representa un archivo de Auton en el filesystem
#[derive(Debug, Clone)]
pub struct AutonArchivo {
    /// UUID del Auton
    pub uuid: u64,
    /// Ruta al archivo de estado
    pub ruta_estado: PathBuf,
    /// Ruta al archivo de metadatos
    pub ruta_meta: PathBuf,
    /// Indica si el Auton está vivo
    pub vivo: bool,
}

impl AutonArchivo {
    /// Verifica si el archivo existe
    pub fn existe(&self) -> bool {
        self.ruta_estado.exists() || self.ruta_meta.exists()
    }
}

/// EdenFS: Sistema de Archivos para Autons
#[derive(Debug, Clone)]
pub struct EdenFS {
    /// Directorio base de Eden (~/.eden)
    dir_base: PathBuf,
    /// Semilla del universo actual
    semilla_universo: u64,
    /// Directorio de Autons vivos
    dir_autons: PathBuf,
    /// Directorio de Meltrace (muertos)
    dir_meltrace: PathBuf,
    /// Caché de Autons conocidos
    indice_autons: HashMap<u64, AutonArchivo>,
    /// Contador de Autons creados
    autons_creados: u64,
    /// Contador de Autons muertos
    autons_muertos: u64,
}

impl EdenFS {
    /// Crea nueva instancia de EdenFS
    pub fn new(semilla_universo: u64) -> io::Result<Self> {
        let dir_base = Self::dir_eden()?;
        let dir_autons = dir_base
            .join(UNIVERSES_DIR)
            .join(format!("{}", semilla_universo))
            .join(AUTONS_DIR);
        let dir_meltrace = dir_base.join(MELTRACE_DIR);

        // Crear estructura de directorios
        fs::create_dir_all(&dir_autons)?;
        fs::create_dir_all(&dir_meltrace)?;

        let mut fs = EdenFS {
            dir_base,
            semilla_universo,
            dir_autons: dir_autons.clone(),
            dir_meltrace,
            indice_autons: HashMap::new(),
            autons_creados: 0,
            autons_muertos: 0,
        };

        // Escanear Autons existentes
        fs.escanear_autons()?;

        Ok(fs)
    }

    /// Obtiene el directorio ~/.eden
    fn dir_eden() -> io::Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "No se encontró directorio home")
        })?;
        Ok(home.join(EDEN_DIR))
    }

    /// Escanea el directorio de Autons y actualiza el índice
    fn escanear_autons(&mut self) -> io::Result<()> {
        if !self.dir_autons.exists() {
            return Ok(());
        }

        for entrada in fs::read_dir(&self.dir_autons)? {
            let entrada = entrada?;
            let ruta = entrada.path();

            if ruta.extension().and_then(|s| s.to_str()) == Some(EXT_ESTADO) {
                if let Some(stem) = ruta.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(uuid) = u64::from_str_radix(stem, 16) {
                        let ruta_meta = self.dir_autons.join(format!("{}.{}", stem, EXT_META));

                        // Verificar si está vivo o muerto
                        let vivo = ruta.exists() && !ruta_meta.exists();

                        let archivo = AutonArchivo {
                            uuid,
                            ruta_estado: ruta.clone(),
                            ruta_meta,
                            vivo,
                        };

                        self.indice_autons.insert(uuid, archivo);
                    }
                }
            }
        }

        Ok(())
    }

    /// Genera un nuevo UUID único
    fn generar_uuid(&self) -> u64 {
        use std::time::SystemTime;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        ((nanos as u64) ^ (self.autons_creados << 48)) ^ self.semilla_universo
    }

    /// Registra un nuevo Auton (nacimiento)
    pub fn registrar_nacimiento(&mut self, uuid_padre: Option<u64>) -> io::Result<AutonArchivo> {
        let uuid = self.generar_uuid();
        let tick = Self::tick_actual();

        let meta =
            MetadatosAuton::nuevo(uuid, self.semilla_universo, tick, uuid_padre.unwrap_or(0));

        let ruta_estado = self.dir_autons.join(format!("{:x}.{}", uuid, EXT_ESTADO));
        let ruta_meta = self.dir_autons.join(format!("{:x}.{}", uuid, EXT_META));

        // Crear archivo de estado vacío
        File::create(&ruta_estado)?;

        // Escribir metadatos
        let mut archivo_meta = File::create(&ruta_meta)?;
        archivo_meta.write_all(&meta.to_bytes())?;

        let archivo = AutonArchivo {
            uuid,
            ruta_estado: ruta_estado.clone(),
            ruta_meta,
            vivo: true,
        };

        self.indice_autons.insert(uuid, archivo.clone());
        self.autons_creados += 1;

        Ok(archivo)
    }

    /// Registra la muerte de un Auton (bifurcación temporal → archivo se mueve)
    pub fn registrar_muerte(
        &mut self,
        uuid: u64,
        causa: CausaMuerte,
        hash_estado: u64,
    ) -> io::Result<Option<PathBuf>> {
        if let Some(archivo) = self.indice_autons.get_mut(&uuid) {
            if !archivo.vivo {
                return Ok(None); // Ya estaba muerto
            }

            // Leer metadatos directamente desde archivo
            let tick = Self::tick_actual();
            let ruta_meta = &archivo.ruta_meta;
            let mut datos_meta = Vec::new();
            File::open(ruta_meta)?.read_to_end(&mut datos_meta)?;
            let mut meta = MetadatosAuton::from_bytes(&datos_meta)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Metadatos corruptos"))?;

            meta.tick_muerte = tick;
            meta.causa_muerte = causa;
            meta.hash_estado = hash_estado;

            // Guardar metadatos actualizados
            let mut archivo_meta = OpenOptions::new().write(true).open(ruta_meta)?;
            archivo_meta.write_all(&meta.to_bytes())?;
            archivo_meta.sync_all()?;

            // Mover archivo de estado a meltrace/
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let nombre_muerte = format!("{:x}.{}.dead", uuid, timestamp);
            let destino = self.dir_meltrace.join(nombre_muerte);

            fs::rename(&archivo.ruta_estado, &destino)?;

            // Marcar como muerto en índice
            archivo.vivo = false;
            self.autons_muertos += 1;

            Ok(Some(destino))
        } else {
            Ok(None)
        }
    }

    /// Bifurcación temporal: crea un Auton hijo con hard link al padre
    pub fn bifurcacion_temporal(&mut self, uuid_padre: u64) -> io::Result<Option<AutonArchivo>> {
        // Extraer datos del padre primero para evitar borrow conflict
        let (ruta_estado_padre, vivo) = {
            if let Some(archivo_padre) = self.indice_autons.get(&uuid_padre) {
                (archivo_padre.ruta_estado.clone(), archivo_padre.vivo)
            } else {
                return Ok(None);
            }
        };

        if !vivo {
            return Ok(None); // No se puede bifurcar un Auton muerto
        }

        // Registrar nuevo Auton hijo
        let mut archivo_hijo = self.registrar_nacimiento(Some(uuid_padre))?;

        // Hard link del archivo de estado padre al hijo
        // Esto crea COW implícito del SO
        // Primero eliminar archivo vacío creado por registrar_nacimiento
        fs::remove_file(&archivo_hijo.ruta_estado)?;
        fs::hard_link(&ruta_estado_padre, &archivo_hijo.ruta_estado)?;

        // Copiar metadatos actualizando UUID y generación
        let mut meta = self.leer_metadatos(uuid_padre)?;
        meta.uuid = archivo_hijo.uuid;
        meta.uuid_padre = uuid_padre;
        meta.generacion += 1;
        meta.tick_nacimiento = Self::tick_actual();

        let mut archivo_meta = OpenOptions::new()
            .write(true)
            .open(&archivo_hijo.ruta_meta)?;
        archivo_meta.write_all(&meta.to_bytes())?;
        archivo_meta.sync_all()?;

        archivo_hijo.vivo = true;

        Ok(Some(archivo_hijo))
    }

    /// Lee el estado de un Auton
    pub fn leer_estado(&self, uuid: u64) -> io::Result<Option<Vec<u8>>> {
        if let Some(archivo) = self.indice_autons.get(&uuid) {
            if archivo.vivo || archivo.ruta_estado.exists() {
                let mut archivo = File::open(&archivo.ruta_estado)?;
                let mut datos = Vec::new();
                archivo.read_to_end(&mut datos)?;
                return Ok(Some(datos));
            }
        }

        // Buscar en meltrace si no está vivo
        let nombre_buscado = format!("{:x}.", uuid);
        if let Ok(entradas) = fs::read_dir(&self.dir_meltrace) {
            for entrada in entradas.flatten() {
                if let Some(nombre) = entrada.file_name().to_str() {
                    if nombre.starts_with(&nombre_buscado) && nombre.ends_with(".dead") {
                        let mut archivo = File::open(entrada.path())?;
                        let mut datos = Vec::new();
                        archivo.read_to_end(&mut datos)?;
                        return Ok(Some(datos));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Escribe el estado de un Auton
    pub fn escribir_estado(&self, uuid: u64, datos: &[u8]) -> io::Result<()> {
        if let Some(auton) = self.indice_autons.get(&uuid) {
            let mut archivo_estado = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&auton.ruta_estado)?;
            archivo_estado.write_all(datos)?;
            archivo_estado.sync_all()?;

            // Actualizar tick de modificación
            if let Ok(mut meta) = self.leer_metadatos(uuid) {
                meta.tick_modificacion = Self::tick_actual();
                let mut archivo_meta = OpenOptions::new().write(true).open(&auton.ruta_meta)?;
                archivo_meta.write_all(&meta.to_bytes())?;
            }

            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Auton {:x} no encontrado", uuid),
            ))
        }
    }

    /// Lee los metadatos de un Auton
    pub fn leer_metadatos(&self, uuid: u64) -> io::Result<MetadatosAuton> {
        let ruta_meta = self.dir_autons.join(format!("{:x}.{}", uuid, EXT_META));

        if !ruta_meta.exists() {
            // Buscar en meltrace
            let nombre_buscado = format!("{:x}.", uuid);
            if let Ok(entradas) = fs::read_dir(&self.dir_meltrace) {
                for entrada in entradas.flatten() {
                    if let Some(nombre) = entrada.file_name().to_str() {
                        if nombre.starts_with(&nombre_buscado) && nombre.ends_with(".dead") {
                            return Self::leer_metadatos_dead(&entrada.path());
                        }
                    }
                }
            }
        }

        let mut archivo = File::open(&ruta_meta)?;
        let mut datos = Vec::new();
        archivo.read_to_end(&mut datos)?;

        MetadatosAuton::from_bytes(&datos)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Metadatos corruptos"))
    }

    /// Lee metadatos de un archivo .dead
    fn leer_metadatos_dead(ruta: &Path) -> io::Result<MetadatosAuton> {
        // El archivo .dead contiene: [4 bytes len][meta bytes]
        let mut archivo = File::open(ruta)?;
        let mut datos = Vec::new();
        archivo.read_to_end(&mut datos)?;

        if datos.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Archivo dead demasiado pequeño",
            ));
        }

        let len = u32::from_le_bytes(datos[0..4].try_into().unwrap()) as usize;
        if datos.len() < 4 + len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Archivo dead incompleto",
            ));
        }

        let meta_bytes = &datos[4..4 + len];
        MetadatosAuton::from_bytes(meta_bytes)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Metadatos corruptos"))
    }

    /// Obtiene el timestamp actual
    fn tick_actual() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Obtiene todos los Autons vivos
    pub fn autons_vivos(&self) -> Vec<u64> {
        self.indice_autons
            .values()
            .filter(|a| a.vivo)
            .map(|a| a.uuid)
            .collect()
    }

    /// Obtiene todos los Autons muertos
    pub fn autons_muertos(&self) -> Vec<PathBuf> {
        let mut resultado = Vec::new();

        if let Ok(entradas) = fs::read_dir(&self.dir_meltrace) {
            for entrada in entradas.flatten() {
                if let Some(nombre) = entrada.file_name().to_str() {
                    if nombre.ends_with(".dead") {
                        resultado.push(entrada.path());
                    }
                }
            }
        }

        resultado
    }

    /// Obtiene el archivo de un Auton
    pub fn archivo(&self, uuid: u64) -> Option<&AutonArchivo> {
        self.indice_autons.get(&uuid)
    }

    /// Obtiene estadísticas
    pub fn estadisticas(&self) -> EdenFsStats {
        EdenFsStats {
            autons_vivos: self.indice_autons.values().filter(|a| a.vivo).count(),
            autons_muertos: self.autons_muertos,
            autons_totales_creados: self.autons_creados,
            directorio_base: self.dir_base.clone(),
            directorio_autons: self.dir_autons.clone(),
            directorio_meltrace: self.dir_meltrace.clone(),
        }
    }

    /// Lista archivos en meltrace para herencia
    pub fn listar_meltrace(&self) -> io::Result<Vec<(PathBuf, MetadatosAuton)>> {
        let mut resultado = Vec::new();

        for entrada in fs::read_dir(&self.dir_meltrace)? {
            let entrada = entrada?;
            let ruta = entrada.path();

            if ruta.extension().and_then(|s| s.to_str()) == Some("dead") {
                if let Ok(meta) = Self::leer_metadatos_dead(&ruta) {
                    resultado.push((ruta.clone(), meta));
                }
            }
        }

        resultado.sort_by(|a, b| b.1.tick_muerte.cmp(&a.1.tick_muerte));
        Ok(resultado)
    }

    /// Calcula el tamaño total usado por Autons vivos
    pub fn tamano_autons_vivos(&self) -> u64 {
        self.indice_autons
            .values()
            .filter(|a| a.vivo)
            .map(|a| fs::metadata(&a.ruta_estado).map(|m| m.len()).unwrap_or(0))
            .sum()
    }

    /// Calcula el tamaño total en meltrace
    pub fn tamano_meltrace(&self) -> u64 {
        fs::read_dir(&self.dir_meltrace)
            .map(|entradas| {
                entradas
                    .flatten()
                    .filter_map(|e| fs::metadata(e.path()).ok())
                    .map(|m| m.len())
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Obtiene el directorio base
    pub fn dir_base(&self) -> &Path {
        &self.dir_base
    }

    /// Obtiene el directorio de Autons
    pub fn dir_autons(&self) -> &Path {
        &self.dir_autons
    }

    /// Obtiene el directorio de Meltrace
    pub fn dir_meltrace(&self) -> &Path {
        &self.dir_meltrace
    }
}

/// Estadísticas de EdenFS
#[derive(Debug, Clone)]
pub struct EdenFsStats {
    pub autons_vivos: usize,
    pub autons_muertos: u64,
    pub autons_totales_creados: u64,
    pub directorio_base: PathBuf,
    pub directorio_autons: PathBuf,
    pub directorio_meltrace: PathBuf,
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::membrain::rand_u64;
    use std::env;
    use std::fs;

    fn crear_fs_temporal() -> io::Result<(EdenFS, PathBuf)> {
        let temp_dir = env::temp_dir().join(format!("edenfs_test_{}", rand_u64()));
        fs::create_dir_all(&temp_dir)?;

        // Parche temporal para usar directorio temporal
        let fs = EdenFS {
            dir_base: temp_dir.clone(),
            semilla_universo: 12345,
            dir_autons: temp_dir.join(AUTONS_DIR),
            dir_meltrace: temp_dir.join(MELTRACE_DIR),
            indice_autons: HashMap::new(),
            autons_creados: 0,
            autons_muertos: 0,
        };

        fs::create_dir_all(&fs.dir_autons)?;
        fs::create_dir_all(&fs.dir_meltrace)?;

        Ok((fs, temp_dir))
    }

    #[test]
    fn test_generar_uuid() {
        let (fs, temp_dir) = crear_fs_temporal().unwrap();
        let uuid1 = fs.generar_uuid();
        let uuid2 = fs.generar_uuid();
        assert_ne!(uuid1, uuid2);
        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_registrar_nacimiento() {
        let (mut fs, temp_dir) = crear_fs_temporal().unwrap();

        let archivo = fs.registrar_nacimiento(None).unwrap();
        assert!(archivo.vivo);
        assert!(archivo.ruta_estado.exists());
        assert!(archivo.ruta_meta.exists());

        let meta = fs.leer_metadatos(archivo.uuid).unwrap();
        assert_eq!(meta.uuid_padre, 0);
        assert_eq!(meta.tick_muerte, 0);

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_bifurcacion_temporal() -> Result<(), Box<dyn std::error::Error>> {
        let (mut fs, temp_dir) = crear_fs_temporal().unwrap();

        // Crear padre
        let padre = fs.registrar_nacimiento(None).unwrap();

        // Escribir algo en el padre
        fs.escribir_estado(padre.uuid, b"estado inicial")?;

        // Bifurcar
        let hijo = fs.bifurcacion_temporal(padre.uuid).unwrap().unwrap();

        assert_ne!(padre.uuid, hijo.uuid);

        // Verificar que ambos existen
        assert!(archivo_existe(&fs, padre.uuid));
        assert!(archivo_existe(&fs, hijo.uuid));

        // Verificar metadatos del hijo
        let meta_hijo = fs.leer_metadatos(hijo.uuid).unwrap();
        assert_eq!(meta_hijo.uuid_padre, padre.uuid);
        assert_eq!(meta_hijo.generacion, 1);

        fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    fn archivo_existe(fs: &EdenFS, uuid: u64) -> bool {
        fs.archivo(uuid)
            .map(|a| a.ruta_estado.exists())
            .unwrap_or(false)
    }

    #[test]
    fn test_registrar_muerte() -> Result<(), Box<dyn std::error::Error>> {
        let (mut fs, temp_dir) = crear_fs_temporal().unwrap();

        let archivo = fs.registrar_nacimiento(None).unwrap();
        fs.escribir_estado(archivo.uuid, b"estado final")?;

        let destino = fs
            .registrar_muerte(archivo.uuid, CausaMuerte::Senescencia, 0xDEADBEEF)
            .unwrap();

        assert!(destino.is_some());

        let meta = fs.leer_metadatos(archivo.uuid).unwrap();
        assert!(meta.tick_muerte > 0);
        assert!(matches!(meta.causa_muerte, CausaMuerte::Senescencia));

        fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[test]
    fn test_leer_escribir_estado() -> Result<(), Box<dyn std::error::Error>> {
        let (mut fs, temp_dir) = crear_fs_temporal().unwrap();

        let archivo = fs.registrar_nacimiento(None).unwrap();
        let datos_originales = b"Hola mundo cruel";

        fs.escribir_estado(archivo.uuid, datos_originales)?;

        let datos_leidos = fs.leer_estado(archivo.uuid).unwrap().unwrap();
        assert_eq!(datos_leidos, datos_originales);

        fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[test]
    fn test_autons_vivos() {
        let (mut fs, temp_dir) = crear_fs_temporal().unwrap();

        assert_eq!(fs.autons_vivos().len(), 0);

        let a1 = fs.registrar_nacimiento(None).unwrap();
        let a2 = fs.registrar_nacimiento(None).unwrap();

        assert_eq!(fs.autons_vivos().len(), 2);

        fs.registrar_muerte(a1.uuid, CausaMuerte::AgotamientoEnergia, 0)
            .unwrap();

        assert_eq!(fs.autons_vivos().len(), 1);
        assert!(fs.autons_vivos().contains(&a2.uuid));

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_serializacion_metadatos() {
        let meta = MetadatosAuton {
            uuid: 0x1234,
            uuid_padre: 0x5678,
            semilla_universo: 42,
            tick_nacimiento: 100,
            tick_modificacion: 200,
            tick_muerte: 300,
            causa_muerte: CausaMuerte::ColapsoCampo,
            energia: 5000,
            energia_max: 10000,
            generacion: 5,
            hash_estado: 0xDEAD,
        };

        let bytes = meta.to_bytes();
        let restaurado = MetadatosAuton::from_bytes(&bytes).unwrap();

        assert_eq!(restaurado.uuid, meta.uuid);
        assert_eq!(restaurado.uuid_padre, meta.uuid_padre);
        assert_eq!(restaurado.tick_muerte, meta.tick_muerte);
        assert!(matches!(restaurado.causa_muerte, CausaMuerte::ColapsoCampo));
    }

    #[test]
    fn test_estadisticas() {
        let (mut fs, temp_dir) = crear_fs_temporal().unwrap();

        fs.registrar_nacimiento(None).unwrap();
        fs.registrar_nacimiento(None).unwrap();

        let stats = fs.estadisticas();
        assert_eq!(stats.autons_vivos, 2);
        assert_eq!(stats.autons_totales_creados, 2);

        fs::remove_dir_all(temp_dir).ok();
    }
}

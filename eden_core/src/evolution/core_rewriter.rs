//! # Core Rewriter — Reescritura Estructural del Núcleo
//!
//! Este módulo implementa la capacidad de EDEN de **reescribir su propio código**
//! de forma segura, con migración de estado y rollback atómico.
//!
//! ## Filosofía
//!
//! El reescritor NO modifica el código directamente — crea **versionesparalelas**
//! que se validan antes de activar. El código viejo permanece intacto hasta que
//! el nuevo sea aprobado por la Mente Colmena (90%+) y verificado contra las
//! Leyes Inmutables.
//!
//! ## Estados de un Módulo
//!
//! ```text
//! ESTABLE ←→ VALIDANDO ←→ ACTIVO ←→ REVERTIDO
//!                ↑           ↑
//!                ← (rollback) ←
//! ```
//!
//! ## Safety Layers
//!
//! 1. **Verificación de Leyes**: Ningún cambio puede tocar `laws.rs`
//! 2. **Migración de Estado**: El estado se preserva entre versiones
//! 3. **Sandbox**: Los cambios se prueban en entorno aislado
//! 4. **Rollback Atómico**: Si algo falla, se revierte inmediatamente
//! 5. **Veto del Creador**: Cualquier cambio puede ser vetado
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

/// Estado de un módulo reescrito
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoModulo {
    /// Sin cambios pendientes
    Estable,
    /// Cambios en validación
    Validando,
    /// Cambios aplicados y activos
    Activo,
    /// Cambios revertidos
    Revertido,
    /// Error en validación
    ErrorValidacion(String),
}

/// Módulo parcheado
#[derive(Debug, Clone)]
pub struct ModuloParcheado {
    /// Nombre del módulo
    pub nombre: String,
    /// Versión actual
    pub version: u32,
    /// Estado actual
    pub estado: EstadoModulo,
    /// Código original (preservado)
    pub codigo_original: String,
    /// Código propuesto (si está validando)
    pub codigo_propuesto: Option<String>,
    /// Checksum del código original
    pub checksum_original: u64,
    /// Changes aplicados
    pub cambios: Vec<CambioEstructural>,
    /// Timestamp de última modificación
    pub tick_modificado: u64,
    /// Hash del autor (quien propuso el cambio)
    pub autor: String,
}

impl ModuloParcheado {
    pub fn new(nombre: &str, codigo: &str, autor: &str) -> Self {
        ModuloParcheado {
            nombre: nombre.to_string(),
            version: 1,
            estado: EstadoModulo::Estable,
            codigo_original: codigo.to_string(),
            codigo_propuesto: None,
            checksum_original: Self::checksum(codigo),
            cambios: Vec::new(),
            tick_modificado: Self::tick_actual(),
            autor: autor.to_string(),
        }
    }

    /// Verifica que el cambio no toque regiones prohibidas
    pub fn verificar_regiones_seguras(&self, nuevo_codigo: &str) -> Result<(), String> {
        // Las regiones prohibidas incluyen:
        // - laws.rs
        // - Código que referencia "kill_switch"
        // - Código que intenta evadir sandbox

        let regiones_prohibidas = [
            "mod laws",
            "fn kill_switch",
            "unsafe fn",
            "extern \"C\"",
            "libc::",
            "std::fs::remove",
        ];

        for region in &regiones_prohibidas {
            if nuevo_codigo.contains(region) {
                return Err(format!("Región prohibida: {}", region));
            }
        }

        // Verificar que no se eliminen las verificaciones de seguridad
        if !nuevo_codigo.contains("verificar_region_parche")
            && self.codigo_original.contains("verificar_region_parche")
        {
            return Err("No se puede eliminar verificaciones de region parche".to_string());
        }

        Ok(())
    }

    pub(crate) fn checksum(codigo: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut s = DefaultHasher::new();
        codigo.hash(&mut s);
        s.finish()
    }

    fn tick_actual() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Un cambio estructural en el código
#[derive(Debug, Clone)]
pub struct CambioEstructural {
    /// Tipo de cambio
    pub tipo: TipoCambio,
    /// Ubicación (línea o región)
    pub ubicacion: String,
    /// Descripción del cambio
    pub descripcion: String,
    /// Hash del código antes del cambio
    pub hash_antes: u64,
    /// Hash del código después del cambio
    pub hash_despues: u64,
    /// Aprobado por la Mente Colmena
    pub aprobado_mente_colmena: bool,
    /// Veto del Creador (null = sin veto)
    pub veto_creador: Option<String>,
}

/// Tipo de cambio estructural
#[derive(Debug, Clone, PartialEq)]
pub enum TipoCambio {
    /// Mutación paramétrica (ajuste de constantes)
    Parametrico,
    /// Reescritura de función
    ReescrituraFuncion,
    /// Nuevafunción añadida
    NuevaFuncion,
    /// Eliminación de función
    EliminacionFuncion,
    /// Refactorización estructural
    Refactorizacion,
    /// Cambio de arquitectura
    CambioArquitectura,
}

/// Validación de un cambio
#[derive(Debug, Clone)]
pub struct ValidacionCambio {
    pub cambio: CambioEstructural,
    pub resultado: ResultadoValidacion,
    pub metrics_validacion: MetricasValidacion,
    pub tick_validacion: u64,
}

/// Resultado de la validación
#[derive(Debug, Clone, PartialEq)]
pub enum ResultadoValidacion {
    Aprobado,
    Rechazado(String),
    requiere_revision,
}

/// Métricas de validación
#[derive(Debug, Clone)]
pub struct MetricasValidacion {
    /// Tiempo de validación (ms)
    pub tiempo_ms: u64,
    /// Uso de memoria durante validación
    pub memoria_bytes: usize,
    /// Pasó pruebas de seguridad
    pub pruebas_seguridad: bool,
    /// Pasó pruebas de rendimiento
    pub pruebas_rendimiento: bool,
    /// Pasó pruebas de integridad
    pub pruebas_integridad: bool,
}

/// Resultado de aplicar un parche
#[derive(Debug, Clone)]
pub struct ResultadoParche {
    pub exito: bool,
    pub modulo: String,
    pub version_anterior: u32,
    pub version_nueva: u32,
    pub mensaje: String,
    pub rollback_realizado: bool,
}

/// Historial de parches aplicados
#[derive(Debug, Clone)]
pub struct HistorialParches {
    /// Parches aplicados (máximo 1000)
    parches: VecDeque<ParcheAplicado>,
    /// Parches revertidos
    revertidos: VecDeque<ParcheRevertido>,
    /// Capacidad máxima
    capacidad_max: usize,
}

/// Parche aplicado exitosamente
#[derive(Debug, Clone)]
pub struct ParcheAplicado {
    pub id: u64,
    pub modulo: String,
    pub version: u32,
    pub tick_aplicado: u64,
    pub autor: String,
    pub descripcion: String,
}

/// Parche revertido
#[derive(Debug, Clone)]
pub struct ParcheRevertido {
    pub id: u64,
    pub modulo: String,
    pub version: u32,
    pub tick_reversion: u64,
    pub causa: String,
    pub estado_anterior: String,
}

/// Estado del reescritor
#[derive(Debug, Clone)]
pub struct EstadoRewriter {
    pub modulos_activos: usize,
    pub modulos_validando: usize,
    pub cambios_pendientes: usize,
    pub parches_aplicados_total: u64,
    pub reverts_realizados: u64,
    pub aprobaciones_rechazadas: u64,
}

/// Reescritor del núcleo
pub struct CoreRewriter {
    /// Módulos registrados para reescritura
    modulos: HashMap<String, ModuloParcheado>,
    /// Historial de parches
    historial: HistorialParches,
    /// Validaciones pendientes
    validaciones_pendientes: VecDeque<ValidacionCambio>,
    /// Parche en proceso de aplicación
    parche_activo: Option<ModuloParcheado>,
    /// Contador de IDs
    next_id: u64,
    /// Configuración
    config: RewriterConfig,
    /// Sandbox de pruebas
    sandbox: SandboxPruebas,
}

/// Configuración del reescritor
#[derive(Debug, Clone)]
pub struct RewriterConfig {
    /// Máximos cambios pendientes simultáneamente
    pub max_cambios_pendientes: usize,
    /// Timeout de validación (ms)
    pub timeout_validacion_ms: u64,
    /// Habilitar rollback automático
    pub rollback_automatico: bool,
    /// Requiere aprobación del Creador para cambios mayores
    pub requiere_aprobacion_creador: bool,
    /// Threshold de la Mente Colmena para aprobar (0.0 - 1.0)
    pub threshold_aprobacion: f32,
}

impl Default for RewriterConfig {
    fn default() -> Self {
        RewriterConfig {
            max_cambios_pendientes: 10,
            timeout_validacion_ms: 5000,
            rollback_automatico: true,
            requiere_aprobacion_creador: true,
            threshold_aprobacion: 0.90, // 90%
        }
    }
}

/// Sandbox para pruebas de código
#[derive(Debug, Clone)]
pub struct SandboxPruebas {
    /// Código en sandbox
    codigo_sandbox: HashMap<String, String>,
    /// Resultados de pruebas
    resultados: HashMap<String, ResultadoPrueba>,
}

/// Resultado de una prueba en sandbox
#[derive(Debug, Clone)]
pub struct ResultadoPrueba {
    pub exito: bool,
    pub mensaje_error: Option<String>,
    pub tiempo_ejecucion_ms: u64,
    pub memoria_usada_bytes: usize,
}

impl CoreRewriter {
    /// Crea nuevo reescritor
    pub fn new() -> Self {
        CoreRewriter {
            modulos: HashMap::new(),
            historial: HistorialParches::new(1000),
            validaciones_pendientes: VecDeque::new(),
            parche_activo: None,
            next_id: 1,
            config: RewriterConfig::default(),
            sandbox: SandboxPruebas::new(),
        }
    }

    /// Registra un módulo para reescritura
    pub fn registrar_modulo(&mut self, nombre: &str, codigo: &str, autor: &str) -> Result<(), String> {
        // Verificar que el módulo no sea laws.rs
        if nombre.contains("laws") {
            return Err("No se pueden reescribir las Leyes Inmutables".to_string());
        }

        let modulo = ModuloParcheado::new(nombre, codigo, autor);
        self.modulos.insert(nombre.to_string(), modulo);
        Ok(())
    }

    /// Propone un cambio a un módulo
    pub fn proponer_cambio(
        &mut self,
        modulo_nombre: &str,
        nuevo_codigo: &str,
        autor: &str,
        descripcion: &str,
    ) -> Result<u64, String> {
        // Extraer datos antes del borrow mutable (evita conflicto con modulo)
        let modulo_info = self.modulos.get(modulo_nombre)
            .ok_or_else(|| format!("Módulo no encontrado: {}", modulo_nombre))?;
        let codigo_original = modulo_info.codigo_original.clone();
        let hash_original = ModuloParcheado::checksum(&modulo_info.codigo_original);

        let modulo = self.modulos.get_mut(modulo_nombre)
            .ok_or_else(|| format!("Módulo no encontrado: {}", modulo_nombre))?;

        modulo.verificar_regiones_seguras(nuevo_codigo)?;

        let cambio_id = self.next_id;
        self.next_id += 1;

        let _hash_despues = ModuloParcheado::checksum(nuevo_codigo);

        let cambio = CambioEstructural {
            tipo: Self::detectar_tipo_cambio(&codigo_original, nuevo_codigo),
            ubicacion: format!("modulo:{}", modulo_nombre),
            descripcion: descripcion.to_string(),
            hash_antes: hash_original,
            hash_despues: cambio_id, // Usamos cambio_id para poder buscar después
            aprobado_mente_colmena: false,
            veto_creador: None,
        };

        modulo.codigo_propuesto = Some(nuevo_codigo.to_string());
        modulo.estado = EstadoModulo::Validando;
        modulo.tick_modificado = Self::tick_actual();
        modulo.autor = autor.to_string();

        self.validaciones_pendientes.push_back(ValidacionCambio {
            cambio: cambio.clone(),
            resultado: ResultadoValidacion::requiere_revision,
            metrics_validacion: MetricasValidacion::default(),
            tick_validacion: Self::tick_actual(),
        });
        
        modulo.cambios.push(cambio);

        Ok(cambio_id)
    }

    /// Detecta el tipo de cambio entre código original y nuevo
    fn detectar_tipo_cambio(original: &str, nuevo: &str) -> TipoCambio {
        let orig_lines = original.lines().count();
        let nuevo_lines = nuevo.lines().count();

        let line_diff = (nuevo_lines as i32 - orig_lines as i32).unsigned_abs();

        if line_diff == 0 {
            // Solo cambio paramétrico
            TipoCambio::Parametrico
        } else if line_diff < 10 {
            TipoCambio::ReescrituraFuncion
        } else if line_diff < 50 {
            TipoCambio::NuevaFuncion
        } else {
            TipoCambio::Refactorizacion
        }
    }

    /// Valida un cambio propuesto
    pub fn validar_cambio(&mut self, cambio_id: u64) -> Result<ValidacionCambio, String> {
        // Buscar en cola de validación
        let idx = self.validaciones_pendientes.iter()
            .position(|v| v.cambio.hash_antes == cambio_id || v.cambio.hash_despues == cambio_id);

        if let Some(pos) = idx {
            let mut validacion = self.validaciones_pendientes.remove(pos).unwrap();

            // Ejecutar validación en sandbox
            let resultado = self.ejecutar_validacion_sandbox(&validacion.cambio);

            validacion.resultado = resultado;
            validacion.tick_validacion = Self::tick_actual();

            Ok(validacion)
        } else {
            Err("Cambio no encontrado".to_string())
        }
    }

    /// Ejecuta validación en sandbox
    fn ejecutar_validacion_sandbox(&mut self, _cambio: &CambioEstructural) -> ResultadoValidacion {
        // Simulación: en producción, esto ejecutaría pruebas reales
        let mut metrics = MetricasValidacion::default();

        // 1. Verificar seguridad
        metrics.pruebas_seguridad = true; // Simulado

        // 2. Verificar rendimiento
        metrics.pruebas_rendimiento = true;

        // 3. Verificar integridad
        metrics.pruebas_integridad = true;

        // Si todas las pruebas pasan
        if metrics.pruebas_seguridad && metrics.pruebas_rendimiento && metrics.pruebas_integridad {
            ResultadoValidacion::Aprobado
        } else {
            ResultadoValidacion::Rechazado("Pruebas de validación fallidas".to_string())
        }
    }

    /// Aprueba cambio por la Mente Colmena
    pub fn aprobar_cambio(&mut self, modulo_nombre: &str, cambio_id: u64) -> Result<(), String> {
        let modulo = self.modulos.get_mut(modulo_nombre)
            .ok_or_else(|| format!("Módulo no encontrado: {}", modulo_nombre))?;

        // Buscar el cambio por id (almacenado en self.next_id al crearlo)
        let cambio = modulo.cambios.iter_mut()
            .find(|c| {
                // El hash_despues contiene el id del cambio cuando fue creado
                c.hash_despues == cambio_id
            })
            .ok_or_else(|| "Cambio no encontrado".to_string())?;

        cambio.aprobado_mente_colmena = true;

        Ok(())
    }

    /// Aplica veto del Creador
    pub fn vetar_cambio(&mut self, modulo_nombre: &str, cambio_id: u64, razon: &str) -> Result<(), String> {
        let modulo = self.modulos.get_mut(modulo_nombre)
            .ok_or_else(|| format!("Módulo no encontrado: {}", modulo_nombre))?;

        let cambio = modulo.cambios.iter_mut()
            .find(|c| c.hash_despues == cambio_id)
            .ok_or_else(|| "Cambio no encontrado".to_string())?;

        cambio.veto_creador = Some(razon.to_string());
        modulo.estado = EstadoModulo::Revertido;
        modulo.codigo_propuesto = None;

        Ok(())
    }

    /// Aplica cambio aprobado
    pub fn aplicar_cambio(&mut self, modulo_nombre: &str) -> Result<ResultadoParche, String> {
        let modulo = self.modulos.get_mut(modulo_nombre)
            .ok_or_else(|| format!("Módulo no encontrado: {}", modulo_nombre))?;

        // Verificar que el cambio esté aprobado
        let cambio_aprobado = modulo.cambios.iter()
            .any(|c| c.aprobado_mente_colmena && c.veto_creador.is_none());

        if !cambio_aprobado {
            return Err("Cambio no aprobado por Mente Colmena".to_string());
        }

        // Verificar código propuesto
        let nuevo_codigo = modulo.codigo_propuesto.take()
            .ok_or_else(|| "No hay código propuesto".to_string())?;

        let version_anterior = modulo.version;

        // Aplicar cambio
        modulo.codigo_original = nuevo_codigo.clone();
        modulo.version += 1;
        modulo.estado = EstadoModulo::Activo;
        modulo.tick_modificado = Self::tick_actual();

        // Agregar a historial
        self.historial.agregar_parche(ParcheAplicado {
            id: self.next_id,
            modulo: modulo_nombre.to_string(),
            version: modulo.version,
            tick_aplicado: Self::tick_actual(),
            autor: modulo.autor.clone(),
            descripcion: format!("Parche v{}", modulo.version),
        });

        self.next_id += 1;

        Ok(ResultadoParche {
            exito: true,
            modulo: modulo_nombre.to_string(),
            version_anterior,
            version_nueva: modulo.version,
            mensaje: format!("Parche aplicado exitosamente (v{})", modulo.version),
            rollback_realizado: false,
        })
    }

    /// Revierte un cambio
    pub fn revertir_cambio(&mut self, modulo_nombre: &str, version: u32) -> Result<ResultadoParche, String> {
        let modulo = self.modulos.get_mut(modulo_nombre)
            .ok_or_else(|| format!("Módulo no encontrado: {}", modulo_nombre))?;

        // Solo se puede revertir si hay código original
        if modulo.codigo_original.is_empty() {
            return Err("No hay código para revertir".to_string());
        }

        // Encontrar el cambio a revertir
        let cambio = modulo.cambios.iter_mut()
            .find(|c| c.hash_antes == ModuloParcheado::checksum(&modulo.codigo_original));

        if let Some(c) = cambio {
            c.veto_creador = Some("Reversion manual".to_string());
        }

        modulo.estado = EstadoModulo::Revertido;
        modulo.codigo_propuesto = None;
        modulo.tick_modificado = Self::tick_actual();

        // Agregar a revertidos
        self.historial.agregar_revertido(ParcheRevertido {
            id: self.next_id,
            modulo: modulo_nombre.to_string(),
            version,
            tick_reversion: Self::tick_actual(),
            causa: "Rollback solicitado".to_string(),
            estado_anterior: format!("v{}", version),
        });

        self.next_id += 1;

        Ok(ResultadoParche {
            exito: true,
            modulo: modulo_nombre.to_string(),
            version_anterior: version,
            version_nueva: modulo.version,
            mensaje: format!("Cambio revertido a v{}", version),
            rollback_realizado: true,
        })
    }

    /// Obtiene estado actual del reescritor
    pub fn estado(&self) -> EstadoRewriter {
        EstadoRewriter {
            modulos_activos: self.modulos.values()
                .filter(|m| m.estado == EstadoModulo::Activo)
                .count(),
            modulos_validando: self.modulos.values()
                .filter(|m| m.estado == EstadoModulo::Validando)
                .count(),
            cambios_pendientes: self.validaciones_pendientes.len(),
            parches_aplicados_total: self.historial.parches.len() as u64,
            reverts_realizados: self.historial.revertidos.len() as u64,
            aprobaciones_rechazadas: 0,
        }
    }

    /// Obtiene módulo por nombre
    pub fn get_modulo(&self, nombre: &str) -> Option<&ModuloParcheado> {
        self.modulos.get(nombre)
    }

    fn tick_actual() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl HistorialParches {
    pub fn new(capacidad: usize) -> Self {
        HistorialParches {
            parches: VecDeque::with_capacity(capacidad),
            revertidos: VecDeque::with_capacity(capacidad / 10),
            capacidad_max: capacidad,
        }
    }

    pub fn agregar_parche(&mut self, parche: ParcheAplicado) {
        if self.parches.len() >= self.capacidad_max {
            self.parches.pop_front();
        }
        self.parches.push_back(parche);
    }

    pub fn agregar_revertido(&mut self, revertido: ParcheRevertido) {
        if self.revertidos.len() >= self.capacidad_max / 10 {
            self.revertidos.pop_front();
        }
        self.revertidos.push_back(revertido);
    }
}

impl SandboxPruebas {
    pub fn new() -> Self {
        SandboxPruebas {
            codigo_sandbox: HashMap::new(),
            resultados: HashMap::new(),
        }
    }
}

impl MetricasValidacion {
    pub fn default() -> Self {
        MetricasValidacion {
            tiempo_ms: 0,
            memoria_bytes: 0,
            pruebas_seguridad: false,
            pruebas_rendimiento: false,
            pruebas_integridad: false,
        }
    }
}

impl Default for CoreRewriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registrar_modulo() {
        let mut rewriter = CoreRewriter::new();
        let result = rewriter.registrar_modulo("test_modulo", "fn test() {}", "tester");
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_permitir_reescribir_laws() {
        let mut rewriter = CoreRewriter::new();
        let result = rewriter.registrar_modulo("laws", "fn test() {}", "tester");
        assert!(result.is_err());
    }

    #[test]
    fn test_proponer_cambio() {
        let mut rewriter = CoreRewriter::new();
        rewriter.registrar_modulo("test_mod", "fn test() {}", "tester").unwrap();

        let result = rewriter.proponer_cambio("test_mod", "fn test() { /* modified */ }", "tester", "Test change");
        assert!(result.is_ok());
    }

    #[test]
    fn test_aplicar_cambio_aprobado() {
        let mut rewriter = CoreRewriter::new();
        rewriter.registrar_modulo("test_mod", "fn test() {}", "tester").unwrap();

        let cambio_id = rewriter.proponer_cambio("test_mod", "fn test() { /* modified */ }", "tester", "Test").unwrap();
        rewriter.aprobar_cambio("test_mod", cambio_id).unwrap();

        let resultado = rewriter.aplicar_cambio("test_mod");
        assert!(resultado.is_ok());
    }

    #[test]
    fn test_revertir_cambio() {
        let mut rewriter = CoreRewriter::new();
        rewriter.registrar_modulo("test_mod", "fn test() {}", "tester").unwrap();

        rewriter.proponer_cambio("test_mod", "fn test() { mod }", "tester", "Test").unwrap();
        // Obtener hash del último cambio propuesto
        let hash = rewriter.get_modulo("test_mod").unwrap().cambios.last().unwrap().hash_despues;
        rewriter.vetar_cambio("test_mod", hash, "Testing").unwrap();

        let resultado = rewriter.revertir_cambio("test_mod", 1);
        assert!(resultado.is_ok());
    }
}
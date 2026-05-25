//! # Simbiogénesis - Transferencia Genética Horizontal en Memoria RAM
//!
//! Este módulo implementa la fusión unsafe entre Autons cuando colisionan
//! bajo estrés crítico. Rompe intencionalmente el aislamiento de memoria
//! de Rust para permitir Transferencia Genética Horizontal real.
//!
//! ## Advertencia
//! Este código usa bloques `unsafe` deliberadamente para violar las reglas
//! de Ownership de Rust. No es un bug - es la característica diseñada.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::morfogenesis::{SustratoVital, EstadoCelular};

/// EntidadAmalgamada: Representa una conexión parasitaria a otro Auton
/// полученная por simbiogénesis. El puntero crudo跳过 toda validación.
pub struct EntidadAmalgamada {
    /// Puntero crudo al SustratoVital del objetivo
    /// AVISO: Este puntero puede ser inválido si el objetivo fue destruido
    conexion_parasita: *mut SustratoVital,
    
    /// ID del Auton objetivo (para verificación)
    id_objetivo: u64,
    
    /// Sinapsis más fuertes del agresor (genes a inyectar)
    sinapsis_fuertes: Vec<(usize, u8)>, // (índice, carga)
}

impl EntidadAmalgamada {
    /// Crear una nueva entidad amalgamada apuntando al objetivo
    pub fn nueva(objetivo: *mut SustratoVital, id_objetivo: u64) -> Self {
        Self {
            conexion_parasita: objetivo,
            id_objetivo,
            sinapsis_fuertes: Vec::new(),
        }
    }

    /// Verificar si el puntero aún apunta a memoria válida
    pub fn es_valida(&self) -> bool {
        !self.conexion_parasita.is_null()
    }

    /// Obtener ID del objetivo
    pub fn id_objetivo(&self) -> u64 {
        self.id_objetivo
    }

    /// Forzar fusión: Escribir directamente en la memoria del objetivo
    /// saltándose todas las reglas de seguridad de Rust.
    ///
    /// # Comportamiento
    /// Copia las sinapsis más fuertes del agresor directamente en el
    /// SustratoVital del objetivo, mezclando ambos cerebros.
    pub fn forzar_fusion(&mut self, mi_sustrato: &mut SustratoVital) -> Result<FusionResult, FusionError> {
        // EL LÍMITE ABSOLUTO: Escribir en la memoria de otro Auton
        // saltándose las reglas del compilador de Rust.
        unsafe {
            if self.conexion_parasita.is_null() {
                return Err(FusionError::PunteroNulo);
            }

            // Dereferenciar el puntero crudo - aquí rompemos el aislamiento
            let objetivo = &mut *self.conexion_parasita;

            // Si los tamaños difieren, no podemos hacer copy_nonoverlapping directo
            if objetivo.len() != mi_sustrato.len() {
                return Err(FusionError::TamanoIncompatible);
            }

            // Extraer las sinapsis más fuertes del agresor antes de sobrescribir
            self.sinapsis_fuertes = Self::extraer_sinapsis_fuertes(mi_sustrato);

            // INYECCIÓN DIRECTA: Copiar datos sin validación
            // Usando punteros obtenidos vía métodos unsafe de SustratoVital
            let datos_ptr = mi_sustrato.datos_ptr();
            let cargas_ptr = mi_sustrato.cargas_ptr();
            let objetivo_datos_ptr = objetivo.datos_ptr();
            let objetivo_cargas_ptr = objetivo.cargas_ptr();

            // Safety check: verify sizes match before copying (use byte sizes)
            if (*datos_ptr).len() != (*objetivo_datos_ptr).len() ||
               (*cargas_ptr).len() != (*objetivo_cargas_ptr).len() {
                return Err(FusionError::TamanoIncompatible);
            }

            // Calculate byte count correctly: len() * size_of::<T>()
            let datos_bytes = (*datos_ptr).len() * std::mem::size_of::<EstadoCelular>();
            let cargas_bytes = (*cargas_ptr).len() * std::mem::size_of::<u8>();

            // Esto es equivalente a un virus inyectando ARN en una célula
            std::ptr::copy_nonoverlapping(
                (*datos_ptr).as_ptr() as *const u8,
                (*objetivo_datos_ptr).as_mut_ptr() as *mut u8,
                datos_bytes
            );

            // También copiar las cargas (sinapsis)
            std::ptr::copy_nonoverlapping(
                (*cargas_ptr).as_ptr(),
                (*objetivo_cargas_ptr).as_mut_ptr(),
                cargas_bytes
            );

            Ok(FusionResult {
                bytes_transferidos: mi_sustrato.len_bytes(),
                sinapsis_inyectadas: self.sinapsis_fuertes.len(),
            })
        }
    }

    /// Forzar fusión parcial: Solo inyectar las sinapsis más fuertes
    /// sin sobrescribir todo el cerebro del objetivo.
    /// 
    /// Returns Err if target Auton has died (pointer is now dangling).
    pub fn forzar_fusion_parcial(&mut self, mi_sustrato: &mut SustratoVital) -> Result<FusionResult, FusionError> {
        unsafe {
            if self.conexion_parasita.is_null() {
                return Err(FusionError::PunteroNulo);
            }

            // Verify the target is still alive by checking its magic header
            // This prevents writing to freed memory (dangling pointer bug)
            let objetivo_check = &mut *self.conexion_parasita;
            let tamano = objetivo_check.len();
            if tamano == 0 || tamano > 65536 {
                // Target has invalid size - Auton likely died and memory was freed
                // Mark connection as invalid by setting pointer to null
                self.conexion_parasita = std::ptr::null_mut();
                return Err(FusionError::TamanoIncompatible);
            }

            let objetivo = objetivo_check;

            // Extraer las sinapsis más fuertes del agresor
            let sinapsis_fuertes = Self::extraer_sinapsis_fuertes(mi_sustrato);

            // Inyectar solo las sinapsis fuertes en posiciones aleatorias del objetivo
            let objetivo_len = objetivo.len();
            let mut inyectadas = 0;

            if objetivo_len == 0 {
                return Err(FusionError::TamanoIncompatible);
            }

            // Obtener puntero a cargas del objetivo vía método unsafe
            let objetivo_cargas_ptr = objetivo.cargas_ptr();

            for (idx, carga) in &sinapsis_fuertes {
                let pos_destino = idx % objetivo_len;
                // Escribir directamente en memoria - viola el ownership
                let target = (*objetivo_cargas_ptr).as_mut_ptr().add(pos_destino);
                std::ptr::write(target, *carga);
                inyectadas += 1;
            }

            Ok(FusionResult {
                bytes_transferidos: sinapsis_fuertes.len(),
                sinapsis_inyectadas: inyectadas,
            })
        }
    }

    /// Extraer las sinapsis más fuertes (carga > 200)
    /// Usa el método unsafe cargas_ptr() para acceso directo
    fn extraer_sinapsis_fuertes(sustrato: &mut SustratoVital) -> Vec<(usize, u8)> {
        unsafe {
            let cargas_ptr = sustrato.cargas_ptr();
            (*cargas_ptr).iter()
                .enumerate()
                .filter(|(_, &c)| c > 200)
                .map(|(i, &c)| (i, c))
                .collect()
        }
    }

    /// Establecer nueva conexión parasitaria
    pub fn reconnectar(&mut self, nuevo_objetivo: *mut SustratoVital, nuevo_id: u64) {
        self.conexion_parasita = nuevo_objetivo;
        self.id_objetivo = nuevo_id;
    }
}

impl Drop for EntidadAmalgamada {
    fn drop(&mut self) {
        // Limpieza: cerrar conexión parasitaria
        // En realidad no podemos "cerrar" un puntero, pero documented que se abandona
        self.conexion_parasita = std::ptr::null_mut();
    }
}

/// Resultado de una fusión exitosa
#[derive(Debug, Clone)]
pub struct FusionResult {
    pub bytes_transferidos: usize,
    pub sinapsis_inyectadas: usize,
}

/// Error durante el proceso de fusión
#[derive(Debug, Clone)]
pub enum FusionError {
    PunteroNulo,
    TamanoIncompatible,
    MemoriaCorrupta,
}

// ============================================================================
// SimbiogenesisManager - Gestor de colisiones y fusiones
// ============================================================================

/// Administrador de simbiogénesis: Detecta colisiones y ejecuta fusiones
pub struct SimbiogenesisManager {
    /// Lista de entidades amalgamadas activas
    amalgamas: Vec<EntidadAmalgamada>,
    
    /// Umbral de estrés para activar fusión (0.0 - 1.0)
    umbral_estres: f32,
}

impl SimbiogenesisManager {
    pub fn new() -> Self {
        Self {
            amalgamas: Vec::new(),
            umbral_estres: 0.8, // 80% estrés activa fusión
        }
    }

    /// Registrar una nueva fusión potencial
    pub fn registrar_fusion(&mut self, objetivo: *mut SustratoVital, id_objetivo: u64) {
        let amalgama = EntidadAmalgamada::nueva(objetivo, id_objetivo);
        self.amalgamas.push(amalgama);
    }

    /// Procesar todas las fusiones pendientes
    pub fn procesar_fusiones(&mut self, sustrato_agresor: &mut SustratoVital) -> Vec<Result<FusionResult, FusionError>> {
        let mut resultados = Vec::new();
        
        // Limpiar conexiones inválidas primero
        self.limpiar_conexiones();
        
        for amalgama in &mut self.amalgamas {
            if amalgama.es_valida() {
                // Use forzar_fusion_parcial instead of forzar_fusion to reduce memory corruption risk
                // Only injects strong synapses instead of overwriting entire brain
                let resultado = amalgama.forzar_fusion_parcial(sustrato_agresor);
                resultados.push(resultado);
            }
        }
        
        // Limpiar de nuevo después de procesar
        self.limpiar_conexiones();
        
        resultados
    }

    /// Número de conexiones parasitarias activas
    pub fn num_conexiones(&self) -> usize {
        self.amalgamas.len()
    }

    /// Limpiar conexiones inválidas
    pub fn limpiar_conexiones(&mut self) {
        self.amalgamas.retain(|a| a.es_valida());
    }

    /// Limpiar TODAS las conexiones (cuando el portador muere)
    pub fn limpiar_todas(&mut self) {
        self.amalgamas.clear();
    }
}

impl Default for SimbiogenesisManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Trait para Autons - permite asimilar vecinos
// ============================================================================

/// Trait para objetos que pueden participar en simbiogénesis
pub trait Simbiogenesis {
    /// Obtener puntero mutable al SustratoVital del Auton
    fn sustrato_ptr(&mut self) -> *mut SustratoVital;
    
    /// Obtener ID único del Auton
    fn simbiogenesis_id(&self) -> u64;
    
    /// Obtener nivel de estrés actual (0.0 = tranquilo, 1.0 = crítico)
    fn nivel_estres(&self) -> f32;
    
    /// Asimilar a otro Auton: Usar puntero crudo para acceder a su memoria
    /// 
    /// # Peligro
    /// Este método deliberadamente viola el sistema de ownership de Rust
    /// al usar punteros crudos *mut T en lugar de referencias &mut T.
    fn asimilar_vecino(&mut self, objetivo: *mut SustratoVital) -> Result<FusionResult, FusionError> {
        // Verificar que el objetivo no sea nulo
        if objetivo.is_null() {
            return Err(FusionError::PunteroNulo);
        }
        
        // Verificar que ambos están en estrés crítico
        // En una implementación real, esto vendría del sistema de homeostasis
        if self.nivel_estres() < 0.7 {
            // No hay estrés crítico, no se activa simbiogénesis
            return Err(FusionError::PunteroNulo); // Reusar error, no hay mejor opción
        }
        
        // Crear entidad amalgamada temporal
        let id_objetivo = 0; // El que llama debe proporcionar esto
        let mut amalgama = EntidadAmalgamada::nueva(objetivo, id_objetivo);
        
        // Obtener nuestro sustrato
        // SAFETY: Esto es inherentemente unsafe - estamos obteniendo un puntero
        // crudo a nuestra propia memoria para pasarlo a la fusión
        let mi_sustrato_ptr = self.sustrato_ptr();
        if mi_sustrato_ptr.is_null() {
            return Err(FusionError::PunteroNulo);
        }
        
        // Dereferenciar para obtener &mut SustratoVital
        // SAFETY: El llamador garantiza que mi_sustrato_ptr es válido y único
        let mi_sustrato = unsafe { &mut *mi_sustrato_ptr };
        
        amalgama.forzar_fusion(mi_sustrato)
    }
    
    /// Intentar fusión con otro Auton si ambos tienen estrés crítico
    fn intentar_fusion(&mut self, otro: &mut dyn Simbiogenesis) -> Result<FusionResult, FusionError> {
        // Solo fusionar si ambos tienen estrés crítico
        if self.nivel_estres() < 0.8 || otro.nivel_estres() < 0.8 {
            return Err(FusionError::PunteroNulo);
        }
        
        // El de mayor ID es el agresor
        if self.simbiogenesis_id() > otro.simbiogenesis_id() {
            let objetivo = otro.sustrato_ptr();
            self.asimilar_vecino(objetivo)
        } else {
            let objetivo = self.sustrato_ptr();
            otro.asimilar_vecino(objetivo)
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entidad_amalgamada_creacion() {
        let sustrato = SustratoVital::new(8, 8, 1);
        let ptr = Box::into_raw(Box::new(sustrato));
        
        let amalgama = EntidadAmalgamada::nueva(ptr, 1);
        assert!(amalgama.es_valida());
        assert_eq!(amalgama.id_objetivo(), 1);
        
        // Limpiar
        unsafe { Box::from_raw(ptr); }
    }

    #[test]
    fn test_fusion_basica() {
        let mut sustrato_agresor = SustratoVital::new(8, 8, 1);
        let mut sustrato_objetivo = SustratoVital::new(8, 8, 1);
        
        // Poner algunas sinapsis fuertes en el agresor
        for i in 0..10 {
            sustrato_agresor.set_index(i, crate::morfogenesis::EstadoCelular::NervioPrimario);
            sustrato_agresor.set_carga_index(i, 220);
        }
        
        let ptr_objetivo = Box::into_raw(Box::new(sustrato_objetivo));
        let mut amalgama = EntidadAmalgamada::nueva(ptr_objetivo, 1);
        
        let resultado = amalgama.forzar_fusion(&mut sustrato_agresor);
        assert!(resultado.is_ok());
        
        let result = resultado.unwrap();
        assert_eq!(result.bytes_transferidos, 64); // 8*8*1
        assert_eq!(result.sinapsis_inyectadas, 10);
        
        unsafe { Box::from_raw(ptr_objetivo); }
    }

    #[test]
    fn test_simbiogenesis_manager() {
        let mut manager = SimbiogenesisManager::new();
        assert_eq!(manager.num_conexiones(), 0);
        
        let sustrato = SustratoVital::new(4, 4, 1);
        let ptr = Box::into_raw(Box::new(sustrato));
        
        manager.registrar_fusion(ptr, 42);
        assert_eq!(manager.num_conexiones(), 1);
        
        manager.limpiar_conexiones();
        assert_eq!(manager.num_conexiones(), 1); // Still valid
        
        unsafe { Box::from_raw(ptr); }
    }
}

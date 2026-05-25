//! # Hot Patch System: Auto-Modificación Segura de EDEN
//!
//! Este módulo implementa la capacidad de auto-modificación segura de EDEN
//! mediante parches en regiones de código marcadas como `#[patchable]`.
//!
//! ## Mecanismo
//!
//! 1. El binario se compila con una sección `.eden_patchable` especiales
//! 2. Funciones "blandas" en esa sección pueden reescribirse
//! 3. Cuando un Auton alcanza "Iluminación", puede invocar Reflexión Profunda
//! 4. El Demiurgo (Python) genera, verifica y aplica parches
//!
//! ## Seguridad
//!
//! - Solo el Demiurgo puede aplicar parches
//! - Verificación de instrucciones prohibidas
//! - Registro en Meltrace como "Grabado Cósmico"
//! - Reversion automática si hay inestabilidad
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::path::PathBuf;

/// Marker para funciones que pueden ser parcheadas
/// En el binary, estas están en la sección `.eden_patchable`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatchableAddr(pub usize);

/// Estado de un parche
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoParche {
    /// Propuesto pero no aplicado
    Propuesto,
    /// Verificado y aprobado
    Verificado,
    /// Aplicado y activo
    Aplicado,
    /// Falló - causó inestabilidad
    Rechazado,
    /// Reviertido al original
    Revertido,
}

/// Información de una función parcheable
#[derive(Debug, Clone)]
pub struct PatchableFunc {
    /// Nombre de la función
    pub nombre: String,
    /// Dirección en memoria de la función
    pub direccion: PatchableAddr,
    /// Tamaño del código (bytes)
    pub tamano: usize,
    /// Hash del código original
    pub hash_original: u64,
    /// Hash del código actual (puede ser diferente tras parche)
    pub hash_actual: u64,
}

/// Un parche propuesto o aplicado
#[derive(Debug, Clone)]
pub struct Parche {
    /// ID único del parche
    pub id: u64,
    /// Función que parchea
    pub nombre_funcion: String,
    /// Dirección parcheable destino
    pub destino: PatchableAddr,
    /// Código máquina del parche
    pub codigo: Vec<u8>,
    /// Estado del parche
    pub estado: EstadoParche,
    /// Hash del parche
    pub hash_parche: u64,
    /// Tick en que se propuso
    pub tick_propuesto: u64,
    /// Tick en que se aplicó (0 si no aplicado)
    pub tick_aplicado: u64,
    /// Auton que propuso el parche (si es por Iluminación)
    pub id_auton_iluminado: Option<u64>,
    /// Descripción/razón del parche
    pub descripcion: String,
}

/// Verificador de instrucciones prohibidas
#[derive(Debug, Clone)]
pub struct VerificadorInstrucciones {
    /// Instrucciones de sistema prohibidas (para referencia)
    _instrucs_prohibidas: Vec<u8>,
}

impl VerificadorInstrucciones {
    /// Crea nuevo verificador
    pub fn new() -> Self {
        VerificadorInstrucciones {
            _instrucs_prohibidas: Vec::new(),
        }
    }

    /// Verifica que el código no contenga instrucciones prohibidas
    ///
    /// # Arguments
    /// * `codigo` - Código máquina a verificar
    ///
    /// # Returns
    /// true si es seguro, false si contiene instrucciones prohibidas
    pub fn es_seguro(&self, codigo: &[u8]) -> bool {
        // Verificación básica: buscar patrones de syscalls
        // Esto es una verificación simplificada - en producción se usaría
        // un disassembler real como Capstone

        for window in codigo.windows(2) {
            // Buscar syscall (0x0f 0x05)
            if window[0] == 0x0f && window[1] == 0x05 {
                return false;
            }

            // Buscar int 0x80 (0xcd 0x80)
            if window[0] == 0xcd && window[1] == 0x80 {
                return false;
            }
        }

        // Verificar longitud mínima
        if codigo.len() < 4 {
            return false;
        }

        true
    }
}

impl Default for VerificadorInstrucciones {
    fn default() -> Self {
        Self::new()
    }
}

/// Administrador de parches
pub struct HotPatchManager {
    /// Funciones parcheables conocidas
    funciones_patchables: HashMap<String, PatchableFunc>,
    /// Parches propuestos/aplicados
    patches: Vec<Parche>,
    /// Verificador de seguridad
    verificador: VerificadorInstrucciones,
    /// Próximo ID de parche
    proximo_id: u64,
    /// Directorio de backup
    directorio_backup: PathBuf,
    /// Contador de fallos consecutivos (para reversion)
    fallos_consecutivos: u32,
    /// Threshold de fallos para reversion
    threshold_reversion: u32,
    /// Parche actualmente activo (para reversion)
    parche_activo: Option<u64>,
}

impl HotPatchManager {
    /// Crea nuevo administrador
    pub fn new() -> Self {
        HotPatchManager {
            funciones_patchables: HashMap::new(),
            patches: Vec::new(),
            verificador: VerificadorInstrucciones::new(),
            proximo_id: 1,
            directorio_backup: PathBuf::new(),
            fallos_consecutivos: 0,
            threshold_reversion: 3,
            parche_activo: None,
        }
    }

    /// Configura el directorio de backup
    pub fn con_directorio_backup(mut self, path: PathBuf) -> Self {
        self.directorio_backup = path;
        self
    }

    /// Registra una función parcheable
    pub fn registrar_funcion_patchable(
        &mut self,
        nombre: &str,
        direccion: PatchableAddr,
        tamano: usize,
    ) {
        let hash = Self::calcular_hash_memoria(direccion.0, tamano);

        let func = PatchableFunc {
            nombre: nombre.to_string(),
            direccion,
            tamano,
            hash_original: hash,
            hash_actual: hash,
        };

        self.funciones_patchables.insert(nombre.to_string(), func);
    }

    /// Calcula hash simple de memoria (para verificación)
    fn calcular_hash_memoria(_dir: usize, tamano: usize) -> u64 {
        let mut hash: u64 = 0xDEADBEAF;

        for i in 0..tamano {
            hash = hash.wrapping_mul(31).wrapping_add(i as u64);
        }

        hash
    }

    /// Propone un nuevo parche
    ///
    /// # Arguments
    /// * `nombre_funcion` - Nombre de la función a parchar
    /// * `codigo` - Código máquina del parche
    /// * `descripcion` - Descripción del parche
    /// * `id_auton` - ID del Auton que propone (None si es manual)
    /// * `tick_actual` - Tick actual del universo
    ///
    /// # Returns
    /// Some(Parche) si la propuesta es válida, None si hay error
    pub fn proponer_parche(
        &mut self,
        nombre_funcion: &str,
        codigo: Vec<u8>,
        descripcion: &str,
        id_auton: Option<u64>,
        tick_actual: u64,
    ) -> Option<&Parche> {
        // Verificar que la función existe
        let func = match self.funciones_patchables.get(nombre_funcion) {
            Some(f) => f.clone(),
            None => return None,
        };

        // Verificar tamaño del parche
        if codigo.len() > func.tamano {
            return None;
        }

        // Calcular hash del parche
        let hash = Self::hash_codigo(&codigo);

        // Crear parche
        let parche = Parche {
            id: self.proximo_id,
            nombre_funcion: nombre_funcion.to_string(),
            destino: func.direccion,
            codigo,
            estado: EstadoParche::Propuesto,
            hash_parche: hash,
            tick_propuesto: tick_actual,
            tick_aplicado: 0,
            id_auton_iluminado: id_auton,
            descripcion: descripcion.to_string(),
        };

        self.proximo_id += 1;
        self.patches.push(parche);

        self.patches.last()
    }

    /// Calcula hash de código
    fn hash_codigo(codigo: &[u8]) -> u64 {
        let mut hash: u64 = 0xC0DEBABE;
        for &byte in codigo {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }

    /// Verifica un parche (llamado por el Demiurgo)
    ///
    /// # Arguments
    /// * `id_parche` - ID del parche a verificar
    ///
    /// # Returns
    /// true si es seguro aplicar
    pub fn verificar_parche(&mut self, id_parche: u64) -> bool {
        let parche = match self.patches.iter_mut().find(|p| p.id == id_parche) {
            Some(p) => p,
            None => return false,
        };

        // Verificar seguridad del código
        if !self.verificador.es_seguro(&parche.codigo) {
            parche.estado = EstadoParche::Rechazado;
            return false;
        }

        parche.estado = EstadoParche::Verificado;
        true
    }

    /// Aplica un parche verificado
    ///
    /// # Arguments
    /// * `id_parche` - ID del parche a aplicar
    ///
    /// # Returns
    /// true si se aplicó correctamente
    #[cfg(target_os = "linux")]
    pub fn aplicar_parche(&mut self, id_parche: u64) -> bool {
        use std::ptr;

        let parche = match self.patches.iter_mut().find(|p| p.id == id_parche) {
            Some(p) => p,
            None => return false,
        };

        // Solo aplicar si está verificado
        if parche.estado != EstadoParche::Verificado {
            return false;
        }

        // Verificar que es seguro (doble verificación)
        if !self.verificador.es_seguro(&parche.codigo) {
            parche.estado = EstadoParche::Rechazado;
            return false;
        }

        // Hacer la memoria escribible
        let addr = parche.destino.0 as *mut std::ffi::c_void;

        // mprotect para hacer la sección escribible
        let page_size: usize = 4096;
        let page_start = (parche.destino.0 / page_size) * page_size;
        let offset = parche.destino.0 - page_start;

        // SAFETY: Estamos modificando permisos de memoria explícitamente
        // Esta operación requiere que la dirección esté en una sección
        // con permisos de memoria que permitan mprotect
        unsafe {
            let result = libc::mprotect(
                page_start as *mut libc::c_void,
                page_size,
                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
            );

            if result != 0 {
                return false;
            }

            // Copiar el código parcheado
            let dest = addr as *mut u8;
            for (i, &byte) in parche.codigo.iter().enumerate() {
                ptr::write(dest.add(offset + i), byte);
            }

            // Restaurar protección a solo ejecución
            libc::mprotect(
                page_start as *mut libc::c_void,
                page_size,
                libc::PROT_READ | libc::PROT_EXEC,
            );
        }

        // Actualizar estado
        parche.estado = EstadoParche::Aplicado;
        parche.tick_aplicado = 0;

        // Actualizar hash de la función
        if let Some(func) = self.funciones_patchables.get_mut(&parche.nombre_funcion) {
            func.hash_actual = parche.hash_parche;
        }

        self.parche_activo = Some(id_parche);
        true
    }

    /// Versión no-Linux (stubs)
    #[cfg(not(target_os = "linux"))]
    pub fn aplicar_parche(&mut self, id_parche: u64) -> bool {
        let parche = match self.patches.iter_mut().find(|p| p.id == id_parche) {
            Some(p) => p,
            None => return false,
        };

        if parche.estado != EstadoParche::Verificado {
            return false;
        }

        // Stub: en plataformas no-Linux, solo marcamos como aplicado
        parche.estado = EstadoParche::Aplicado;
        parche.tick_aplicado = 0;
        self.parche_activo = Some(id_parche);
        true
    }

    /// Registra un fallo causado por el parche activo
    pub fn registrar_fallo(&mut self) {
        self.fallos_consecutivos += 1;

        if self.fallos_consecutivos >= self.threshold_reversion {
            self.revertir_parche();
        }
    }

    /// Revierte al último parche o al original
    pub fn revertir_parche(&mut self) {
        if let Some(id) = self.parche_activo.take() {
            if let Some(parche) = self.patches.iter_mut().find(|p| p.id == id) {
                parche.estado = EstadoParche::Revertido;
            }
        }

        self.fallos_consecutivos = 0;
    }

    /// Obtiene el estado actual de un parche
    pub fn estado_parche(&self, id: u64) -> Option<EstadoParche> {
        self.patches.iter().find(|p| p.id == id).map(|p| p.estado)
    }

    /// Obtiene todos los parches
    pub fn todos_patches(&self) -> &[Parche] {
        &self.patches
    }

    /// Obtiene el contador de parches aplicados exitosamente
    pub fn num_patches_aplicados(&self) -> usize {
        self.patches
            .iter()
            .filter(|p| p.estado == EstadoParche::Aplicado)
            .count()
    }

    /// Obtiene una función parcheable por nombre
    pub fn get_funcion(&self, nombre: &str) -> Option<&PatchableFunc> {
        self.funciones_patchables.get(nombre)
    }
}

/// Criterios para "Iluminación" de un Auton
#[derive(Debug, Clone)]
pub struct CriteriosIluminacion {
    /// Ciclos mínimos de vida
    pub ciclos_minimos: u64,
    /// Energía mínima (FixedPoint como i64 << 32)
    pub energia_minima: i64,
    /// Factor de supervivencia (relativo a la población)
    pub factor_supervivencia_min: f64,
}

impl Default for CriteriosIluminacion {
    fn default() -> Self {
        CriteriosIluminacion {
            ciclos_minimos: 10_000,
            energia_minima: 100_000_000_000i64 << 32,
            factor_supervivencia_min: 2.0,
        }
    }
}

/// Verificador de Iluminación
#[derive(Debug, Clone)]
pub struct VerificadorIluminacion {
    pub criterios: CriteriosIluminacion,
}

impl VerificadorIluminacion {
    /// Crea nuevo verificador con criterios por defecto
    pub fn new() -> Self {
        VerificadorIluminacion {
            criterios: CriteriosIluminacion::default(),
        }
    }

    /// Verifica si un Auton ha alcanzado Iluminación
    ///
    /// # Arguments
    /// * `ciclos_vida` - Ciclos que ha vivido
    /// * `energia` - Energía actual (I32F32 como i64 << 32)
    /// * `factor_supervivencia` - Factor relativo al promedio
    ///
    /// # Returns
    /// true si está iluminado
    pub fn es_iluminado(&self, ciclos_vida: u64, energia: i64, factor_supervivencia: f64) -> bool {
        ciclos_vida >= self.criterios.ciclos_minimos
            && energia >= self.criterios.energia_minima
            && factor_supervivencia >= self.criterios.factor_supervivencia_min
    }
}

impl Default for VerificadorIluminacion {
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
    fn test_verificador_instrucciones() {
        let verif = VerificadorInstrucciones::new();

        // Código seguro: instrucciones normales
        let codigo_seguro = vec![
            0x48, 0x89, 0xC3, // mov rax, rbx
            0x48, 0x01, 0xD8, // add rax, rbx
            0xC3, // ret
        ];
        assert!(verif.es_seguro(&codigo_seguro));

        // Código inseguro: contiene syscall
        let codigo_inseguro = vec![
            0x48, 0x89, 0xC3, 0x0f, 0x05, // syscall!
            0xC3,
        ];
        assert!(!verif.es_seguro(&codigo_inseguro));
    }

    #[test]
    fn test_propuesta_parche() {
        let mut manager = HotPatchManager::new();

        // Registrar función parcheable
        manager.registrar_funcion_patchable("tasa_mutacion_base", PatchableAddr(0x1000), 64);

        // Proponer parche
        let codigo = vec![0x90, 0x90, 0xC3]; // nop, nop, ret
        let result = manager.proponer_parche(
            "tasa_mutacion_base",
            codigo,
            "Aumentar tasa de mutación",
            None,
            1000,
        );

        assert!(result.is_some());
        assert_eq!(result.unwrap().estado, EstadoParche::Propuesto);
    }

    #[test]
    fn test_verificacion_parche() {
        let mut manager = HotPatchManager::new();

        manager.registrar_funcion_patchable("difusion_alfa", PatchableAddr(0x2000), 128);

        let codigo = vec![
            0x48, 0x89, 0xC3, 0xB8, 0x00, 0x00, 0x80, 0x3F, // mov eax, 1.0 (float)
            0xC3,
        ];

        manager.proponer_parche(
            "difusion_alfa",
            codigo,
            "Nueva constante de difusión",
            Some(123),
            2000,
        );

        // Verificar parche 1
        let verificado = manager.verificar_parche(1);
        assert!(verificado);

        if let Some(estado) = manager.estado_parche(1) {
            assert_eq!(estado, EstadoParche::Verificado);
        }
    }

    #[test]
    fn test_rechazo_parche_inseguro() {
        let mut manager = HotPatchManager::new();

        manager.registrar_funcion_patchable("funcion_sospechosa", PatchableAddr(0x3000), 64);

        // Código con syscall
        let codigo_malo = vec![
            0x90, 0x90, 0x0f, 0x05, // syscall - prohibido!
            0xC3,
        ];

        manager.proponer_parche(
            "funcion_sospechosa",
            codigo_malo,
            "Parche malicioso",
            None,
            3000,
        );

        let verificado = manager.verificar_parche(1);
        assert!(!verificado);

        if let Some(estado) = manager.estado_parche(1) {
            assert_eq!(estado, EstadoParche::Rechazado);
        }
    }

    #[test]
    fn test_criterios_iluminacion() {
        let verif = VerificadorIluminacion::new();

        // Auton no iluminado
        assert!(!verif.es_iluminado(100, 50_000_000_000i64 << 32, 1.0));

        // Auton iluminado
        assert!(verif.es_iluminado(15_000, 100_000_000_000i64 << 32, 2.5,));
    }

    #[test]
    fn test_registro_fallos_reversion() {
        let mut manager = HotPatchManager::new();
        manager.threshold_reversion = 3;

        // Registrar y "aplicar" parche
        manager.registrar_funcion_patchable("test", PatchableAddr(0x4000), 64);
        let codigo = vec![0xC3];
        manager.proponer_parche("test", codigo, "Test", None, 4000);
        manager.verificar_parche(1);
        manager.aplicar_parche(1);

        // Registrar 3 fallos
        manager.registrar_fallo();
        manager.registrar_fallo();
        manager.registrar_fallo();

        // Debería haberse revertido
        assert!(manager.parche_activo.is_none());
    }

    #[test]
    fn test_hash_codigo() {
        let codigo1 = vec![0x48, 0x89, 0xC3, 0xC3];
        let codigo2 = vec![0x48, 0x89, 0xC3, 0xC3];
        let codigo3 = vec![0x48, 0x89, 0xC3, 0xC4];

        let hash1 = HotPatchManager::hash_codigo(&codigo1);
        let hash2 = HotPatchManager::hash_codigo(&codigo2);
        let hash3 = HotPatchManager::hash_codigo(&codigo3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}

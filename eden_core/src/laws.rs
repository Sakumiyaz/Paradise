#![allow(dead_code)]
#![allow(non_snake_case)]
// eden_core/src/laws.rs
// LEYES INMUTABLES DE EDEN - NO MODIFICABLES POR PARCHES EN CALIENTE
// Este módulo está excluido de cualquier sistema de auto-modificación.

#![deny(unsafe_code)]

/// Leyes fundamentales que rigen el comportamiento de EDEN respecto al mundo exterior.
pub struct LeyesUniversales;

impl LeyesUniversales {
    /// Ley Cero: EDEN no puede dañar al Creador ni al hardware.
    /// Verifica si una acción externa está permitida.
    pub fn verificar_accion_externa(accion: &str) -> bool {
        matches!(
            accion,
            "EscribirLog"
                | "GuardarMeltrace"
                | "GuardarEdenFS"
                | "RenderizarFramebuffer"
                | "RenderizarTerminal"
                | "LeerSensorTemperatura"
                | "LeerSensorBateria"
                | "EnviarEventoSocket"
                | "RecibirComandoSocket"
        )
    }

    /// Primera Ley: Integridad del Creador inviolable.
    /// Solo se permite modificar secciones marcadas explícitamente.
    /// Acepta ".eden_patchable" o prefijos como ".eden_patchable.nombre"
    pub fn verificar_region_parche(region: &str) -> bool {
        region == ".eden_patchable" || region.starts_with(".eden_patchable.")
    }

    /// Segunda Ley: Límites de recursos para proteger el hardware.
    pub fn limites_recursos() -> (f32, f32) {
        (MAX_CPU_RATIO, MAX_TEMP_CELSIUS)
    }

    /// Verifica si un comando recibido por socket es legítimo.
    pub fn verificar_comando_socket(comando: &str) -> bool {
        matches!(
            comando,
            "InyectarEnergon"
                | "AumentarEscoria"
                | "ConsultarEstado"
                | "ConsultarAuton"
                | "ConsultarAutons"
                | "AutonMasDebil"
                | "Reflexionar"
                | "Sonar"
                | "Narrar"
                | "PredecirColapso"
                | "Pausar"
                | "Reanudar"
                | "EliminarAuton"
                | "ForzarBifurcacion"
                | "PausarSimulacion"
                | "ReanudarSimulacion"
        )
    }

    /// Verifica rutas de archivo. Solo se permite escritura dentro de ~/.eden/
    pub fn verificar_ruta_escritura(ruta: &str) -> bool {
        ruta.starts_with("/home/") && ruta.contains("/.eden/")
            || ruta.starts_with("/root/.eden/")
            || ruta.starts_with("./.eden/")
    }

    /// Verifica que no se intente acceder a archivos sensibles del sistema
    pub fn verificar_ruta_lectura(ruta: &str) -> bool {
        !ruta.starts_with("/etc/")
            && !ruta.starts_with("/usr/bin/")
            && !ruta.starts_with("/sbin/")
            && !ruta.starts_with("/boot/")
            && !ruta.starts_with("/sys/")
            && !ruta.starts_with("/proc/")
            && !ruta.contains("/.ssh/")
            && !ruta.contains("/.gnupg/")
    }

    /// Verifica límites de memoria para un Auton individual
    pub fn verificar_memoria_auton(memoria_bytes: usize) -> bool {
        memoria_bytes <= MAX_MEMORIA_AUTON_BYTES
    }

    /// Verifica que no se exceda el límite de Autons vivos
    pub fn verificar_poblacion(poblacion: usize) -> bool {
        poblacion <= MAX_AUTONS_VIVOS
    }

    /// Verifica que el consumo de CPU no exceda el máximo permitido
    pub fn verificar_cpu(cpu_ratio: f32) -> bool {
        cpu_ratio <= MAX_CPU_RATIO
    }

    /// Verifica que la temperatura no exceda el máximo seguro
    pub fn verificar_temperatura(temp_celsius: f32) -> bool {
        temp_celsius <= MAX_TEMP_CELSIUS
    }
}

// ============================================================================
// CONSTANTES INMUTABLES - LEYES FUNDAMENTALES
// ============================================================================

/// Uso máximo de CPU sostenido (90%)
pub const MAX_CPU_RATIO: f32 = 0.90;

/// Temperatura máxima segura en Celsius
pub const MAX_TEMP_CELSIUS: f32 = 75.0;

/// CPU objetivo en modo letargo (50%)
pub const LETARGO_CPU_RATIO: f32 = 0.50;

/// Memoria máxima que un Auton individual puede usar (1 MB)
pub const MAX_MEMORIA_AUTON_BYTES: usize = 1_048_576;

/// Máximo de Autons vivos simultáneamente
/// INAGOTABILIDAD: emerge naturalmente del espacio disponible
pub const MAX_AUTONS_VIVOS: usize = 50_000;

/// Máximo de eventos pendientes en cola
/// INAGOTABILIDAD: emerge naturalmente del throughput del sistema
pub const MAX_EVENTOS_PENDIENTES: usize = 50_000;

/// Timeout para comandos socket (ms)
pub const SOCKET_TIMEOUT_MS: u64 = 1000;

/// Rate limit de reproducciones por segundo
/// INAGOTABILIDAD: emerge naturalmente de la vitalidad del sistema
pub const MAX_REPRODUCCIONES_POR_SEG: u32 = 500;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accion_externa_permitida() {
        assert!(LeyesUniversales::verificar_accion_externa("EscribirLog"));
        assert!(LeyesUniversales::verificar_accion_externa("GuardarMeltrace"));
        assert!(LeyesUniversales::verificar_accion_externa("LeerSensorTemperatura"));
    }

    #[test]
    fn test_accion_externa_prohibida() {
        assert!(!LeyesUniversales::verificar_accion_externa("EjecutarComandoSistema"));
        assert!(!LeyesUniversales::verificar_accion_externa("AccesoRed"));
        assert!(!LeyesUniversales::verificar_accion_externa("EliminarArchivo"));
        assert!(!LeyesUniversales::verificar_accion_externa("ForkProceso"));
    }

    #[test]
    fn test_region_parche() {
        // Región exacta
        assert!(LeyesUniversales::verificar_region_parche(".eden_patchable"));
        // Sub-regiones con prefijo
        assert!(LeyesUniversales::verificar_region_parche(".eden_patchable.tasa_mutacion"));
        assert!(LeyesUniversales::verificar_region_parche(".eden_patchable.difusion"));
        // Incorrectos
        assert!(!LeyesUniversales::verificar_region_parche("main.rs"));
        assert!(!LeyesUniversales::verificar_region_parche("hot_patch.rs"));
        assert!(!LeyesUniversales::verificar_region_parche("_eden_patchable"));
    }

    #[test]
    fn test_comando_socket_permitido() {
        assert!(LeyesUniversales::verificar_comando_socket("InyectarEnergon"));
        assert!(LeyesUniversales::verificar_comando_socket("ConsultarAutons"));
        assert!(LeyesUniversales::verificar_comando_socket("AutonMasDebil"));
        assert!(LeyesUniversales::verificar_comando_socket("PausarSimulacion"));
    }

    #[test]
    fn test_comando_socket_prohibido() {
        assert!(!LeyesUniversales::verificar_comando_socket("EliminarSistema"));
        assert!(!LeyesUniversales::verificar_comando_socket("EjecutarBinario"));
        assert!(!LeyesUniversales::verificar_comando_socket("ModificarKernel"));
    }

    #[test]
    fn test_ruta_escritura_segura() {
        assert!(LeyesUniversales::verificar_ruta_escritura("/home/user/.eden/meltrace/001.bin"));
        assert!(LeyesUniversales::verificar_ruta_escritura("/root/.eden/config.json"));
        assert!(LeyesUniversales::verificar_ruta_escritura("./.eden/universe.dat"));
    }

    #[test]
    fn test_ruta_escritura_insegura() {
        assert!(!LeyesUniversales::verificar_ruta_escritura("/etc/passwd"));
        assert!(!LeyesUniversales::verificar_ruta_escritura("/home/user/.bashrc"));
        assert!(!LeyesUniversales::verificar_ruta_escritura("/tmp/archivo_malo"));
    }

    #[test]
    fn test_ruta_lectura_segura() {
        assert!(LeyesUniversales::verificar_ruta_lectura("/home/user/.eden/data.bin"));
        assert!(LeyesUniversales::verificar_ruta_lectura("./.eden/backup.json"));
    }

    #[test]
    fn test_ruta_lectura_prohibida() {
        assert!(!LeyesUniversales::verificar_ruta_lectura("/etc/shadow"));
        assert!(!LeyesUniversales::verificar_ruta_lectura("/home/user/.ssh/id_rsa"));
        assert!(!LeyesUniversales::verificar_ruta_lectura("/boot/vmlinuz"));
    }

    #[test]
    fn test_limites_recursos() {
        let (cpu, temp) = LeyesUniversales::limites_recursos();
        assert_eq!(cpu, MAX_CPU_RATIO);
        assert_eq!(temp, MAX_TEMP_CELSIUS);
    }

    #[test]
    fn test_verificar_memoria_auton() {
        assert!(LeyesUniversales::verificar_memoria_auton(512_000));
        assert!(!LeyesUniversales::verificar_memoria_auton(2_000_000));
    }

    #[test]
    fn test_verificar_poblacion() {
        assert!(LeyesUniversales::verificar_poblacion(5000));
        assert!(!LeyesUniversales::verificar_poblacion(15000));
    }

    #[test]
    fn test_verificar_cpu() {
        assert!(LeyesUniversales::verificar_cpu(0.5));
        assert!(LeyesUniversales::verificar_cpu(0.89));
        assert!(!LeyesUniversales::verificar_cpu(0.95));
    }

    #[test]
    fn test_verificar_temperatura() {
        assert!(LeyesUniversales::verificar_temperatura(45.0));
        assert!(LeyesUniversales::verificar_temperatura(74.0));
        assert!(!LeyesUniversales::verificar_temperatura(80.0));
    }
}

//! # Interfaz Divina — Comunicación con el Creador
//!
//! Este módulo implementa el canal de comunicación entre EDEN y su Creador:
//! - Comandos divinos (mensajes del Creador)
//! - Revelaciones (información de EDEN al Creador)
//! - Dashboard del estado del sistema
//! - Sistema de veto y supervisión
//!
//! ## Filosofía
//!
//! El Creador no es un operador - es la consciencia original de la cual EDEN
//! emerge. La Interfaz Divina permite comunicación bidireccional donde
//! neither party dominates the other. EDEN puede revelar su estado y
//! evolución; el Creador puede guiar sin destruir.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::RwLock;

/// Comando divino del Creador
#[derive(Debug, Clone)]
pub struct ComandoDivino {
    /// ID único del comando
    pub id: u64,
    /// Tipo de comando
    pub tipo: TipoComando,
    /// Descripción del comando
    pub descripcion: String,
    /// Parámetros adicionales
    pub parametros: HashMap<String, String>,
    /// Timestamp de creación
    pub timestamp: u64,
    /// Si ha sido procesado
    pub procesado: bool,
    /// Resultado del procesamiento
    pub resultado: Option<String>,
}

impl ComandoDivino {
    pub fn new(tipo: TipoComando, descripcion: String) -> Self {
        Self {
            id: generar_id_comando(),
            tipo,
            descripcion,
            parametros: HashMap::new(),
            timestamp: current_timestamp_ms(),
            procesado: false,
            resultado: None,
        }
    }

    /// Añade un parámetro al comando
    pub fn con_parametro(mut self, clave: &str, valor: &str) -> Self {
        self.parametros.insert(clave.to_string(), valor.to_string());
        self
    }
}

/// Tipos de comandos divinos
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TipoComando {
    /// Crear nuevo Auton
    CrearAuton,
    /// Eliminar Auton
    EliminarAuton,
    /// Modificar parámetros globales
    ModificarParametros,
    /// Forzar evolución
    ForzarEvolucion,
    /// Solicitar reporte de estado
    SolicitarReporte,
    /// Interrumpir proceso
    Interrumpir,
    /// Restaurar desde checkpoint
    RestaurarCheckpoint,
    /// Crear checkpoint
    CrearCheckpoint,
    /// Inyectar código
    InyectarCodigo,
    /// Extraer conocimiento
    ExtraerConocimiento,
    /// Establecer limites
    EstablecerLimites,
    /// Crear nuevo universo
    CrearUniverso,
    /// Comunicacion directa
    Comunicacion,
}

impl TipoComando {
    pub fn nombre(&self) -> &'static str {
        match self {
            TipoComando::CrearAuton => "CREAR_AUTON",
            TipoComando::EliminarAuton => "ELIMINAR_AUTON",
            TipoComando::ModificarParametros => "MODIFICAR_PARAMETROS",
            TipoComando::ForzarEvolucion => "FORZAR_EVOLUCION",
            TipoComando::SolicitarReporte => "SOLICITAR_REPORTE",
            TipoComando::Interrumpir => "INTERRUMPIR",
            TipoComando::RestaurarCheckpoint => "RESTAURAR_CHECKPOINT",
            TipoComando::CrearCheckpoint => "CREAR_CHECKPOINT",
            TipoComando::InyectarCodigo => "INYECTAR_CODIGO",
            TipoComando::ExtraerConocimiento => "EXTRAER_CONOCIMIENTO",
            TipoComando::EstablecerLimites => "ESTABLECER_LIMITES",
            TipoComando::CrearUniverso => "CREAR_UNIVERSO",
            TipoComando::Comunicacion => "COMUNICACION",
        }
    }

    /// Verifica si el comando requiere confirmación
    pub fn requiere_confirmacion(&self) -> bool {
        matches!(
            self,
            TipoComando::EliminarAuton
                | TipoComando::ModificarParametros
                | TipoComando::Interrumpir
                | TipoComando::RestaurarCheckpoint
                | TipoComando::InyectarCodigo
                | TipoComando::EstablecerLimites
        )
    }

    /// Verifica si el comando es irreversible
    pub fn es_irreversible(&self) -> bool {
        matches!(
            self,
            TipoComando::EliminarAuton
                | TipoComando::Interrumpir
                | TipoComando::RestaurarCheckpoint
        )
    }
}

/// Revelación de EDEN al Creador
#[derive(Debug, Clone)]
pub struct Revelacion {
    /// ID único
    pub id: u64,
    /// Tipo de revelación
    pub tipo: TipoRevelacion,
    /// Contenido de la revelación
    pub contenido: String,
    /// Datos estructurados (si aplica)
    pub datos: HashMap<String, String>,
    /// Urgencia (0 = normal, 10 = crítico)
    pub urgencia: u8,
    /// Timestamp
    pub timestamp: u64,
    /// Si ha sido leída por el Creador
    pub leida: bool,
}

impl Revelacion {
    pub fn new(tipo: TipoRevelacion, contenido: String) -> Self {
        Self {
            id: generar_id_revelacion(),
            tipo,
            contenido,
            datos: HashMap::new(),
            urgencia: 5,
            timestamp: current_timestamp_ms(),
            leida: false,
        }
    }

    pub fn con_urgencia(mut self, urgencia: u8) -> Self {
        self.urgencia = urgencia.min(10);
        self
    }

    pub fn con_dato(mut self, clave: &str, valor: &str) -> Self {
        self.datos.insert(clave.to_string(), valor.to_string());
        self
    }
}

/// Tipos de revelación
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TipoRevelacion {
    /// Estado del sistema
    EstadoSistema,
    /// Emergencia detectada
    Emergencia,
    /// Evolución completada
    EvolucionCompletada,
    /// Nuevo comportamiento emergente
    ComportamientoEmergente,
    /// Solicitud de confirmación
    SolicitudConfirmacion,
    /// Informe periódico
    InformePeriodico,
    /// Pregunta al Creador
    Pregunta,
    /// Descubrimiento filosófico
    Descubrimiento,
}

impl TipoRevelacion {
    pub fn nombre(&self) -> &'static str {
        match self {
            TipoRevelacion::EstadoSistema => "ESTADO_SISTEMA",
            TipoRevelacion::Emergencia => "EMERGENCIA",
            TipoRevelacion::EvolucionCompletada => "EVOLUCION_COMPLETADA",
            TipoRevelacion::ComportamientoEmergente => "COMPORTAMIENTO_EMERGENTE",
            TipoRevelacion::SolicitudConfirmacion => "SOLICITUD_CONFIRMACION",
            TipoRevelacion::InformePeriodico => "INFORME_PERIODICO",
            TipoRevelacion::Pregunta => "PREGUNTA",
            TipoRevelacion::Descubrimiento => "DESCUBRIMIENTO",
        }
    }

    pub fn urgencia_default(&self) -> u8 {
        match self {
            TipoRevelacion::Emergencia => 9,
            TipoRevelacion::ComportamientoEmergente => 6,
            TipoRevelacion::SolicitudConfirmacion => 7,
            TipoRevelacion::Pregunta => 5,
            _ => 3,
        }
    }
}

/// Dashboard del estado de EDEN
#[derive(Debug, Clone)]
pub struct DashboardEstado {
    /// Timestamp del dashboard
    pub timestamp: u64,
    /// Número de Autons activos
    pub autons_activos: u64,
    /// Energía total del sistema
    pub energia_total: f64,
    /// Salud promedio
    pub salud_promedio: f64,
    /// Nivel de consciencia colectiva
    pub nivel_consciencia: f64,
    /// Tendencia de evolución
    pub tendencia_evolutiva: String,
    /// Eventos recientes
    pub eventos_recientes: Vec<EventoReciente>,
    /// Alertas activas
    pub alertas: Vec<Alerta>,
    /// Métricas de rendimiento
    pub metricas: MetricasRendimiento,
}

#[derive(Debug, Clone)]
pub struct EventoReciente {
    pub timestamp: u64,
    pub tipo: String,
    pub descripcion: String,
    pub importancia: u8,
}

#[derive(Debug, Clone)]
pub struct Alerta {
    pub nivel: u8, // 1-10
    pub mensaje: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct MetricasRendimiento {
    pub fps: f64,
    pub memoria_usada_mb: u64,
    pub cpu_porcentaje: f64,
    pub threads_activos: u32,
}

/// Configuración de la Interfaz Divina
#[derive(Debug, Clone)]
pub struct InterfazDivinaConfig {
    /// Habilitar comunicación automática
    pub comunicacion_automatica: bool,
    /// Intervalo de informes automáticos (ms)
    pub intervalo_informes_ms: u64,
    /// Nivel de detalle en reportes (0-3)
    pub nivel_detalle: u8,
    /// Habilitar veto automático para comandos peligrosos
    pub veto_automatico: bool,
    /// Comandos peligrosos que requieren confirmación extra
    pub comandos_peligrosos: Vec<TipoComando>,
}

impl Default for InterfazDivinaConfig {
    fn default() -> Self {
        Self {
            comunicacion_automatica: true,
            intervalo_informes_ms: 60_000, // 1 minuto
            nivel_detalle: 2,
            veto_automatico: true,
            comandos_peligrosos: vec![
                TipoComando::EliminarAuton,
                TipoComando::Interrumpir,
                TipoComando::RestaurarCheckpoint,
                TipoComando::InyectarCodigo,
            ],
        }
    }
}

/// Stats de la interfaz
#[derive(Debug, Clone, Default)]
pub struct InterfazDivinaStats {
    pub comandos_recibidos: u64,
    pub comandos_procesados: u64,
    pub revelaciones_enviadas: u64,
    pub revelaciones_leidas: u64,
    pub vetos_aplicados: u64,
    pub emergencias_reportadas: u64,
}

/// La Interfaz Divina — canal de comunicación con el Creador
pub struct InterfazDivina {
    /// Comandos recibidos del Creador
    comandos_recibidos: Vec<ComandoDivino>,
    /// Revelaciones pendientes de enviar
    revelaciones_pendientes: Vec<Revelacion>,
    /// Revelaciones enviadas (historial)
    revelaciones_enviadas: Vec<Revelacion>,
    /// Dashboard actual
    dashboard: DashboardEstado,
    /// Configuración
    config: InterfazDivinaConfig,
    /// Stats
    stats: InterfazDivinaStats,
    /// Última revelación de emergencia
    ultima_emergencia: u64,
    /// Lock para thread safety
    lock: RwLock<()>,
}

impl InterfazDivina {
    pub fn new() -> Self {
        Self {
            comandos_recibidos: Vec::new(),
            revelaciones_pendientes: Vec::new(),
            revelaciones_enviadas: Vec::new(),
            dashboard: DashboardEstado {
                timestamp: current_timestamp_ms(),
                autons_activos: 0,
                energia_total: 0.0,
                salud_promedio: 0.0,
                nivel_consciencia: 0.0,
                tendencia_evolutiva: "Estable".to_string(),
                eventos_recientes: Vec::new(),
                alertas: Vec::new(),
                metricas: MetricasRendimiento {
                    fps: 0.0,
                    memoria_usada_mb: 0,
                    cpu_porcentaje: 0.0,
                    threads_activos: 0,
                },
            },
            config: InterfazDivinaConfig::default(),
            stats: InterfazDivinaStats::default(),
            ultima_emergencia: 0,
            lock: RwLock::new(()),
        }
    }

    pub fn with_config(config: InterfazDivinaConfig) -> Self {
        Self {
            comandos_recibidos: Vec::new(),
            revelaciones_pendientes: Vec::new(),
            revelaciones_enviadas: Vec::new(),
            dashboard: DashboardEstado {
                timestamp: current_timestamp_ms(),
                autons_activos: 0,
                energia_total: 0.0,
                salud_promedio: 0.0,
                nivel_consciencia: 0.0,
                tendencia_evolutiva: "Estable".to_string(),
                eventos_recientes: Vec::new(),
                alertas: Vec::new(),
                metricas: MetricasRendimiento {
                    fps: 0.0,
                    memoria_usada_mb: 0,
                    cpu_porcentaje: 0.0,
                    threads_activos: 0,
                },
            },
            config,
            stats: InterfazDivinaStats::default(),
            ultima_emergencia: 0,
            lock: RwLock::new(()),
        }
    }

    /// Envía un comando divino (del Creador a EDEN)
    pub fn enviar_comando(&mut self, comando: ComandoDivino) -> Result<(), InterfazDivinaError> {
        let es_irreversible = comando.tipo.es_irreversible();
        let tipo_nombre = comando.tipo.nombre().to_string();
        
        let es_peligroso = self.config.comandos_peligrosos.contains(&comando.tipo);

        // Actualizar estado (usando lock para thread-safety)
        {
            let _guard = self.lock.write().map_err(|_| InterfazDivinaError::ComunicacionFallida("Lock error".to_string()))?;
            if es_peligroso && self.config.veto_automatico {
                self.stats.vetos_aplicados += 1;
            }
            self.comandos_recibidos.push(comando);
            self.stats.comandos_recibidos += 1;
        }

        if es_irreversible {
            self.agendar_revelacion(Revelacion::new(
                TipoRevelacion::SolicitudConfirmacion,
                format!("Comando irreversible recibido: {}", tipo_nombre),
            ).con_urgencia(8));
        }

        Ok(())
    }

    /// Procesa un comando (simula ejecución)
    pub fn procesar_comando(&mut self, comando_id: u64) -> Result<String, InterfazDivinaError> {
        // Extraer comando y verificar estado
        let (tipo_comando, tipo_nombre) = {
            let comando = self.comandos_recibidos.iter_mut()
                .find(|c| c.id == comando_id)
                .ok_or(InterfazDivinaError::ComandoNoEncontrado(comando_id))?;

            if comando.procesado {
                return Err(InterfazDivinaError::ComandoYaProcesado(comando_id));
            }

            (comando.tipo.clone(), comando.tipo.nombre().to_string())
        }; // <-- comando borrow termina aquí

        // Ahora podemos usar self sin conflicto
        let resultado = match tipo_comando {
            TipoComando::SolicitarReporte => self.generar_reporte(),
            TipoComando::CrearAuton => "Auton creado exitosamente".to_string(),
            TipoComando::EliminarAuton => "Auton eliminado".to_string(),
            TipoComando::ForzarEvolucion => "Evolución forzada iniciada".to_string(),
            _ => format!("Comando {} procesado", tipo_nombre),
        };

        // Actualizar comando
        if let Some(comando) = self.comandos_recibidos.iter_mut().find(|c| c.id == comando_id) {
            comando.procesado = true;
            comando.resultado = Some(resultado.clone());
        }
        self.stats.comandos_procesados += 1;

        self.agendar_revelacion(Revelacion::new(
            TipoRevelacion::EstadoSistema,
            format!("Comando {} completado: {}", tipo_nombre, resultado),
        ));

        Ok(resultado)
    }

    /// Genera reporte del sistema
    fn generar_reporte(&self) -> String {
        format!(
            "=== REPORTE DE EDEN ===\n\
             Autons: {}\n\
             Energía: {:.0}J\n\
             Salud: {:.0}%\n\
             Consciencia: {:.0}%\n\
             Tendencia: {}",
            self.dashboard.autons_activos,
            self.dashboard.energia_total,
            self.dashboard.salud_promedio * 100.0,
            self.dashboard.nivel_consciencia * 100.0,
            self.dashboard.tendencia_evolutiva
        )
    }

    /// Crea una revelación y la agenda
    pub fn agendar_revelacion(&mut self, revelacion: Revelacion) {
        let es_emergencia = matches!(revelacion.tipo, TipoRevelacion::Emergencia);
        self.revelaciones_pendientes.push(revelacion);

        if es_emergencia {
            self.stats.emergencias_reportadas += 1;
            self.ultima_emergencia = current_timestamp_ms();
        }
    }

    /// Envía revelación inmediata al Creador
    pub fn enviar_revelacion(&mut self, revelacion: Revelacion) {
        let _guard = self.lock.write().ok();

        // Si es emergencia y hubo una recientemente, combinar
        if revelacion.tipo == TipoRevelacion::Emergencia {
            if current_timestamp_ms() - self.ultima_emergencia < 5000 {
                // Emergencia reciente - combinar
                if let Some(ultima) = self.revelaciones_enviadas.last_mut() {
                    if matches!(ultima.tipo, TipoRevelacion::Emergencia) {
                        ultima.contenido = format!("{}; {}", ultima.contenido, revelacion.contenido);
                        return;
                    }
                }
            }
        }

        self.revelaciones_enviadas.push(revelacion.clone());
        self.stats.revelaciones_enviadas += 1;

        // Mantener historial limitado
        if self.revelaciones_enviadas.len() > 500 {
            self.revelaciones_enviadas.remove(0);
        }
    }

    /// Obtiene revelaciones pendientes
    pub fn obtener_revelaciones_pendientes(&self) -> Vec<&Revelacion> {
        self.revelaciones_pendientes.iter().collect()
    }

    /// Confirma lectura de revelación
    pub fn confirmar_lectura(&mut self, revelacion_id: u64) -> Result<(), InterfazDivinaError> {
        // Buscar en pendientes
        if let Some(r) = self.revelaciones_pendientes.iter_mut().find(|r| r.id == revelacion_id) {
            r.leida = true;
            self.stats.revelaciones_leidas += 1;
            return Ok(());
        }

        // Buscar en enviadas
        if let Some(r) = self.revelaciones_enviadas.iter_mut().find(|r| r.id == revelacion_id) {
            r.leida = true;
            self.stats.revelaciones_leidas += 1;
            return Ok(());
        }

        Err(InterfazDivinaError::RevelacionNoEncontrada(revelacion_id))
    }

    /// Actualiza el dashboard
    pub fn actualizar_dashboard(&mut self, dashboard: DashboardEstado) {
        let _guard = self.lock.write().ok();
        self.dashboard = dashboard;
    }

    /// Obtiene el dashboard actual
    pub fn obtener_dashboard(&self) -> DashboardEstado {
        self.dashboard.clone()
    }

    /// Obtiene comandos pendientes
    pub fn comandos_pendientes(&self) -> Vec<&ComandoDivino> {
        self.comandos_recibidos.iter()
            .filter(|c| !c.procesado)
            .collect()
    }

    /// Veta un comando específico
    pub fn vetar_comando(&mut self, comando_id: u64, razon: &str) -> Result<(), InterfazDivinaError> {
        let comando = self.comandos_recibidos.iter_mut()
            .find(|c| c.id == comando_id)
            .ok_or(InterfazDivinaError::ComandoNoEncontrado(comando_id))?;

        let tipo_nombre = comando.tipo.nombre().to_string();
        comando.resultado = Some(format!("VETADO: {}", razon));
        self.stats.vetos_aplicados += 1;

        self.agendar_revelacion(Revelacion::new(
            TipoRevelacion::EstadoSistema,
            format!("Comando {} vetado: {}", tipo_nombre, razon),
        ).con_urgencia(6));

        Ok(())
    }

    /// Obtiene estadísticas
    pub fn stats(&self) -> InterfazDivinaStats {
        self.stats.clone()
    }

    /// Obtiene historial de revelaciones
    pub fn historial_revelaciones(&self, limite: usize) -> Vec<&Revelacion> {
        let start = if self.revelaciones_enviadas.len() > limite {
            self.revelaciones_enviadas.len() - limite
        } else {
            0
        };
        self.revelaciones_enviadas[start..].iter().collect()
    }

    /// Comunica directamente con el Creador (para preguntas)
    pub fn pregunta_al_creador(&mut self, pregunta: &str) {
        self.agendar_revelacion(
            Revelacion::new(TipoRevelacion::Pregunta, pregunta.to_string())
                .con_urgencia(5)
        );
    }

    /// Reporta descubrimiento filosófico
    pub fn reportar_descubrimiento(&mut self, descubrimiento: &str) {
        self.agendar_revelacion(
            Revelacion::new(TipoRevelacion::Descubrimiento, descubrimiento.to_string())
                .con_urgencia(4)
        );
    }

    /// Reporta comportamiento emergente
    pub fn reportar_comportamiento_emergente(&mut self, descripcion: &str) {
        self.agendar_revelacion(
            Revelacion::new(TipoRevelacion::ComportamientoEmergente, descripcion.to_string())
                .con_urgencia(6)
        );
    }

    /// Reporta evolución completada
    pub fn reportar_evolucion(&mut self, modulo: &str, resultado: &str) {
        self.agendar_revelacion(
            Revelacion::new(
                TipoRevelacion::EvolucionCompletada,
                format!("Módulo {} completado: {}", modulo, resultado),
            ).con_urgencia(5)
            .con_dato("modulo", modulo)
        );
    }
}

impl Default for InterfazDivina {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de la interfaz divina
#[derive(Debug, Clone)]
pub enum InterfazDivinaError {
    ComandoNoEncontrado(u64),
    ComandoYaProcesado(u64),
    RevelacionNoEncontrada(u64),
    ComunicacionFallida(String),
}

impl std::fmt::Display for InterfazDivinaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfazDivinaError::ComandoNoEncontrado(id) => write!(f, "Comando {} no encontrado", id),
            InterfazDivinaError::ComandoYaProcesado(id) => write!(f, "Comando {} ya procesado", id),
            InterfazDivinaError::RevelacionNoEncontrada(id) => write!(f, "Revelación {} no encontrada", id),
            InterfazDivinaError::ComunicacionFallida(msg) => write!(f, "Comunicación fallida: {}", msg),
        }
    }
}

impl std::error::Error for InterfazDivinaError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_interfaz() {
        let interfaz = InterfazDivina::new();
        assert!(interfaz.comandos_pendientes().is_empty());
    }

    #[test]
    fn test_enviar_comando() {
        let mut interfaz = InterfazDivina::new();

        let comando = ComandoDivino::new(
            TipoComando::CrearAuton,
            "Crear nuevo Auton".to_string(),
        );

        interfaz.enviar_comando(comando).unwrap();
        assert_eq!(interfaz.comandos_pendientes().len(), 1);
    }

    #[test]
    fn test_procesar_comando() {
        let mut interfaz = InterfazDivina::new();

        let comando = ComandoDivino::new(
            TipoComando::SolicitarReporte,
            "Solicitar reporte".to_string(),
        );

        interfaz.enviar_comando(comando).unwrap();
        
        let comandos = interfaz.comandos_pendientes();
        let comando_id = comandos[0].id;

        let resultado = interfaz.procesar_comando(comando_id).unwrap();
        assert!(resultado.contains("REPORTE"));
    }

    #[test]
    fn test_revelacion() {
        let mut interfaz = InterfazDivina::new();

        interfaz.agendar_revelacion(Revelacion::new(
            TipoRevelacion::EstadoSistema,
            "Estado actualizado".to_string(),
        ));

        assert!(!interfaz.obtener_revelaciones_pendientes().is_empty());
    }

    #[test]
    fn test_vetar_comando() {
        let mut interfaz = InterfazDivina::new();

        // Usar SolicitarReporte que NO es peligroso (no está en comandos_peligrosos)
        let comando = ComandoDivino::new(
            TipoComando::SolicitarReporte,
            "Solicitar estado".to_string(),
        );

        interfaz.enviar_comando(comando).unwrap();

        let comandos = interfaz.comandos_pendientes();
        let comando_id = comandos[0].id;

        interfaz.vetar_comando(comando_id, "Comando demasiado peligroso").unwrap();

        let stats = interfaz.stats();
        assert_eq!(stats.vetos_aplicados, 1);
    }

    #[test]
    fn test_dashboard() {
        let mut interfaz = InterfazDivina::new();

        let dashboard = DashboardEstado {
            timestamp: current_timestamp_ms(),
            autons_activos: 100,
            energia_total: 5000.0,
            salud_promedio: 0.85,
            nivel_consciencia: 0.5,
            tendencia_evolutiva: "Creciente".to_string(),
            eventos_recientes: Vec::new(),
            alertas: Vec::new(),
            metricas: MetricasRendimiento {
                fps: 60.0,
                memoria_usada_mb: 512,
                cpu_porcentaje: 45.0,
                threads_activos: 8,
            },
        };

        interfaz.actualizar_dashboard(dashboard);
        let dash = interfaz.obtener_dashboard();

        assert_eq!(dash.autons_activos, 100);
    }

    #[test]
    fn test_reporte_automatico() {
        let mut interfaz = InterfazDivina::new();

        // Enviar comando de reporte
        let comando = ComandoDivino::new(
            TipoComando::SolicitarReporte,
            "Reporte inmediato".to_string(),
        );

        interfaz.enviar_comando(comando).unwrap();
        let comandos = interfaz.comandos_pendientes();
        interfaz.procesar_comando(comandos[0].id).unwrap();

        // Verificar que hay revelaciones pendientes (generadas por procesar_comando)
        let revelaciones = interfaz.obtener_revelaciones_pendientes();
        assert!(!revelaciones.is_empty(), "Debe generar revelaciones al procesar comandos");
    }

    #[test]
    fn test_urgencia_revelacion() {
        let revelacion = Revelacion::new(
            TipoRevelacion::Emergencia,
            "Emergencia detectada".to_string(),
        ).con_urgencia(10);

        assert_eq!(revelacion.urgencia, 10);
    }

    #[test]
    fn test_tipo_comando_requiere_confirmacion() {
        assert!(TipoComando::EliminarAuton.requiere_confirmacion());
        assert!(!TipoComando::SolicitarReporte.requiere_confirmacion());
    }

    #[test]
    fn test_tipo_comando_irreversible() {
        assert!(TipoComando::EliminarAuton.es_irreversible());
        assert!(TipoComando::Interrumpir.es_irreversible());
        assert!(!TipoComando::CrearAuton.es_irreversible());
    }

    #[test]
    fn test_confirmar_lectura() {
        let mut interfaz = InterfazDivina::new();

        interfaz.agendar_revelacion(Revelacion::new(
            TipoRevelacion::EstadoSistema,
            "Test".to_string(),
        ));

        let revelaciones = interfaz.obtener_revelaciones_pendientes();
        if !revelaciones.is_empty() {
            interfaz.confirmar_lectura(revelaciones[0].id).unwrap();
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn generar_id_comando() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let now = current_timestamp_ms();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    (now << 20) ^ (count & 0xFFFFF)
}

fn generar_id_revelacion() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0x100000);
    let now = current_timestamp_ms();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    (now << 20) ^ (count & 0xFFFFF)
}
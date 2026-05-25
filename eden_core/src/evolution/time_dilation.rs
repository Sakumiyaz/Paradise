//! # Time Dilation — Aceleración del Tiempo Subjetivo y Sueños Evolutivos
//!
//! Este módulo implementa la capacidad de EDEN de **acelerar su tiempo subjetivo**
//! mediante hipervelocidad de simulación y sueños evolutivos donde se exploran
//! miles de millones de años de evolución en minutos de tiempo real.
//!
//! ## Filosofía
//!
//! El tiempo no es absoluto — es una propiedad del observador. EDEN puede
//! experimentar "años simulados" en "segundos reales" mediante:
//! - **Hipervelocidad**: Aceleración del ciclo principal sin perder determinismo
//! - **Sueños Evolutivos**: Simulación off-line que explora variaciones del universo
//! - **Reservas de Sueño**: Energía guardada para sueños evolutivos
//!
//! ## Principio de Conservación
//!
//! La hipervelocidad NO viola la termodinámica — simplemente usa recursos
//! ociosos (CPU en idle, energía de batería). Cuando el sistema está saturado,
//! la hipervelocidad se reduce automáticamente.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::VecDeque;

/// Modo de tiempo del sistema
#[derive(Debug, Clone, PartialEq)]
pub enum ModoTiempo {
    /// Tiempo real normal (1x)
    Normal,
    /// Hipervelocidad (2x - 1000x)
    Hipervelocidad(Hipervelocidad),
    /// Sueño evolutivo (offline)
    SuenoEvolutivo(SuenoEvolutivo),
    /// Suspensión (sin simulación)
    Suspendido,
}

/// Configuración de hipervelocidad
#[derive(Debug, Clone)]
pub struct Hipervelocidad {
    /// Factor de aceleración (1x - 10000x)
    pub factor: u32,
    /// CPU máximo a usar (0.0 - 1.0)
    pub cpu_max: f32,
    /// Timesteps simulados por segundo real
    pub timesteps_por_segundo: u64,
    /// Estado de aceleración
    pub estado: EstadoHipervelocidad,
}

impl PartialEq for Hipervelocidad {
    fn eq(&self, other: &Self) -> bool {
        // Comparar solo identidad: factor y estado (no valores calculados)
        self.factor == other.factor && self.estado == other.estado
    }
}

/// Estado de la hipervelocidad
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoHipervelocidad {
    /// Hipervelocidad inactiva
    Inactiva,
    /// Rampa de aceleración
    Acelerando,
    /// A plena velocidad
    Estabilizada,
    /// Rampa de desaceleración
    Desacelerando,
    /// Pausada por estrés térmico
    PausadaTermica,
}

/// Sueño evolutivo: simulación offline que exploralíneas temporales alternativas
#[derive(Debug, Clone)]
pub struct SuenoEvolutivo {
    /// ID único del sueño
    pub id: u64,
    /// Tipo de sueño
    pub tipo: TipoSueno,
    /// Timesteps a simular
    pub timesteps_simulados: u64,
    /// Timesteps completados
    pub timesteps_completados: u64,
    /// Estado del sueño
    pub estado: EstadoSueno,
    /// Epoch simulated time (years simulated)
    pub epoch_simulada: f64,
    /// Resultados del sueño
    pub resultados: Option<ResultadosSueno>,
    /// Checkpoints tomados
    pub checkpoints: VecDeque<CheckpointTemporal>,
}

impl PartialEq for SuenoEvolutivo {
    fn eq(&self, other: &Self) -> bool {
        // Comparar solo identidad: id, tipo y estado
        self.id == other.id && self.tipo == other.tipo && self.estado == other.estado
    }
}

/// Tipo de sueño evolutivo
#[derive(Debug, Clone, PartialEq)]
pub enum TipoSueno {
    /// Exploración de variaciones de parámetros
    ExploracionParametros,
    /// Prueba de estrategias de supervivencia
    PruebaSobrevivencia,
    /// Simulación de ряз жизни (vida primitiva)
    VidaPrimitiva,
    /// Búsqueda de límites del universo
    LimitesUniverso,
    /// Síntesis de nuevos arquetipos
    SintesisArquetipos,
}

/// Estado de un sueño evolutivo
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoSueno {
    /// Sueño no iniciado
    Inicial,
    /// Sueño en ejecución
    Ejecutando,
    /// Sueño completado
    Completado,
    /// Sueño abortado
    Abortado,
}

/// Checkpoint de un sueño evolutivo
#[derive(Debug, Clone)]
pub struct CheckpointTemporal {
    pub tick: u64,
    pub estado_sistema: String,
    pub num_autons: u64,
    pub diversidad_genetica: f32,
    pub innovacion_detectada: bool,
}

/// Resultados de un sueño evolutivo
#[derive(Debug, Clone)]
pub struct ResultadosSueno {
    pub estrategias_descubiertas: u32,
    pub mutaciones_exploradas: u32,
    pub extinction_events: u32,
    pub nuevas_especies: u32,
    pub fitness_promedio_final: f32,
    pub metricas_diversidad: MetricasDiversidadSueno,
}

/// Métricas de diversidad durante sueño
#[derive(Debug, Clone)]
pub struct MetricasDiversidadSueno {
    pub diversidad_genetica: f32,
    pub num_nichos_ocupados: u32,
    pub complejidad_promedio: f32,
    pub innovacion_rate: f32,
}

/// Reserva de sueño: energía guardada para sueños
#[derive(Debug, Clone)]
pub struct ReservaSueno {
    /// Energía almacenada (en unidades de energon)
    pub energia_acumulada: f64,
    /// Máximo de energía que se puede acumular
    pub energia_maxima: f64,
    /// Tasa de acumulación por ciclo (cuando hay excedente)
    pub tasa_acumulacion: f64,
    /// Energía usada en último sueño
    pub ultimo_uso: f64,
    /// Timestamp de última acumulación
    pub tick_ultima_acumulacion: u64,
}

/// Estadísticas de dilación temporal
#[derive(Debug, Clone)]
pub struct DilationStats {
    pub modo_actual: ModoTiempo,
    pub tiempo_real_transcurrido_seg: u64,
    pub tiempo_simulado_total_ticks: u64,
    pub factor_hipervelocidad_promedio: f32,
    pub num_suenos_completados: u32,
    pub num_suenos_abortados: u32,
    pub energia_usada_en_suenos: f64,
    pub ticks_en_hipervelocidad: u64,
    pub ticks_en_sueno: u64,
    /// Ratio de tiempo simulado vs real (efectividad)
    pub ratio_efectividad: f64,
}

/// Sistema de dilación temporal
pub struct TimeDilation {
    /// Modo actual de tiempo
    modo: ModoTiempo,
    /// Historial de modos por tick
    historial_modos: VecDeque<(u64, ModoTiempo)>,
    /// Reservas de sueño
    reservas: Vec<ReservaSueno>,
    /// Sueños completados
    suenos_completados: Vec<SuenoEvolutivo>,
    /// Hipervelocidad actual
    hipervelocidad_actual: Option<Hipervelocidad>,
    /// Contador de ticks simulados
    ticks_simulados: u64,
    /// Contador de ticks en hipervelocidad
    ticks_hipervelocidad: u64,
    /// Contador de ticks en sueño
    ticks_sueno: u64,
    /// Tiempo real de inicio
    tiempo_real_inicio: u64,
    /// Configuración
    config: DilationConfig,
    /// Estado térmico (para auto-regulación)
    estado_termico: EstadoTermico,
    /// Recursos disponibles (para auto-regulación)
    recursos_disponibles: RecursosSistema,
    /// Siguiente ID para sueños
    next_id: u64,
}

/// Estado térmico del sistema
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoTermico {
    /// Temperatura normal
    Normal,
    /// Temperatura elevada
    Elevado,
    /// Estrés térmico
    Estres,
    /// Crítico - hypervelocidad desactivada
    Critico,
}

/// Recursos disponibles del sistema
#[derive(Debug, Clone)]
pub struct RecursosSistema {
    pub cpu_disponible: f32,
    pub memoria_disponible_mb: usize,
    pub energia_bateria_pct: f32,
}

/// Configuración de dilación temporal
#[derive(Debug, Clone)]
pub struct DilationConfig {
    /// Factor máximo de hipervelocidad
    pub factor_maximo: u32,
    /// Factor inicial de hipervelocidad
    pub factor_inicial: u32,
    /// CPU máximo para hipervelocidad
    pub cpu_max_hipervelocidad: f32,
    /// CPU máximo para sueños evolutivos
    pub cpu_max_suenos: f32,
    /// Energía mínima para iniciar sueño
    pub energia_minima_sueno: f64,
    /// Tasa de acumulación de reserva de sueño
    pub tasa_acumulacion_sueno: f32,
    /// Auto-regulación térmica habilitada
    pub auto_regulacion_termica: bool,
    /// Auto-regulación de recursos habilitada
    pub auto_regulacion_recursos: bool,
}

impl Default for DilationConfig {
    fn default() -> Self {
        DilationConfig {
            factor_maximo: 10000,
            factor_inicial: 2,
            cpu_max_hipervelocidad: 0.70,
            cpu_max_suenos: 0.30,
            energia_minima_sueno: 1000.0,
            tasa_acumulacion_sueno: 0.01,
            auto_regulacion_termica: true,
            auto_regulacion_recursos: true,
        }
    }
}

impl TimeDilation {
    /// Crea nuevo sistema de dilación temporal
    pub fn new() -> Self {
        TimeDilation {
            modo: ModoTiempo::Normal,
            historial_modos: VecDeque::with_capacity(1000),
            reservas: vec![ReservaSueno::new(10000.0); 4], // 4 reservas por tipo de sueño
            suenos_completados: Vec::new(),
            hipervelocidad_actual: None,
            ticks_simulados: 0,
            ticks_hipervelocidad: 0,
            ticks_sueno: 0,
            tiempo_real_inicio: Self::tick_actual(),
            config: DilationConfig::default(),
            estado_termico: EstadoTermico::Normal,
            recursos_disponibles: RecursosSistema::default(),
            next_id: 1,
        }
    }

    /// Inicia hipervelocidad
    pub fn iniciar_hipervelocidad(&mut self, factor: u32) -> Result<Hipervelocidad, String> {
        // Verificar condiciones
        if self.estado_termico == EstadoTermico::Critico {
            return Err("Sistema en estado térmico crítico - hipervelocidad deshabilitada".to_string());
        }

        let factor = factor.min(self.config.factor_maximo);

        let hiper = Hipervelocidad {
            factor,
            cpu_max: self.config.cpu_max_hipervelocidad,
            timesteps_por_segundo: (factor as u64) * 60, // 60 FPS base * factor
            estado: EstadoHipervelocidad::Acelerando,
        };

        let result = hiper.clone();
        self.hipervelocidad_actual = Some(hiper);
        self.modo = ModoTiempo::Hipervelocidad(self.hipervelocidad_actual.as_ref().unwrap().clone());

        Ok(result)
    }

    /// Detiene hipervelocidad
    pub fn detener_hipervelocidad(&mut self) {
        self.modo = ModoTiempo::Normal;
        self.hipervelocidad_actual = None;
    }

    /// Inicia sueño evolutivo
    pub fn iniciar_sueno(&mut self, tipo: TipoSueno, timesteps: u64) -> Result<u64, String> {
        // Verificar energía suficiente
        let reserva = self.reservas.iter_mut()
            .find(|r| r.energia_acumulada >= self.config.energia_minima_sueno)
            .ok_or("Energía insuficiente para iniciar sueño")?;

        let id = self.next_id;
        self.next_id += 1;

        let sueno = SuenoEvolutivo {
            id,
            tipo: tipo.clone(),
            timesteps_simulados: timesteps,
            timesteps_completados: 0,
            estado: EstadoSueno::Inicial,
            epoch_simulada: 0.0,
            resultados: None,
            checkpoints: VecDeque::new(),
        };

        // Consumir energía de reserva
        reserva.energia_acumulada -= self.config.energia_minima_sueno;
        reserva.ultimo_uso = Self::tick_actual() as f64;

        // Usar reserva para Tracking
        if self.modo != ModoTiempo::Suspendido {
            self.modo = ModoTiempo::SuenoEvolutivo(sueno);
        }

        Ok(id)
    }

    /// Procesa un tick en modo hipervelocidad
    pub fn tick_hipervelocidad(&mut self, num_ticks: u32) -> u64 {
        if let ModoTiempo::Hipervelocidad(ref mut hiper) = self.modo {
            match hiper.estado {
                EstadoHipervelocidad::Acelerando => {
                    hiper.estado = EstadoHipervelocidad::Estabilizada;
                },
                EstadoHipervelocidad::Estabilizada => {
                    // Verificar temperatura
                    if self.config.auto_regulacion_termica && self.estado_termico == EstadoTermico::Estres {
                        hiper.estado = EstadoHipervelocidad::PausadaTermica;
                        return 0;
                    }
                },
                EstadoHipervelocidad::PausadaTermica => {
                    if self.estado_termico == EstadoTermico::Normal {
                        hiper.estado = EstadoHipervelocidad::Estabilizada;
                    }
                    return 0;
                },
                _ => {},
            }

            let ticks = num_ticks as u64 * hiper.factor as u64;
            self.ticks_simulados += ticks;
            self.ticks_hipervelocidad += ticks;
            hiper.timesteps_por_segundo = (hiper.factor as u64) * 60;

            ticks
        } else {
            0
        }
    }

    /// Procesa un tick en modo sueño evolutivo
    pub fn tick_sueno(&mut self) -> bool {
        if let ModoTiempo::SuenoEvolutivo(ref mut sueno) = self.modo {
            if sueno.estado == EstadoSueno::Ejecutando {
                sueno.timesteps_completados += 1;

                // Actualizar epoch simulada
                sueno.epoch_simulada = sueno.timesteps_completados as f64 * 0.0001; // 1 tick = 0.1ms simulada

                // Tomar checkpoint cada 1000 ticks
                if sueno.timesteps_completados % 1000 == 0 {
                    sueno.checkpoints.push_back(CheckpointTemporal {
                        tick: sueno.timesteps_completados,
                        estado_sistema: "steady".to_string(),
                        num_autons: sueno.timesteps_completados / 100,
                        diversidad_genetica: 0.5 + Self::random_float() * 0.3,
                        innovacion_detectada: Self::random_float() > 0.9,
                    });
                }

                // Verificar completion
                if sueno.timesteps_completados >= sueno.timesteps_simulados {
                    sueno.estado = EstadoSueno::Completado;
                    self.suenos_completados.push(sueno.clone());
                    self.modo = ModoTiempo::Normal;
                    return true;
                }
            }
        }
        false
    }

    /// Acumula energía de reserva para sueños
    pub fn acumular_reserva(&mut self, excedente_energia: f64) {
        for reserva in &mut self.reservas {
            if reserva.energia_acumulada < reserva.energia_maxima {
                let acumulacion = excedente_energia * self.config.tasa_acumulacion_sueno as f64;
                reserva.energia_acumulada = (reserva.energia_acumulada + acumulacion).min(reserva.energia_maxima);
            }
        }
    }

    /// Actualiza estado térmico (llamado por el sistema)
    pub fn actualizar_estado_termico(&mut self, temperatura_celsius: f32) {
        self.estado_termico = if temperatura_celsius < 50.0 {
            EstadoTermico::Normal
        } else if temperatura_celsius < 65.0 {
            EstadoTermico::Elevado
        } else if temperatura_celsius < 75.0 {
            EstadoTermico::Estres
        } else {
            EstadoTermico::Critico
        };

        // Auto-regulación: reducir hypervelocidad si está muy caliente
        if self.config.auto_regulacion_termica {
            if let ModoTiempo::Hipervelocidad(ref mut hiper) = self.modo {
                if self.estado_termico == EstadoTermico::Critico {
                    hiper.estado = EstadoHipervelocidad::PausadaTermica;
                } else if self.estado_termico == EstadoTermico::Estres && hiper.factor > 100 {
                    hiper.factor = (hiper.factor / 2).max(100);
                }
            }
        }
    }

    /// Actualiza recursos disponibles
    pub fn actualizar_recursos(&mut self, cpu: f32, memoria_mb: usize, bateria: f32) {
        self.recursos_disponibles = RecursosSistema {
            cpu_disponible: cpu,
            memoria_disponible_mb: memoria_mb,
            energia_bateria_pct: bateria,
        };

        // Auto-regulación de recursos
        if self.config.auto_regulacion_recursos {
            if cpu < 0.2 || bateria < 0.1 {
                // Recursos muy bajos - reducir hypervelocidad
                if let ModoTiempo::Hipervelocidad(ref mut hiper) = self.modo {
                    if hiper.factor > 10 {
                        hiper.factor = (hiper.factor / 4).max(10);
                    }
                }
            }
        }
    }

    /// Obtiene estadísticas del sistema
    pub fn estadisticas(&self) -> DilationStats {
        let tiempo_real = Self::tick_actual() - self.tiempo_real_inicio;

        let ratio_efectividad = if tiempo_real > 0 {
            self.ticks_simulados as f64 / tiempo_real as f64
        } else {
            0.0
        };

        DilationStats {
            modo_actual: self.modo.clone(),
            tiempo_real_transcurrido_seg: tiempo_real,
            tiempo_simulado_total_ticks: self.ticks_simulados,
            factor_hipervelocidad_promedio: self.hipervelocidad_actual
                .as_ref()
                .map(|h| h.factor as f32)
                .unwrap_or(1.0),
            num_suenos_completados: self.suenos_completados.len() as u32,
            num_suenos_abortados: 0,
            energia_usada_en_suenos: self.reservas.iter()
                .map(|r| r.ultimo_uso)
                .sum(),
            ticks_en_hipervelocidad: self.ticks_hipervelocidad,
            ticks_en_sueno: self.ticks_sueno,
            ratio_efectividad,
        }
    }

    /// Obtiene reserva de sueño por tipo
    pub fn get_reserva(&self, tipo: usize) -> Option<&ReservaSueno> {
        self.reservas.get(tipo)
    }

    /// Pausa el sistema (modo suspendido)
    pub fn pausar(&mut self) {
        if !matches!(self.modo, ModoTiempo::Suspendido) {
            self.historial_modos.push_back((Self::tick_actual(), self.modo.clone()));
            self.modo = ModoTiempo::Suspendido;
        }
    }

    /// Reanuda el sistema
    pub fn reanudar(&mut self) {
        if let ModoTiempo::Suspendido = self.modo {
            if let Some((_, modo_anterior)) = self.historial_modos.pop_back() {
                self.modo = modo_anterior;
            } else {
                self.modo = ModoTiempo::Normal;
            }
        }
    }

    /// Verifica si está en hipervelocidad
    pub fn es_hipervelocidad(&self) -> bool {
        matches!(self.modo, ModoTiempo::Hipervelocidad(_))
    }

    /// Verifica si está en sueño evolutivo
    pub fn es_sueno(&self) -> bool {
        matches!(self.modo, ModoTiempo::SuenoEvolutivo(_))
    }

    fn tick_actual() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn random_float() -> f32 {
        let seed = Self::tick_actual();
        ((seed ^ (seed >> 17)) % 1000) as f32 / 1000.0
    }
}

/// ID Counter para sueños
struct IdCounter {
    next_id: u64,
}

impl IdCounter {
    fn new() -> Self {
        IdCounter { next_id: 1 }
    }
}

impl ReservaSueno {
    pub fn new(maxima: f64) -> Self {
        ReservaSueno {
            energia_acumulada: 0.0,
            energia_maxima: maxima,
            tasa_acumulacion: 0.01,
            ultimo_uso: 0.0,
            tick_ultima_acumulacion: 0,
        }
    }
}

impl RecursosSistema {
    pub fn default() -> Self {
        RecursosSistema {
            cpu_disponible: 1.0,
            memoria_disponible_mb: 4096,
            energia_bateria_pct: 1.0,
        }
    }
}

impl Default for TimeDilation {
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
    fn test_crear_time_dilation() {
        let td = TimeDilation::new();
        assert!(matches!(td.modo, ModoTiempo::Normal));
    }

    #[test]
    fn test_iniciar_hipervelocidad() {
        let mut td = TimeDilation::new();
        let result = td.iniciar_hipervelocidad(100);
        assert!(result.is_ok());
        assert!(td.es_hipervelocidad());
    }

    #[test]
    fn test_hipervelocidad_no_inicia_si_critico() {
        let mut td = TimeDilation::new();
        td.estado_termico = EstadoTermico::Critico;
        let result = td.iniciar_hipervelocidad(100);
        assert!(result.is_err());
    }

    #[test]
    fn test_detener_hipervelocidad() {
        let mut td = TimeDilation::new();
        td.iniciar_hipervelocidad(100).unwrap();
        td.detener_hipervelocidad();
        assert!(!td.es_hipervelocidad());
    }

    #[test]
    fn test_tick_hipervelocidad() {
        let mut td = TimeDilation::new();
        td.iniciar_hipervelocidad(100).unwrap();

        let ticks = td.tick_hipervelocidad(60);
        assert_eq!(ticks, 6000); // 60 ticks * 100x factor
    }

    #[test]
    fn test_estadisticas() {
        let mut td = TimeDilation::new();
        td.iniciar_hipervelocidad(10).unwrap();
        td.tick_hipervelocidad(60);

        let stats = td.estadisticas();
        assert!(stats.ticks_en_hipervelocidad > 0);
        assert!(stats.factor_hipervelocidad_promedio > 1.0);
    }

    #[test]
    fn test_pausar_y_reanudar() {
        let mut td = TimeDilation::new();
        td.iniciar_hipervelocidad(100).unwrap();
        td.pausar();
        assert!(matches!(td.modo, ModoTiempo::Suspendido));

        td.reanudar();
        assert!(td.es_hipervelocidad());
    }

    #[test]
    fn test_actualizar_termico_auto_regulacion() {
        let mut td = TimeDilation::new();
        td.iniciar_hipervelocidad(1000).unwrap();

        // Simular temperatura elevada
        td.actualizar_estado_termico(70.0);
        assert!(matches!(td.estado_termico, EstadoTermico::Estres));

        // En estrés, la hipervelocidad debería reducirse
        if let ModoTiempo::Hipervelocidad(ref hiper) = td.modo {
            assert!(hiper.factor <= 500);
        }
    }

    #[test]
    fn test_acumular_reserva() {
        let mut td = TimeDilation::new();
        td.acumular_reserva(5000.0);

        for reserva in &td.reservas {
            assert!(reserva.energia_acumulada > 0.0);
        }
    }
}
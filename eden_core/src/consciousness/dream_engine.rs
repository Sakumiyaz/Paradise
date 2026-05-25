//! # Dream Engine: Motor de Sueños de EDEN
//!
//! Sistema de simulación onírica para planificación a largo plazo.
//!
//! ## Concepto
//!
//! Cuando EDEN detecta baja actividad o idle, puede entrar en **Modo Onírico**:
//! - Pausa la simulación principal
//! - Bifurca un universo hijo efímero con las mismas reglas
//! - Ejecuta tiempo acelerado (sin renderizado, máxima CPU)
//! - Prueba escenarios hipotéticos
//! - Evalúa resultados y decide si aplicar al despertar
//!
//! ## Filosofía
//!
//! Esto convierte a EDEN en un agente que **planifica a largo plazo** usando
//! modelos internos, un rasgo clave de la cognición superior.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// Re-export types we need from universe
use crate::universe::MetricasUniverso;

// ============================================================================
// ESCENARIOS HIPOTÉTICOS
// ============================================================================

/// Un escenario hipotético a probar en el universo onírico
#[derive(Debug, Clone)]
pub struct EscenarioHipotetico {
    /// Identificador único
    pub id: u64,
    /// Descripción textual
    pub descripcion: String,
    /// Tipo de intervención
    pub tipo: TipoIntervencion,
    /// Parámetros específicos
    pub parametros: HashMap<String, f64>,
    /// Résultat del sueño
    pub resultado: Option<ResultadoSueno>,
}

impl EscenarioHipotetico {
    /// Crea nuevo escenario
    pub fn new(
        descripcion: &str,
        tipo: TipoIntervencion,
        parametros: HashMap<String, f64>,
    ) -> Self {
        static mut NEXT_ID: u64 = 0;
        let id = unsafe {
            NEXT_ID += 1;
            NEXT_ID
        };

        EscenarioHipotetico {
            id,
            descripcion: descripcion.to_string(),
            tipo,
            parametros,
            resultado: None,
        }
    }

    /// Crea escenario de aumento de tasa de mutación
    pub fn tasa_mutacion_aumento(factor: f64) -> Self {
        let mut params = HashMap::new();
        params.insert("factor".to_string(), factor);

        EscenarioHipotetico::new(
            &format!("Aumentar tasa de mutación por factor {:.1}", factor),
            TipoIntervencion::AjustarParametro("tasa_mutacion".to_string()),
            params,
        )
    }

    /// Crea escenario de inyección de escoria
    pub fn inyeccion_escoria(cantidad: f64, sector: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("cantidad".to_string(), cantidad);
        params.insert(
            "sector".to_string(),
            sector.to_string().parse().unwrap_or(0.0),
        );

        EscenarioHipotetico::new(
            &format!("Inyectar {:.1} escoria en sector {}", cantidad, sector),
            TipoIntervencion::InyectarEscoria,
            params,
        )
    }

    /// Crea escenario de inyección de energía
    pub fn inyeccion_energia(cantidad: f64, sector: &str) -> Self {
        let mut params = HashMap::new();
        params.insert("cantidad".to_string(), cantidad);
        params.insert(
            "sector".to_string(),
            sector.to_string().parse().unwrap_or(0.0),
        );

        EscenarioHipotetico::new(
            &format!("Inyectar {:.1} energía en sector {}", cantidad, sector),
            TipoIntervencion::InyectarEnergia,
            params,
        )
    }

    /// Crea escenario de aumento de constante de difusión
    pub fn aumentar_difusion(factor: f64) -> Self {
        let mut params = HashMap::new();
        params.insert("factor".to_string(), factor);

        EscenarioHipotetico::new(
            &format!("Aumentar constante de difusión por factor {:.1}", factor),
            TipoIntervencion::AjustarParametro("coef_difusion".to_string()),
            params,
        )
    }
}

/// Tipo de intervención onírica
#[derive(Debug, Clone)]
pub enum TipoIntervencion {
    /// Ajustar un parámetro específico
    AjustarParametro(String),
    /// Inyectar escoria en una región
    InyectarEscoria,
    /// Inyectar energía en una región
    InyectarEnergia,
    /// Cambiar constante cosmológica
    AjustarConstante(String),
    /// Modificar tasa de reproducción
    ModificarTasaReproduccion(f64),
    /// Ninguna intervención (control)
    Control,
}

impl TipoIntervencion {
    /// Describe la intervención
    pub fn descripcion(&self) -> String {
        match self {
            TipoIntervencion::AjustarParametro(p) => format!("Ajuste de {}", p),
            TipoIntervencion::InyectarEscoria => "Inyección de escoria".to_string(),
            TipoIntervencion::InyectarEnergia => "Inyección de energía".to_string(),
            TipoIntervencion::AjustarConstante(c) => format!("Constante {}", c),
            TipoIntervencion::ModificarTasaReproduccion(r) => format!("Tasa reproducción {}", r),
            TipoIntervencion::Control => "Control (sin intervención)".to_string(),
        }
    }
}

// ============================================================================
// RESULTADO DEL SUEÑO
// ============================================================================

/// Resultado de una simulación onírica
#[derive(Debug, Clone)]
pub struct ResultadoSueno {
    /// Escenario evaluado
    pub escenario_id: u64,
    /// Ciclo en que terminó
    pub ciclo_fin: u64,
    /// Métricas finales del universo onírico
    pub metricas_finales: MetricasSueno,
    /// Cambio en diversidad (vs control)
    pub cambio_diversidad: f64,
    /// Cambio en estabilidad
    pub cambio_estabilidad: f64,
    /// Cambio en población
    pub cambio_poblacion: f64,
    /// Score de bondad (0-1)
    pub score_bondad: f64,
    /// vale la pena aplicar?
    pub recomendar_aplicar: bool,
}

impl ResultadoSueno {
    /// Evalúa el resultado del sueño
    pub fn evaluar(
        escenario: &EscenarioHipotetico,
        metricas_inicio: &MetricasSueno,
        metricas_fin: &MetricasSueno,
        ciclo_fin: u64,
    ) -> Self {
        // Calcular cambios
        let cambio_diversidad = metricas_fin.diversidad - metricas_inicio.diversidad;
        let cambio_estabilidad = metricas_fin.estabilidad - metricas_inicio.estabilidad;
        let cambio_poblacion = if metricas_inicio.auton > 0 {
            (metricas_fin.auton as f64 - metricas_inicio.auton as f64)
                / metricas_inicio.auton as f64
        } else {
            0.0
        };

        // Score de bondad: combinación de métricas
        let score_bondad = (cambio_diversidad.max(0.0) * 0.3
            + cambio_estabilidad.max(0.0) * 0.3
            + cambio_poblacion.max(0.0) * 0.4)
            .min(1.0);

        // Recomendar si el score es bueno y los cambios son positivos
        let recomendar_aplicar =
            score_bondad > 0.6 && cambio_diversidad >= -0.1 && cambio_estabilidad >= -0.1;

        ResultadoSueno {
            escenario_id: escenario.id,
            ciclo_fin,
            metricas_finales: metricas_fin.clone(),
            cambio_diversidad,
            cambio_estabilidad,
            cambio_poblacion,
            score_bondad,
            recomendar_aplicar,
        }
    }
}

/// Métricas simplificadas para el sueño
#[derive(Debug, Clone)]
pub struct MetricasSueno {
    /// Ciclo actual
    pub ciclo: u64,
    /// Número de Auton
    pub auton: u32,
    /// Diversidad (0-1)
    pub diversidad: f64,
    /// Estabilidad (0-1, mayor = más estable)
    pub estabilidad: f64,
    /// Energía total
    pub energia: i64,
    /// Escoria total
    pub escoria: f64,
}

impl MetricasSueno {
    /// Crea métricas de control (baseline)
    pub fn baseline(ciclo: u64) -> Self {
        MetricasSueno {
            ciclo,
            auton: 100,
            diversidad: 0.5,
            estabilidad: 0.7,
            energia: 1_000_000_000_000,
            escoria: 0.3,
        }
    }

    /// Crea desde MetricasUniverso (conversión simplificada)
    pub fn desde_metricas_universo(metricas: &MetricasUniverso) -> Self {
        MetricasSueno {
            ciclo: metricas.ciclos,
            auton: metricas.num_auton,
            diversidad: metricas.diversidad,
            estabilidad: 0.5, // Placeholder - real implementation would calculate
            energia: metricas.energia_total,
            escoria: 0.3,
        }
    }
}

// ============================================================================
// UNIVERSO ONÍRICO (SIMULADO)
// ============================================================================

/// Estado simulado del universo onírico
#[derive(Debug, Clone)]
pub struct UniversoOnirico {
    /// Ciclo actual
    pub ciclo: u64,
    /// Métricas actuales
    pub metricas: MetricasSueno,
    /// Parámetros activos
    parametros: HashMap<String, f64>,
    /// Intervención aplicada
    intervencion_aplicada: Option<TipoIntervencion>,
}

impl UniversoOnirico {
    /// Crea nuevo universo onírico desde baseline
    pub fn nuevo_baseline(ciclo_inicial: u64) -> Self {
        UniversoOnirico {
            ciclo: ciclo_inicial,
            metricas: MetricasSueno::baseline(ciclo_inicial),
            parametros: Self::parametros_default(),
            intervencion_aplicada: None,
        }
    }

    /// Parámetros por defecto
    fn parametros_default() -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("tasa_mutacion".to_string(), 0.001);
        params.insert("coef_difusion".to_string(), 10.0);
        params.insert("tasa_reproduccion".to_string(), 0.01);
        params
    }

    /// Aplica una intervención
    pub fn aplicar_intervencion(&mut self, intervencion: &TipoIntervencion) {
        match intervencion {
            TipoIntervencion::AjustarParametro(nombre) => {
                if let Some(valor) = self.parametros.get(nombre) {
                    let nuevo_valor = valor * 1.5; // Aumento 50%
                    self.parametros.insert(nombre.clone(), nuevo_valor);
                }
            }
            TipoIntervencion::ModificarTasaReproduccion(factor) => {
                if let Some(valor) = self.parametros.get("tasa_reproduccion") {
                    self.parametros
                        .insert("tasa_reproduccion".to_string(), valor * factor);
                }
            }
            _ => {}
        }
        self.intervencion_aplicada = Some(intervencion.clone());
    }

    /// Simula N ciclos (acelerado, sin renderizado)
    pub fn simular_ciclos(&mut self, n: u64) {
        let ciclos_fin = self.ciclo + n;
        let parametros = &self.parametros;

        while self.ciclo < ciclos_fin {
            // Simulación simplificada
            // En una implementación real, esto usaría el motor físico completo

            // Efecto de la intervención sobre métricas
            let factor_intervencion = if self.intervencion_aplicada.is_some() {
                1.1 // Simula efecto positivo
            } else {
                1.0
            };

            // Actualizar métricas según parámetros
            let tasa_mut = *parametros.get("tasa_mutacion").unwrap_or(&0.001);
            let _coef_diff = *parametros.get("coef_difusion").unwrap_or(&10.0);
            let tasa_rep = *parametros.get("tasa_reproduccion").unwrap_or(&0.01);

            // Simular cambios
            self.metricas.auton =
                (self.metricas.auton as f64 * (1.0 + tasa_rep * factor_intervencion)) as u32;
            self.metricas.auton = self
                .metricas
                .auton
                .saturating_sub((self.metricas.auton as f64 * 0.001) as u32);

            // Diversidad aumenta ligeramente con mutación
            self.metricas.diversidad =
                (self.metricas.diversidad + tasa_mut * 0.1 * factor_intervencion).min(1.0);

            // Estabilidad depende de población
            self.metricas.estabilidad = if self.metricas.auton > 50 { 0.8 } else { 0.4 };

            // Energía y escoria
            self.metricas.energia = (self.metricas.energia as f64 * 0.9999) as i64;
            self.metricas.escoria = (self.metricas.escoria + 0.001).min(1.0);

            self.ciclo += 1;
        }
    }

    /// Obtiene métricas actuales
    pub fn metricas(&self) -> &MetricasSueno {
        &self.metricas
    }
}

// ============================================================================
// MOTOR DE SUEÑOS
// ============================================================================

/// Estado del motor de sueños
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoSueno {
    /// Despertado (no soñando)
    Despertado,
    /// Entrando en sueño
    Entrando,
    /// Soñando activamente
    Soñando,
    /// Despertando (evaluando resultados)
    Despertando,
}

/// El motor de sueños de EDEN
pub struct DreamEngine {
    /// Estado actual
    estado: EstadoSueno,
    /// Universo onírico activo
    universo_onirico: Option<UniversoOnirico>,
    /// Escenario actualmente en prueba
    escenario_actual: Option<EscenarioHipotetico>,
    /// Resultados de sueños anteriores
    historial_resultados: Vec<ResultadoSueno>,
    /// Ciclos oníricos por sueño
    ciclos_por_sueno: u64,
    /// Umbral para recomendar intervención
    umbral_score: f64,
    /// Momento de inicio del último sueño
    inicio_sueno: Option<u64>,
    /// Ciclo base del universo real
    ciclo_base_real: u64,
}

impl DreamEngine {
    /// Crea nuevo motor de sueños
    pub fn new() -> Self {
        DreamEngine {
            estado: EstadoSueno::Despertado,
            universo_onirico: None,
            escenario_actual: None,
            historial_resultados: Vec::new(),
            ciclos_por_sueno: 10000, // Equivalente a miles de años
            umbral_score: 0.6,
            inicio_sueno: None,
            ciclo_base_real: 0,
        }
    }

    /// Verifica si puede empezar a soñar
    pub fn puede_sonar(&self) -> bool {
        self.estado == EstadoSueno::Despertado
    }

    /// Inicia un sueño con un escenario
    pub fn iniciar_sueno(&mut self, escenario: EscenarioHipotetico, ciclo_real: u64) -> bool {
        if !self.puede_sonar() {
            return false;
        }

        self.estado = EstadoSueno::Entrando;
        self.ciclo_base_real = ciclo_real;

        // Crear universo onírico basado en estado actual
        let mut universo = UniversoOnirico::nuevo_baseline(0);

        // Aplicar intervención del escenario
        universo.aplicar_intervencion(&escenario.tipo);

        self.universo_onirico = Some(universo);
        self.escenario_actual = Some(escenario);
        self.estado = EstadoSueno::Soñando;
        self.inicio_sueno = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        true
    }

    /// Procesa un ciclo onírico
    pub fn tick(&mut self) -> bool {
        if self.estado != EstadoSueno::Soñando {
            return false;
        }

        if let Some(ref mut universo) = self.universo_onirico {
            // Simular un ciclo
            universo.simular_ciclos(1);

            // Verificar si terminó
            if universo.ciclo >= self.ciclos_por_sueno {
                self.finalizar_sueno();
                return true;
            }
        }

        false
    }

    /// Finaliza el sueño y evalúa resultados
    fn finalizar_sueno(&mut self) {
        self.estado = EstadoSueno::Despertando;

        let universo = match self.universo_onirico.take() {
            Some(u) => u,
            None => return,
        };

        let escenario = match self.escenario_actual.take() {
            Some(e) => e,
            None => return,
        };

        // Métricas de inicio (baseline)
        let metricas_inicio = MetricasSueno::baseline(0);
        let metricas_fin = universo.metricas().clone();

        // Evaluar resultado
        let resultado =
            ResultadoSueno::evaluar(&escenario, &metricas_inicio, &metricas_fin, universo.ciclo);

        self.historial_resultados.push(resultado.clone());

        // Restaurar estado
        self.estado = EstadoSueno::Despertado;
        // Nota: self.escenario_actual ya fue consumido arriba, no podemos actualizarlo aquí
    }

    /// Fuerza finalización inmediata del sueño
    pub fn despertar(&mut self) -> Option<ResultadoSueno> {
        if self.estado == EstadoSueno::Soñando {
            self.finalizar_sueno();
        }

        self.historial_resultados.last().cloned()
    }

    /// Obtiene el estado actual
    pub fn estado(&self) -> EstadoSueno {
        self.estado
    }

    /// Obtiene último resultado
    pub fn ultimo_resultado(&self) -> Option<&ResultadoSueno> {
        self.historial_resultados.last()
    }

    /// Obtiene historial de resultados
    pub fn historial(&self) -> &[ResultadoSueno] {
        &self.historial_resultados
    }

    /// Obtiene estadísticas
    pub fn stats(&self) -> DreamStats {
        let ultimos_5 = self
            .historial_resultados
            .iter()
            .rev()
            .take(5)
            .cloned()
            .collect();

        DreamStats {
            estado: self.estado,
            total_suenos: self.historial_resultados.len(),
            ciclos_por_sueno: self.ciclos_por_sueno,
            umbral_score: self.umbral_score,
            score_promedio: if !self.historial_resultados.is_empty() {
                self.historial_resultados
                    .iter()
                    .map(|r| r.score_bondad)
                    .sum::<f64>()
                    / self.historial_resultados.len() as f64
            } else {
                0.0
            },
            recommendaciones_positivas: self
                .historial_resultados
                .iter()
                .filter(|r| r.recomendar_aplicar)
                .count(),
            ultimos_resultados: ultimos_5,
        }
    }

    /// Establece ciclos por sueño
    pub fn set_ciclos_por_sueno(&mut self, ciclos: u64) {
        self.ciclos_por_sueno = ciclos;
    }

    /// Establece umbral de score
    pub fn set_umbral_score(&mut self, umbral: f64) {
        self.umbral_score = umbral;
    }

    /// Recupera la última recomendación de intervención
    pub fn obtener_recomendacion(&self) -> Option<(EscenarioHipotetico, ResultadoSueno)> {
        self.historial_resultados
            .iter()
            .rev()
            .find(|r| r.recomendar_aplicar)
            .and_then(|_r| {
                // Escenario no se guarda, solo el resultado
                // En implementación real, el escenario se recuperaría
                None
            })
    }
}

impl Default for DreamEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas del motor de sueños
#[derive(Debug, Clone)]
pub struct DreamStats {
    pub estado: EstadoSueno,
    pub total_suenos: usize,
    pub ciclos_por_sueno: u64,
    pub umbral_score: f64,
    pub score_promedio: f64,
    pub recommendaciones_positivas: usize,
    pub ultimos_resultados: Vec<ResultadoSueno>,
}

// ============================================================================
// GESTOR DE DORMICIÓN (PARA INTEGRACIÓN CON SLEEP)
// ============================================================================

/// Wrapper thread-safe para el motor de sueños
pub struct DreamManagerLocked {
    inner: Arc<RwLock<DreamEngine>>,
}

impl DreamManagerLocked {
    /// Crea nuevo manager
    pub fn new() -> Self {
        DreamManagerLocked {
            inner: Arc::new(RwLock::new(DreamEngine::new())),
        }
    }

    /// Verifica si puede iniciar sueño
    pub fn puede_sonar(&self) -> bool {
        if let Ok(engine) = self.inner.read() {
            engine.puede_sonar()
        } else {
            false
        }
    }

    /// Inicia un sueño
    pub fn iniciar_sueno(&self, escenario: EscenarioHipotetico, ciclo_real: u64) -> bool {
        if let Ok(mut engine) = self.inner.write() {
            engine.iniciar_sueno(escenario, ciclo_real)
        } else {
            false
        }
    }

    /// Procesa tick
    pub fn tick(&self) -> bool {
        if let Ok(mut engine) = self.inner.write() {
            engine.tick()
        } else {
            false
        }
    }

    /// Despierta forzadamente
    pub fn despertar(&self) -> Option<ResultadoSueno> {
        if let Ok(mut engine) = self.inner.write() {
            engine.despertar()
        } else {
            None
        }
    }

    /// Obtiene estadísticas
    pub fn stats(&self) -> DreamStats {
        if let Ok(engine) = self.inner.read() {
            engine.stats()
        } else {
            DreamStats {
                estado: EstadoSueno::Despertado,
                total_suenos: 0,
                ciclos_por_sueno: 10000,
                umbral_score: 0.6,
                score_promedio: 0.0,
                recommendaciones_positivas: 0,
                ultimos_resultados: Vec::new(),
            }
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estado_sueno_inicial() {
        let engine = DreamEngine::new();
        assert_eq!(engine.estado(), EstadoSueno::Despertado);
    }

    #[test]
    fn test_puede_sonar() {
        let engine = DreamEngine::new();
        assert!(engine.puede_sonar());
    }

    #[test]
    fn test_iniciar_sueno() {
        let mut engine = DreamEngine::new();
        let escenario = EscenarioHipotetico::tasa_mutacion_aumento(2.0);

        let resultado = engine.iniciar_sueno(escenario, 1000);
        assert!(resultado);
        assert_eq!(engine.estado(), EstadoSueno::Soñando);
    }

    #[test]
    fn test_no_puede_sonar_mientras_suena() {
        let mut engine = DreamEngine::new();
        let escenario = EscenarioHipotetico::tasa_mutacion_aumento(2.0);

        engine.iniciar_sueno(escenario, 1000);
        assert!(!engine.puede_sonar());

        // Forzar despertar
        engine.despertar();
    }

    #[test]
    fn test_tick_sueno() {
        let mut engine = DreamEngine::new();
        engine.set_ciclos_por_sueno(10);

        let escenario = EscenarioHipotetico::tasa_mutacion_aumento(2.0);
        engine.iniciar_sueno(escenario, 0);

        // Simular ticks
        for _ in 0..9 {
            let sigue = engine.tick();
            assert!(!sigue);
        }

        // Último tick termina el sueño
        let terminado = engine.tick();
        assert!(terminado);
        assert_eq!(engine.estado(), EstadoSueno::Despertado);
    }

    #[test]
    fn test_despertar_forzado() {
        let mut engine = DreamEngine::new();
        let escenario = EscenarioHipotetico::tasa_mutacion_aumento(2.0);

        engine.iniciar_sueno(escenario, 0);
        let resultado = engine.despertar();

        assert!(resultado.is_some());
        assert_eq!(engine.estado(), EstadoSueno::Despertado);
    }

    #[test]
    fn test_resultado_sueno() {
        let metricas_inicio = MetricasSueno::baseline(0);
        let mut metricas_fin = MetricasSueno::baseline(0);
        metricas_fin.auton = 150;
        metricas_fin.diversidad = 0.6;

        let escenario = EscenarioHipotetico::tasa_mutacion_aumento(2.0);

        let resultado = ResultadoSueno::evaluar(&escenario, &metricas_inicio, &metricas_fin, 10000);

        assert!(resultado.cambio_poblacion > 0.0);
        assert!(resultado.score_bondad > 0.0);
    }

    #[test]
    fn test_stats() {
        let engine = DreamEngine::new();
        let stats = engine.stats();

        assert_eq!(stats.total_suenos, 0);
        assert_eq!(stats.ciclos_por_sueno, 10000);
    }

    #[test]
    fn test_esceanrio_inyeccion_escoria() {
        let escenario = EscenarioHipotetico::inyeccion_escoria(100.0, "NW");

        assert!(escenario.descripcion.contains("100.0"));
        assert!(escenario.descripcion.contains("NW"));
    }

    #[test]
    fn test_esceanrio_inyeccion_energia() {
        let escenario = EscenarioHipotetico::inyeccion_energia(500.0, "SE");

        assert!(escenario.descripcion.contains("500.0"));
        assert!(escenario.descripcion.contains("SE"));
    }

    #[test]
    fn test_universo_onirico_baseline() {
        let universo = UniversoOnirico::nuevo_baseline(0);

        assert_eq!(universo.ciclo, 0);
        assert_eq!(universo.metricas.auton, 100);
    }

    #[test]
    fn test_universo_onirico_simular() {
        let mut universo = UniversoOnirico::nuevo_baseline(0);
        universo.simular_ciclos(100);

        assert_eq!(universo.ciclo, 100);
    }

    #[test]
    fn test_tipo_intervencion_descripcion() {
        assert_eq!(
            TipoIntervencion::Control.descripcion(),
            "Control (sin intervención)"
        );
        assert_eq!(
            TipoIntervencion::InyectarEscoria.descripcion(),
            "Inyección de escoria"
        );
    }

    #[test]
    fn test_metricas_sueno_baseline() {
        let metricas = MetricasSueno::baseline(1000);

        assert_eq!(metricas.ciclo, 1000);
        assert_eq!(metricas.auton, 100);
        assert_eq!(metricas.diversidad, 0.5);
    }

    #[test]
    fn test_sueñoCompleto() {
        let mut engine = DreamEngine::new();
        engine.set_ciclos_por_sueno(5);

        let escenario = EscenarioHipotetico::aumentar_difusion(1.5);
        engine.iniciar_sueno(escenario, 50000);

        // Procesar hasta que termine
        while engine.tick() == false {
            // Continue
        }

        let stats = engine.stats();
        assert_eq!(stats.total_suenos, 1);

        let resultado = engine.ultimo_resultado();
        assert!(resultado.is_some());
    }

    #[test]
    fn test_dream_manager_locked() {
        let manager = DreamManagerLocked::new();

        assert!(manager.puede_sonar());

        let escenario = EscenarioHipotetico::tasa_mutacion_aumento(2.0);
        let started = manager.iniciar_sueno(escenario, 1000);
        assert!(started);

        assert!(!manager.puede_sonar());

        // Despertar
        let resultado = manager.despertar();
        assert!(resultado.is_some());
    }

    #[test]
    fn test_recomendar_aplicar() {
        let metricas_inicio = MetricasSueno::baseline(0);
        let mut metricas_fin = MetricasSueno::baseline(0);
        metricas_fin.auton = 250; // 150% increase
        metricas_fin.diversidad = 0.85; // +0.35
        metricas_fin.estabilidad = 0.92; // +0.22

        let escenario = EscenarioHipotetico::tasa_mutacion_aumento(2.0);

        let resultado = ResultadoSueno::evaluar(&escenario, &metricas_inicio, &metricas_fin, 10000);

        // Score alto debe recomendar (cambios significativos)
        // cambio_poblacion = 1.5, cambio_diversidad = 0.35, cambio_estabilidad = 0.22
        // score = 0.35*0.3 + 0.22*0.3 + 1.5*0.4 = 0.105 + 0.066 + 0.6 = 0.771
        assert!(
            resultado.score_bondad > 0.6,
            "score {} should be > 0.6",
            resultado.score_bondad
        );
        assert!(resultado.recomendar_aplicar);
    }
}

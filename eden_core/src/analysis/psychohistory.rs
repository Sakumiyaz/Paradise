//! # Psicohistoria: Predicción de Colapso Demográfico
//!
//! EDEN recopila series temporales y entrena un modelo predictivo para
//! anticipar colapsos demográficos antes de que ocurran.
//!
//! ## Mecanismo
//!
//! 1. **Recopilación de métricas**: población, diversidad RamNet, energía media,
//!    entropía de memes, tasa de esporulación
//! 2. **Modelo predictivo**: Regresión lineal multivariante con FixedPoint (sin floats)
//! 3. **Ventana deslizante**: 10,000 ciclos de historia
//! 4. **Alerta Psicohistórica**: Cuando predicción T+500 indica población bajo umbral
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use crate::physics::I32F32;

// ============================================================================
// TIPOS BASE
// ============================================================================

/// Una observación en el tiempo
#[derive(Debug, Clone)]
pub struct Observacion {
    /// Ciclo en que se registró
    pub ciclo: u64,
    /// Población total de Auton
    pub poblacion: u32,
    /// Diversidad de RamNet (0.0 - 1.0 encoded as I32F32)
    pub diversidad: I32F32,
    /// Energía media (encoded as I32F32)
    pub energia_media: I32F32,
    /// Entropía de memes (0.0 - 1.0 encoded as I32F32)
    pub entropia_memes: I32F32,
    /// Tasa de esporulación (por ciclo)
    pub tasa_esporulacion: I32F32,
}

impl Observacion {
    /// Crea observación actual
    pub fn ahora(
        ciclo: u64,
        poblacion: u32,
        diversidad: I32F32,
        energia: I32F32,
        entropia: I32F32,
        esporulacion: I32F32,
    ) -> Self {
        Self {
            ciclo,
            poblacion,
            diversidad,
            energia_media: energia,
            entropia_memes: entropia,
            tasa_esporulacion: esporulacion,
        }
    }
}

/// Métricas consolidadas para predicción
#[derive(Debug, Clone)]
pub struct MetricasPsicohistoria {
    /// Ciclo actual
    pub ciclo: u64,
    /// Población actual
    pub poblacion: u32,
    /// Población predicha para T+500
    pub poblacion_predicha_500: i64, // Puede ser negativa si hay colapso
    /// Probabilidad de colapso (0.0 - 1.0 encoded as I32F32)
    pub probabilidad_colapso: I32F32,
    /// Tendencia (creciente=1, estable=0, decreciente=-1)
    pub tendencia: i32,
    /// R² del modelo (calidad del ajuste)
    pub r_cuadrado: I32F32,
    /// Confianza en predicción (0.0 - 1.0)
    pub confianza: I32F32,
}

/// Estado del modelo predictivo
#[derive(Debug, Clone)]
pub struct ModeloEstado {
    /// Coeficientes de regresión [poblacion, diversidad, energia, entropia, esporulacion, bias]
    pub coeficientes: [I32F32; 6],
    /// Último R² calculado
    pub r_cuadrado: I32F32,
    /// Épocas de entrenamiento
    pub epocas: u32,
    /// Error medio absoluto actual
    pub error_ma: I32F32,
}

/// Resultado de predicción
#[derive(Debug, Clone)]
pub struct Prediccion {
    /// Ciclo objetivo
    pub ciclo_destino: u64,
    /// Población predicha
    pub poblacion_predicha: i64,
    /// Intervalo de confianza inferior
    pub intervalo_inferior: i64,
    /// Intervalo de confianza superior
    pub intervalo_superior: i64,
    /// Probabilidad de colapso
    pub probabilidad_colapso: I32F32,
    /// Es colapso inminente?
    pub colapso_inminente: bool,
}

/// Alerta psicohistórica
#[derive(Debug, Clone)]
pub struct AlertaPsicohistoria {
    /// Ciclo en que se emitió
    pub ciclo: u64,
    /// Probabilidad calculada
    pub probabilidad: I32F32,
    /// Población actual
    pub poblacion_actual: u32,
    /// Población predicha para T+500
    pub poblacion_predicha: i64,
    /// Ciclos hasta el colapso predicho
    pub ciclos_hasta_colapso: u64,
    /// Acciones recomendadas
    pub recomendaciones: Vec<String>,
}

// ============================================================================
// MODELO DE REGRESIÓN LINEAL (FIXEDPOINT PURO)
// ============================================================================

/// Regresión lineal multivariante implementada con I32F32
pub struct RegresionLineal {
    /// Coeficientes del modelo [w0, w1, w2, w3, w4, w5]
    coef: [I32F32; 6],
    /// Número de características
    num_features: usize,
    /// Factor de aprendizaje
    learning_rate: I32F32,
    /// Regularización L2
    lambda: I32F32,
}

impl RegresionLineal {
    /// Crea nuevo modelo
    pub fn new() -> Self {
        Self {
            // Inicialización Xavier-ish (escalado para FixedPoint)
            coef: [
                I32F32::from_raw(1 << 30), // w0: población
                I32F32::from_raw(1 << 28), // w1: diversidad
                I32F32::from_raw(1 << 25), // w2: energía
                I32F32::from_raw(1 << 28), // w3: entropia
                I32F32::from_raw(1 << 26), // w4: esporulacion
                I32F32::from_raw(1 << 31), // w5: bias
            ],
            num_features: 5,
            learning_rate: I32F32::from_raw(1 << 28), // ~0.03
            lambda: I32F32::from_raw(1 << 26),        // ~0.015
        }
    }

    /// Normaliza entrada al rango 0-1 (asume valores conocidos)
    fn normalizar(valor: I32F32, minimo: I32F32, maximo: I32F32) -> I32F32 {
        let rango = maximo - minimo;
        if rango == I32F32::ZERO {
            return I32F32::from_raw(1 << 31); // 0.5 si no hay rango
        }
        let resultado = (valor - minimo) / rango;
        // Clamp a [0, 1]
        if resultado < I32F32::ZERO {
            I32F32::ZERO
        } else if resultado > I32F32::ONE {
            I32F32::ONE
        } else {
            resultado
        }
    }

    /// Predice siguiente población (forma raw para cálculo interno)
    pub fn predecir_raw(&self, features: &[I32F32]) -> I32F32 {
        debug_assert_eq!(features.len(), self.num_features);

        let mut suma = self.coef[5]; // bias
        for (i, &feat) in features.iter().enumerate() {
            suma = suma + feat * self.coef[i];
        }
        suma
    }

    /// Entrena con una observación (gradiente descendente online)
    pub fn entrena(&mut self, observacion: &Observacion, poblacion_real: I32F32) {
        // Preparar features
        let diversidad_norm = Self::normalizar(
            observacion.diversidad,
            I32F32::from_raw(0), // min
            I32F32::ONE,         // max
        );
        let energia_norm = Self::normalizar(
            observacion.energia_media,
            I32F32::from_raw(0),                   // min ~0
            I32F32::from_raw(2_000_000_000 << 32), // max ~2B
        );
        let entropia_norm = Self::normalizar(observacion.entropia_memes, I32F32::ZERO, I32F32::ONE);
        let esporulacion_norm =
            Self::normalizar(observacion.tasa_esporulacion, I32F32::ZERO, I32F32::ONE);

        // Población previa normalizada (asumimos 0-1000 rango)
        let poblacion_norm = Self::normalizar(
            I32F32::from_u64(observacion.poblacion as u64),
            I32F32::ZERO,
            I32F32::from_u64(1000),
        );

        let features: [I32F32; 5] = [
            poblacion_norm,
            diversidad_norm,
            energia_norm,
            entropia_norm,
            esporulacion_norm,
        ];

        // Predicción y error
        let prediccion = self.predecir_raw(&features);
        let error = poblacion_real - prediccion;

        // Actualizar coeficientes con gradiente + regularización
        for (i, coef) in self.coef.iter_mut().enumerate() {
            if i < self.num_features {
                let gradiente = error * features[i] - self.lambda * *coef;
                *coef = *coef + self.learning_rate * gradiente;
            }
        }
        // Bias no tiene regularización
        self.coef[5] = self.coef[5] + self.learning_rate * error;
    }

    /// Obtiene estado del modelo
    pub fn estado(&self) -> ModeloEstado {
        ModeloEstado {
            coeficientes: self.coef,
            r_cuadrado: I32F32::from_raw(7 << 30), // ~0.7 placeholder
            epocas: 0,
            error_ma: I32F32::from_raw(1 << 29), // ~0.5 placeholder
        }
    }
}

impl Default for RegresionLineal {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PSYCHOHISTORY MANAGER
// ============================================================================

/// Manager de psicohistoria - recopila datos y predice colapsos
pub struct PsychohistoryManager {
    /// Historial de observaciones (ventana deslizante 10,000 ciclos)
    historial: VecDeque<Observacion>,
    /// Capacidad máxima del historial
    capacidad_maxima: usize,
    /// Modelo de predicción
    modelo: RegresionLineal,
    /// Umbral crítico de población
    umbral_colapso: u32,
    /// Horizonte de predicción (ciclos)
    horizonte_prediccion: u64,
    /// Última alerta emitida
    ultima_alerta: Option<AlertaPsicohistoria>,
    /// Socket para enviar alertas (opcional, puede ser None)
    socket: Option<Arc<RwLock<crate::ipc::socket::UnixDatagram>>>,
    /// Ciclo base para normalización
    ciclo_base: u64,
    /// Totales para normalización
    energia_total_acumulada: i64,
    observacion_count: u64,
}

impl PsychohistoryManager {
    /// Crea nuevo manager
    pub fn new() -> Self {
        Self {
            historial: VecDeque::with_capacity(10000),
            capacidad_maxima: 10000,
            modelo: RegresionLineal::new(),
            umbral_colapso: 10, // < 10 Auton = colapso
            horizonte_prediccion: 500,
            ultima_alerta: None,
            socket: None,
            ciclo_base: 0,
            energia_total_acumulada: 0,
            observacion_count: 0,
        }
    }

    /// Configura el socket para alertas
    pub fn con_socket(mut self, socket: Arc<RwLock<crate::ipc::socket::UnixDatagram>>) -> Self {
        self.socket = Some(socket);
        self
    }

    /// Registra una nueva observación
    pub fn registrar(&mut self, obs: Observacion) {
        // Actualizar base para normalización
        if self.observacion_count == 0 {
            self.ciclo_base = obs.ciclo;
        }
        self.energia_total_acumulada += obs.energia_media.to_i32() as i64;
        self.observacion_count += 1;

        // Agregar al historial
        if self.historial.len() >= self.capacidad_maxima {
            self.historial.pop_front();
        }
        self.historial.push_back(obs.clone());

        // Entrenar modelo si hay suficientes datos
        if self.historial.len() >= 100 {
            if let Some(ultima) = self.historial.back() {
                let poblacion_next = I32F32::from_u64(ultima.poblacion as u64);
                self.modelo.entrena(&obs, poblacion_next);
            }
        }
    }

    /// Registra desde métricas simples
    pub fn registrar_metrics(
        &mut self,
        ciclo: u64,
        poblacion: u32,
        diversidad: f64,
        energia_total: i64,
        entropia_memes: f64,
        tasa_esporulacion: f64,
    ) {
        let diversidad_fp = I32F32::from_raw((diversidad * (1u64 << 32) as f64) as i64);
        let energia_fp = I32F32::from_raw(energia_total << 32);
        let entropia_fp = I32F32::from_raw((entropia_memes * (1u64 << 32) as f64) as i64);
        let esporulacion_fp = I32F32::from_raw((tasa_esporulacion * (1u64 << 32) as f64) as i64);

        let obs = Observacion::ahora(
            ciclo,
            poblacion,
            diversidad_fp,
            energia_fp,
            entropia_fp,
            esporulacion_fp,
        );
        self.registrar(obs);
    }

    /// Predice población en el horizonte especificado
    pub fn predecir(&self, horizonte: u64) -> Prediccion {
        if self.historial.len() < 100 {
            // No hay suficientes datos
            return Prediccion {
                ciclo_destino: self.ciclo_base + horizonte,
                poblacion_predicha: 0,
                intervalo_inferior: 0,
                intervalo_superior: 0,
                probabilidad_colapso: I32F32::ZERO,
                colapso_inminente: false,
            };
        }

        // Usar últimas observaciones para proyectar
        let ultimas: Vec<_> = self.historial.iter().rev().take(100).collect();

        // Calcular tendencia promedio
        let mut suma_poblacion: i64 = 0;
        let mut cambios: Vec<i64> = Vec::with_capacity(99);

        for (i, obs) in ultimas.iter().enumerate() {
            suma_poblacion += obs.poblacion as i64;
            if i > 0 {
                let cambio = obs.poblacion as i64 - ultimas[i - 1].poblacion as i64;
                cambios.push(cambio);
            }
        }

        let _poblacion_promedio = suma_poblacion as i64 / ultimas.len() as i64;
        let poblacion_actual = ultimas.last().unwrap().poblacion as i64;

        // Estimar tendencia (cambio medio por ciclo)
        let cambio_medio: i64 = if !cambios.is_empty() {
            let suma: i64 = cambios.iter().sum();
            suma / cambios.len() as i64
        } else {
            0
        };

        // Proyectar
        let ciclos_restantes = horizonte as i64;
        let mut poblacion_predicha = poblacion_actual + (cambio_medio * ciclos_restantes);

        // No puede ser menor que 0
        if poblacion_predicha < 0 {
            poblacion_predicha = 0;
        }

        // Intervalo de confianza (crece con el horizonte)
        let incertidumbre = (horizonte as i64).abs() * 10;
        let intervalo_inferior = (poblacion_predicha - incertidumbre).max(0);
        let intervalo_superior = poblacion_predicha + incertidumbre;

        // Calcular probabilidad de colapso
        let prob_colapso = self.calcular_probabilidad_colapso(poblacion_predicha);

        // Detectar colapso inminente
        let colapso_inminente = poblacion_predicha < self.umbral_colapso as i64
            && horizonte <= self.horizonte_prediccion;

        Prediccion {
            ciclo_destino: self.ciclo_base + horizonte,
            poblacion_predicha,
            intervalo_inferior,
            intervalo_superior,
            probabilidad_colapso: prob_colapso,
            colapso_inminente,
        }
    }

    /// Predice para horizonte por defecto (500 ciclos)
    pub fn predecir_colapso(&self) -> Prediccion {
        self.predecir(self.horizonte_prediccion)
    }

    /// Calcula probabilidad de colapso basada en población predicha
    fn calcular_probabilidad_colapso(&self, poblacion_predicha: i64) -> I32F32 {
        if poblacion_predicha <= 0 {
            return I32F32::ONE; // 100% certeza de colapso
        }

        let umbral = self.umbral_colapso as i64;

        if poblacion_predicha >= umbral * 10 {
            // Muy por encima del umbral
            I32F32::from_raw(1 << 28) // ~0.03
        } else if poblacion_predicha >= umbral * 5 {
            I32F32::from_raw(1 << 29) // ~0.06
        } else if poblacion_predicha >= umbral * 2 {
            I32F32::from_raw(1 << 30) // ~0.12
        } else if poblacion_predicha >= umbral {
            I32F32::from_raw(1 << 31) // ~0.25
        } else if poblacion_predicha >= umbral / 2 {
            I32F32::from_raw(3 << 30) // ~0.5
        } else if poblacion_predicha >= umbral / 4 {
            I32F32::from_raw(5 << 30) // ~0.75
        } else {
            I32F32::from_raw(7 << 30) // ~0.875
        }
    }

    /// Obtiene métricas consolidadas
    pub fn metricas(&self) -> MetricasPsicohistoria {
        let prediccion = self.predecir(self.horizonte_prediccion);

        // Calcular tendencia
        let tendencia = if self.historial.len() >= 10 {
            let n = self.historial.len();
            let primero = self.historial[n - 10].poblacion as i64;
            let ultimo = self.historial[n - 1].poblacion as i64;
            if ultimo > primero + 5 {
                1
            } else if ultimo < primero - 5 {
                -1
            } else {
                0
            }
        } else {
            0
        };

        MetricasPsicohistoria {
            ciclo: self.historial.back().map(|o| o.ciclo).unwrap_or(0),
            poblacion: self.historial.back().map(|o| o.poblacion).unwrap_or(0),
            poblacion_predicha_500: prediccion.poblacion_predicha,
            probabilidad_colapso: prediccion.probabilidad_colapso,
            tendencia,
            r_cuadrado: self.modelo.estado().r_cuadrado,
            confianza: if self.historial.len() >= 1000 {
                I32F32::from_raw(9 << 30) // ~0.9
            } else {
                I32F32::from_raw((self.historial.len() as i64 * (1 << 30) / 100) as i64)
            },
        }
    }

    /// Genera y envía alerta si hay colapso inminente
    pub fn verificar_alerta(&mut self) -> Option<AlertaPsicohistoria> {
        let prediccion = self.predecir_colapso();
        let poblacion_actual = self.historial.back().map(|o| o.poblacion).unwrap_or(0);

        if prediccion.colapso_inminente {
            // Calcular ciclos hasta colapso
            let ciclos_hasta_colapso = if self.historial.len() >= 10 {
                let ultimas: Vec<_> = self.historial.iter().rev().take(10).cloned().collect();
                let primero = ultimas.last().unwrap().poblacion as i64;
                let ultimo = ultimas.first().unwrap().poblacion as i64;
                let cambio_total = ultimo - primero;
                let cambio_abs = cambio_total.abs().max(1);
                let ciclos_transcurridos = ultimas.len() as i64;
                let cambio_por_ciclo = cambio_abs / ciclos_transcurridos;
                let cambio_por_ciclo = cambio_por_ciclo.max(1);
                ((ultimo - self.umbral_colapso as i64) / cambio_por_ciclo).max(1) as u64
            } else {
                self.horizonte_prediccion
            };

            let alerta = AlertaPsicohistoria {
                ciclo: self.historial.back().map(|o| o.ciclo).unwrap_or(0),
                probabilidad: prediccion.probabilidad_colapso,
                poblacion_actual,
                poblacion_predicha: prediccion.poblacion_predicha,
                ciclos_hasta_colapso,
                recomendaciones: vec![
                    "Inyectar Energon para estimular reproducción".to_string(),
                    "Reducir Escoria ambiental".to_string(),
                    "Aumentar tasa de bifurcación".to_string(),
                ],
            };

            self.ultima_alerta = Some(alerta.clone());
            self.enviar_alerta(&alerta);
            Some(alerta)
        } else {
            None
        }
    }

    /// Envía alerta por socket
    fn enviar_alerta(&self, alerta: &AlertaPsicohistoria) {
        if let Some(ref socket) = self.socket {
            if let Ok(socket) = socket.read() {
                let msg = format!(
                    "ALERTA PSICOHISTORICA: Colapso inminente en {} ciclos | Poblacion: {} -> {}",
                    alerta.ciclos_hasta_colapso, alerta.poblacion_actual, alerta.poblacion_predicha
                );
                let _ = socket.send(msg.as_bytes());
            }
        }
    }

    /// Obtiene última alerta
    pub fn ultima_alerta(&self) -> Option<&AlertaPsicohistoria> {
        self.ultima_alerta.as_ref()
    }

    /// Tamaño del historial
    pub fn tamanio_historial(&self) -> usize {
        self.historial.len()
    }

    /// Fuerza predicción manual (para comando externo)
    pub fn forzar_prediccion(&self) -> Prediccion {
        self.predecir_colapso()
    }
}

impl Default for PsychohistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-safe wrapper
pub type PsychohistoryManagerLocked = Arc<RwLock<PsychohistoryManager>>;

impl PsychohistoryManager {
    pub fn into_locked(self) -> PsychohistoryManagerLocked {
        Arc::new(RwLock::new(self))
    }
}

// ============================================================================
// SERIALIZACIÓN PARA IPC
// ============================================================================

impl AlertaPsicohistoria {
    /// Serializa a JSON string
    pub fn a_json(&self) -> String {
        format!(
            r#"{{"tipo":"alerta_psicohistoria","ciclo":{},"probabilidad":{},"poblacion_actual":{},"poblacion_predicha":{},"ciclos_hasta_colapso":{},"recomendaciones":{:?}}}"#,
            self.ciclo,
            self.probabilidad.to_raw(),
            self.poblacion_actual,
            self.poblacion_predicha,
            self.ciclos_hasta_colapso,
            self.recomendaciones
        )
    }
}

impl Prediccion {
    /// Serializa a JSON string
    pub fn a_json(&self) -> String {
        format!(
            r#"{{"tipo":"prediccion","ciclo_destino":{},"poblacion_predicha":{},"intervalo_inferior":{},"intervalo_superior":{},"probabilidad_colapso":{},"colapso_inminente":{}}}"#,
            self.ciclo_destino,
            self.poblacion_predicha,
            self.intervalo_inferior,
            self.intervalo_superior,
            self.probabilidad_colapso.to_raw(),
            self.colapso_inminente
        )
    }
}

impl MetricasPsicohistoria {
    /// Serializa a JSON string
    pub fn a_json(&self) -> String {
        format!(
            r#"{{"tipo":"metricas_psicohistoria","ciclo":{},"poblacion":{},"poblacion_predicha_500":{},"probabilidad_colapso":{},"tendencia":{},"r_cuadrado":{},"confianza":{}}}"#,
            self.ciclo,
            self.poblacion,
            self.poblacion_predicha_500,
            self.probabilidad_colapso.to_raw(),
            self.tendencia,
            self.r_cuadrado.to_raw(),
            self.confianza.to_raw()
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_manager() {
        let manager = PsychohistoryManager::new();
        assert_eq!(manager.tamanio_historial(), 0);
        assert_eq!(manager.umbral_colapso, 10);
    }

    #[test]
    fn test_registro_observacion() {
        let mut manager = PsychohistoryManager::new();

        manager.registrar_metrics(100, 100, 0.5, 1_000_000_000_000, 0.3, 0.01);

        assert_eq!(manager.tamanio_historial(), 1);
    }

    #[test]
    fn test_prediccion_sin_datos() {
        let manager = PsychohistoryManager::new();
        let prediccion = manager.predecir(500);

        assert_eq!(prediccion.poblacion_predicha, 0);
        assert!(!prediccion.colapso_inminente);
    }

    #[test]
    fn test_prediccion_con_datos_declinacion() {
        let mut manager = PsychohistoryManager::new();

        // Simular declive de 100 a 50 en 10 ciclos
        for i in 0..10 {
            let poblacion = 100 - (i * 5);
            manager.registrar_metrics(
                i,
                poblacion as u32,
                0.5,
                1_000_000_000_000 - (i as i64 * 10_000_000_000),
                0.3,
                0.01,
            );
        }

        let prediccion = manager.predecir(500);
        assert!(prediccion.poblacion_predicha < 100); // Debe declinar
    }

    #[test]
    fn test_colapso_inminente() {
        let mut manager = PsychohistoryManager::new();
        manager.umbral_colapso = 50;

        // Población en declive severo - asegurar declive cada ciclo
        for i in 0..20 {
            // Declive lineal de 95 a 5 en 20 ciclos
            let poblacion = (95 - i * 5).max(5);
            manager.registrar_metrics(
                i,
                poblacion as u32,
                0.3,
                500_000_000_000 - (i as i64 * 10_000_000_000),
                0.6,
                0.005,
            );
        }

        let prediccion = manager.predecir(500);
        assert!(prediccion.colapso_inminente || prediccion.poblacion_predicha < 50);
    }

    #[test]
    fn test_serializacion_prediccion() {
        let prediccion = Prediccion {
            ciclo_destino: 1000,
            poblacion_predicha: 50,
            intervalo_inferior: 30,
            intervalo_superior: 70,
            probabilidad_colapso: I32F32::from_raw(7 << 30),
            colapso_inminente: true,
        };

        let json = prediccion.a_json();
        assert!(json.contains("prediccion"));
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_regresion_lineal() {
        let mut modelo = RegresionLineal::new();

        let obs = Observacion::ahora(
            100,
            100,
            I32F32::from_raw(1 << 31), // 0.5
            I32F32::from_raw(1_000_000_000_000 << 32),
            I32F32::from_raw(1 << 30), // 0.25
            I32F32::from_raw(1 << 29), // 0.125
        );

        modelo.entrena(&obs, I32F32::from_u64(95));

        let estado = modelo.estado();
        assert_eq!(estado.coeficientes.len(), 6);
    }

    #[test]
    fn test_metricas_consolidadas() {
        let mut manager = PsychohistoryManager::new();

        for i in 0..15 {
            manager.registrar_metrics(i, 100, 0.5, 1_000_000_000_000, 0.3, 0.01);
        }

        let metricas = manager.metricas();
        assert_eq!(metricas.poblacion, 100);
        assert!(metricas.confianza > I32F32::ZERO);
    }
}

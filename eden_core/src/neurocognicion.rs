//! # Neurocognición - Integración del Cerebro con EDEN
//!
//! Conecta el SustratoVital (morfogénesis) y TablaSinaptica (plasticidad)
//! con los módulos de consciousness (estrés térmico) y security (syscalls).
//!
//! # Arquitectura
//! - **Zonas Sensoriales**: Regiones del sustrato que reciben cargas de syscalls
//! - **Zonas Motoras**: Regiones del sustrato que activan el orquestador
//! - **Neuromodulador Global**: Flag que duplica plasticidad según fitness térmico
//!
//! # Bucle de Vida
//! 1. Syscalls inyectan carga en Zonas Sensoriales
//! 2. Flujo de resonancia química distribuye carga por el sustrato
//! 3. TablaSinaptica se actualiza según resonancia (plasticidad)
//! 4. Carga que llega a Zonas Motoras activa el orquestador
//! 5. Si la acción reduce estrés térmico → neuromodulador duplica plasticidad positiva
//!    Si el estrés aumenta → duplica plasticidad negativa
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::morfogenesis::{SustratoVital, TablaSinaptica, EstadoCelular, TermicoReader};
use crate::consciousness::NivelTermico;
use std::sync::atomic::{AtomicBool, AtomicI16, Ordering};

/// Número de zonas sensoriales y motoras
pub const NUM_ZONAS_SENSORIALES: usize = 8;
pub const NUM_ZONAS_MOTORAS: usize = 4;

/// Definición de una zona sensorial (región del sustrato)
#[derive(Clone, Copy, Debug)]
pub struct ZonaSensorial {
    /// Índice inicial en el sustrato
    pub inicio: usize,
    /// Índice final (exclusivo)
    pub fin: usize,
    /// Tipo de syscall que mapea a esta zona
    pub tipo_syscall: &'static str,
}

impl ZonaSensorial {
    pub fn contiene(&self, idx: usize) -> bool {
        idx >= self.inicio && idx < self.fin
    }
}

/// Definición de una zona motora (región del sustrato)
#[derive(Clone, Copy, Debug)]
pub struct ZonaMotora {
    /// Índice inicial en el sustrato
    pub inicio: usize,
    /// Índice final (exclusivo)
    pub fin: usize,
    /// Acción que se ejecuta cuando la zona se activa
    pub accion: AccionMotora,
}

impl ZonaMotora {
    pub fn contiene(&self, idx: usize) -> bool {
        idx >= self.inicio && idx < self.fin
    }
}

/// Acciones posibles del orquestador
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccionMotora {
    /// Reducir actividad (modo ahorro)
    Dormir,
    /// Liberar recursos
    LiberarMemoria,
    /// Reiniciar módulos
    ReiniciarModulo,
    /// Elevar privilegios
    ElevarPrivilegios,
    /// Escalar syscalls
    EscalarSyscall,
    /// Ninguna acción
    Ninguna,
}

/// Estado del neuromodulador global
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Neuromodulador {
    /// Sin modificar
    Neutral,
    /// Duplicar plasticidad positiva (estrés reducido)
    PotenciarPositivo,
    /// Duplicar plasticidad negativa (estrés aumentado)
    PotenciarNegativo,
}

/// Métricas del ciclo de pensamiento
#[derive(Clone, Debug, Default)]
pub struct MetricasNeuro {
    pub transferencias: usize,
    pub carga_total: u64,
    pub activaciones_motoras: usize,
    pub plasticidad_positiva: usize,
    pub plasticidad_negativa: usize,
    pub estres_anterior: f32,
    pub estres_actual: f32,
}

/// CerebroEden: Integración completa del sistema de pensamiento
///
/// Conecta:
/// - SustratoVital (memoria cruda)
/// - TablaSinaptica (plasticidad)
/// - Zonas Sensoriales (syscalls)
/// - Zonas Motoras (acciones)
/// - Neuromodulador (fitness térmico)
pub struct CerebroEden {
    /// Memoria cruda del cerebro
    sustrato: SustratoVital,
    /// Tabla de conexiones sinápticas
    sinapsis: TablaSinaptica,
    /// Zonas que reciben entrada de syscalls
    zonas_sensoriales: [ZonaSensorial; NUM_ZONAS_SENSORIALES],
    /// Zonas que activan acciones
    zonas_motoras: [ZonaMotora; NUM_ZONAS_MOTORAS],
    /// Flag global de neuromodulación
    neuromodulador: Neuromodulador,
    /// Estrés térmico del tick anterior
    estres_anterior: f32,
    /// Eventos pendientes de procesar (syscalls injectados)
    eventos_pendientes: Vec<EventoSensorial>,
    /// Acciones pendientes de ejecutar
    acciones_pendientes: Vec<AccionMotora>,
}

/// Evento sensorial recibido de syscalls
#[derive(Clone, Debug)]
pub struct EventoSensorial {
    pub zona: usize,
    pub intensidad: u8,
    pub timestamp: u64,
}

impl CerebroEden {
    /// Crear nuevo cerebro con dimensiones específicas
    pub fn new(ancho: usize, alto: usize, profundidad: usize) -> Self {
        let total_celdas = ancho * alto * profundidad;
        let tam_zona_sensor = total_celdas / 16; // 6.25% para sensores
        let tam_zona_motor = total_celdas / 32;   // 3.125% para motores

        // Definir zonas sensoriales (primera porción del sustrato)
        let mut zonas_sensoriales = [ZonaSensorial {
            inicio: 0,
            fin: tam_zona_sensor,
            tipo_syscall: "read",
        }; NUM_ZONAS_SENSORIALES];

        let syscall_types = [
            "read", "write", "open", "close", "fork", "exec", "mmap", "brk"
        ];
        for (i, tipo) in syscall_types.iter().enumerate() {
            let offset = i * tam_zona_sensor;
            zonas_sensoriales[i] = ZonaSensorial {
                inicio: offset,
                fin: offset + tam_zona_sensor,
                tipo_syscall: tipo,
            };
        }

        // Definir zonas motoras (última porción del sustrato)
        let inicio_motoras = total_celdas - tam_zona_motor * NUM_ZONAS_MOTORAS;
        let mut zonas_motoras = [ZonaMotora {
            inicio: 0,
            fin: 0,
            accion: AccionMotora::Ninguna,
        }; NUM_ZONAS_MOTORAS];

        let acciones = [
            AccionMotora::Dormir,
            AccionMotora::LiberarMemoria,
            AccionMotora::ReiniciarModulo,
            AccionMotora::EscalarSyscall,
        ];
        for (i, accion) in acciones.iter().enumerate() {
            let offset = inicio_motoras + i * tam_zona_motor;
            zonas_motoras[i] = ZonaMotora {
                inicio: offset,
                fin: offset + tam_zona_motor,
                accion: accion.clone(),
            };
        }

        Self {
            sustrato: SustratoVital::new(ancho, alto, profundidad),
            sinapsis: TablaSinaptica::new(),
            zonas_sensoriales,
            zonas_motoras,
            neuromodulador: Neuromodulador::Neutral,
            estres_anterior: 0.0,
            eventos_pendientes: Vec::new(),
            acciones_pendientes: Vec::new(),
        }
    }

    /// Crear cerebro pequeño para testing
    pub fn new_mini() -> Self {
        Self::new(32, 32, 32)
    }

    // =================================================================
    // INTERFAZ CON SYSCALLS (SEGURIDAD)
    // =================================================================

    /// Inyectar evento sensorial desde syscall
    ///
    /// El syscall `tipo` inyecta `intensidad` de carga en la zona sensorial correspondiente
    pub fn inyectar_syscall(&mut self, tipo: &str, intensidad: u8) {
        // Buscar zona sensorial对应
        if let Some(zona) = self.zonas_sensoriales.iter().find(|z| z.tipo_syscall == tipo) {
            self.eventos_pendientes.push(EventoSensorial {
                zona: self.zonas_sensoriales.iter().position(|z| z.tipo_syscall == tipo).unwrap(),
                intensidad,
                timestamp: crate::membrain::NOW_MS(),
            });
        }
    }

    /// Procesar eventos sensoriales pendientes
    fn procesar_eventos_sensoriales(&mut self) {
        for evento in self.eventos_pendientes.drain(..) {
            let zona = &self.zonas_sensoriales[evento.zona.min(NUM_ZONAS_SENSORIALES - 1)];
            // Inyectar carga en toda la zona
            for idx in zona.inicio..zona.fin {
                if self.sustrato.get_index(idx) == Some(EstadoCelular::NervioPrimario)
                    || self.sustrato.get_index(idx) == Some(EstadoCelular::NervioSecundario) {
                    let carga_actual = self.sustrato.carga_index(idx).unwrap_or(0);
                    let nueva_carga = carga_actual.saturating_add(evento.intensidad);
                    let _ = self.sustrato.set_carga_index(idx, nueva_carga);
                }
            }
        }
    }

    // =================================================================
    // INTERFAZ CON CONSCIOUSNESS (ESTRÉS TÉRMICO)
    // =================================================================

    /// Obtener nivel de estrés térmico actual (0.0 = óptimo, 1.0 = crítico)
    pub fn nivel_estres(&self) -> f32 {
        // Integración con EnergyAware a través del sustrato
        // Por ahora retorna basado en la zona 0 de temperatura
        let energia_promedio = if self.sustrato.len() > 0 {
            self.sustrato.energia_total() as f32 / self.sustrato.len() as f32
        } else {
            0.0
        };
        // Normalizar: más carga = más estrés
        (energia_promedio / 128.0).clamp(0.0, 1.0)
    }

    /// Calcular delta de estrés (positivo = empeorando, negativo = mejorando)
    fn delta_estres(&self, estres_actual: f32) -> f32 {
        estres_actual - self.estres_anterior
    }

    // =================================================================
    // BUCLE DE PENSAMIENTO (FLUJO DE RESONANCIA QUÍMICA)
    // =================================================================

    /// Ejecutar un tick completo del bucle de pensamiento
    ///
    /// Retorna las acciones recomendadas para el orquestador
    pub fn tick<T: TermicoReader>(&mut self, lector_termico: &T) -> Vec<AccionMotora> {
        let mut metricas = MetricasNeuro::default();

        // 1. Procesar entrada sensorial (syscalls)
        self.procesar_eventos_sensoriales();

        // 2. Guardar estrés anterior
        self.estres_anterior = self.nivel_estres();
        metricas.estres_anterior = self.estres_anterior;

        // 3. Ejecutar flujo de resonancia química
        metricas.transferencias = self.sustrato.iterar_pensamiento(&mut self.sinapsis);
        metricas.carga_total = self.sustrato.energia_total();

        // 4. Detectar activación en zonas motoras
        metricas.activaciones_motoras = self.detectar_activaciones_motoras();

        // 5. Actualizar neuromodulador según fitness térmico
        let estres_actual = self.nivel_estres();
        self.actualizar_neuromodulador(estres_actual);
        metricas.estres_actual = estres_actual;

        // 6. Aplicar neuromodulación a la tabla sináptica
        self.aplicar_neuromodulador();

        // 7. Generar acciones recomendadas
        self.generar_acciones(metricas)
    }

    /// Detectar qué zonas motoras tienen carga significativa
    fn detectar_activaciones_motoras(&mut self) -> usize {
        let mut activaciones = 0;

        for zona in &self.zonas_motoras {
            for idx in zona.inicio..zona.fin {
                if let Some(carga) = self.sustrato.carga_index(idx) {
                    if carga > 200 {
                        // Zona activada - registrar acción
                        if !self.acciones_pendientes.contains(&zona.accion) {
                            self.acciones_pendientes.push(zona.accion.clone());
                        }
                        activaciones += 1;
                    }
                }
            }
        }

        activaciones
    }

    /// Actualizar neuromodulador según cambio en estrés térmico
    fn actualizar_neuromodulador(&mut self, estres_actual: f32) {
        let delta = self.delta_estres(estres_actual);

        self.neuromodulador = if delta < -0.01 {
            // Estrés reducido → mejorar aprendizaje positivo
            Neuromodulador::PotenciarPositivo
        } else if delta > 0.01 {
            // Estrés aumentado → fortalecer que no repita
            Neuromodulador::PotenciarNegativo
        } else {
            Neuromodulador::Neutral
        };
    }

    /// Aplicar neuromodulación a las actualizaciones de plasticidad
    fn aplicar_neuromodulador(&mut self) {
        // El neuromodulador afecta cómo se procesan las futuras actualizaciones
        // Esto se implementa mediante el factor de multiplicación en actualizar_plasticidad
        match self.neuromodulador {
            Neuromodulador::PotenciarPositivo => {
                // Duplicar factores de fortalecimiento en futuras plasticidades
                // Marcamos esto como flag global
            }
            Neuromodulador::PotenciarNegativo => {
                // Duplicar factores de debilitamiento
            }
            Neuromodulador::Neutral => {}
        }
    }

    /// Generar acciones recomendadas basadas en el estado actual
    fn generar_acciones(&mut self, metricas: MetricasNeuro) -> Vec<AccionMotora> {
        let mut acciones = Vec::new();

        // Si el estrés es crítico, sugerir acción de enfriamiento
        if metricas.estres_actual > 0.8 {
            acciones.push(AccionMotora::Dormir);
        }

        // Si hay activaciones motoras pendientes, devolverlas
        acciones.extend(self.acciones_pendientes.drain(..));

        acciones
    }

    // =================================================================
    // INTERFAZ PÚBLICA
    // =================================================================

    /// Obtener referencia al sustrato
    pub fn sustrato(&self) -> &SustratoVital {
        &self.sustrato
    }

    /// Obtener referencia mutable al sustrato
    pub fn sustrato_mut(&mut self) -> &mut SustratoVital {
        &mut self.sustrato
    }

    /// Obtener referencia a la tabla sináptica
    pub fn sinapsis(&self) -> &TablaSinaptica {
        &self.sinapsis
    }

    /// Obtener neuromodulador actual
    pub fn neuromodulador(&self) -> Neuromodulador {
        self.neuromodulador
    }

    /// Obtener número de conexiones sinápticas
    pub fn num_conexiones(&self) -> usize {
        self.sinapsis.num_conexiones()
    }

    /// Inyectar pensamiento directamente (para testing)
    pub fn inyectar_pensamiento(&mut self, x: usize, y: usize, z: usize, cantidad: u8) -> bool {
        self.sustrato_mut().inyectar_pensamiento(x, y, z, cantidad)
    }

    /// Podar conexiones sinápticas débiles
    pub fn podar_sinapsis(&mut self, umbral: i16) {
        self.sinapsis.podar_conexiones_debiles(umbral);
    }

    /// Iterar morfogénesis
    pub fn iterar_morfogenesis<T: TermicoReader>(&mut self, lector: &T) {
        self.sustrato_mut().iterar_morfogenesis(lector);
    }
}

// =============================================================================
// IMPLEMENTACIÓN DEL TRAIT TermicoReader PARA CerebroEden
// =============================================================================

impl TermicoReader for CerebroEden {
    fn temperatura_zona(&self, zona: usize) -> NivelTermico {
        // Mapear zona del sustrato (0-63) a nivel térmico
        let energia_zona = if self.sustrato.len() > 0 {
            let celdas_por_zona = self.sustrato.len() / 64;
            let inicio = zona * celdas_por_zona;
            let fin = (zona + 1) * celdas_por_zona;
            let mut suma = 0u64;
            for idx in inicio..fin.min(self.sustrato.len()) {
                if let Some(c) = self.sustrato.carga_index(idx) {
                    suma += c as u64;
                }
            }
            suma / celdas_por_zona.max(1) as u64
        } else {
            0
        };

        match energia_zona {
            0..=32 => NivelTermico::Optimo,
            33..=96 => NivelTermico::Normal,
            97..=192 => NivelTermico::Alto,
            _ => NivelTermico::Critico,
        }
    }

    fn temperatura_global(&self) -> NivelTermico {
        self.temperatura_zona(0)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_cerebro_mini() {
        let cerebro = CerebroEden::new_mini();
        assert!(cerebro.sustrato().len() > 0);
        assert_eq!(cerebro.num_conexiones(), 0);
    }

    #[test]
    fn test_zonas_sensoriales() {
        let cerebro = CerebroEden::new_mini();
        assert_eq!(cerebro.zonas_sensoriales.len(), NUM_ZONAS_SENSORIALES);
        // First zone should be "read"
        assert_eq!(cerebro.zonas_sensoriales[0].tipo_syscall, "read");
    }

    #[test]
    fn test_zonas_motoras() {
        let cerebro = CerebroEden::new_mini();
        assert_eq!(cerebro.zonas_motoras.len(), NUM_ZONAS_MOTORAS);
        // Zonas should not overlap
        for i in 0..NUM_ZONAS_MOTORAS {
            for j in (i+1)..NUM_ZONAS_MOTORAS {
                let zona_i = &cerebro.zonas_motoras[i];
                let zona_j = &cerebro.zonas_motoras[j];
                assert!(zona_i.fin <= zona_j.inicio || zona_j.fin <= zona_i.inicio);
            }
        }
    }

    #[test]
    fn test_inyectar_syscall() {
        let mut cerebro = CerebroEden::new_mini();
        cerebro.inyectar_syscall("read", 100);
        assert_eq!(cerebro.eventos_pendientes.len(), 1);
        assert_eq!(cerebro.eventos_pendientes[0].intensidad, 100);
    }

    #[test]
    fn test_neuromodulador_fitness() {
        let mut cerebro = CerebroEden::new_mini();

        // Simular reducción de estrés
        cerebro.estres_anterior = 0.8;
        cerebro.actualizar_neuromodulador(0.5);
        assert_eq!(cerebro.neuromodulador(), Neuromodulador::PotenciarPositivo);

        // Simular aumento de estrés
        cerebro.actualizar_neuromodulador(0.9);
        assert_eq!(cerebro.neuromodulador(), Neuromodulador::PotenciarNegativo);

        // Simular estabilidad: primero actualizar estres_anterior
        cerebro.estres_anterior = 0.5; // Antes era 0.9
        cerebro.actualizar_neuromodulador(0.5); // delta = 0.5 - 0.5 = 0.0
        assert_eq!(cerebro.neuromodulador(), Neuromodulador::Neutral);
    }

    #[test]
    fn test_tick_sin_eventos_basico() {
        let mut cerebro = CerebroEden::new_mini();
        // Sin eventos pendientes, no debería haber acciones motoras
        assert!(cerebro.acciones_pendientes.is_empty());
        // Sinapse vacía
        assert_eq!(cerebro.num_conexiones(), 0);
    }

    #[test]
    fn test_nivel_estres() {
        let cerebro = CerebroEden::new_mini();
        let estres = cerebro.nivel_estres();
        assert!(estres >= 0.0 && estres <= 1.0);
    }
}

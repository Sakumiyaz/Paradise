//! # Estados Vitales: Estrategias de Supervivencia a Largo Plazo
//!
//! Este módulo implementa los estados vitales de un Auton:
//! - **Activo**: Comportamiento normal (metabolismo par-impar)
//! - **Letargo (Hibernación)**: Estado de baja energía para sobrevivir condiciones adversas
//! - **Espora**: Forma comprimida de supervivencia extrema
//!
//! ## Transiciones de Estado
//!
//! ```text
//! Activo ──────► Letargo ──────► Espora
//!     │              │                │
//!     │              │                │
//!     ◄──────────────┘                │
//!         (mejora condiciones)          │
//!                                       │
//!              ◄────────────────────────┘
//!                   (germinación)
//! ```
//!
//! ## Letargo
//!
//! Se activa cuando:
//! - `energia_interna` < UMBRAL_LETARGO
//! - Densidad local de Escoria > UMBRAL_ESCORIA_LETARGO
//!
//! En letargo:
//! - El Campo Estructural se congela (no resuelve EDP)
//! - El metabolismo se detiene
//! - La Umbra permanece intacta
//! - Consumo mínimo: 1% del costo normal
//!
//! ## Espora
//!
//! Transformación extrema desde letargo:
//! - Campo Estructural compactado en esfera densa
//! - RamNet almacenada comprimida
//! - Inmune a la Escoria
//! - Movimiento pasivo por corrientes del Mar Morfóseo
//!
//! Germinación (Espora → Activo):
//! - Requiere zona con alta Energon y baja Escoria
//! - Mutación masiva (reinicialización parcial de RamNet)
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::life::ramnet::RamNet;

/// Umbral de energía para entrar en letargo (20.0 unidades)
pub const UMBRAL_LETARGO: i64 = 0x00000014_00000000;

/// Umbral de escoria local para entrar en letargo (densidad normalizada 0..1)
pub const UMBRAL_ESCORIA_LETARGO: f64 = 0.6;

/// Umbral de energía muy bajo para esporulación (5.0 unidades)
pub const UMBRAL_ESPORULACION: i64 = 0x00000005_00000000;

/// Umbral de escoria muy alto para esporulación
pub const UMBRAL_ESCORIA_ESPORULACION: f64 = 0.85;

/// Consumo de energía en letargo (1% del normal)
pub const CONSUMO_LETARGO: f64 = 0.01;

/// Ciclos entre verificaciones de condiciones en letargo
pub const CICLOS_VERIFICACION_LETARGO: u32 = 100;

/// Umbral de energía para germinación (30.0 unidades)
pub const UMBRAL_GERMINACION: i64 = 0x0000001E_00000000;

/// Umbral de escoria bajo para germinación
pub const UMBRAL_ESCORIA_GERMINACION: f64 = 0.3;

/// Estados vitales de un Auton
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoVital {
    /// Comportamiento normal
    Activo,
    /// Hibernación para conservar energía
    Letargo,
    /// Espora: forma comprimida de supervivencia extrema
    Espora,
}

impl Default for EstadoVital {
    fn default() -> Self {
        EstadoVital::Activo
    }
}

impl std::fmt::Display for EstadoVital {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EstadoVital::Activo => write!(f, "Activo"),
            EstadoVital::Letargo => write!(f, "Letargo"),
            EstadoVital::Espora => write!(f, "Espora"),
        }
    }
}

/// Datos de un Auton en estado de Letargo
#[derive(Debug, Clone)]
pub struct DatosLetargo {
    /// Ciclos acumulados en letargo
    pub ciclos_en_letargo: u32,
    /// Último tick cuando se verificaron las condiciones
    pub ultimo_tick_verificacion: u64,
    /// Posición congelada del Auton
    pub posicion_congelada: Option<(f64, f64)>,
    /// Energía al entrar en letargo
    pub energia_entrada: i64,
}

impl DatosLetargo {
    /// Crea nuevos datos de letargo
    pub fn nuevo(tick: u64, energia: i64) -> Self {
        DatosLetargo {
            ciclos_en_letargo: 0,
            ultimo_tick_verificacion: tick,
            posicion_congelada: None,
            energia_entrada: energia,
        }
    }

    /// Avanza un ciclo en letargo
    pub fn avanzar_ciclo(&mut self, tick: u64) {
        self.ciclos_en_letargo += 1;
        self.ultimo_tick_verificacion = tick;
    }
}

/// Datos comprimidos de una Espora
#[derive(Debug, Clone)]
pub struct DatosEspora {
    /// Posición actual en el Mar Morfóseo
    pub posicion: (f64, f64),
    /// Energía acumulada mientras es espora
    pub energia_acumulada: i64,
    /// Semilla de la RamNet original (para reconstruir con mutación)
    pub ramnet_semilla: u64,
    /// Hash del campo original para verificación
    pub hash_campo_original: u64,
    /// Ciclos como espora
    pub ciclos_como_espora: u64,
    /// Mutación acumulada (para germinación)
    pub mutacion_acumulada: f64,
}

impl DatosEspora {
    /// Crea datos de espora
    pub fn nuevo(posicion: (f64, f64), energia: i64, ramnet_semilla: u64, hash_campo: u64) -> Self {
        DatosEspora {
            posicion,
            energia_acumulada: energia,
            ramnet_semilla,
            hash_campo_original: hash_campo,
            ciclos_como_espora: 0,
            mutacion_acumulada: 0.0,
        }
    }

    /// Avanza un ciclo como espora
    pub fn avanzar_ciclo(&mut self, energia_ganada: i64) {
        self.ciclos_como_espora += 1;
        self.energia_acumulada += energia_ganada;
        // Acumular mutación pasiva
        self.mutacion_acumulada += 0.01;
    }

    /// Reconstruye la RamNet desde datos almacenados
    /// Nota: La RamNet reconstruida tendrá mutaciones acumuladas
    pub fn reconstruir_ramnet(&self) -> RamNet {
        // Crear nueva RamNet con la semilla + mutación acumulada
        let semilla_mutada = self
            .ramnet_semilla
            .wrapping_add((self.mutacion_acumulada * 1000.0) as u64);
        RamNet::new(8, 2, semilla_mutada)
    }

    /// Verifica si la espora puede germinar
    pub fn puede_germinar(&self, energia_local: i64, escoria_local: f64) -> bool {
        energia_local >= UMBRAL_GERMINACION && escoria_local < UMBRAL_ESCORIA_GERMINACION
    }
}

/// Resultado de procesar los estados vitales
#[derive(Debug, Clone)]
pub struct ResultadoEstadosVitales {
    /// Nuevo estado del Auton
    pub estado: EstadoVital,
    /// Si hubo transición de estado
    pub transicion: Option<TransicionEstado>,
    /// Evento a emitir (si hay)
    pub evento: Option<String>,
}

/// Transición de estado que ocurrió
#[derive(Debug, Clone)]
pub enum TransicionEstado {
    /// Entró en letargo
    EntradaLetargo {
        energia_al_entrar: i64,
        causa: CausaLetargo,
    },
    /// Salió del letargo (volvió a activo)
    SalidaLetargo {
        ciclos_en_letargo: u32,
        energia_al_salir: i64,
    },
    /// Se transformó en espora
    Esporulacion {
        energia_al_entrar: i64,
        causa: CausaEsporulacion,
    },
    /// Germinó (espora → activo)
    Germinacion {
        ciclos_como_espora: u64,
        mutacion_aplicada: f64,
    },
}

/// Causa de entrada en letargo
#[derive(Debug, Clone, Copy)]
pub enum CausaLetargo {
    /// Energía muy baja
    EnergiaBaja,
    /// Escoria local muy alta
    EscoriaAlta,
}

/// Causa de esporulación
#[derive(Debug, Clone, Copy)]
pub enum CausaEsporulacion {
    /// Energía extremadamente baja
    EnergiaExtrema,
    /// Escoria local extremadamente alta
    EscoriaExtrema,
    /// Ambas condiciones extremas
    CondicionesExtrema,
}

/// Evaluación de condiciones para un Auton
#[derive(Debug, Clone)]
pub struct CondicionesAmbientales {
    /// Energía interna actual
    pub energia: i64,
    /// Densidad de escoria local (0..1)
    pub escoria_local: f64,
    /// Energon disponible en la posición
    pub energon_disponible: i64,
}

impl CondicionesAmbientales {
    /// Evalúa si las condiciones son favorables para germinación
    pub fn favorables_germinacion(&self) -> bool {
        self.energia >= UMBRAL_GERMINACION && self.escoria_local < UMBRAL_ESCORIA_GERMINACION
    }

    /// Evalúa si las condiciones son extremas (esporulación)
    pub fn extremas(&self) -> bool {
        self.escoria_local >= UMBRAL_ESCORIA_ESPORULACION || self.energia <= UMBRAL_ESPORULACION
    }

    /// Evalúa si las condiciones justifican letargo
    pub fn justifican_letargo(&self) -> bool {
        self.energia <= UMBRAL_LETARGO || self.escoria_local >= UMBRAL_ESCORIA_LETARGO
    }
}

/// Procesador de estados vitales
pub struct ProcesadorEstadosVitales {
    /// Estado actual
    pub estado_actual: EstadoVital,
    /// Datos de letargo (si está en ese estado)
    pub datos_letargo: Option<DatosLetargo>,
    /// Datos de espora (si está en ese estado)
    pub datos_espora: Option<DatosEspora>,
    /// Último tick procesado
    ultimo_tick: u64,
}

impl ProcesadorEstadosVitales {
    /// Crea un nuevo procesador
    pub fn nuevo() -> Self {
        ProcesadorEstadosVitales {
            estado_actual: EstadoVital::Activo,
            datos_letargo: None,
            datos_espora: None,
            ultimo_tick: 0,
        }
    }

    /// Obtiene el estado actual
    pub fn estado(&self) -> EstadoVital {
        self.estado_actual
    }

    /// Obtiene datos de letargo si existen
    pub fn letargo(&self) -> Option<&DatosLetargo> {
        self.datos_letargo.as_ref()
    }

    /// Obtiene datos de espora si existen
    pub fn espora(&self) -> Option<&DatosEspora> {
        self.datos_espora.as_ref()
    }

    /// Procesa un ciclo de estados vitales
    ///
    /// # Arguments
    /// * `tick` - Tick actual de la simulación
    /// * `condiciones` - Condiciones ambientales actuales
    /// * `metabolismo_activo` - Si el metabolismo normal está activo
    ///
    /// # Returns
    /// Resultado con nuevo estado, posible transición y evento
    pub fn procesar_ciclo(
        &mut self,
        tick: u64,
        condiciones: &CondicionesAmbientales,
        metabolismo_activo: bool,
    ) -> ResultadoEstadosVitales {
        self.ultimo_tick = tick;

        match self.estado_actual {
            EstadoVital::Activo => self.procesar_activo(tick, condiciones),
            EstadoVital::Letargo => self.procesar_letargo(tick, condiciones, metabolismo_activo),
            EstadoVital::Espora => self.procesar_espora(tick, condiciones),
        }
    }

    /// Procesa estado activo
    fn procesar_activo(
        &mut self,
        tick: u64,
        condiciones: &CondicionesAmbientales,
    ) -> ResultadoEstadosVitales {
        // Verificar si debe entrar en letargo
        if condiciones.justifican_letargo() {
            let causa = if condiciones.escoria_local >= UMBRAL_ESCORIA_LETARGO {
                CausaLetargo::EscoriaAlta
            } else {
                CausaLetargo::EnergiaBaja
            };

            self.estado_actual = EstadoVital::Letargo;
            self.datos_letargo = Some(DatosLetargo::nuevo(tick, condiciones.energia));
            self.datos_espora = None;

            let transicion = TransicionEstado::EntradaLetargo {
                energia_al_entrar: condiciones.energia,
                causa,
            };

            return ResultadoEstadosVitales {
                estado: EstadoVital::Letargo,
                transicion: Some(transicion),
                evento: Some(format!(
                    "{{\"tipo\":\"Letargo\",\"id\":{},\"causa\":\"{}\"}}",
                    causa.to_string().to_lowercase(),
                    if matches!(causa, CausaLetargo::EnergiaBaja) {
                        "energia_baja"
                    } else {
                        "escoria_alta"
                    }
                )),
            };
        }

        ResultadoEstadosVitales {
            estado: EstadoVital::Activo,
            transicion: None,
            evento: None,
        }
    }

    /// Procesa estado de letargo
    fn procesar_letargo(
        &mut self,
        tick: u64,
        condiciones: &CondicionesAmbientales,
        _metabolismo_activo: bool,
    ) -> ResultadoEstadosVitales {
        // Tomar ownership del dato de letargo para evitar borrow conflicts
        let mut datos_letargo = match self.datos_letargo.take() {
            Some(d) => d,
            None => {
                // Error: datos de letargo faltantes
                self.estado_actual = EstadoVital::Activo;
                return ResultadoEstadosVitales {
                    estado: EstadoVital::Activo,
                    transicion: None,
                    evento: None,
                };
            }
        };

        // Avanza el ciclo de letargo
        datos_letargo.avanzar_ciclo(tick);

        // Guardar referencia para uso posterior antes de potentially consume
        let ciclos_actuales = datos_letargo.ciclos_en_letargo;
        let _energia_entrada = datos_letargo.energia_entrada;

        // Verificar condiciones cada CICLOS_VERIFICACION_LETARGO ciclos
        // Siempre verificamos si debemos salir del letargo cuando pasamos el umbral
        let verificar_ahora = datos_letargo.ciclos_en_letargo == 1  // Primera verificación tras entrar
            || datos_letargo.ciclos_en_letargo % CICLOS_VERIFICACION_LETARGO == 0;

        if verificar_ahora {
            // Verificar si debe esporular
            if condiciones.extremas() {
                // Devolver el dato antes de transicionar
                self.datos_letargo = Some(datos_letargo);
                return self.transicionar_a_espora(tick, condiciones);
            }

            // Verificar si puede salir del letargo
            if !condiciones.justifican_letargo() {
                let ciclos = ciclos_actuales;
                self.estado_actual = EstadoVital::Activo;
                // No devolvemos datos_letargo porque salimos del letargo

                let transicion = TransicionEstado::SalidaLetargo {
                    ciclos_en_letargo: ciclos,
                    energia_al_salir: condiciones.energia,
                };

                return ResultadoEstadosVitales {
                    estado: EstadoVital::Activo,
                    transicion: Some(transicion),
                    evento: Some(format!("{{\"tipo\":\"LetargoFin\",\"ciclos\":{}}}", ciclos)),
                };
            }
        }

        // Devolver el dato actualizado
        self.datos_letargo = Some(datos_letargo);

        // En letargo no hay metabolismo normal, consumo mínimo
        ResultadoEstadosVitales {
            estado: EstadoVital::Letargo,
            transicion: None,
            evento: None,
        }
    }

    /// Procesa estado de espora
    fn procesar_espora(
        &mut self,
        _tick: u64,
        condiciones: &CondicionesAmbientales,
    ) -> ResultadoEstadosVitales {
        let datos_espora = self.datos_espora.as_mut().unwrap();

        // Avanza ciclo de espora
        datos_espora.avanzar_ciclo(condiciones.energia);

        // Verificar si puede germinar
        if datos_espora.puede_germinar(condiciones.energia, condiciones.escoria_local) {
            self.estado_actual = EstadoVital::Activo;
            let ciclos = datos_espora.ciclos_como_espora;
            let mutacion = datos_espora.mutacion_acumulada;
            self.datos_espora = None;
            self.datos_letargo = None;

            let transicion = TransicionEstado::Germinacion {
                ciclos_como_espora: ciclos,
                mutacion_aplicada: mutacion,
            };

            return ResultadoEstadosVitales {
                estado: EstadoVital::Activo,
                transicion: Some(transicion),
                evento: Some(format!(
                    "{{\"tipo\":\"Germinacion\",\"ciclos\":{},\"mutacion\":{}}}",
                    ciclos, mutacion
                )),
            };
        }

        ResultadoEstadosVitales {
            estado: EstadoVital::Espora,
            transicion: None,
            evento: None,
        }
    }

    /// Transición de letargo a espora
    fn transicionar_a_espora(
        &mut self,
        _tick: u64,
        condiciones: &CondicionesAmbientales,
    ) -> ResultadoEstadosVitales {
        // Determinar causa ANTES de tomar ownership
        let causa_str = if condiciones.escoria_local >= UMBRAL_ESCORIA_ESPORULACION {
            "escoria_extrema"
        } else if condiciones.energia <= UMBRAL_ESPORULACION {
            "energia_extrema"
        } else {
            "condiciones_extremas"
        };

        let causa = if condiciones.escoria_local >= UMBRAL_ESCORIA_ESPORULACION {
            CausaEsporulacion::EscoriaExtrema
        } else if condiciones.energia <= UMBRAL_ESPORULACION {
            CausaEsporulacion::EnergiaExtrema
        } else {
            CausaEsporulacion::CondicionesExtrema
        };

        // Limpiar datos de letargo
        self.datos_letargo = None;

        self.estado_actual = EstadoVital::Espora;
        self.datos_espora = Some(DatosEspora::nuevo(
            (0.5, 0.5), // Posición por defecto, se actualiza desde AutonVivo
            condiciones.energia,
            0, // semilla se actualiza desde AutonVivo
            0, // hash se actualiza desde AutonVivo
        ));

        let transicion = TransicionEstado::Esporulacion {
            energia_al_entrar: condiciones.energia,
            causa,
        };

        ResultadoEstadosVitales {
            estado: EstadoVital::Espora,
            transicion: Some(transicion),
            evento: Some(format!(
                "{{\"tipo\":\"Esporulacion\",\"causa\":\"{}\"}}",
                causa_str
            )),
        }
    }

    /// Inicia germinación (usado desde AutonVivo)
    ///
    /// # Arguments
    /// * `posicion` - Nueva posición al germinar
    /// * `ramnet` - RamNet reconstruida
    /// * `hash_campo` - Hash del campo original
    pub fn iniciar_germinacion(&mut self, posicion: (f64, f64), hash_campo: u64) {
        if let Some(ref mut datos_espora) = self.datos_espora {
            // Actualizar posición y hash antes de germinar
            datos_espora.posicion = posicion;
            datos_espora.hash_campo_original = hash_campo;
        }
    }

    /// Calcula consumo de energía según el estado
    ///
    /// # Arguments
    /// * `consumo_base` - Consumo base del metabolismo normal
    ///
    /// # Returns
    /// Energía a consumir este ciclo
    pub fn calcular_consumo(&self, consumo_base: i64) -> i64 {
        match self.estado_actual {
            EstadoVital::Activo => consumo_base,
            EstadoVital::Letargo => {
                // 1% del consumo normal
                ((consumo_base as f64) * CONSUMO_LETARGO) as i64
            }
            EstadoVital::Espora => {
                // La espora no consume energía activamente
                // pero puede ganar energía pasivamente del Mar
                0
            }
        }
    }
}

impl Default for ProcesadorEstadosVitales {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl CausaLetargo {
    /// Convierte a string para eventos
    pub fn to_string(&self) -> &str {
        match self {
            CausaLetargo::EnergiaBaja => "energia_baja",
            CausaLetargo::EscoriaAlta => "escoria_alta",
        }
    }
}

impl CausaEsporulacion {
    /// Convierte a string para eventos
    pub fn to_string(&self) -> &str {
        match self {
            CausaEsporulacion::EnergiaExtrema => "energia_extrema",
            CausaEsporulacion::EscoriaExtrema => "escoria_extrema",
            CausaEsporulacion::CondicionesExtrema => "condiciones_extremas",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper para crear I32F32 desde valor simple
    fn energia_f32(valor: f64) -> i64 {
        (valor as i64) << 32
    }

    fn crear_condiciones(energia: i64, escoria: f64) -> CondicionesAmbientales {
        CondicionesAmbientales {
            energia,
            escoria_local: escoria,
            energon_disponible: energia,
        }
    }

    #[test]
    fn test_pasivo_permanece_activo() {
        let mut procesador = ProcesadorEstadosVitales::nuevo();
        let condiciones = crear_condiciones(energia_f32(50.0), 0.1); // 50.0 energía, baja escoria

        let resultado = procesador.procesar_ciclo(1, &condiciones, true);

        assert_eq!(resultado.estado, EstadoVital::Activo);
        assert!(resultado.transicion.is_none());
    }

    #[test]
    fn test_entrada_en_letargo_por_energia() {
        let mut procesador = ProcesadorEstadosVitales::nuevo();
        let condiciones = crear_condiciones(energia_f32(15.0), 0.1); // 15.0 energía (bajo umbral)

        let resultado = procesador.procesar_ciclo(1, &condiciones, true);

        assert_eq!(resultado.estado, EstadoVital::Letargo);
        assert!(resultado.evento.is_some());
        assert!(resultado.evento.unwrap().contains("Letargo"));
    }

    #[test]
    fn test_entrada_en_letargo_por_escoria() {
        let mut procesador = ProcesadorEstadosVitales::nuevo();
        let condiciones = crear_condiciones(energia_f32(50.0), 0.7); // Alta escoria

        let resultado = procesador.procesar_ciclo(1, &condiciones, true);

        assert_eq!(resultado.estado, EstadoVital::Letargo);
        assert!(resultado.transicion.is_some());
    }

    #[test]
    fn test_consumo_en_letargo() {
        let mut procesador = ProcesadorEstadosVitales::nuevo();

        // Entrar en letargo
        let condiciones = crear_condiciones(energia_f32(15.0), 0.1);
        procesador.procesar_ciclo(1, &condiciones, true);

        // Verificar consumo mínimo
        let consumo = procesador.calcular_consumo(energia_f32(100.0)); // 100 base
        assert!(consumo < energia_f32(2.0)); // Menos que 2% del consumo
    }

    #[test]
    fn test_salida_de_letargo() {
        let mut procesador = ProcesadorEstadosVitales::nuevo();

        // Entrar en letargo
        let condiciones = crear_condiciones(energia_f32(15.0), 0.1);
        procesador.procesar_ciclo(1, &condiciones, true);
        assert_eq!(procesador.estado(), EstadoVital::Letargo);

        // Mejorar condiciones
        let condiciones_buenas = crear_condiciones(energia_f32(30.0), 0.2);
        let resultado = procesador.procesar_ciclo(
            1 + CICLOS_VERIFICACION_LETARGO as u64,
            &condiciones_buenas,
            true,
        );

        assert_eq!(resultado.estado, EstadoVital::Activo);
        assert!(resultado.evento.is_some());
    }

    #[test]
    fn test_transicion_a_espora() {
        let mut procesador = ProcesadorEstadosVitales::nuevo();

        // Entrar en letargo primero
        let condiciones = crear_condiciones(energia_f32(15.0), 0.1);
        procesador.procesar_ciclo(1, &condiciones, true);

        // Condiciones extremas
        let condiciones_extremas = crear_condiciones(energia_f32(3.0), 0.9);
        let resultado = procesador.procesar_ciclo(
            1 + CICLOS_VERIFICACION_LETARGO as u64,
            &condiciones_extremas,
            true,
        );

        assert_eq!(resultado.estado, EstadoVital::Espora);
        assert!(resultado.evento.is_some());
        assert!(resultado.evento.unwrap().contains("Esporulacion"));
    }

    #[test]
    fn test_germinacion() {
        let mut procesador = ProcesadorEstadosVitales::nuevo();

        // Entrar en espora
        let condiciones = crear_condiciones(energia_f32(15.0), 0.1);
        procesador.procesar_ciclo(1, &condiciones, true);

        let condiciones_extremas = crear_condiciones(energia_f32(3.0), 0.9);
        procesador.procesar_ciclo(
            1 + CICLOS_VERIFICACION_LETARGO as u64,
            &condiciones_extremas,
            true,
        );
        assert_eq!(procesador.estado(), EstadoVital::Espora);

        // Condiciones favorables para germinar
        let condiciones_germinacion = crear_condiciones(energia_f32(35.0), 0.2);
        let resultado = procesador.procesar_ciclo(100, &condiciones_germinacion, true);

        assert_eq!(resultado.estado, EstadoVital::Activo);
        assert!(resultado.evento.is_some());
        assert!(resultado.evento.unwrap().contains("Germinacion"));
    }

    #[test]
    fn test_estado_default() {
        let procesador = ProcesadorEstadosVitales::nuevo();
        assert_eq!(procesador.estado(), EstadoVital::Activo);
    }
}

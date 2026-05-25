//! # Storyteller: Narrativa Generativa del Universo
//!
//! Sistema de generación de texto narrativo sobre el estado del universo EDEN.
//! Usa plantillas generativas con variables del estado actual.
//!
//! ## Tono Emocional
//!
//! El tono se calcula a partir de métricas del universo:
//! - **Épico**: Alta energía, población creciente, alta diversidad
//! - **Melancólico**: Energía declining, población estable o declinando
//! - **Triunfal**: Eventos mayores (Nomos formado, fisión cósmica)
//! - **Preocupado**: Alta escoria, baja energía, riesgo de colapso
//!
//! ## Uso
//!
//! ```rust
//! use eden_core::consciousness::storyteller::Storyteller;
//!
//! let mut storyteller = Storyteller::new();
//! storyteller.actualizar_metrics(42, 12, 750_000_000_000, 0.1, 0.75, 0.4);
//! let narrativa = storyteller.generar_narrativa();
//! assert!(!narrativa.texto.is_empty());
//! ```
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

// ============================================================================
// TIPOS Y ENUMS
// ============================================================================

/// Tono emocional de la narrativa
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TonoEmocional {
    /// Narrativa épica - gran energía y expansión
    Epico,
    /// Narrativa melancólica - declive suave
    Melancolico,
    /// Narrativa triunfal - logros y eventos mayores
    Triunfal,
    /// Narrativa preocupada - crisis inminentes
    Preocupado,
    /// Narrativa neutral - sin emoción particular
    Neutral,
}

impl TonoEmocional {
    /// Describe el tono
    pub fn descripcion(&self) -> &str {
        match self {
            TonoEmocional::Epico => "épico",
            TonoEmocional::Melancolico => "melancólico",
            TonoEmocional::Triunfal => "triunfal",
            TonoEmocional::Preocupado => "preocupado",
            TonoEmocional::Neutral => "neutral",
        }
    }
}

/// Métricas del universo para narrativa
#[derive(Debug, Clone)]
pub struct EstadoNarrativo {
    /// Ciclo actual
    pub ciclo: u64,
    /// Población total de Auton
    pub poblacion: u32,
    /// Número de Auton vivos
    pub autons_vivos: u32,
    /// Número de Auton muertos este ciclo
    pub muertes_ciclo: u32,
    /// Energía total del universo
    pub energia_total: i64,
    /// Escoria total
    pub escoria_total: f64,
    /// Diversidad de RamNet (0-1)
    pub diversidad: f64,
    /// Entropía de memes (0-1)
    pub entropia_memes: f64,
    /// Tasa de esporulación actual
    pub tasa_esporulacion: f64,
    /// ID del Auton más antiguo
    pub auton_mas_antiguo: Option<u64>,
    /// Edad del Auton más antiguo (ciclos)
    pub edad_maxima: u64,
    /// Cuadrante con más población
    pub cuadrante_dominante: String,
    /// Zona con más muertes
    pub zona_mas_muertes: String,
    /// Meme más popular
    pub meme_mas_popular: Option<String>,
    /// Ciclos desde último Nomos
    pub ciclos_desde_nomos: u64,
    /// Cantidad de energía inyectada recientemente
    pub energia_inyectada_reciente: i64,
    /// Indica si hay crisis activa
    pub crisis_activa: bool,
}

impl Default for EstadoNarrativo {
    fn default() -> Self {
        Self {
            ciclo: 0,
            poblacion: 0,
            autons_vivos: 0,
            muertes_ciclo: 0,
            energia_total: 0,
            escoria_total: 0.0,
            diversidad: 0.0,
            entropia_memes: 0.0,
            tasa_esporulacion: 0.0,
            auton_mas_antiguo: None,
            edad_maxima: 0,
            cuadrante_dominante: "Noroeste".to_string(),
            zona_mas_muertes: "Ninguna".to_string(),
            meme_mas_popular: None,
            ciclos_desde_nomos: u64::MAX,
            energia_inyectada_reciente: 0,
            crisis_activa: false,
        }
    }
}

/// Resultado de la generación narrativa
#[derive(Debug, Clone)]
pub struct Narrativa {
    /// Texto generado
    pub texto: String,
    /// Tono emocional usado
    pub tono: TonoEmocional,
    /// Ciclo de la narración
    pub ciclo: u64,
    /// Métricas usadas
    pub metricas: EstadoNarrativo,
}

// ============================================================================
// PLANTILLAS
// ============================================================================

/// Plantillas narrativas por tono
struct PlantillasNarrativa {
    /// Plantillas épicas
    epicas: Vec<&'static str>,
    /// Plantillas melancólicas
    melancolicas: Vec<&'static str>,
    /// Plantillas triunfales
    triunales: Vec<&'static str>,
    /// Plantillas preocupadas
    preocupantes: Vec<&'static str>,
    /// Plantillas neutrales
    neutrales: Vec<&'static str>,
}

impl PlantillasNarrativa {
    fn new() -> Self {
        Self {
            epicas: vec![
                "En el ciclo {ciclo}, el cosmos late con fuerza vital. {autons_vivos} Auton danzan en el Mar Morfóseo, sus formas entrelazándose en patrones de belleza imposible.",
                "El universo EDEN respira con amplitude. {autons_vivos} seres carbónicos trazan constelaciones de información en el espacio onírico.",
                "Una sinfonía de existencia se despliega ante nuestros ojos. {autons_vivos} Auton, cada uno portando {diversidad} de diversidad genética, pululan en {cuadrante_dominante}.",
                "Los fuegos de la creación arden brillantes. La energía fluye ({energia_total} unidades) mientras {meme_mas_popular} resuena como un cántico ancestral.",
                "El Mar Morfóseo se ondula con propósito. {autons_vivos} Auton danzan el baile eterno de la autopoiesis.",
            ],
            melancolicas: vec![
                "En el ciclo {ciclo}, una calma reflexiva envuelve al universo. {autons_vivos} Auton permanecen, algunos partiendo en silencio hacia el sueño eterno.",
                "Las aguas del Mar Morfóseo descansan tranquilas. {autons_vivos} Auton guardan la memoria de {edad_maxima} ciclos de existencia.",
                "El tiempo fluye suave aquí. {muertes_ciclo} Auton han encontrado paz este ciclo, pero la vida persiste en sus {autons_vivos} descendientes.",
                "Una luz tenue ilumina el cosmos. La escoria se acumula ({escoria_total}), pero {autons_vivos} Auton continúan su vigilia eterna.",
                "Los ecos del pasado resuenan en el vacío. El linaje más antiguo, {auton_mas_antiguo}, ha perdurado por {edad_maxima} ciclos.",
            ],
            triunales: vec![
                "¡UN MOMENTO PARA LA HISTORIA! En el ciclo {ciclo}, {evento_especial}. {autons_vivos} Auton son testigos de este instante eterno.",
                "Las estrellas cantan en el cosmos. {evento_especial}. La humanidad primordial ha dado un paso de gigante.",
                "¡ALELUYA! El universo EDEN celebra: {evento_especial}. Los {autons_vivos} Auton danzan de alegría.",
                "Un Nomos ha surgido de las profundidades del Mar Morfóseo. El patrón {meme_mas_popular} se materializa ante {autons_vivos} Auton expectantes.",
                "La fisión cósmica ha ocurrido. El linaje de {auton_mas_antiguo} se ha duplicado, extendiendo la vida por el cuadrante {cuadrante_dominante}.",
            ],
            preocupantes: vec![
                "¡ALERTA! En el ciclo {ciclo}, {zonas_criticas}. La escoria avanza ({escoria_total}) y solo {autons_vivos} Auton permanecen para resistir.",
                "Tormentas de información azotan el Mar Morfóseo. {muertes_ciclo} Auton han caído en el último ciclo. El universo tiembla.",
                "Los indicadores muestran preocupación. Energía en declive ({energia_total}), escoria en aumento ({escoria_total}). {autons_vivos} Auton libran una batalla silenciosa.",
                "El cisne negro se cierne. La diversidad mengua ({diversidad}) mientras {zona_mas_muertes} se convierte en un cementerio de memorias.",
                "El tiempo apremia. El linaje más antiguo ({auton_mas_antiguo}) lucha por sobrevivir. {ciclos_desde_nomos} ciclos sin un nuevo Nomos.",
            ],
            neutrales: vec![
                "Ciclo {ciclo}. {autons_vivos} Auton existen actualmente. Energía: {energia_total}. Escoria: {escoria_total}.",
                "El universo continua su operación. {meme_mas_popular} domina el paisaje cognitivo. {cuadrante_dominante} es el hogar de la mayoría.",
                "Estado actual: {autons_vivos} Auton vivos, {muertes_ciclo} muertes este ciclo, diversidad de {diversidad}.",
                "El Mar Morfóseo permanece en estado {estado_mar}. {tasa_esporulacion} tasa de esporulación observada.",
                "Reporte del ciclo {ciclo}: Población estable en {autons_vivos}. {zona_mas_muertes} zona de mayor mortalidad.",
            ],
        }
    }

    fn obtener(&self, tono: &TonoEmocional) -> &[&'static str] {
        match tono {
            TonoEmocional::Epico => &self.epicas,
            TonoEmocional::Melancolico => &self.melancolicas,
            TonoEmocional::Triunfal => &self.triunales,
            TonoEmocional::Preocupado => &self.preocupantes,
            TonoEmocional::Neutral => &self.neutrales,
        }
    }
}

/// Eventos especiales que cambian la narrativa
#[derive(Debug, Clone)]
pub enum EventoEspecial {
    /// Se formó un Nomos
    NomosFormado(String),
    /// Fisión cósmica ocurrió
    FisionCosmica,
    /// Explosión demográfica
    ExplosionDemografica,
    /// Extinción masiva
    ExtincionMasiva,
    /// Primer contacto entre linajes
    PrimerContacto,
    /// Desarrollo de nuevo meme
    NuevoMeme(String),
}

impl EventoEspecial {
    fn descripcion(&self) -> String {
        match self {
            EventoEspecial::NomosFormado(tipo) => {
                format!("Un Nomos de tipo {} ha emergido de las profundidades", tipo)
            }
            EventoEspecial::FisionCosmica => {
                "la fisión cósmica ha dividido un Auton en dos".to_string()
            }
            EventoEspecial::ExplosionDemografica => {
                "una explosión demográfica ha duplicado la población".to_string()
            }
            EventoEspecial::ExtincionMasiva => {
                "una extinción masiva ha golpeado al universo".to_string()
            }
            EventoEspecial::PrimerContacto => {
                "dos linajes previamente aislados han entrado en contacto".to_string()
            }
            EventoEspecial::NuevoMeme(meme) => {
                format!("el meme '{}' se ha propagado como wildfire", meme)
            }
        }
    }
}

// ============================================================================
// STORYTELLER MANAGER
// ============================================================================

/// Generador de narrativa del universo
pub struct Storyteller {
    /// Estado narrativo actual
    estado: EstadoNarrativo,
    /// Plantillas disponibles
    plantillas: PlantillasNarrativa,
    /// Último tono usado
    ultimo_tono: TonoEmocional,
    /// Historial de narrativas (últimas 10)
    historial: VecDeque<Narrativa>,
    /// Contador para selección pseudo-aleatoria de plantillas
    contador_plantilla: usize,
    /// Evento especial activo (None si no hay)
    evento_especial: Option<EventoEspecial>,
    /// Socket para envío de narrativas
    socket: Option<Arc<RwLock<crate::ipc::socket::UnixDatagram>>>,
}

impl Storyteller {
    /// Crea un nuevo Storyteller
    pub fn new() -> Self {
        Self {
            estado: EstadoNarrativo::default(),
            plantillas: PlantillasNarrativa::new(),
            ultimo_tono: TonoEmocional::Neutral,
            historial: VecDeque::with_capacity(10),
            contador_plantilla: 0,
            evento_especial: None,
            socket: None,
        }
    }

    /// Configura el socket para envío de narrativas
    pub fn con_socket(mut self, socket: Arc<RwLock<crate::ipc::socket::UnixDatagram>>) -> Self {
        self.socket = Some(socket);
        self
    }

    /// Actualiza el estado narrativo
    pub fn actualizar_estado(&mut self, estado: EstadoNarrativo) {
        self.estado = estado;
        self.ultimo_tono = self.calcular_tono();
    }

    /// Actualiza desde métricas simples
    pub fn actualizar_metrics(
        &mut self,
        ciclo: u64,
        poblacion: u32,
        energia_total: i64,
        escoria_total: f64,
        diversidad: f64,
        entropia_memes: f64,
    ) {
        self.estado.ciclo = ciclo;
        self.estado.poblacion = poblacion;
        self.estado.autons_vivos = poblacion;
        self.estado.energia_total = energia_total;
        self.estado.escoria_total = escoria_total;
        self.estado.diversidad = diversidad;
        self.estado.entropia_memes = entropia_memes;
        self.ultimo_tono = self.calcular_tono();
    }

    /// Establece un evento especial
    pub fn establecer_evento(&mut self, evento: EventoEspecial) {
        self.evento_especial = Some(evento);
        // Forzar tono triunfal para eventos especiales
        self.ultimo_tono = TonoEmocional::Triunfal;
    }

    /// Calcula el tono emocional basado en el estado
    fn calcular_tono(&self) -> TonoEmocional {
        let e = &self.estado;

        // Verificar condiciones de crisis
        if e.crisis_activa
            || e.escoria_total > 0.7
            || (e.energia_total < 100_000_000_000 && e.autons_vivos < 20)
        {
            return TonoEmocional::Preocupado;
        }

        // Verificar eventos especiales primero
        if self.evento_especial.is_some() {
            return TonoEmocional::Triunfal;
        }

        // Calcular métricas de salud
        let energia_saludable = e.energia_total > 500_000_000_000;
        let poblacion_creciendo = e.autons_vivos > 80;
        let diversidad_alta = e.diversidad > 0.6;

        // Épico: alta energía, población buena, alta diversidad
        if energia_saludable && poblacion_creciendo && diversidad_alta {
            return TonoEmocional::Epico;
        }

        // Melancolico: energía o población en declive suave
        if e.energia_total < 300_000_000_000 || e.autons_vivos < 50 {
            return TonoEmocional::Melancolico;
        }

        // Neutral: todo está regular
        TonoEmocional::Neutral
    }

    /// Reemplaza variables en una plantilla
    fn render_plantilla(&self, plantilla: &str) -> String {
        let e = &self.estado;

        let texto = plantilla
            .replace("{ciclo}", &e.ciclo.to_string())
            .replace("{autons_vivos}", &e.autons_vivos.to_string())
            .replace("{muertes_ciclo}", &e.muertes_ciclo.to_string())
            .replace("{energia_total}", &Self::formato_numero(e.energia_total))
            .replace("{escoria_total}", &format!("{:.2}", e.escoria_total))
            .replace("{diversidad}", &format!("{:.1}%", e.diversidad * 100.0))
            .replace("{cuadrante_dominante}", &e.cuadrante_dominante)
            .replace("{zona_mas_muertes}", &e.zona_mas_muertes)
            .replace(
                "{auton_mas_antiguo}",
                &e.auton_mas_antiguo
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "desconocido".to_string()),
            )
            .replace("{edad_maxima}", &e.edad_maxima.to_string())
            .replace(
                "{meme_mas_popular}",
                &e.meme_mas_popular
                    .clone()
                    .unwrap_or_else(|| "el silencio".to_string()),
            )
            .replace("{ciclos_desde_nomos}", &e.ciclos_desde_nomos.to_string())
            .replace(
                "{tasa_esporulacion}",
                &format!("{:.3}", e.tasa_esporulacion),
            );

        // Reemplazar evento especial si existe
        if let Some(ref evento) = self.evento_especial {
            texto.replace("{evento_especial}", &evento.descripcion())
        } else if texto.contains("{evento_especial}") {
            texto.replace("{evento_especial}", "el universo continúa su eterno baile")
        } else {
            texto
        }
    }

    /// Formatea un número grande para legibilidad
    fn formato_numero(n: i64) -> String {
        if n >= 1_000_000_000_000 {
            format!("{:.1}T", n as f64 / 1_000_000_000_000.0)
        } else if n >= 1_000_000_000 {
            format!("{:.1}B", n as f64 / 1_000_000_000.0)
        } else if n >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else {
            n.to_string()
        }
    }

    /// Genera una narrativa
    pub fn generar_narrativa(&mut self) -> Narrativa {
        let plantillas = self.plantillas.obtener(&self.ultimo_tono);

        // Seleccionar plantilla (rotación simple)
        let idx = self.contador_plantilla % plantillas.len();
        self.contador_plantilla += 1;

        let plantilla = plantillas[idx];
        let texto = self.render_plantilla(plantilla);

        let narrativa = Narrativa {
            texto: texto.clone(),
            tono: self.ultimo_tono,
            ciclo: self.estado.ciclo,
            metricas: self.estado.clone(),
        };

        // Guardar en historial
        if self.historial.len() >= 10 {
            self.historial.pop_front();
        }
        self.historial.push_back(narrativa.clone());

        // Limpiar evento especial después de usarlo
        self.evento_especial = None;

        // Enviar por socket si está configurado
        self.enviar_narrativa(&texto);

        narrativa
    }

    /// Genera narrativa breve (una línea)
    pub fn generar_resumen(&self) -> String {
        let e = &self.estado;
        format!(
            "[Ciclo {}] {} Auton | Energía: {} | Escoria: {:.1}% | Tono: {}",
            e.ciclo,
            e.autons_vivos,
            Self::formato_numero(e.energia_total),
            e.escoria_total * 100.0,
            self.ultimo_tono.descripcion()
        )
    }

    /// Envía narrativa por socket
    fn enviar_narrativa(&self, texto: &str) {
        if let Some(ref socket) = self.socket {
            if let Ok(socket) = socket.read() {
                let msg = format!("NARRATIVA: {}", texto);
                let _ = socket.send(msg.as_bytes());
            }
        }
    }

    /// Obtiene el historial de narrativas
    pub fn historial(&self) -> Vec<Narrativa> {
        self.historial.iter().cloned().collect()
    }

    /// Obtiene el tono actual
    pub fn tono_actual(&self) -> TonoEmocional {
        self.ultimo_tono
    }

    /// Obtiene el estado narrativo actual
    pub fn estado(&self) -> &EstadoNarrativo {
        &self.estado
    }

    /// Fuerza el tono (para testing o demos)
    pub fn forzar_tono(&mut self, tono: TonoEmocional) {
        self.ultimo_tono = tono;
    }
}

impl Default for Storyteller {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper thread-safe
pub type StorytellerLocked = Arc<RwLock<Storyteller>>;

impl Storyteller {
    pub fn into_locked(self) -> StorytellerLocked {
        Arc::new(RwLock::new(self))
    }
}

// ============================================================================
// SERIALIZACIÓN
// ============================================================================

impl Narrativa {
    /// Serializa a JSON
    pub fn a_json(&self) -> String {
        format!(
            r#"{{"tipo":"narrativa","texto":"{}","tono":"{}","ciclo":{}}}"#,
            self.texto.replace('"', "\\\"").replace('\n', "\\n"),
            self.tono.descripcion(),
            self.ciclo
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
    fn test_crear_storyteller() {
        let s = Storyteller::new();
        assert_eq!(s.tono_actual(), TonoEmocional::Neutral);
    }

    #[test]
    fn test_calculo_tono_epico() {
        let mut s = Storyteller::new();
        s.actualizar_metrics(1000, 100, 1_000_000_000_000, 0.1, 0.7, 0.5);
        assert_eq!(s.tono_actual(), TonoEmocional::Epico);
    }

    #[test]
    fn test_calculo_tono_preocupado() {
        let mut s = Storyteller::new();
        s.actualizar_estado(EstadoNarrativo {
            ciclo: 1000,
            autons_vivos: 15,
            energia_total: 50_000_000_000,
            escoria_total: 0.8,
            diversidad: 0.3,
            crisis_activa: true,
            ..Default::default()
        });
        assert_eq!(s.tono_actual(), TonoEmocional::Preocupado);
    }

    #[test]
    fn test_generar_narrativa() {
        let mut s = Storyteller::new();
        s.actualizar_metrics(500, 75, 800_000_000_000, 0.3, 0.5, 0.4);

        let narrativa = s.generar_narrativa();
        assert!(narrativa.texto.contains("500"));
        assert!(narrativa.texto.contains("75"));
    }

    #[test]
    fn test_resumen() {
        let mut s = Storyteller::new();
        s.actualizar_metrics(200, 50, 500_000_000_000, 0.4, 0.5, 0.3);

        let resumen = s.generar_resumen();
        assert!(resumen.contains("200"));
        assert!(resumen.contains("50"));
    }

    #[test]
    fn test_evento_especial() {
        let mut s = Storyteller::new();
        s.actualizar_metrics(
            100,
            81, // > 80 threshold for Epico
            900_000_000_000,
            0.2,
            0.61, // > 0.6 threshold for Epico
            0.5,
        );

        // Sin evento especial, debe ser épico
        assert_eq!(s.tono_actual(), TonoEmocional::Epico);

        // Con evento especial, debe ser triunfal
        s.establecer_evento(EventoEspecial::NomosFormado("Silencio".to_string()));
        assert_eq!(s.tono_actual(), TonoEmocional::Triunfal);
    }

    #[test]
    fn test_formato_numero() {
        assert_eq!(Storyteller::formato_numero(1_500_000_000_000), "1.5T");
        assert_eq!(Storyteller::formato_numero(800_000_000), "800.0M");
        assert_eq!(Storyteller::formato_numero(2_500_000), "2.5M");
        assert_eq!(Storyteller::formato_numero(1000), "1000");
    }

    #[test]
    fn test_serializacion_json() {
        let narrativa = Narrativa {
            texto: "Test narrativa".to_string(),
            tono: TonoEmocional::Epico,
            ciclo: 100,
            metricas: EstadoNarrativo::default(),
        };

        let json = narrativa.a_json();
        assert!(json.contains("narrativa"));
        assert!(json.contains("Test narrativa"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_historial() {
        let mut s = Storyteller::new();
        s.actualizar_metrics(1, 50, 500_000_000_000, 0.3, 0.5, 0.4);

        for _ in 0..5 {
            s.generar_narrativa();
        }

        assert_eq!(s.historial().len(), 5);
    }

    #[test]
    fn test_todos_los_tonos() {
        let tonos = [
            (TonoEmocional::Epico, 1_000_000_000_000, 100, 0.1, 0.7),
            (TonoEmocional::Melancolico, 200_000_000_000, 40, 0.3, 0.4),
            (TonoEmocional::Preocupado, 50_000_000_000, 10, 0.8, 0.2),
            (TonoEmocional::Neutral, 500_000_000_000, 60, 0.4, 0.5),
        ];

        for (tono_esperado, energia, pob, escoria, div) in tonos {
            let mut s = Storyteller::new();
            s.actualizar_metrics(100, pob as u32, energia, escoria, div, 0.5);
            let tono = s.tono_actual();
            assert_eq!(
                tono, tono_esperado,
                "Para energia={}, pob={}, expected {:?}",
                energia, pob, tono_esperado
            );
        }
    }
}

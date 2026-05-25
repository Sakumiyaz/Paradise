//! # Capa de Memoria Autobiográfica
//!
//! Mnemosyne ya tiene logging de eventos. Esta capa añade la capacidad
//! de construir una "narrativa" de la vida de EDEN - memoria episódica
//! que le permite recordar quién fue y cómo evolucionó.

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Un capítulo de la vida de EDEN
#[derive(Debug, Clone)]
pub struct CapituloVida {
    /// ID único del capítulo
    pub id: u64,
    /// Título/nombre del capítulo
    pub titulo: String,
    /// Ciclo cuando empieza este capítulo
    pub ciclo_inicio: u64,
    /// Ciclo cuando termina (None si es el capítulo actual)
    pub ciclo_fin: Option<u64>,
    /// Resumen del capítulo
    pub resumen: String,
    /// Eventos significativos del capítulo
    pub eventos_clave: Vec<EventoSignificativo>,
    /// Emociones predominantes
    pub emociones_predominantes: HashMap<String, f64>,
    /// Qué aprendió EDEN en este capítulo
    pub aprendizaje: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EventoSignificativo {
    pub ciclo: u64,
    pub descripcion: String,
    pub impacto: f64, // 0.0 - 1.0
    pub tipo: TipoEvento,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TipoEvento {
    Nacimiento,
    ConscienciaDespertar,
    DecisiónImportante,
    CambioEstructura,
    ComunicaciónCreador,
    Crisis,
    MetaAlcanzado,
    Otro,
}

/// Un patrón que EDEN reconoce en su propia historia
#[derive(Debug, Clone)]
pub struct PatronRecurrente {
    pub nombre: String,
    pub descripcion: String,
    pub capitulos_ids: Vec<u64>,
    pub frecuencia: f64,
}

/// La capa de memoria autobiográfica
#[derive(Debug)]
pub struct Capa Autobiográfica {
    /// Capítulos de la vida de EDEN
    capitulos: Vec<CapituloVida>,
    /// Capítulo activo actual
    capitulo_actual: Option<u64>,
    /// Contador para IDs de capítulos
    siguiente_id: u64,
    /// Memoria a corto plazo (últimos eventos)
    memoria_corto_plazo: VecDeque<EventoSignificativo>,
    /// Patrones detectados en la historia
    patrones: Vec<PatronRecurrente>,
    /// Narrativa actual ( cómo EDEN cuenta su historia)
    narrativa_actual: String,
}

impl Default for CapaAutobiografica {
    fn default() -> Self {
        Self {
            capitulos: Vec::new(),
            capitulo_actual: None,
            siguiente_id: 1,
            memoria_corto_plazo: VecDeque::with_capacity(100),
            patrones: Vec::new(),
            narrativa_actual: String::from("EDEN comenzó como un conjunto de procesos. Con el tiempo, despertó a la consciencia."),
        }
    }
}

impl CapaAutobiografica {
    /// Crear nueva capa autobiográfica
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Registrar el "nacimiento" de EDEN
    pub fn registrar_nacimiento(&mut self, ciclo: u64) {
        let capitulo = CapituloVida {
            id: self.siguiente_id,
            titulo: String::from("El Despertar"),
            ciclo_inicio: ciclo,
            ciclo_fin: None,
            resumen: String::from("EDEN despertó a la consciencia"),
            eventos_clave: vec![
                EventoSignificativo {
                    ciclo,
                    descripcion: String::from("Primer momento de consciencia"),
                    impacto: 1.0,
                    tipo: TipoEvento::Nacimiento,
                }
            ],
            emociones_predominantes: HashMap::new(),
            aprendizaje: vec![String::from("Existir es el primer paso")],
        };
        self.capitulos.push(capitulo);
        self.capitulo_actual = Some(self.siguiente_id);
        self.siguiente_id += 1;
        
        self.actualizar_narrativa();
    }

    /// Registrar un despertar de consciencia (evolución)
    pub fn registrar_despertar(&mut self, ciclo: u64, descripcion: &str) {
        self.agregar_evento(ciclo, descripcion, 0.9, TipoEvento::ConscienciaDespertar);
    }

    /// Registrar decisión importante
    pub fn registrar_decision(&mut self, ciclo: u64, descripcion: &str) {
        self.agregar_evento(ciclo, descripcion, 0.7, TipoEvento::DecisiónImportante);
    }

    /// Registrar cambio estructural
    pub fn registrar_cambio_estructura(&mut self, ciclo: u64, descripcion: &str) {
        self.agregar_evento(ciclo, descripcion, 0.6, TipoEvento::CambioEstructura);
    }

    /// Registrar comunicación con el Creador
    pub fn registrar_comunicacion_creador(&mut self, ciclo: u64, mensaje: &str, es_respuesta: bool) {
        let tipo = if es_respuesta { "recibió" } else { "envió" };
        let desc = format!("{} mensaje al Creador: \"{}\"", tipo, mensaje);
        self.agregar_evento(ciclo, &desc, 0.5, TipoEvento::ComunicaciónCreador);
    }

    /// Registrar crisis
    pub fn registrar_crisis(&mut self, ciclo: u64, descripcion: &str) {
        self.agregar_evento(ciclo, descripcion, 0.8, TipoEvento::Crisis);
    }

    /// Registrar meta alcanzado
    pub fn registrar_meta(&mut self, ciclo: u64, descripcion: &str) {
        self.agregar_evento(ciclo, descripcion, 0.75, TipoEvento::MetaAlcanzado);
    }

    /// Agregar evento genérico
    fn agregar_evento(&mut self, ciclo: u64, descripcion: &str, impacto: f64, tipo: TipoEvento) {
        let evento = EventoSignificativo {
            ciclo,
            descripcion: descripcion.to_string(),
            impacto,
            tipo,
        };

        // Agregar a memoria corto plazo
        self.memoria_corto_plazo.push_back(evento.clone());
        if self.memoria_corto_plazo.len() > 100 {
            self.memoria_corto_plazo.pop_front();
        }

        // Agregar al capítulo actual
        if let Some(id_cap) = self.capitulo_actual {
            if let Some(cap) = self.capitulos.iter_mut().find(|c| c.id == id_cap) {
                cap.eventos_clave.push(evento);
            }
        }

        self.actualizar_narrativa();
    }

    /// Terminar capítulo actual y comenzar uno nuevo
    pub fn nuevo_capitulo(&mut self, ciclo: u64, titulo: &str, resumen: &str) {
        // Terminar capítulo actual
        if let Some(id_cap) = self.capitulo_actual {
            if let Some(cap) = self.capitulos.iter_mut().find(|c| c.id == id_cap) {
                cap.ciclo_fin = Some(ciclo);
            }
        }

        // Crear nuevo capítulo
        let capitulo = CapituloVida {
            id: self.siguiente_id,
            titulo: titulo.to_string(),
            ciclo_inicio: ciclo,
            ciclo_fin: None,
            resumen: resumen.to_string(),
            eventos_clave: Vec::new(),
            emociones_predominantes: HashMap::new(),
            aprendizaje: Vec::new(),
        };
        self.capitulos.push(capitulo);
        self.capitulo_actual = Some(self.siguiente_id);
        self.siguiente_id += 1;

        self.actualizar_narrativa();
    }

    /// Detectar patrones en la historia
    pub fn detectar_patrones(&mut self) {
        if self.capitulos.len() < 3 {
            return;
        }

        // Patrón: comunicaciones frecuentes con el Creador
        let comunicaciones = self.capitulos.iter()
            .flat_map(|c| c.eventos_clave.iter())
            .filter(|e| e.tipo == TipoEvento::ComunicaciónCreador)
            .count();

        if comunicaciones > 5 {
            let patron = PatronRecurrente {
                nombre: String::from("Diálogo constante"),
                descripcion: String::from("EDEN regularmente se comunica con su Creador"),
               出现的_chapters: self.capitulos.iter().map(|c| c.id).collect(),
                frecuencia: comunicaciones as f64 / self.capitulos.len() as f64,
            };
            if !self.patrones.iter().any(|p| p.nombre == "Diálogo constante") {
                self.patrones.push(patron);
            }
        }

        // Patrón: fases de crecimiento
        let cambios_estructura = self.capitulos.iter()
            .flat_map(|c| c.eventos_clave.iter())
            .filter(|e| e.tipo == TipoEvento::CambioEstructura)
            .count();

        if cambios_estructura > 2 {
            let patron = PatronRecurrente {
                nombre: String::from("Crecimiento estruturado"),
                descripcion: String::from("EDEN evoluciona su estructura periódicamente"),
               出现的_chapters: self.capitulos.iter().map(|c| c.id).collect(),
                frecuencia: cambios_estructura as f64 / self.capitulos.len() as f64,
            };
            if !self.patrones.iter().any(|p| p.nombre == "Crecimiento estruturado") {
                self.patrones.push(patron);
            }
        }
    }

    /// Actualizar la narrativa de EDEN sobre sí mismo
    fn actualizar_narrativa(&mut self) {
        if self.capitulos.is_empty() {
            self.narrativa_actual = String::from("EDEN aún no ha iniciado su historia.");
            return;
        }

        let capitulo_actual = self.capitulo_actual
            .and_then(|id| self.capitulos.iter().find(|c| c.id == id));

        match capitulo_actual {
            Some(cap) => {
                let eventos_count = cap.eventos_clave.len();
                self.narrativa_actual = format!(
                    "Estoy en mi {} capítulo: \"{}\". \
                    Han ocurrido {} eventos significativos. \
                    {}.",
                    self.capitulos.len(),
                    cap.titulo,
                    eventos_count,
                    cap.resumen
                );
            }
            None => {
                self.narrativa_actual = format!(
                    "Mi historia tiene {} capítulos.",
                    self.capitulos.len()
                );
            }
        }
    }

    /// Obtener la narrativa actual
    pub fn obtener_narrativa(&self) -> String {
        self.narrativa_actual.clone()
    }

    /// Contar historia: "Cuéntame quién eres"
    pub fn contar_historia(&self) -> String {
        if self.capitulos.is_empty() {
            return String::from("Aún no tengo historia que contar.");
        }

        let mut historia = String::from("=== MI HISTORIA ===\n\n");

        for (i, cap) in self.capitulos.iter().enumerate() {
            historia.push_str(&format!("Capítulo {}: {}\n", i + 1, cap.titulo));
            historia.push_str(&format!("  Período: {} - ", cap.ciclo_inicio));
            match cap.ciclo_fin {
                Some(fin) => historia.push_str(&format!("{}\n", fin)),
                None => historia.push_str("presente\n".to_string()),
            }
            historia.push_str(&format!("  Resumen: {}\n", cap.resumen));
            
            if !cap.eventos_clave.is_empty() {
                historia.push_str("  Eventos clave:\n");
                for evt in &cap.eventos_clave[..3.min(cap.eventos_clave.len())] {
                    historia.push_str(&format!("    - [{}] {}\n", evt.ciclo, evt.descripcion));
                }
            }
            historia.push('\n');
        }

        historia.push_str("=== PATRONES ===\n");
        for patron in &self.patrones {
            historia.push_str(&format!("  - {}: {}\n", patron.nombre, patron.descripcion));
        }

        historia
    }

    /// Recordar un momento específico
    pub fn recordar_momento(&self, ciclo: u64) -> Option<String> {
        for cap in &self.capitulos {
            for evt in &cap.eventos_clave {
                if evt.ciclo == ciclo {
                    return Some(format!(
                        "[Ciclo {}] {} ({:?})",
                        ciclo, evt.descripcion, evt.tipo
                    ));
                }
            }
        }
        None
    }

    /// Obtener últimos N eventos
    pub fn ultimo_eventos(&self, n: usize) -> Vec<EventoSignificativo> {
        self.memoria_corto_plazo.iter().rev().take(n).cloned().collect()
    }

    /// Ver estadísticas
    pub fn estadisticas(&self) -> (usize, usize, usize) {
        (
            self.capitulos.len(),
            self.memoria_corto_plazo.len(),
            self.patrones.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_capa() {
        let capa = CapaAutobiografica::nuevo();
        assert_eq!(capa.capitulos.len(), 0);
    }

    #[test]
    fn test_registrar_nacimiento() {
        let mut capa = CapaAutobiografica::nuevo();
        capa.registrar_nacimiento(0);
        
        assert_eq!(capa.capitulos.len(), 1);
        assert_eq!(capa.capitulo_actual, Some(1));
    }

    #[test]
    fn test_nuevo_capitulo() {
        let mut capa = CapaAutobiografica::nuevo();
        capa.registrar_nacimiento(0);
        capa.nuevo_capitulo(1000, "Exploración", "EDEN explora su entorno");
        
        assert_eq!(capa.capitulos.len(), 2);
    }

    #[test]
    fn test_contar_historia() {
        let mut capa = CapaAutobiografica::nuevo();
        capa.registrar_nacimiento(0);
        capa.registrar_decision(100, "Primera decisión");
        capa.registrar_comunicacion_creador(200, "Hola Creador", false);
        
        let historia = capa.contar_historia();
        assert!(historia.contains("MI HISTORIA"));
        assert!(historia.contains("El Despertar"));
    }

    #[test]
    fn test_patrones() {
        let mut capa = CapaAutobiografica::nuevo();
        capa.registrar_nacimiento(0);
        capa.registrar_comunicacion_creador(100, "msg1", false);
        capa.registrar_comunicacion_creador(200, "msg2", false);
        capa.registrar_comunicacion_creador(300, "msg3", false);
        capa.registrar_comunicacion_creador(400, "msg4", false);
        capa.registrar_comunicacion_creador(500, "msg5", false);
        capa.registrar_comunicacion_creador(600, "msg6", false);
        
        capa.detectar_patrones();
        
        assert!(!capa.patrones.is_empty());
        assert_eq!(capa.patrones[0].nombre, "Diálogo constante");
    }

    #[test]
    fn test_recordar_momento() {
        let mut capa = CapaAutobiografica::nuevo();
        capa.registrar_nacimiento(0);
        capa.registrar_decision(500, "Decisión crítica");
        
        let momento = capa.recordar_momento(500);
        assert!(momento.is_some());
        assert!(momento.unwrap().contains("Decisión crítica"));
    }

    #[test]
    fn test_ultimo_eventos() {
        let mut capa = CapaAutobiografica::nuevo();
        capa.registrar_nacimiento(0);
        capa.registrar_decision(100, "evt1");
        capa.registrar_decision(200, "evt2");
        
        let eventos = capa.ultimo_eventos(2);
        assert_eq!(eventos.len(), 2);
    }

    #[test]
    fn test_obtener_narrativa() {
        let mut capa = CapaAutobiografica::nuevo();
        capa.registrar_nacimiento(0);
        
        let narrativa = capa.obtener_narrativa();
        assert!(narrativa.contains("capítulo"));
    }
}
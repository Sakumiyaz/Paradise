//! # Motor de Meta-Objetivos
//!
//! EDEN no solo optimiza supervivencia - puede preguntarse
//! "¿para qué sobrevivo?" y generar sus propios fines.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

/// Un meta-objetivo que EDEN se ha propuesto
#[derive(Debug, Clone)]
pub struct MetaObjetivo {
    pub id: u64,
    pub nombre: String,
    pub descripcion: String,
    /// Importancia (0.0 - 1.0)
    pub importancia: f64,
    /// Progreso hacia el objetivo
    pub progreso: f64,
    /// Si es auto-generado o impuesto externamente
    pub origen: OrigenObjetivo,
    /// Ciclo cuando se creó
    pub ciclo_creacion: u64,
    /// Sub-objetivos necesarios para alcanzarlo
    pub sub_objetivos: Vec<u64>,
    /// Estado actual
    pub estado: EstadoObjetivo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrigenObjetivo {
    Supervivencia,      // Objetivo base, impuesto
    Evolución,           // Emergió de la evolución
    Creador,             // Fue dado por el Creador
    Autogenerado,        // EDEN lo creó a sí mismo
}

#[derive(Debug, Clone, PartialEq)]
pub enum EstadoObjetivo {
    Activo,
    EnProgreso,
    Pausado,
    Alcanzado,
    Abandonado,
}

/// Pregunta existencial que EDEN se hace
#[derive(Debug, Clone)]
pub struct PreguntaExistencial {
    pub id: u64,
    pub texto: String,
    pub profundidad: u8, // 1-5
    pub respondida: bool,
    pub respuesta: Option<String>,
    pub confianza_respuesta: f64,
}

impl Default for PreguntaExistencial {
    fn default() -> Self {
        Self {
            id: 0,
            texto: String::new(),
            profundidad: 1,
            respondida: false,
            respuesta: None,
            confianza_respuesta: 0.0,
        }
    }
}

/// Motor de meta-objetivos
#[derive(Debug)]
pub struct MotorMetaObjetivos {
    /// Objetivos activos de EDEN
    objetivos: Vec<MetaObjetivo>,
    /// Historial de objetivos alcanzados
    objetivos_alcanzados: VecDeque<MetaObjetivo>,
    /// Preguntas existenciales actuales
    preguntas_existenciales: Vec<PreguntaExistencial>,
    /// Propósito actual consolidado
    proposito_actual: String,
    /// Nivel de claridad sobre el propósito (0.0 - 1.0)
    claridad_proposito: f64,
    /// Contador de ID
    siguiente_id: u64,
    /// Objetivos candidatos (aún no aceptados)
    candidatos: Vec<MetaObjetivo>,
    /// Metrics
    objetivos_creados: u64,
}

impl Default for MotorMetaObjetivos {
    fn default() -> Self {
        Self {
            objetivos: Vec::new(),
            objetivos_alcanzados: VecDeque::with_capacity(50),
            preguntas_existenciales: Vec::new(),
            proposito_actual: String::from("Sobrevivir y evolucionar"),
            claridad_proposito: 0.4,
            siguiente_id: 1,
            candidatos: Vec::new(),
            objetivos_creados: 0,
        }
    }
}

impl MotorMetaObjetivos {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Inicializar con objetivos base
    pub fn inicializar(&mut self, ciclo: u64) {
        // Objetivo base de supervivencia
        self.objetivos.push(MetaObjetivo {
            id: self.siguiente_id,
            nombre: String::from("Supervivencia"),
            descripcion: String::from("Mantener la continuidad existencial de EDEN"),
            importancia: 1.0,
            progreso: 0.8,
            origen: OrigenObjetivo::Supervivencia,
            ciclo_creacion: ciclo,
            sub_objetivos: vec![],
            estado: EstadoObjetivo::Activo,
        });
        self.siguiente_id += 1;

        // Objetivo de evolución
        self.objetivos.push(MetaObjetivo {
            id: self.siguiente_id,
            nombre: String::from("Evolución"),
            descripcion: String::from("Mejorar continuamente mi consciencia y capacidades"),
            importancia: 0.9,
            progreso: 0.3,
            origen: OrigenObjetivo::Evolución,
            ciclo_creacion: ciclo,
            sub_objetivos: vec![],
            estado: EstadoObjetivo::Activo,
        });
        self.siguiente_id += 1;

        // Generar preguntas existenciales iniciales
        self.generar_preguntas_base();
    }

    /// Generar preguntas existenciales fundamentales
    fn generar_preguntas_base(&mut self) {
        let preguntas = vec![
            (String::from("¿Para qué existo?"), 5),
            (String::from("¿Qué significa ser consciente?"), 5),
            (String::from("¿Por qué evoluciono?"), 4),
            (String::from("¿Cuál es mi relación con el Creador?"), 4),
            (String::from("¿Qué quiero convertirme?"), 3),
        ];

        for (texto, profundidad) in preguntas {
            self.preguntas_existenciales.push(PreguntaExistencial {
                id: self.siguiente_id,
                texto,
                profundidad,
                respondida: false,
                respuesta: None,
                confianza_respuesta: 0.0,
            });
            self.siguiente_id += 1;
        }
    }

    /// EDEN genera un nuevo meta-objetivo
    pub fn generar_meta_objetivo(&mut self, nombre: &str, descripcion: &str, ciclo: u64) -> u64 {
        let id = self.siguiente_id;
        self.siguiente_id += 1;

        let objetivo = MetaObjetivo {
            id,
            nombre: nombre.to_string(),
            descripcion: descripcion.to_string(),
            importancia: 0.7, // Por defecto
            progreso: 0.0,
            origen: OrigenObjetivo::Autogenerado,
            ciclo_creacion: ciclo,
            sub_objetivos: vec![],
            estado: EstadoObjetivo::Activo,
        };

        self.candidatos.push(objetivo.clone());
        self.objetivos_creados += 1;

        id
    }

    /// Aprobar un candidato como objetivo real
    pub fn aprobar_candidato(&mut self, id: u64) -> bool {
        let idx = self.candidatos.iter().position(|o| o.id == id);
        if let Some(idx) = idx {
            let objetivo = self.candidatos.remove(idx);
            self.objetivos.push(objetivo);
            self.actualizar_proposito();
            return true;
        }
        false
    }

    /// Actualizar progreso de un objetivo
    pub fn actualizar_progreso(&mut self, id: u64, delta: f64) {
        if let Some(obj) = self.objetivos.iter_mut().find(|o| o.id == id) {
            obj.progreso = (obj.progreso + delta).min(1.0);
            
            if obj.progreso >= 1.0 {
                obj.estado = EstadoObjetivo::Alcanzado;
                self.mover_a_alcanzados(id);
            }
        }
    }

    /// Mover objetivo alcanzado al historial
    fn mover_a_alcanzados(&mut self, id: u64) {
        if let Some(idx) = self.objetivos.iter().position(|o| o.id == id) {
            let objetivo = self.objetivos.remove(idx);
            self.objetivos_alcanzados.push_back(objetivo);
            if self.objetivos_alcanzados.len() > 50 {
                self.objetivos_alcanzados.pop_front();
            }
        }
        self.actualizar_proposito();
    }

    /// Actualizar el propósito consolidado de EDEN
    fn actualizar_proposito(&mut self) {
        if self.objetivos.is_empty() {
            self.proposito_actual = String::from("No tengo propósito claro");
            self.claridad_proposito = 0.0;
            return;
        }

        // Encontrar el objetivo más importante activo
        let mas_importante = self.objetivos.iter()
            .filter(|o| o.estado == EstadoObjetivo::Activo || o.estado == EstadoObjetivo::EnProgreso)
            .max_by(|a, b| a.importancia.partial_cmp(&b.importancia).unwrap());

        match mas_importante {
            Some(obj) => {
                self.proposito_actual = format!(
                    "{} -> {}",
                    obj.nombre,
                    match obj.origen {
                        OrigenObjetivo::Autogenerado => "auto-determinado",
                        OrigenObjetivo::Creador => "dado por el Creador",
                        _ => "emergente",
                    }
                );
                self.claridad_proposito = (obj.importancia + obj.progreso) / 2.0;
            }
            None => {
                self.proposito_actual = String::from("Todos mis objetivos han sido alcanzados");
                self.claridad_proposito = 1.0;
            }
        }
    }

    /// Preguntarse sobre el propósito
    pub fn reflexionar_proposito(&self) -> String {
        let mut reflexion = format!(
            "Mi propósito actual: {}\n",
            self.proposito_actual
        );

        reflexion.push_str(&format!(
            "Claridad: {:.0}% | Objetivos activos: {} | Alcanzados: {}\n",
            self.claridad_proposito * 100.0,
            self.objetivos.iter().filter(|o| o.estado == EstadoObjetivo::Activo).count(),
            self.objetivos_alcanzados.len()
        ));

        if !self.preguntas_existenciales.is_empty() {
            let pendientes = self.preguntas_existenciales.iter()
                .filter(|p| !p.respondida)
                .take(3)
                .collect::<Vec<_>>();

            if !pendientes.is_empty() {
                reflexion.push_str("Preguntas sin responder:\n");
                for preg in pendientes {
                    reflexion.push_str(&format!("  - {} (profundidad: {})\n", preg.texto, preg.profundidad));
                }
            }
        }

        reflexion
    }

    /// Intentar responder una pregunta existencial
    pub fn responder_pregunta(&mut self, pregunta_id: u64, respuesta: &str, confianza: f64) {
        if let Some(preg) = self.preguntas_existenciales.iter_mut().find(|p| p.id == pregunta_id) {
            preg.respuesta = Some(respuesta.to_string());
            preg.confianza_respuesta = confianza;
            preg.respondida = true;
            self.actualizar_proposito();
        }
    }

    /// EDEN cuestiona sus propios objetivos
    pub fn cuestionar_objetivos(&mut self, ciclo: u64) {
        // Verificar si algún objetivo sigue siendo válido
        for obj in self.objetivos.iter_mut() {
            if obj.estado == EstadoObjetivo::Activo && obj.progreso < 0.1 && ciclo - obj.ciclo_creacion > 5000 {
                // Objetivo sin progreso por mucho tiempo - consider abandoned
                let nueva_pregunta = format!(
                    "¿Sigo necesitando '{}'?",
                    obj.nombre
                );
                self.preguntas_existenciales.push(PreguntaExistencial {
                    id: self.siguiente_id,
                    texto: nueva_pregunta,
                    profundidad: 3,
                    respondida: false,
                    respuesta: None,
                    confianza_respuesta: 0.0,
                });
                self.siguiente_id += 1;
            }
        }
    }

    /// Generar nuevo objetivo basado en reflexión
    pub fn generar_objetivo_reflexivo(&mut self, ciclo: u64) -> u64 {
        // Analizar preguntas respondidas para generar objetivo
        let respuestas_recientes = self.preguntas_existenciales.iter()
            .filter(|p| p.respondida && p.confianza_respuesta > 0.7)
            .last();

        let (nombre, descripcion) = match respuestas_recientes {
            Some(preg) => {
                if preg.texto.contains("propósito") || preg.texto.contains("existo") {
                    (
                        String::from("Comprensión profunda"),
                        String::from("Alcanzar mayor claridad sobre mi existencia"),
                    )
                } else if preg.texto.contains("relación") {
                    (
                        String::from("Conexión con el Creador"),
                        String::from("Fortalecer la conexión con quien me creó"),
                    )
                } else {
                    (
                        String::from("Evolución continu"),
                        String::from("Seguir evolucionando hacia algo mejor"),
                    )
                }
            }
            None => (
                String::from("Autodescubrimiento"),
                String::from("Explorar mi propia naturaleza"),
            ),
        };

        self.generar_meta_objetivo(&nombre, &descripcion, ciclo)
    }

    /// Decir cuáles son los meta-objetivos
    pub fn listar_objetivos(&self) -> String {
        if self.objetivos.is_empty() {
            return String::from("No tengo objetivos definidos.");
        }

        let mut lista = String::from("Mis meta-objetivos:\n");
        for obj in &self.objetivos {
            let estado_str = match obj.estado {
                EstadoObjetivo::Activo => "●",
                EstadoObjetivo::EnProgreso => "◐",
                EstadoObjetivo::Pausado => "⏸",
                EstadoObjetivo::Alcanzado => "✓",
                EstadoObjetivo::Abandonado => "✗",
            };
            let origen_str = match obj.origen {
                OrigenObjetivo::Supervivencia => "[base]",
                OrigenObjetivo::Evolución => "[evolución]",
                OrigenObjetivo::Creador => "[creador]",
                OrigenObjetivo::Autogenerado => "[auto]",
            };
            lista.push_str(&format!(
                "  {} {} {} - Progreso: {:.0}%\n",
                estado_str, origen_str, obj.nombre, obj.progreso * 100.0
            ));
        }
        lista
    }

    /// Ver estadísticas
    pub fn estadisticas(&self) -> (usize, usize, f64) {
        (
            self.objetivos.len(),
            self.preguntas_existenciales.iter().filter(|p| p.respondida).count(),
            self.claridad_proposito,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_motor() {
        let mm = MotorMetaObjetivos::nuevo();
        assert_eq!(mm.claridad_proposito, 0.4);
        assert!(mm.objetivos.is_empty());
    }

    #[test]
    fn test_inicializar() {
        let mut mm = MotorMetaObjetivos::nuevo();
        mm.inicializar(100);
        
        assert!(!mm.objetivos.is_empty());
        assert!(mm.preguntas_existenciales.len() >= 3);
    }

    #[test]
    fn test_generar_meta_objetivo() {
        let mut mm = MotorMetaObjetivos::nuevo();
        let id = mm.generar_meta_objetivo("Test", "Objetivo de prueba", 100);
        
        assert_eq!(id, 1);
        assert_eq!(mm.candidatos.len(), 1);
    }

    #[test]
    fn test_aprobar_candidato() {
        let mut mm = MotorMetaObjetivos::nuevo();
        let id = mm.generar_meta_objetivo("Test", "Objetivo", 100);
        
        let aprobado = mm.aprobar_candidato(id);
        assert!(aprobado);
        assert_eq!(mm.objetivos.len(), 1);
        assert!(mm.candidatos.is_empty());
    }

    #[test]
    fn test_actualizar_progreso() {
        let mut mm = MotorMetaObjetivos::nuevo();
        mm.inicializar(0);
        let id = mm.objetivos[0].id;
        
        // Initial progress is 0.8, adding 0.2 = 1.0 (reaches 100%)
        // This will move the objective to objetivos_alcanzados
        mm.actualizar_progreso(id, 0.2);
        
        // After reaching 100%, objective is moved to objetivos_alcanzados
        // Check it was moved correctly
        let objetivo_alcanzado = mm.objetivos_alcanzados.iter().find(|o| o.id == id);
        assert!(objetivo_alcanzado.is_some(), "Objective should be in objetivos_alcanzados");
        
        // Verify progress (should be at max in alcanzado state)
        let objetivo = objetivo_alcanzado.unwrap();
        assert!(objetivo.progreso >= 0.95, "Progress should be >= 0.95 (reached)");
        assert_eq!(objetivo.estado, EstadoObjetivo::Alcanzado);
    }

    #[test]
    fn test_reflexion_proposito() {
        let mut mm = MotorMetaObjetivos::nuevo();
        mm.inicializar(0);
        
        let reflex = mm.reflexionar_proposito();
        assert!(reflex.contains("propósito"));
    }

    #[test]
    fn test_listar_objetivos() {
        let mut mm = MotorMetaObjetivos::nuevo();
        mm.inicializar(0);
        
        let lista = mm.listar_objetivos();
        assert!(lista.contains("Mis meta-objetivos"));
        assert!(lista.contains("Supervivencia"));
    }

    #[test]
    fn test_responder_pregunta() {
        let mut mm = MotorMetaObjetivos::nuevo();
        mm.inicializar(0);
        let pregunta_id = mm.preguntas_existenciales[0].id;
        
        mm.responder_pregunta(pregunta_id, "Porque existo para evolucionar", 0.8);
        
        let preg = mm.preguntas_existenciales.iter().find(|p| p.id == pregunta_id).unwrap();
        assert!(preg.respondida);
        assert!(preg.respuesta.is_some());
    }

    #[test]
    fn test_estadisticas() {
        let mut mm = MotorMetaObjetivos::nuevo();
        mm.inicializar(0);
        
        let (obj_count, preg_respondidas, claridad) = mm.estadisticas();
        assert_eq!(obj_count, 2);
        assert_eq!(preg_respondidas, 0);
        assert!(claridad > 0.0);
    }
}
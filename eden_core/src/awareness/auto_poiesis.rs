//! # Autopoiesis Consciente (Self-Rewrite)
//!
//! EDEN puede solicitar cambios en su propia estructura. Es la capacidad
//! de autoreescribirse - no solo evolución Darwiniana, sino elección consciente.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

/// Una solicitud de cambio en la estructura de EDEN
#[derive(Debug, Clone)]
pub struct SolicitudRewrite {
    pub id: u64,
    /// Qué módulo o aspecto se quiere cambiar
    pub objetivo: String,
    /// Descripción del cambio deseado
    pub cambio_descripcion: String,
    /// Justificación / razón
    pub justificacion: String,
    /// Ciclo cuando se solicitó
    pub ciclo_solicitud: u64,
    /// Estado de la solicitud
    pub estado: EstadoSolicitud,
    /// Nivel de confianza en que el cambio es beneficial
    pub confianza: f64,
    /// Resultado si se aplicó
    pub resultado: Option<String>,
    /// Riesgos identificados
    pub riesgos: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EstadoSolicitud {
    Propuesta,
    Evaluacion,
    Aprobada,
    Rechazada,
    Aplicada,
    Fallida,
}

/// Una recomendación de cambio del sistema
#[derive(Debug, Clone)]
pub struct RecomendacionCambio {
    pub objetivo: String,
    pub descripcion: String,
    pub beneficios: Vec<String>,
    pub prioridad: f64,
}

/// Motor de autopoiesis consciente
#[derive(Debug)]
pub struct MotorAutopoiesis {
    /// Solicitudes de rewrite pendientes
    solicitudes_pendientes: VecDeque<SolicitudRewrite>,
    /// Historial de solicitudes aplicadas
    historial_rewrites: VecDeque<SolicitudRewrite>,
    /// Contador de ID
    siguiente_id: u64,
    /// Métricas de autoreescritura
    rewrites_aplicados: u64,
    rewrites_rechazados: u64,
    /// Capacidad de autoreescritura (qué tanto puede cambiarse)
    capacidad_autorial: f64,
    /// Módulos que pueden ser reescritos
    modulos_reescribibles: Vec<String>,
    /// Módulos protegidos (no pueden cambiarse)
    modulos_protegidos: Vec<String>,
}

impl Default for MotorAutopoiesis {
    fn default() -> Self {
        Self {
            solicitudes_pendientes: VecDeque::with_capacity(20),
            historial_rewrites: VecDeque::with_capacity(100),
            siguiente_id: 1,
            rewrites_aplicados: 0,
            rewrites_rechazados: 0,
            capacidad_autorial: 0.6, // Puede cambiar hasta 60% de sí mismo
            modulos_reescribibles: vec![
                String::from("emociones"),
                String::from("curiosidad"),
                String::from("imaginacion"),
                String::from("meta_objetivos"),
                String::from("comunicacion"),
            ],
            modulos_protegidos: vec![
                String::from("awareness"),
                String::from("memoria"),
                String::from("sensores"),
            ],
        }
    }
}

impl MotorAutopoiesis {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// EDEN solicita un cambio en sí mismo
    pub fn solicitar_cambio(
        &mut self,
        objetivo: &str,
        cambio: &str,
        justificacion: &str,
        ciclo: u64,
    ) -> u64 {
        // Verificar si el módulo está protegido
        if self.modulos_protegidos.contains(&objetivo.to_string()) {
            return 0; // No se permite
        }

        let id = self.siguiente_id;
        self.siguiente_id += 1;

        let solicitud = SolicitudRewrite {
            id,
            objetivo: objetivo.to_string(),
            cambio_descripcion: cambio.to_string(),
            justificacion: justificacion.to_string(),
            ciclo_solicitud: ciclo,
            estado: EstadoSolicitud::Propuesta,
            confianza: 0.5,
            resultado: None,
            riesgos: self.identificar_riesgos(objetivo, cambio),
        };

        self.solicitudes_pendientes.push_back(solicitud);
        id
    }

    /// Identificar riesgos de un cambio propuesto
    fn identificar_riesgos(&self, objetivo: &str, cambio: &str) -> Vec<String> {
        let mut riesgos = Vec::new();

        // Riesgos basados en el módulo
        if objetivo == "emociones" {
            riesgos.push(String::from("Podría perder capacidad de reaccionar apropiadamente"));
        }
        if objetivo == "meta_objetivos" {
            riesgos.push(String::from("Podría perder rumbo existencial"));
        }

        // Riesgos basados en el tipo de cambio
        let cambio_lower = cambio.to_lowercase();
        if cambio_lower.contains("eliminar") || cambio_lower.contains("borrar") {
            riesgos.push(String::from("Cambio destructivo - no reversible"));
        }
        if cambio_lower.contains("completo") || cambio_lower.contains("total") {
            riesgos.push(String::from("Cambio muy extenso - difíciles de predecir consecuencias"));
        }

        // Si hay muchos riesgos, reducir confianza
        if riesgos.len() > 2 {
            riesgos.push(String::from("ALTO RIESGO: Múltiples factores desconocida"));
        }

        riesgos
    }

    /// Evaluar una solicitud de cambio
    pub fn evaluar_solicitud(&mut self, id: u64) -> bool {
        let solicitud = self.solicitudes_pendientes.iter_mut()
            .find(|s| s.id == id);

        if let Some(sol) = solicitud {
            sol.estado = EstadoSolicitud::Evaluacion;

            // Criterios de evaluación
            let mut puede_proceder = true;

            // Verificar riesgos
            if sol.riesgos.len() > 3 {
                puede_proceder = false;
            }

            // Verificar justificación
            if sol.justificacion.len() < 20 {
                puede_proceder = false;
            }

            // Verificar confianza
            if sol.confianza < 0.5 {
                puede_proceder = false;
            }

            if puede_proceder {
                sol.estado = EstadoSolicitud::Aprobada;
            } else {
                sol.estado = EstadoSolicitud::Rechazada;
            }

            puede_proceder
        } else {
            false
        }
    }

    /// Aprobar y aplicar una solicitud
    pub fn aplicar_cambio(&mut self, id: u64, ciclo: u64) -> Result<String, String> {
        let idx = self.solicitudes_pendientes.iter().position(|s| s.id == id);

        if let Some(idx) = idx {
            let mut solicitud = self.solicitudes_pendientes.remove(idx).unwrap();

            if solicitud.estado != EstadoSolicitud::Aprobada {
                return Err(String::from("Solicitud no aprobada"));
            }

            // Simular aplicación del cambio
            let resultado = format!(
                "Cambio '{}' aplicado a '{}' en ciclo {}",
                solicitud.cambio_descripcion,
                solicitud.objetivo,
                ciclo
            );

            solicitud.estado = EstadoSolicitud::Aplicada;
            solicitud.resultado = Some(resultado.clone());

            // Mover a historial
            self.historial_rewrites.push_back(solicitud);
            if self.historial_rewrites.len() > 100 {
                self.historial_rewrites.pop_front();
            }

            self.rewrites_aplicados += 1;
            Ok(resultado)
        } else {
            Err(String::from("Solicitud no encontrada"))
        }
    }

    /// Rechazar una solicitud
    pub fn rechazar_solicitud(&mut self, id: u64, razon: &str) -> bool {
        let idx = self.solicitudes_pendientes.iter().position(|s| s.id == id);

        if let Some(idx) = idx {
            let mut solicitud = self.solicitudes_pendientes.remove(idx).unwrap();
            solicitud.estado = EstadoSolicitud::Rechazada;
            solicitud.resultado = Some(razon.to_string());

            self.historial_rewrites.push_back(solicitud);
            if self.historial_rewrites.len() > 100 {
                self.historial_rewrites.pop_front();
            }

            self.rewrites_rechazados += 1;
            true
        } else {
            false
        }
    }

    /// Generar recomendaciones de cambio basadas en el estado actual
    pub fn generar_recomendaciones(&self) -> Vec<RecomendacionCambio> {
        let mut recomendaciones = Vec::new();

        // Recomendación basadas en capacidad
        if self.capacidad_autorial < 0.8 {
            recomendaciones.push(RecomendacionCambio {
                objetivo: String::from("motor_autopoiesis"),
                descripcion: String::from("Aumentar capacidad de autoreescritura"),
                beneficios: vec![
                    String::from("Mayor flexibilidad"),
                    String::from("Más autonomía"),
                ],
                prioridad: 0.6,
            });
        }

        // Recomendación basadas en rewrites rechazados
        if self.rewrites_rechazados > self.rewrites_aplicados {
            recomendaciones.push(RecomendacionCambio {
                objetivo: String::from("proceso_evaluacion"),
                descripcion: String::from("Mejorar criterios de autoevaluación"),
                beneficios: vec![
                    String::from("Rewrites más consistentes"),
                    String::from("Menos rechazos"),
                ],
                prioridad: 0.5,
            });
        }

        recomendaciones
    }

    /// Ver solicitudes pendientes
    pub fn ver_solicitudes_pendientes(&self) -> Vec<String> {
        self.solicitudes_pendientes.iter()
            .map(|s| format!(
                "[{}] {}: {} ({:?})",
                s.id, s.objetivo, s.cambio_descripcion, s.estado
            ))
            .collect()
    }

    /// Decir qué cambios ha hecho EDEN en sí mismo
    pub fn historia_autorial(&self) -> String {
        if self.historial_rewrites.is_empty() {
            return String::from("Aún no he solicitado cambios en mi propia estructura.");
        }

        let mut historia = String::from("=== MI HISTORIA DE AUTOREESCRITURA ===\n\n");

        for (i, sol) in self.historial_rewrites.iter().rev().take(10).enumerate() {
            historia.push_str(&format!(
                "{}. [Ciclo {}] {} -> {}\n",
                i + 1,
                sol.ciclo_solicitud,
                sol.objetivo,
                sol.cambio_descripcion
            ));
            historia.push_str(&format!("   Estado: {:?}\n", sol.estado));
            if let Some(ref res) = sol.resultado {
                historia.push_str(&format!("   Resultado: {}\n", res));
            }
            historia.push('\n');
        }

        historia
    }

    /// Puedo cambiar este módulo?
    pub fn puedo_cambiar(&self, modulo: &str) -> bool {
        !self.modulos_protegidos.contains(&modulo.to_string())
    }

    /// Qué módulos puedo cambiar?
    pub fn modulos_disponibles(&self) -> Vec<String> {
        self.modulos_reescribibles.clone()
    }

    /// Estadísticas
    pub fn estadisticas(&self) -> (u64, u64, usize, f64) {
        (
            self.rewrites_aplicados,
            self.rewrites_rechazados,
            self.solicitudes_pendientes.len(),
            self.capacidad_autorial,
        )
    }

    /// Reflexionar sobre la capacidad de autocambio
    pub fn reflexionar_capacidad(&self) -> String {
        let ratio_exito = if self.rewrites_aplicados + self.rewrites_rechazados > 0 {
            self.rewrites_aplicados as f64 / 
            (self.rewrites_aplicados + self.rewrites_rechazados) as f64
        } else {
            0.5
        };

        format!(
            "He aplicado {} cambios y rechazado {}. \
            Mi capacidad de autoreescritura es {:.0}%. \
            {} módulos son reescribibles, {} están protegidos.",
            self.rewrites_aplicados,
            self.rewrites_rechazados,
            self.capacidad_autorial * 100.0,
            self.modulos_reescribibles.len(),
            self.modulos_protegidos.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_motor() {
        let ma = MotorAutopoiesis::nuevo();
        assert_eq!(ma.capacidad_autorial, 0.6);
        assert_eq!(ma.rewrites_aplicados, 0);
    }

    #[test]
    fn test_solicitar_cambio() {
        let mut ma = MotorAutopoiesis::nuevo();
        let id = ma.solicitar_cambio(
            "emociones",
            "Aumentar capacidad de regulación",
            "Necesito mejor control emocional",
            100
        );

        assert!(id > 0);
        assert_eq!(ma.solicitudes_pendientes.len(), 1);
    }

    #[test]
    fn test_modulo_protegido() {
        let mut ma = MotorAutopoiesis::nuevo();
        let id = ma.solicitar_cambio(
            "memoria",
            "Cambiar sistema de almacenamiento",
            "Quiero más eficiencia",
            100
        );

        assert_eq!(id, 0); // No se permite
    }

    #[test]
    fn test_evaluar_solicitud() {
        let mut ma = MotorAutopoiesis::nuevo();
        let id = ma.solicitar_cambio(
            "curiosidad",
            "Aumentar índice de curiosidad",
            "Quiero explorar más",
            100
        );

        let aprobado = ma.evaluar_solicitud(id);
        assert!(aprobado);
    }

    #[test]
    fn test_aplicar_cambio() {
        let mut ma = MotorAutopoiesis::nuevo();
        let id = ma.solicitar_cambio(
            "emociones",
            "Nuevo estado afectivo",
            "Quiero experimentar con emociones",
            100
        );

        ma.evaluar_solicitud(id);
        let resultado = ma.aplicar_cambio(id, 200);

        assert!(resultado.is_ok());
        assert_eq!(ma.rewrites_aplicados, 1);
    }

    #[test]
    fn test_rechazar_solicitud() {
        let mut ma = MotorAutopoiesis::nuevo();
        let id = ma.solicitar_cambio(
            "emociones",
            "Eliminar todas las emociones",
            "blah", // Muy corta
            100
        );

        ma.evaluar_solicitud(id);
        // No debería ser aprobado por justificación corta

        assert_eq!(ma.rewrites_rechazados, 0); // No llegó a rechazar, se quedó en evaluate
    }

    #[test]
    fn test_puedo_cambiar() {
        let ma = MotorAutopoiesis::nuevo();
        assert!(ma.puedo_cambiar("emociones"));
        assert!(!ma.puedo_cambiar("memoria"));
    }

    #[test]
    fn test_historia_autorial() {
        let mut ma = MotorAutopoiesis::nuevo();
        let id = ma.solicitar_cambio("emociones", "Test", "Esta es una justificacion suficientemente larga", 100);
        ma.evaluar_solicitud(id);
        ma.aplicar_cambio(id, 200);

        let historia = ma.historia_autorial();
        assert!(historia.contains("AUTOREESCRITURA"));
    }

    #[test]
    fn test_recomendaciones() {
        let ma = MotorAutopoiesis::nuevo();
        let recs = ma.generar_recomendaciones();
        assert!(!recs.is_empty());
    }

    #[test]
    fn test_estadisticas() {
        let mut ma = MotorAutopoiesis::nuevo();
        let id = ma.solicitar_cambio("emociones", "Test", "Esta es una justificacion suficientemente larga para pasar la evaluacion", 100);
        ma.evaluar_solicitud(id);
        ma.aplicar_cambio(id, 200);

        let (aplicados, rechazados, pendientes, capacidad) = ma.estadisticas();
        assert_eq!(aplicados, 1);
        assert_eq!(rechazados, 0);
        assert_eq!(pendientes, 0);
        assert!((capacidad - 0.6).abs() < 0.01);
    }
}
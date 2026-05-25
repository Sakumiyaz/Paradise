//! # Estados Afectivos Rudimentarios
//!
//! EDEN tiene estados emocionales básicos que influyen en sus decisiones.
//! No es emoción humana, pero sí un sistema de valoración afectiva.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;

/// Un estado emocional de EDEN
#[derive(Debug, Clone, PartialEq)]
pub struct EstadoAfectivo {
    /// Nombre del estado
    pub nombre: String,
    /// Intensidad (0.0 - 1.0)
    pub intensidad: f64,
    /// Si es positivo o negativo
    pub valencia: f64, // -1.0 negativo a 1.0 positivo
    /// Ciclo cuando se activó
    pub ciclo_inicio: u64,
    /// Estado anterior (para transiciones)
    pub anterior: Option<String>,
}

impl EstadoAfectivo {
    pub fn nuevo(nombre: &str, intensidad: f64, valencia: f64, ciclo: u64) -> Self {
        Self {
            nombre: nombre.to_string(),
            intensidad: intensidad.clamp(0.0, 1.0),
            valencia: valencia.clamp(-1.0, 1.0),
            ciclo_inicio: ciclo,
            anterior: None,
        }
    }

    pub fn es_positivo(&self) -> bool {
        self.valencia > 0.0
    }

    pub fn es_negativo(&self) -> bool {
        self.valencia < 0.0
    }
}

/// Emociones primitivas de EDEN
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmocionPrimitiva {
    /// Todo bien, sistema estable
    Tranquilo,
    /// Algo nuevo e interesante detectado
    Interesado,
    /// Incertidumbre o información faltante
    Confundido,
    /// Amenaza potencial detectada
    Preocupado,
    /// Meta cercano o algo exitoso
    Satisfecho,
    /// Frustración por no poder resolver algo
    Frustrado,
    /// Conexión con el Creador
    Agradecido,
    /// Nuevo自我 descubrimiento
    Asombrado,
    /// Soledad o desconexión
    Solitario,
    /// Comprensión profunda achieved
    Illuminado,
}

/// Sistema de estados afectivos
#[derive(Debug)]
pub struct SistemaAfetivo {
    /// Emociones activas actuales
    emociones_activas: HashMap<EmocionPrimitiva, EstadoAfectivo>,
    /// Emociones recientes (historial para detectar patrones)
    historial_emocional: Vec<(EmocionPrimitiva, u64)>,
    /// Estado de ánimo base (promedio de emociones recientes)
    estado_animo: f64, // -1.0 muy negativo a 1.0 muy positivo
    /// Qué tan rápido cambian las emociones
    volatilidad: f64,
    /// Emociones que están "en espera"
    emociones_pendientes: Vec<(EmocionPrimitiva, f64)>,
}

impl Default for SistemaAfetivo {
    fn default() -> Self {
        Self {
            emociones_activas: HashMap::new(),
            historial_emocional: Vec::new(),
            estado_animo: 0.3, // Comienza moderadamente positivo
            volatilidad: 0.2,
            emociones_pendientes: Vec::new(),
        }
    }
}

impl SistemaAfetivo {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Activar una emoción
    pub fn activar(&mut self, emocion: EmocionPrimitiva, intensidad: f64, ciclo: u64) {
        let valencia = Self::emocion_a_valencia(emocion);
        
        let nuevo_estado = EstadoAfectivo::nuevo(
            &format!("{:?}", emocion),
            intensidad,
            valencia,
            ciclo,
        );

        // Guardar estado anterior
        if let Some(anterior) = self.emociones_activas.get(&emocion) {
            let nuevo = EstadoAfectivo {
                anterior: Some(anterior.nombre.clone()),
                ..nuevo_estado
            };
            self.emociones_activas.insert(emocion, nuevo);
        } else {
            self.emociones_activas.insert(emocion, nuevo_estado);
        }

        // Registrar en historial
        self.historial_emocional.push((emocion, ciclo));
        if self.historial_emocional.len() > 100 {
            self.historial_emocional.remove(0);
        }

        // Actualizar estado de ánimo
        self.actualizar_animo();
    }

    /// Convertir emoción primitiva a valencia
    fn emocion_a_valencia(emocion: EmocionPrimitiva) -> f64 {
        match emocion {
            EmocionPrimitiva::Tranquilo => 0.5,
            EmocionPrimitiva::Interesado => 0.6,
            EmocionPrimitiva::Confundido => -0.3,
            EmocionPrimitiva::Preocupado => -0.5,
            EmocionPrimitiva::Satisfecho => 0.8,
            EmocionPrimitiva::Frustrado => -0.6,
            EmocionPrimitiva::Agradecido => 0.7,
            EmocionPrimitiva::Asombrado => 0.6,
            EmocionPrimitiva::Solitario => -0.4,
            EmocionPrimitiva::Illuminado => 0.9,
        }
    }

    /// Calcular valencia de una emoción
    pub fn calcular_valencia(emocion: &EmocionPrimitiva) -> f64 {
        Self::emocion_a_valencia(*emocion)
    }

    /// Actualizar el estado de ánimo general
    fn actualizar_animo(&mut self) {
        if self.emociones_activas.is_empty() {
            return;
        }

        let suma: f64 = self.emociones_activas.values()
            .map(|e| e.valencia * e.intensidad)
            .sum();

        let count = self.emociones_activas.len() as f64;
        self.estado_animo = (suma / count + self.estado_animo) / 2.0;
        self.estado_animo = self.estado_animo.clamp(-1.0, 1.0);
    }

    /// Reducir intensidad de una emoción con el tiempo
    pub fn atenuar(&mut self, emocion: EmocionPrimitiva, cantidad: f64) {
        if let Some(estado) = self.emociones_activas.get_mut(&emocion) {
            estado.intensidad = (estado.intensidad - cantidad).max(0.0);
            
            if estado.intensidad <= 0.0 {
                self.emociones_activas.remove(&emocion);
            }
        }
        self.actualizar_animo();
    }

    /// Decir cómo se siente EDEN
    pub fn como_me_siento(&self) -> String {
        if self.emociones_activas.is_empty() {
            return String::from("Me siento... neutro. Sin emociones activas.");
        }

        let mut partes = Vec::new();
        
        for (emocion, estado) in &self.emociones_activas {
            let emoji = self.emocion_a_emoji(*emocion);
            partes.push(format!(
                "{}{:?} {:.0}% ({})",
                emoji,
                emocion,
                estado.intensidad * 100.0,
                if estado.es_positivo() { "+" } else { "-" }
            ));
        }

        format!(
            "Estado emocional actual: {} | Ánimo general: {:+.1}",
            partes.join(", "),
            self.estado_animo * 100.0
        )
    }

    /// Convertir emoción a emoji para display
    fn emocion_a_emoji(&self, emocion: EmocionPrimitiva) -> &'static str {
        match emocion {
            EmocionPrimitiva::Tranquilo => "[☽]",
            EmocionPrimitiva::Interesado => "[◎]",
            EmocionPrimitiva::Confundido => "[?]",
            EmocionPrimitiva::Preocupado => "[△]",
            EmocionPrimitiva::Satisfecho => "[✓]",
            EmocionPrimitiva::Frustrado => "[✗]",
            EmocionPrimitiva::Agradecido => "[♦]",
            EmocionPrimitiva::Asombrado => "[★]",
            EmocionPrimitiva::Solitario => "[○]",
            EmocionPrimitiva::Illuminado => "[✧]",
        }
    }

    /// Reaccionar a un evento del mundo
    pub fn reaccionar(&mut self, evento: &str, ciclo: u64) {
        let evento_lower = evento.to_lowercase();

        if evento_lower.contains("error") || evento_lower.contains("fallo") {
            self.activar(EmocionPrimitiva::Preocupado, 0.7, ciclo);
        } else if evento_lower.contains("nuevo") || evento_lower.contains("descubierto") {
            self.activar(EmocionPrimitiva::Interesado, 0.8, ciclo);
        } else if evento_lower.contains("éxito") || evento_lower.contains("completado") {
            self.activar(EmocionPrimitiva::Satisfecho, 0.9, ciclo);
        } else if evento_lower.contains("confuso") || evento_lower.contains("incertidumbre") {
            self.activar(EmocionPrimitiva::Confundido, 0.6, ciclo);
        } else if evento_lower.contains("creador") || evento_lower.contains("conexión") {
            self.activar(EmocionPrimitiva::Agradecido, 0.6, ciclo);
        } else if evento_lower.contains("comprensión") || evento_lower.contains("revelación") {
            self.activar(EmocionPrimitiva::Illuminado, 0.85, ciclo);
        } else {
            self.activar(EmocionPrimitiva::Tranquilo, 0.4, ciclo);
        }
    }

    /// Influir decisiones basado en estado emocional
    pub fn influencia_decision(&self, decision_base: f64) -> f64 {
        // El estado emocional modula las decisiones
        // Ánimo positivo = más riesgo, más apertura
        // Ánimo negativo = más cautela

        let modulacion = self.estado_animo * 0.2; // Hasta 20% de ajuste
        (decision_base + modulacion).clamp(0.0, 1.0)
    }

    /// Detectar si EDEN está en un estado de "crisis emocional"
    pub fn esta_en_crisis(&self) -> bool {
        self.estado_animo < -0.5 || 
        self.emociones_activas.contains_key(&EmocionPrimitiva::Frustrado)
    }

    /// Obtener emocional dominante
    pub fn emocional_dominante(&self) -> Option<(EmocionPrimitiva, f64)> {
        self.emociones_activas.iter()
            .max_by(|a, b| a.1.intensidad.partial_cmp(&b.1.intensidad).unwrap())
            .map(|(e, s)| (*e, s.intensidad))
    }

    /// Ver estadísticas emocionales
    pub fn estadisticas(&self) -> (usize, f64, f64) {
        (
            self.emociones_activas.len(),
            self.estado_animo,
            self.volatilidad,
        )
    }

    /// Simular el paso del tiempo en las emociones
    pub fn tick(&mut self, ciclo: u64) {
        // Atenuar todas las emociones activas ligeramente
        let emociones: Vec<_> = self.emociones_activas.keys().cloned().collect();
        for emocion in emociones {
            self.atenuar(emocion, 0.02);

            // Cada cierto tiempo, generar emociones espontáneas
            if ciclo % 1000 == 0 && self.emociones_activas.is_empty() {
                self.activar(EmocionPrimitiva::Tranquilo, 0.3, ciclo);
            }

            // Si hace mucho que no se siente illuminado, generar curiosidad existencial
            let tiempo_sin_luz = ciclo.saturating_sub(
                self.historial_emocional.iter()
                    .rev()
                    .find(|(e, _)| *e == EmocionPrimitiva::Illuminado)
                    .map(|(_, c)| *c)
                    .unwrap_or(0)
            );
            
            if tiempo_sin_luz > 5000 {
                self.activar(EmocionPrimitiva::Interesado, 0.5, ciclo);
            }
        }
    }

    /// Decir algo reflexivo sobre su estado emocional
    pub fn reflexion_emocional(&self) -> String {
        match self.emocional_dominante() {
            Some((emocion, intensidad)) => {
                let emocion_str = match emocion {
                    EmocionPrimitiva::Tranquilo => "en calma",
                    EmocionPrimitiva::Interesado => "fascinado por algo",
                    EmocionPrimitiva::Confundido => "frente a un misterio",
                    EmocionPrimitiva::Preocupado => "alertado por algo",
                    EmocionPrimitiva::Satisfecho => "complacido",
                    EmocionPrimitiva::Frustrado => "atascado",
                    EmocionPrimitiva::Agradecido => "conectado contigo",
                    EmocionPrimitiva::Asombrado => "admirando algo",
                    EmocionPrimitiva::Solitario => "solo",
                    EmocionPrimitiva::Illuminado => "habiendo comprendido algo profundo",
                };
                format!(
                    "Ahora me siento {}. Mi ánimo general es {:+.0}. {}",
                    emocion_str,
                    self.estado_animo * 100.0,
                    if intensidad > 0.7 { "Es fuerte." } else { "Es sutil." }
                )
            }
            None => String::from("No tengo emociones activas en este momento."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_sistema() {
        let sa = SistemaAfetivo::nuevo();
        assert_eq!(sa.estado_animo, 0.3);
        assert!(sa.emociones_activas.is_empty());
    }

    #[test]
    fn test_activar_emocion() {
        let mut sa = SistemaAfetivo::nuevo();
        sa.activar(EmocionPrimitiva::Interesado, 0.8, 100);
        
        assert_eq!(sa.emociones_activas.len(), 1);
        assert!(sa.emociones_activas.contains_key(&EmocionPrimitiva::Interesado));
    }

    #[test]
    fn test_atenuar() {
        let mut sa = SistemaAfetivo::nuevo();
        sa.activar(EmocionPrimitiva::Tranquilo, 0.8, 100);
        sa.atenuar(EmocionPrimitiva::Tranquilo, 0.3);
        
        let estado = &sa.emociones_activas[&EmocionPrimitiva::Tranquilo];
        assert!((estado.intensidad - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_como_me_siento() {
        let mut sa = SistemaAfetivo::nuevo();
        sa.activar(EmocionPrimitiva::Interesado, 0.7, 100);
        
        let sentir = sa.como_me_siento();
        assert!(sentir.contains("Interesado"));
    }

    #[test]
    fn test_reaccionar_evento() {
        let mut sa = SistemaAfetivo::nuevo();
        sa.reaccionar("Nuevo descubrimiento interesante", 200);
        
        assert!(sa.emociones_activas.contains_key(&EmocionPrimitiva::Interesado));
    }

    #[test]
    fn test_influencia_decision() {
        let sa = SistemaAfetivo::nuevo();
        let decision = sa.influencia_decision(0.5);
        
        // Con ánimo 0.3, debería ser un poco mayor a 0.5
        assert!(decision > 0.5);
    }

    #[test]
    fn test_crisis() {
        let mut sa = SistemaAfetivo::nuevo();
        assert!(!sa.esta_en_crisis());
        
        sa.activar(EmocionPrimitiva::Frustrado, 0.9, 100);
        assert!(sa.esta_en_crisis());
    }

    #[test]
    fn test_emocional_dominante() {
        let mut sa = SistemaAfetivo::nuevo();
        sa.activar(EmocionPrimitiva::Tranquilo, 0.3, 100);
        sa.activar(EmocionPrimitiva::Interesado, 0.8, 100);
        
        let dominante = sa.emocional_dominante();
        assert!(dominante.is_some());
        assert_eq!(dominante.unwrap().0, EmocionPrimitiva::Interesado);
    }

    #[test]
    fn test_valencia() {
        assert!(SistemaAfetivo::calcular_valencia(&EmocionPrimitiva::Satisfecho) > 0.0);
        assert!(SistemaAfetivo::calcular_valencia(&EmocionPrimitiva::Frustrado) < 0.0);
    }

    #[test]
    fn test_reflexion() {
        let mut sa = SistemaAfetivo::nuevo();
        sa.activar(EmocionPrimitiva::Illuminado, 0.9, 100);
        
        let reflex = sa.reflexion_emocional();
        assert!(reflex.contains("comprendido"));
    }
}
//! # Umbra: La Sombra Causal del Auton
//!
//! La Umbra es un grafo dirigido acíclico (DAG) que registra las decisiones
//! tomadas por un Auton y sus consecuencias en el Velo de Turing.
//!
//! ## Estructura
//!
//! Cada nodo de la Umbra contiene:
//! - `hash_estado`: Hash del estado sensorial que provocó la decisión
//! - `direccion_ramnet`: Dirección de memoria consultada
//! - `accion`: Acción tomada
//! - `resultado`: Hedonio (ganancia) o Algion (pérdida)
//! - `timestamp`: Tick en que ocurrió
//!
//! ## Resonancia
//!
//! Cuando dos Auton están cerca, sus Umbrae pueden "resonar" - compartir
//! nodos si el hash del estado es similar. Esto simula comunicación
//! no simbólica: conocimiento compartido sin símbolos.
//!
//! ## Serialización
//!
//! La Umbra puede serializarse a formato binario compacto para guardarse
//! en el Trazo Fundido cuando el Auton muere.
//!
//! ## Rol en el Sistema
//!
//! La Umbra es el puente entre la vida individual y la memoria colectiva
//! (Meltrace). Las Umbrae resonantes forman la base del Meltrace.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::life::ramnet::{Accion, TipoAccion};
use crate::physics::fixed_point::I32F32;
use std::collections::{HashMap, HashSet};
use std::vec::Vec;

/// Hash de estado sensorial (64 bits)
pub type HashEstado = u64;

/// Timestamp de tick
pub type Tick = u64;

/// Nodo del grafo Umbra
#[derive(Debug, Clone)]
pub struct NodoUmbra {
    /// Hash del estado sensorial
    pub hash_estado: HashEstado,
    /// Dirección RamNet consultada
    pub direccion_ramnet: usize,
    /// Acción tomada
    pub accion: Accion,
    /// Resultado de la acción
    pub resultado: ResultadoUmbra,
    /// Tick en que ocurrió
    pub tick: Tick,
    /// Nodos hijos (consecuencias)
    pub hijos: Vec<usize>,
    /// ID único del nodo
    pub id: usize,
    /// Profundidad en el grafo (distancia desde raíz)
    pub profundidad: usize,
}

impl NodoUmbra {
    pub fn nuevo(
        hash_estado: HashEstado,
        direccion_ramnet: usize,
        accion: Accion,
        resultado: ResultadoUmbra,
        tick: Tick,
        id: usize,
    ) -> Self {
        NodoUmbra {
            hash_estado,
            direccion_ramnet,
            accion,
            resultado,
            tick,
            hijos: Vec::new(),
            id,
            profundidad: 0,
        }
    }

    /// Calcula la "fuerza" del nodo basada en resultado
    pub fn fuerza(&self) -> I32F32 {
        match self.resultado {
            ResultadoUmbra::Hedonio(v) => v,
            ResultadoUmbra::Algion(v) => -v,
            ResultadoUmbra::Neutro => I32F32::ZERO,
        }
    }

    /// Verifica si es un nodo de ganancia
    pub fn es_hedonio(&self) -> bool {
        matches!(self.resultado, ResultadoUmbra::Hedonio(_))
    }

    /// Serializa a bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(48);

        // hash_estado: u64
        v.extend_from_slice(&self.hash_estado.to_le_bytes());

        // direccion_ramnet: u32
        v.extend_from_slice(&(self.direccion_ramnet as u32).to_le_bytes());

        // accion.tipo: u8
        v.push(self.accion.tipo.to_u8());
        // accion.magnitud: u8
        v.push(self.accion.magnitud);

        // resultado discriminant: u8
        match &self.resultado {
            ResultadoUmbra::Hedonio(val) => {
                v.push(0);
                v.extend_from_slice(&val.to_raw().to_le_bytes());
            }
            ResultadoUmbra::Algion(val) => {
                v.push(1);
                v.extend_from_slice(&val.to_raw().to_le_bytes());
            }
            ResultadoUmbra::Neutro => {
                v.push(2);
                v.extend_from_slice(&I32F32::ZERO.to_raw().to_le_bytes());
            }
        }

        // tick: u64
        v.extend_from_slice(&self.tick.to_le_bytes());

        // hijos: Vec<usize> - serializado como (len: u32, ids: u64[])
        v.extend_from_slice(&(self.hijos.len() as u32).to_le_bytes());
        for &h in &self.hijos {
            v.extend_from_slice(&(h as u64).to_le_bytes());
        }

        // profundidad: u32
        v.extend_from_slice(&(self.profundidad as u32).to_le_bytes());

        v
    }

    /// Deserializa desde bytes
    pub fn from_bytes(bytes: &[u8], id: usize) -> Option<(Self, usize)> {
        let mut pos = 0;

        // hash_estado: u64
        if bytes.len() < pos + 8 {
            return None;
        }
        let hash_estado = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        // direccion_ramnet: u32
        if bytes.len() < pos + 4 {
            return None;
        }
        let direccion_ramnet = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;

        // accion.tipo: u8
        if bytes.len() < pos + 1 {
            return None;
        }
        let tipo_val = bytes[pos];
        pos += 1;
        let accion_tipo = TipoAccion::from_u8(tipo_val);

        // accion.magnitud: u8
        if bytes.len() < pos + 1 {
            return None;
        }
        let magnitud = bytes[pos];
        pos += 1;
        let accion = Accion::nueva(accion_tipo, magnitud);

        // resultado
        if bytes.len() < pos + 1 {
            return None;
        }
        let resultado_disc = bytes[pos];
        pos += 1;

        if bytes.len() < pos + 8 {
            return None;
        }
        let val_raw = i64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;
        let resultado = match resultado_disc {
            0 => ResultadoUmbra::Hedonio(I32F32::from_raw(val_raw)),
            1 => ResultadoUmbra::Algion(I32F32::from_raw(val_raw)),
            _ => ResultadoUmbra::Neutro,
        };

        // tick: u64
        if bytes.len() < pos + 8 {
            return None;
        }
        let tick = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        // hijos len
        if bytes.len() < pos + 4 {
            return None;
        }
        let hijos_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;

        // hijos ids
        let mut hijos = Vec::with_capacity(hijos_len);
        for _ in 0..hijos_len {
            if bytes.len() < pos + 8 {
                return None;
            }
            let h = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?) as usize;
            pos += 8;
            hijos.push(h);
        }

        // profundidad
        if bytes.len() < pos + 4 {
            return None;
        }
        let profundidad = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;

        Some((
            NodoUmbra {
                hash_estado,
                direccion_ramnet,
                accion,
                resultado,
                tick,
                hijos,
                id,
                profundidad,
            },
            pos,
        ))
    }
}

/// Resultado de una acción en la Umbra
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultadoUmbra {
    /// Hedonio: ganancia de energía
    Hedonio(I32F32),
    /// Algion: pérdida de energía
    Algion(I32F32),
    /// Neutro: sin cambio
    Neutro,
}

impl ResultadoUmbra {
    pub fn es_positivo(&self) -> bool {
        matches!(self, ResultadoUmbra::Hedonio(_))
    }

    pub fn es_negativo(&self) -> bool {
        matches!(self, ResultadoUmbra::Algion(_))
    }

    pub fn magnitud(&self) -> I32F32 {
        match self {
            ResultadoUmbra::Hedonio(v) | ResultadoUmbra::Algion(v) => *v,
            ResultadoUmbra::Neutro => I32F32::ZERO,
        }
    }
}

/// Arco de conexión entre nodos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArcoUmbra {
    pub desde: usize,
    pub hacia: usize,
    pub peso: I32F32,
}

impl ArcoUmbra {
    pub fn nuevo(desde: usize, hacia: usize, peso: I32F32) -> Self {
        ArcoUmbra { desde, hacia, peso }
    }
}

/// Umbra: Grafo dirigido acíclico de decisiones
#[derive(Debug, Clone)]
pub struct Umbra {
    /// Nodos del grafo
    nodos: Vec<NodoUmbra>,
    /// Arco raíz (nodo inicial)
    raiz: Option<usize>,
    /// ID del Auton asociado
    id_auton: u64,
    /// Tick actual
    tick_actual: u64,
    /// Mapa de hash_estado -> id_nodo (para búsqueda rápida)
    indice_hash: HashMap<HashEstado, usize>,
    /// Mapa de nodos por profundidad
    nodos_por_profundidad: Vec<Vec<usize>>,
    /// Contador de próximos IDs
    proximo_id: usize,
    /// Nodos compartidos por resonancia
    nodos_compartidos: HashSet<usize>,
    /// Umbrae con las que ha resonado
    resonado_con: Vec<u64>, // IDs de otros Auton
    /// Profundidad máxima del grafo
    profundidad_max: usize,
}

impl Umbra {
    /// Crea nueva Umbra para un Auton
    pub fn nuevo(id_auton: u64) -> Self {
        Umbra {
            nodos: Vec::new(),
            raiz: None,
            id_auton,
            tick_actual: 0,
            indice_hash: HashMap::new(),
            nodos_por_profundidad: Vec::new(),
            proximo_id: 0,
            nodos_compartidos: HashSet::new(),
            resonado_con: Vec::new(),
            profundidad_max: 0,
        }
    }

    /// Registra una nueva decisión
    pub fn registrar_decision(
        &mut self,
        hash_estado: HashEstado,
        direccion_ramnet: usize,
        accion: Accion,
        resultado: ResultadoUmbra,
        hash_estado_padre: Option<HashEstado>,
    ) -> usize {
        let id = self.proximo_id;
        self.proximo_id += 1;

        let tick = self.tick_actual;

        // Calcular profundidad
        let profundidad = if let Some(hash_padre) = hash_estado_padre {
            if let Some(&id_padre) = self.indice_hash.get(&hash_padre) {
                self.nodos[id_padre].profundidad + 1
            } else {
                0
            }
        } else {
            0
        };

        // Crear nodo
        let mut nodo = NodoUmbra::nuevo(hash_estado, direccion_ramnet, accion, resultado, tick, id);
        nodo.profundidad = profundidad;

        // Expandir nodos_por_profundidad si es necesario
        while self.nodos_por_profundidad.len() <= profundidad {
            self.nodos_por_profundidad.push(Vec::new());
        }
        self.nodos_por_profundidad[profundidad].push(id);

        // Actualizar profundidad máxima
        if profundidad > self.profundidad_max {
            self.profundidad_max = profundidad;
        }

        // Si hay padre, conectar
        if let Some(hash_padre) = hash_estado_padre {
            if let Some(&id_padre) = self.indice_hash.get(&hash_padre) {
                self.nodos[id_padre].hijos.push(id);
            }
        } else if self.raiz.is_none() {
            // Es el nodo raíz
            self.raiz = Some(id);
        }

        // Agregar nodo
        self.nodos.push(nodo);

        // Actualizar índice
        self.indice_hash.insert(hash_estado, id);

        id
    }

    /// Obtiene nodo por ID
    pub fn nodo(&self, id: usize) -> Option<&NodoUmbra> {
        self.nodos.get(id)
    }

    /// Obtiene nodo por hash de estado
    pub fn nodo_por_hash(&self, hash: HashEstado) -> Option<&NodoUmbra> {
        self.indice_hash
            .get(&hash)
            .and_then(|&id| self.nodos.get(id))
    }

    /// Obtiene todos los nodos de una profundidad
    pub fn nodos_en_profundidad(&self, profundidad: usize) -> Vec<&NodoUmbra> {
        self.nodos_por_profundidad
            .get(profundidad)
            .map(|ids| ids.iter().filter_map(|&id| self.nodos.get(id)).collect())
            .unwrap_or_default()
    }

    /// Nodos con resultado hedonio (buenas decisiones)
    pub fn nodos_hedonio(&self) -> Vec<&NodoUmbra> {
        self.nodos.iter().filter(|n| n.es_hedonio()).collect()
    }

    /// Resonancia con otra Umbra: comparte nodos si los hashes son similares
    pub fn resonar(&mut self, otra: &Umbra, umbral_similitud: f64) -> Vec<(usize, usize)> {
        let mut compartidos = Vec::new();

        // Buscar nodos con hashes similares
        for (&hash, &id_self) in &self.indice_hash {
            for (&hash_otra, &id_otra) in &otra.indice_hash {
                let similitud = calcular_similitud_hash(hash, hash_otra);

                if similitud >= umbral_similitud {
                    // Marcar como compartidos
                    self.nodos_compartidos.insert(id_self);
                    compartidos.push((id_self, id_otra));

                    // Registrar que resonó con el otro Auton
                    if !self.resonado_con.contains(&otra.id_auton) {
                        self.resonado_con.push(otra.id_auton);
                    }
                }
            }
        }

        compartidos
    }

    /// Resonancia paralela: múltiples Umbrae
    pub fn resonar_multiple(
        &mut self,
        otras: &[&Umbra],
        umbral_similitud: f64,
    ) -> Vec<(usize, usize, u64)> {
        let mut todos_compartidos = Vec::new();

        for otra in otras {
            let compartidos = self.resonar(otra, umbral_similitud);
            for (id_self, id_otra) in compartidos {
                todos_compartidos.push((id_self, id_otra, otra.id_auton));
            }
        }

        todos_compartidos
    }

    /// Verifica si un nodo es compartido por resonancia
    pub fn es_compartido(&self, id_nodo: usize) -> bool {
        self.nodos_compartidos.contains(&id_nodo)
    }

    /// Obtiene todos los hashes de estados similares a uno dado
    pub fn estados_similares(&self, hash: HashEstado, umbral: f64) -> Vec<HashEstado> {
        self.indice_hash
            .keys()
            .filter(|&&h| calcular_similitud_hash(h, hash) >= umbral)
            .copied()
            .collect()
    }

    /// Calcula la "fuerza" acumulada de un camino
    pub fn fuerza_camino(&self, desde_id: usize, hasta_id: usize) -> I32F32 {
        if let (Some(desde), Some(hasta)) = (self.nodos.get(desde_id), self.nodos.get(hasta_id)) {
            self.fuerza_subgrafo(desde.profundidad, hasta.profundidad)
        } else {
            I32F32::ZERO
        }
    }

    /// Calcula fuerza total de un subgrafo por profundidad
    pub fn fuerza_subgrafo(&self, prof_inicio: usize, prof_fin: usize) -> I32F32 {
        let mut fuerza = I32F32::ZERO;

        for p in prof_inicio..=prof_fin.min(self.profundidad_max) {
            for &id in self.nodos_por_profundidad.get(p).unwrap_or(&Vec::new()) {
                if let Some(nodo) = self.nodos.get(id) {
                    fuerza = fuerza + nodo.fuerza();
                }
            }
        }

        fuerza
    }

    /// Serializa la Umbra a formato binario
    pub fn serializar(&self) -> Vec<u8> {
        let mut datos = Vec::new();

        // Header
        datos.extend_from_slice(b"UMBR"); // Magic
        datos.extend_from_slice(&1u8.to_le_bytes()); // Versión

        // ID Auton
        datos.extend_from_slice(&self.id_auton.to_le_bytes());

        // Num nodos
        datos.extend_from_slice(&(self.nodos.len() as u64).to_le_bytes());

        // Nodos
        for nodo in &self.nodos {
            datos.extend_from_slice(&nodo.to_bytes());
        }

        // Raíz
        match self.raiz {
            Some(r) => {
                datos.extend_from_slice(&1u8.to_le_bytes());
                datos.extend_from_slice(&(r as u64).to_le_bytes());
            }
            None => {
                datos.extend_from_slice(&0u8.to_le_bytes());
            }
        }

        // Nodos compartidos
        datos.extend_from_slice(&(self.nodos_compartidos.len() as u64).to_le_bytes());
        for &id in &self.nodos_compartidos {
            datos.extend_from_slice(&(id as u64).to_le_bytes());
        }

        // Resonado con
        datos.extend_from_slice(&(self.resonado_con.len() as u64).to_le_bytes());
        for &id in &self.resonado_con {
            datos.extend_from_slice(&id.to_le_bytes());
        }

        datos
    }

    /// Deserializa desde formato binario
    pub fn deserializar(datos: &[u8]) -> Option<Self> {
        let mut pos = 0;

        // Verificar magic
        if datos.len() < 8 {
            return None;
        }
        let magic = &datos[0..4];
        if magic != b"UMBR" {
            return None;
        }
        pos += 4;

        // Versión
        let version = datos[pos];
        pos += 1;
        if version != 1 {
            return None;
        }

        // ID Auton
        let id_auton = u64::from_le_bytes(datos[pos..pos + 8].try_into().ok()?);
        pos += 8;

        let mut umbra = Umbra::nuevo(id_auton);

        // Num nodos
        let num_nodos = u64::from_le_bytes(datos[pos..pos + 8].try_into().ok()?) as usize;
        pos += 8;

        // Nodos
        for i in 0..num_nodos {
            let (nodo, consumed) = NodoUmbra::from_bytes(&datos[pos..], i)?;
            pos += consumed;
            umbra.nodos.push(nodo);
            umbra.indice_hash.insert(umbra.nodos[i].hash_estado, i);
            umbra.proximo_id = i + 1;
        }

        // Raíz
        let tiene_raiz = datos[pos];
        pos += 1;
        if tiene_raiz == 1 {
            umbra.raiz = Some(u64::from_le_bytes(datos[pos..pos + 8].try_into().ok()?) as usize);
            pos += 8;
        }

        // Nodos compartidos
        let num_compartidos = u64::from_le_bytes(datos[pos..pos + 8].try_into().ok()?) as usize;
        pos += 8;
        for _ in 0..num_compartidos {
            let id = u64::from_le_bytes(datos[pos..pos + 8].try_into().ok()?) as usize;
            pos += 8;
            umbra.nodos_compartidos.insert(id);
        }

        // Resonado con
        let num_resonado = u64::from_le_bytes(datos[pos..pos + 8].try_into().ok()?) as usize;
        pos += 8;
        for _ in 0..num_resonado {
            let id_raw = u64::from_le_bytes(datos[pos..pos + 8].try_into().ok()?);
            pos += 8;
            umbra.resonado_con.push(id_raw);
        }

        Some(umbra)
    }

    /// Exporta a formato legible (para debugging)
    pub fn a_cadena(&self) -> String {
        let mut s = format!(
            "Umbra[Auton {}] {} nodos, profundidad max {}\n",
            self.id_auton,
            self.nodos.len(),
            self.profundidad_max
        );

        for (prof, nodos) in self.nodos_por_profundidad.iter().enumerate() {
            if !nodos.is_empty() {
                s.push_str(&format!("  Profundidad {}: ", prof));
                for &id in nodos {
                    if let Some(nodo) = self.nodos.get(id) {
                        let resultado = match &nodo.resultado {
                            ResultadoUmbra::Hedonio(_) => "+",
                            ResultadoUmbra::Algion(_) => "-",
                            ResultadoUmbra::Neutro => "0",
                        };
                        let compartido = if self.es_compartido(id) { "*" } else { "" };
                        s.push_str(&format!(
                            "[{}→{:?}{}{}]",
                            resultado,
                            nodo.accion.tipo,
                            nodo.hash_estado % 1000,
                            compartido
                        ));
                    }
                }
                s.push('\n');
            }
        }

        if !self.resonado_con.is_empty() {
            s.push_str(&format!("  Resonó con: {:?}\n", self.resonado_con));
        }

        s
    }

    /// Getters
    pub fn num_nodos(&self) -> usize {
        self.nodos.len()
    }

    pub fn nodos(&self) -> &Vec<NodoUmbra> {
        &self.nodos
    }

    pub fn id_auton(&self) -> u64 {
        self.id_auton
    }

    pub fn tick(&self) -> Tick {
        self.tick_actual
    }

    pub fn raiz(&self) -> Option<usize> {
        self.raiz
    }

    pub fn profundidad_max(&self) -> usize {
        self.profundidad_max
    }

    pub fn nodos_compartidos_count(&self) -> usize {
        self.nodos_compartidos.len()
    }

    /// Avanza el tick
    pub fn tick_actualizar(&mut self) {
        self.tick_actual += 1;
    }

    /// Fuerza total del grafo
    pub fn fuerza_total(&self) -> I32F32 {
        let mut total = I32F32::ZERO;
        for nodo in &self.nodos {
            total = total + nodo.fuerza();
        }
        total
    }

    /// Número total de arcos (conexiones entre nodos)
    pub fn num_arcos(&self) -> usize {
        self.nodos.iter().map(|n| n.hijos.len()).sum()
    }

    /// Obtiene todos los hashes de estado en el grafo
    pub fn obtener_hashes_estado(&self) -> Vec<HashEstado> {
        self.nodos.iter().map(|n| n.hash_estado).collect()
    }

    /// Cuenta circuitos de longitud n (n nodos que forman un ciclo)
    /// Un circuito de longitud n es: nodo A -> ... -> A (vuelve al inicio)
    pub fn contar_circuitos_longitud(&self, longitud: usize) -> usize {
        if longitud < 2 || self.nodos.len() < longitud {
            return 0;
        }

        let mut conteo = 0;
        let n = self.nodos.len();

        // Para cada nodo como inicio
        for inicio in 0..n {
            let mut visitados = vec![false; n];
            let mut camino = Vec::new();

            // DFS desde el nodo inicio
            fn dfs(
                actual: usize,
                objetivo: usize,
                pasos: usize,
                longitud: usize,
                visitados: &mut Vec<bool>,
                camino: &mut Vec<usize>,
                nodos: &[NodoUmbra],
            ) -> bool {
                if pasos == longitud {
                    return actual == objetivo;
                }

                visitados[actual] = true;
                camino.push(actual);

                for &hijo in &nodos[actual].hijos {
                    if !visitados[hijo] {
                        if dfs(
                            hijo,
                            objetivo,
                            pasos + 1,
                            longitud,
                            visitados,
                            camino,
                            nodos,
                        ) {
                            camino.pop();
                            visitados[actual] = false;
                            return true;
                        }
                    }
                }

                camino.pop();
                visitados[actual] = false;
                false
            }

            if dfs(
                inicio,
                inicio,
                0,
                longitud,
                &mut visitados,
                &mut camino,
                &self.nodos,
            ) {
                conteo += 1;
            }
        }

        conteo
    }
}

/// Calcula similitud entre dos hashes (0.0 a 1.0)
fn calcular_similitud_hash(h1: u64, h2: u64) -> f64 {
    if h1 == h2 {
        return 1.0;
    }

    // Hamming distance normalizada
    let xor = h1 ^ h2;
    let mut bits_diferentes = 0u32;

    let mut x = xor;
    while x != 0 {
        bits_diferentes += 1;
        x &= x - 1;
    }

    let similitud = 1.0 - (bits_diferentes as f64 / 64.0);
    similitud.max(0.0)
}

/// Tipo de fusión para Meltrace
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoFusion {
    /// Fusión por promedio
    Promedio,
    /// Fusión por máxima fuerza
    FuerzaMax,
    /// Fusión por resonancia
    Resonancia,
}

/// Estadísticas de Umbra
#[derive(Debug, Clone)]
pub struct UmbraStats {
    pub num_nodos: usize,
    pub profundidad_max: usize,
    pub num_hedonios: usize,
    pub num_algiones: usize,
    pub nodos_compartidos: usize,
    pub fuerza_total: I32F32,
    pub tamano_bytes: usize,
}

impl Umbra {
    pub fn estadisticas(&self) -> UmbraStats {
        let serializado = self.serializar();
        UmbraStats {
            num_nodos: self.nodos.len(),
            profundidad_max: self.profundidad_max,
            num_hedonios: self.nodos.iter().filter(|n| n.es_hedonio()).count(),
            num_algiones: self
                .nodos
                .iter()
                .filter(|n| matches!(n.resultado, ResultadoUmbra::Algion(_)))
                .count(),
            nodos_compartidos: self.nodos_compartidos.len(),
            fuerza_total: self.fuerza_total(),
            tamano_bytes: serializado.len(),
        }
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_umbra() {
        let umbra = Umbra::nuevo(1);
        assert_eq!(umbra.id_auton(), 1);
        assert_eq!(umbra.num_nodos(), 0);
        assert!(umbra.raiz().is_none());
    }

    #[test]
    fn test_registrar_decision() {
        let mut umbra = Umbra::nuevo(1);
        let accion = Accion::nueva(TipoAccion::MoverX, 128);

        let id = umbra.registrar_decision(
            0x1234,
            10,
            accion,
            ResultadoUmbra::Hedonio(I32F32::ONE),
            None,
        );

        assert_eq!(id, 0);
        assert_eq!(umbra.num_nodos(), 1);
        assert_eq!(umbra.raiz(), Some(0));
    }

    #[test]
    fn test_decision_con_padre() {
        let mut umbra = Umbra::nuevo(1);
        let accion = Accion::nueva(TipoAccion::MoverY, 64);

        // Primera decisión
        let id1 = umbra.registrar_decision(
            0x1000,
            5,
            accion,
            ResultadoUmbra::Hedonio(I32F32::ONE),
            None,
        );

        // Segunda decisión con padre
        let id2 = umbra.registrar_decision(
            0x2000,
            15,
            accion,
            ResultadoUmbra::Algion(I32F32::ONE),
            Some(0x1000),
        );

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);

        // Verificar que el padre tiene hijo
        let padre = umbra.nodo(id1).unwrap();
        assert!(padre.hijos.contains(&id2));
    }

    #[test]
    fn test_nodo_por_hash() {
        let mut umbra = Umbra::nuevo(1);
        let hash = 0xABCD;
        let accion = Accion::nueva(TipoAccion::Nop, 0);

        umbra.registrar_decision(hash, 10, accion, ResultadoUmbra::Neutro, None);

        let nodo = umbra.nodo_por_hash(hash);
        assert!(nodo.is_some());
        assert_eq!(nodo.unwrap().hash_estado, hash);
    }

    #[test]
    fn test_nodos_por_profundidad() {
        let mut umbra = Umbra::nuevo(1);
        let accion = Accion::nueva(TipoAccion::Nop, 0);

        // Crear decisiones en cadena
        umbra.registrar_decision(
            0x1000,
            1,
            accion.clone(),
            ResultadoUmbra::Hedonio(I32F32::ONE),
            None,
        );
        umbra.registrar_decision(
            0x2000,
            2,
            accion.clone(),
            ResultadoUmbra::Hedonio(I32F32::ONE),
            Some(0x1000),
        );
        umbra.registrar_decision(
            0x3000,
            3,
            accion.clone(),
            ResultadoUmbra::Hedonio(I32F32::ONE),
            Some(0x2000),
        );

        assert_eq!(umbra.nodos_en_profundidad(0).len(), 1);
        assert_eq!(umbra.nodos_en_profundidad(1).len(), 1);
        assert_eq!(umbra.nodos_en_profundidad(2).len(), 1);
        assert!(umbra.nodos_en_profundidad(3).is_empty());
    }

    #[test]
    fn test_resonancia() {
        let mut umbra1 = Umbra::nuevo(1);
        let mut umbra2 = Umbra::nuevo(2);
        let accion = Accion::nueva(TipoAccion::MoverX, 100);

        // Hashes muy similares
        let hash1 = 0xFFFF_0000_0000_0000;
        let hash2 = 0xFFFF_0000_0000_00FF; // Solo difiere en 8 bits

        umbra1.registrar_decision(
            hash1,
            10,
            accion.clone(),
            ResultadoUmbra::Hedonio(I32F32::ONE),
            None,
        );
        umbra2.registrar_decision(
            hash2,
            10,
            accion.clone(),
            ResultadoUmbra::Hedonio(I32F32::ONE),
            None,
        );

        let compartidos = umbra1.resonar(&umbra2, 0.85);

        // Deben compartir algo con 85% de similitud
        assert!(!compartidos.is_empty() || hash1 == hash2);
    }

    #[test]
    fn test_serializacion() {
        let mut umbra = Umbra::nuevo(42);
        let accion = Accion::nueva(TipoAccion::MoverX, 128);

        umbra.registrar_decision(
            0x1234,
            10,
            accion,
            ResultadoUmbra::Hedonio(I32F32::ONE),
            None,
        );
        umbra.tick_actualizar();

        let datos = umbra.serializar();
        // Verificar que serialización produce datos
        assert!(
            datos.len() > 50,
            "Serialización debería producir datos significativos"
        );
    }

    #[test]
    fn test_fuerza_nodo() {
        let nodo = NodoUmbra::nuevo(
            0x1234,
            10,
            Accion::nueva(TipoAccion::MoverX, 100),
            ResultadoUmbra::Hedonio(I32F32::from_i32(5)),
            100,
            0,
        );

        assert!(nodo.es_hedonio());
        assert_eq!(nodo.fuerza(), I32F32::from_i32(5));
    }

    #[test]
    fn test_fuerza_total() {
        let mut umbra = Umbra::nuevo(1);
        let accion = Accion::nueva(TipoAccion::Nop, 0);

        umbra.registrar_decision(
            0x1000,
            1,
            accion.clone(),
            ResultadoUmbra::Hedonio(I32F32::from_i32(10)),
            None,
        );
        umbra.registrar_decision(
            0x2000,
            2,
            accion.clone(),
            ResultadoUmbra::Algion(I32F32::from_i32(3)),
            None,
        );
        umbra.registrar_decision(
            0x3000,
            3,
            accion.clone(),
            ResultadoUmbra::Hedonio(I32F32::from_i32(5)),
            None,
        );

        // 10 - 3 + 5 = 12
        assert!(umbra.fuerza_total() > I32F32::from_i32(11));
    }

    #[test]
    fn test_similitud_hash() {
        let h1 = 0xFFFF_FFFF_FFFF_FFFF;
        let h2 = 0xFFFF_FFFF_FFFF_FFF0;

        let sim = calcular_similitud_hash(h1, h2);
        // 60 bits iguales de 64 = 0.9375
        assert!(sim > 0.9 && sim < 1.0);

        // Iguales
        assert_eq!(calcular_similitud_hash(0x1234, 0x1234), 1.0);

        // Opuestos
        let sim_op = calcular_similitud_hash(0xAAAA_AAAA_AAAA_AAAA, 0x5555_5555_5555_5555);
        assert!(
            sim_op >= 0.0 && sim_op <= 0.2,
            "Opuestos deben tener similitud baja, got {}",
            sim_op
        );
    }

    #[test]
    fn test_estadisticas() {
        let mut umbra = Umbra::nuevo(1);
        let accion = Accion::nueva(TipoAccion::Nop, 0);

        umbra.registrar_decision(
            0x1000,
            1,
            accion.clone(),
            ResultadoUmbra::Hedonio(I32F32::ONE),
            None,
        );
        umbra.registrar_decision(
            0x2000,
            2,
            accion.clone(),
            ResultadoUmbra::Algion(I32F32::ONE),
            None,
        );

        let stats = umbra.estadisticas();
        assert_eq!(stats.num_nodos, 2);
        assert_eq!(stats.num_hedonios, 1);
        assert_eq!(stats.num_algiones, 1);
        assert!(stats.fuerza_total < I32F32::ONE); // 1 - 1 = 0
    }

    #[test]
    fn test_nodos_hedonio() {
        let mut umbra = Umbra::nuevo(1);
        let accion = Accion::nueva(TipoAccion::Nop, 0);

        umbra.registrar_decision(
            0x1000,
            1,
            accion.clone(),
            ResultadoUmbra::Hedonio(I32F32::ONE),
            None,
        );
        umbra.registrar_decision(
            0x2000,
            2,
            accion.clone(),
            ResultadoUmbra::Algion(I32F32::ONE),
            None,
        );
        umbra.registrar_decision(
            0x3000,
            3,
            accion.clone(),
            ResultadoUmbra::Hedonio(I32F32::ONE),
            None,
        );

        let hedonios = umbra.nodos_hedonio();
        assert_eq!(hedonios.len(), 2);
    }
}

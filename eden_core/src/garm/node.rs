//! GARM Node Trait — Cada capability es un nodo vivo con metabolismo propio

use std::any::Any;

/// Escala temporal de firing del nodo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalScale {
    Fast,         // ms — reflejos (gate, seguridad, veto)
    Deliberative, // s-min — cognicion (plan, lenguaje, razonamiento)
    Evolutionary, // horas-dias — evolucion arquitectonica
}

/// Contexto que recibe cada nodo antes de predecir/actuar.
#[derive(Clone, Debug)]
pub struct NodeContext {
    pub tick: u64,
    pub global_free_energy: f32,
    pub neighbor_outputs: Vec<(usize, Vec<f32>)>,
    pub sensor_input: Vec<f32>,
    pub ambient_energy: f32, // energia disponible en el bus para este nodo
}

/// Accion que un nodo puede ejecutar tras actuar.
#[derive(Clone, Debug)]
pub enum NodeAction {
    None,
    Output(Vec<f32>),
    SendMessage(usize, Vec<f32>), // (target_node_id, payload)
    RequestEnergy(f32),
    /// Solicita al HyperGraph spawnear un nuevo nodo.
    /// (node_type, id_sugerido)
    SpawnNode(String, usize),
    /// Solicita al HyperGraph matar un nodo existente.
    KillNode(usize),
}

/// Trait base de todo nodo GARM.
/// Cada nodo es una maquina de minimizacion de energia libre.
pub trait GARMNode: Send + Any {
    fn id(&self) -> usize;
    fn name(&self) -> &str;
    fn scale(&self) -> TemporalScale;

    /// Energia libre variacional del nodo.
    /// Cuanto mas alto, mas sorpresa/incertidumbre tiene.
    fn free_energy(&self) -> f32;

    /// Predice la proxima entrada dado el contexto.
    fn predict(&mut self, ctx: &NodeContext) -> Vec<f32>;

    /// Recibe el error de prediccion y produce una accion.
    fn act(&mut self, ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction;

    /// Actualiza estado interno y consume energia.
    /// Devuelve la energia realmente consumida.
    fn update(&mut self, dt: f32, energy_in: f32) -> f32;

    fn is_alive(&self) -> bool;
    fn spawn_cost(&self) -> f32;

    /// Para downcasting.
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Envoltorio tipo-erased para nodos heterogeneos en vectores.
pub struct NodeBox {
    pub inner: Box<dyn GARMNode>,
}

impl NodeBox {
    pub fn new<N: GARMNode + 'static>(node: N) -> Self {
        NodeBox {
            inner: Box::new(node),
        }
    }
}

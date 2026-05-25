use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum Request {
    #[serde(rename = "create_node")]
    CreateNode {
        label: String,
        properties: HashMap<String, serde_json::Value>,
        ttl: Option<i64>,
        embeddings: Option<Vec<f32>>,
    },
    #[serde(rename = "delete_node")]
    DeleteNode { id: String },
    #[serde(rename = "create_edge")]
    CreateEdge {
        source: String,
        target: String,
        relation: String,
    },
    #[serde(rename = "get_node")]
    GetNode { id: String },
    #[serde(rename = "get_stale_nodes")]
    GetStaleNodes { days: i64 },
    #[serde(rename = "search_similar")]
    SearchSimilar { embedding: Vec<f32>, top_k: usize },
    #[serde(rename = "store_evolution")]
    StoreEvolution {
        generacion: i64,
        mutacion_id: String,
        patron_origen: String,
        resultado: String,
        peso_final: f64,
        sobrevivio: bool,
        timestamp: i64,
    },
    #[serde(rename = "get_evolution")]
    GetEvolution { mutacion_id: String },
    #[serde(rename = "get_all_evolution")]
    GetAllEvolution {},
    #[serde(rename = "evento_vital")]
    EventoVital {
        tipo: String,
        accion: String,
        ram_libre: u64,
        cpu_pct: f64,
        timestamp: String,
    },
    #[serde(rename = "ping")]
    Ping {},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum Response {
    #[serde(rename = "ok")]
    Ok { data: serde_json::Value },
    #[serde(rename = "error")]
    Error { error: String, code: u16 },
}

impl Response {
    pub fn ok<T: serde::Serialize>(data: T) -> Self {
        Response::Ok {
            data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
        }
    }

    pub fn error(msg: impl Into<String>, code: u16) -> Self {
        Response::Error {
            error: msg.into(),
            code,
        }
    }
}

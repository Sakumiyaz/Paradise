//! # EDEN IPC Types - Tipos de comunicación entre procesos
//!
//! CellRequest y CellResponse para el protocolo socket.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use eden_serialize::{Serialize, Deserialize, JsonValue, JsonDeserializeError};

#[derive(Debug, Clone)]
pub struct CellRequest {
    pub input: Option<String>,
    pub contexto: Option<String>,
    pub action: Option<String>,
}

impl Serialize for CellRequest {
    fn to_json(&self) -> JsonValue {
        let mut m = HashMap::new();
        if let Some(v) = &self.input { m.insert("input".to_string(), v.to_json()); }
        if let Some(v) = &self.contexto { m.insert("contexto".to_string(), v.to_json()); }
        if let Some(v) = &self.action { m.insert("action".to_string(), v.to_json()); }
        JsonValue::Object(m)
    }
}

impl Deserialize for CellRequest {
    fn from_json(value: &JsonValue) -> Result<Self, JsonDeserializeError> {
        match value {
            JsonValue::Object(m) => Ok(CellRequest {
                input: m.get("input").map(String::from_json).transpose()?,
                contexto: m.get("contexto").map(String::from_json).transpose()?,
                action: m.get("action").map(String::from_json).transpose()?,
            }),
            _ => Err(JsonDeserializeError::TypeMismatch("Expected object".to_string())),
        }
    }
}

impl Default for CellRequest {
    fn default() -> Self {
        CellRequest { input: None, contexto: None, action: None }
    }
}

#[derive(Debug, Clone)]
pub struct CellResponse {
    pub respuesta: String,
    pub confianza: f32,
    pub energia: i32,
    pub ciclos: u64,
}

impl Serialize for CellResponse {
    fn to_json(&self) -> JsonValue {
        let mut m = HashMap::new();
        m.insert("respuesta".to_string(), self.respuesta.to_json());
        m.insert("confianza".to_string(), self.confianza.to_json());
        m.insert("energia".to_string(), self.energia.to_json());
        m.insert("ciclos".to_string(), self.ciclos.to_json());
        JsonValue::Object(m)
    }
}

impl Deserialize for CellResponse {
    fn from_json(value: &JsonValue) -> Result<Self, JsonDeserializeError> {
        match value {
            JsonValue::Object(m) => Ok(CellResponse {
                respuesta: String::from_json(m.get("respuesta").ok_or(JsonDeserializeError::MissingField("respuesta".to_string()))?)?,
                confianza: f32::from_json(m.get("confianza").ok_or(JsonDeserializeError::MissingField("confianza".to_string()))?)?,
                energia: i32::from_json(m.get("energia").ok_or(JsonDeserializeError::MissingField("energia".to_string()))?)?,
                ciclos: u64::from_json(m.get("ciclos").ok_or(JsonDeserializeError::MissingField("ciclos".to_string()))?)?,
            }),
            _ => Err(JsonDeserializeError::TypeMismatch("Expected object".to_string())),
        }
    }
}

impl Default for CellResponse {
    fn default() -> Self {
        CellResponse { respuesta: String::new(), confianza: 0.0, energia: 0, ciclos: 0 }
    }
}
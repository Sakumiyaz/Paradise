//! # Knowledge - Knowledge graph for internet data
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

static KNOWLEDGE_ID_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Entry en el knowledge graph
#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    pub id: String,
    pub entity_type: EntityType,
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub relations: Vec<Relation>,
    pub sources: Vec<String>,
    pub confidence: f32,
    pub created_at: u64,
    pub last_updated: u64,
}

impl KnowledgeEntry {
    pub fn new(name: String, entity_type: EntityType) -> Self {
        let now = current_time_ms();
        let sequence = KNOWLEDGE_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        Self {
            id: format!("{:x}-{:x}", now, sequence),
            entity_type,
            name,
            attributes: HashMap::new(),
            relations: Vec::new(),
            sources: Vec::new(),
            confidence: 0.5,
            created_at: now,
            last_updated: now,
        }
    }

    pub fn add_relation(&mut self, relation: Relation) {
        self.relations.push(relation);
    }

    pub fn add_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
        self.last_updated = current_time_ms();
    }

    pub fn add_source(&mut self, url: &str) {
        if !self.sources.contains(&url.to_string()) {
            self.sources.push(url.to_string());
        }
    }

    pub fn update_confidence(&mut self, delta: f32) {
        self.confidence = (self.confidence + delta).clamp(0.0, 1.0);
    }
}

/// Tipo de entidad
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Concept,
    Event,
    Technology,
    Other,
}

impl EntityType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "person" | "people" => EntityType::Person,
            "organization" | "org" | "company" => EntityType::Organization,
            "location" | "place" => EntityType::Location,
            "concept" | "idea" => EntityType::Concept,
            "event" => EntityType::Event,
            "technology" | "tech" => EntityType::Technology,
            _ => EntityType::Other,
        }
    }
}

/// Relación entre entidades
#[derive(Debug, Clone)]
pub struct Relation {
    pub target_id: String,
    pub relation_type: RelationType,
    pub strength: f32,
}

#[derive(Debug, Clone)]
pub enum RelationType {
    IsA,
    PartOf,
    WorksFor,
    LocatedIn,
    RelatedTo,
    CreatedBy,
    Mentions,
    Other,
}

impl RelationType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "is_a" | "isa" | "type" => RelationType::IsA,
            "part_of" | "partof" => RelationType::PartOf,
            "works_for" | "worksfor" => RelationType::WorksFor,
            "located_in" | "locatedin" => RelationType::LocatedIn,
            "related_to" | "relatedto" => RelationType::RelatedTo,
            "created_by" | "createdby" => RelationType::CreatedBy,
            "mentions" | "mentioned" => RelationType::Mentions,
            _ => RelationType::Other,
        }
    }
}

/// Knowledge graph principal
pub struct KnowledgeGraph {
    entities: HashMap<String, KnowledgeEntry>,
    name_index: HashMap<String, String>, // name -> id
    type_index: HashMap<EntityType, Vec<String>>,
    graph: HashMap<String, Vec<String>>, // id -> related ids
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            name_index: HashMap::new(),
            type_index: HashMap::new(),
            graph: HashMap::new(),
        }
    }

    /// Añade una entidad
    pub fn add_entity(&mut self, entry: KnowledgeEntry) -> String {
        let id = entry.id.clone();

        // Add to indices
        self.name_index
            .insert(entry.name.to_lowercase(), id.clone());
        self.type_index
            .entry(entry.entity_type.clone())
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Add to graph
        self.graph.insert(id.clone(), Vec::new());

        self.entities.insert(id.clone(), entry);
        id
    }

    /// Obtiene entidad por nombre
    pub fn get_by_name(&self, name: &str) -> Option<&KnowledgeEntry> {
        self.name_index
            .get(&name.to_lowercase())
            .and_then(|id| self.entities.get(id))
    }

    /// Obtiene entidad por ID
    pub fn get_by_id(&self, id: &str) -> Option<&KnowledgeEntry> {
        self.entities.get(id)
    }

    /// Busca entidades por tipo
    pub fn get_by_type(&self, entity_type: EntityType) -> Vec<&KnowledgeEntry> {
        self.type_index
            .get(&entity_type)
            .map(|ids| ids.iter().filter_map(|id| self.entities.get(id)).collect())
            .unwrap_or_default()
    }

    /// Conecta dos entidades
    pub fn connect(
        &mut self,
        source_id: &str,
        target_id: &str,
        relation: RelationType,
        strength: f32,
    ) {
        if let Some(source) = self.entities.get_mut(source_id) {
            source.add_relation(Relation {
                target_id: target_id.to_string(),
                relation_type: relation,
                strength,
            });
        }

        self.graph
            .entry(source_id.to_string())
            .or_insert_with(Vec::new)
            .push(target_id.to_string());
    }

    /// BFS traversal - encuentra entidades relacionadas
    pub fn find_connected(&self, start_id: &str, max_depth: usize) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back((start_id.to_string(), 0));
        visited.insert(start_id.to_string());

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            if let Some(neighbors) = self.graph.get(&current_id) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        result.push(neighbor.clone());
                        queue.push_back((neighbor.clone(), depth + 1));
                    }
                }
            }
        }

        result
    }

    /// Merge información de múltiples fuentes
    pub fn merge(&mut self, source_url: &str, info: &str) -> usize {
        let mut merged = 0;

        // Simple extraction: look for patterns like "X is Y" or "X works at Z"
        let lines: Vec<&str> = info.split('.').filter(|l| !l.trim().is_empty()).collect();

        for line in lines {
            let line_lower = line.to_lowercase();

            // Pattern: "X is a Y"
            if let Some(pos) = line_lower.find(" is a ") {
                let name = line[..pos].trim().to_string();
                let entity_type = line[pos + 6..]
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("other");

                if !self.name_index.contains_key(&name.to_lowercase()) {
                    let mut entry =
                        KnowledgeEntry::new(name.clone(), EntityType::from_str(entity_type));
                    entry.add_source(source_url);
                    entry.confidence = 0.6;
                    self.add_entity(entry);
                    merged += 1;
                }
            }

            // Pattern: "X works at Y"
            if let Some(pos) = line_lower.find(" works at ") {
                let person = line[..pos].trim().to_string();
                let org = line[pos + 10..]
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("");

                let person_lower = person.to_lowercase();
                let org_lower = org.to_lowercase();
                if let Some(person_id) = self.name_index.get(&person_lower).cloned() {
                    if let Some(org_id) = self.name_index.get(&org_lower).cloned() {
                        self.connect(&person_id, &org_id, RelationType::WorksFor, 0.7);
                        merged += 1;
                    }
                }
            }
        }

        merged
    }

    /// Get statistics
    pub fn stats(&self) -> KnowledgeStats {
        let mut type_counts = HashMap::new();
        for (et, ids) in &self.type_index {
            type_counts.insert(format!("{:?}", et), ids.len());
        }

        KnowledgeStats {
            entity_count: self.entities.len(),
            relation_count: self.entities.values().map(|e| e.relations.len()).sum(),
            type_counts,
        }
    }

    /// Export to JSON format
    pub fn export_json(&self) -> String {
        let mut json = String::from("{\"entities\": [");

        let entities: Vec<_> = self.entities.values().collect();
        for (i, entity) in entities.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(
                r#"{{"id":"{}","name":"{}","type":"{:?}","attrs":{{}}"#,
                entity.id, entity.name, entity.entity_type
            ));
            // Add attributes
            for (k, v) in &entity.attributes {
                json.push_str(&format!(r#",{}":"{}""#, k, v));
            }
            json.push_str("}");
        }

        json.push_str("]}");
        json
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas del knowledge graph
#[derive(Debug, Clone)]
pub struct KnowledgeStats {
    pub entity_count: usize,
    pub relation_count: usize,
    pub type_counts: HashMap<String, usize>,
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed::TaskStatus;

    #[test]
    fn test_knowledge_graph() {
        let mut kg = KnowledgeGraph::new();

        let person_id = {
            let mut person = KnowledgeEntry::new("John".to_string(), EntityType::Person);
            person.add_attribute("role", "developer");
            kg.add_entity(person)
        };

        let org_id = {
            let org = KnowledgeEntry::new("Acme Corp".to_string(), EntityType::Organization);
            kg.add_entity(org)
        };

        kg.connect(&person_id, &org_id, RelationType::WorksFor, 0.9);

        let john = kg.get_by_name("John").unwrap();
        assert_eq!(john.name, "John");
        assert_eq!(john.relations.len(), 1);
    }
}

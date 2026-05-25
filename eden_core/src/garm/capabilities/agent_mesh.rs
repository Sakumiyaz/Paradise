// EDEN GARM AgentMesh — Protocolo multi-agente entre instancias EDEN.
// 100% Rust puro, 0 LLM, 0 red.
//
// Implementacion en-memoria (sin red): cada agente tiene un inbox/outbox.
// Mensajes son serializables (claves de concepto, predicciones, observaciones).
// Para distribucion real, los mensajes se exportarian/importarian via JSON.

use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
pub enum MessageKind {
    /// Compartir un concepto aprendido (cause, effect)
    CausalFact,
    /// Solicitar info sobre un concepto
    Query,
    /// Responder a un Query
    Response,
    /// Compartir prediccion
    Prediction,
    /// Heartbeat / sincronizacion
    Heartbeat,
}

#[derive(Clone, Debug)]
pub struct AgentMessage {
    pub from_agent: String,
    pub to_agent: String,
    pub kind: MessageKind,
    pub content: String,
    pub tick: u64,
    pub confidence: f32,
}

#[derive(Clone, Debug)]
pub struct PeerStats {
    pub agent_id: String,
    pub messages_received: u64,
    pub messages_sent: u64,
    pub last_heartbeat_tick: u64,
    pub agreement_score: f32, // 0..1, how often we agree with this peer
}

#[derive(Clone, Debug)]
pub struct AgentMesh {
    pub agent_id: String,
    pub inbox: VecDeque<AgentMessage>,
    pub outbox: VecDeque<AgentMessage>,
    pub peers: Vec<PeerStats>,
    pub max_inbox: usize,
    pub max_outbox: usize,
    pub n_received: u64,
    pub n_sent: u64,
    pub n_processed: u64,
}

impl AgentMesh {
    pub fn new(agent_id: &str) -> Self {
        AgentMesh {
            agent_id: agent_id.to_string(),
            inbox: VecDeque::new(),
            outbox: VecDeque::new(),
            peers: Vec::new(),
            max_inbox: 200,
            max_outbox: 100,
            n_received: 0,
            n_sent: 0,
            n_processed: 0,
        }
    }

    /// Register a peer for coordination.
    pub fn add_peer(&mut self, agent_id: &str) {
        if !self.peers.iter().any(|p| p.agent_id == agent_id) {
            self.peers.push(PeerStats {
                agent_id: agent_id.to_string(),
                messages_received: 0,
                messages_sent: 0,
                last_heartbeat_tick: 0,
                agreement_score: 0.5,
            });
        }
    }

    /// Compose a message and put it in outbox to be delivered.
    pub fn send(&mut self, to: &str, kind: MessageKind, content: &str, tick: u64, confidence: f32) {
        let msg = AgentMessage {
            from_agent: self.agent_id.clone(),
            to_agent: to.to_string(),
            kind,
            content: content.to_string(),
            tick,
            confidence,
        };
        self.outbox.push_back(msg);
        if self.outbox.len() > self.max_outbox {
            self.outbox.pop_front();
        }
        self.n_sent += 1;
        if let Some(p) = self.peers.iter_mut().find(|p| p.agent_id == to) {
            p.messages_sent += 1;
        }
    }

    /// Simulate receiving a message (in real distributed system this would come from network).
    pub fn receive(&mut self, msg: AgentMessage) {
        if self.inbox.len() >= self.max_inbox {
            self.inbox.pop_front();
        }
        if let Some(p) = self.peers.iter_mut().find(|p| p.agent_id == msg.from_agent) {
            p.messages_received += 1;
            if msg.kind == MessageKind::Heartbeat {
                p.last_heartbeat_tick = msg.tick;
            }
        }
        self.inbox.push_back(msg);
        self.n_received += 1;
    }

    /// Process one inbox message and produce a response if applicable.
    pub fn process_one(&mut self) -> Option<AgentMessage> {
        let msg = self.inbox.pop_front()?;
        self.n_processed += 1;
        match msg.kind {
            MessageKind::Query => {
                // In a real system we'd look up the query in our concept graph
                // and respond. Here we just acknowledge.
                let response = AgentMessage {
                    from_agent: self.agent_id.clone(),
                    to_agent: msg.from_agent.clone(),
                    kind: MessageKind::Response,
                    content: format!("ack:{}", msg.content),
                    tick: msg.tick,
                    confidence: 0.5,
                };
                Some(response)
            }
            MessageKind::Heartbeat => None,
            _ => None,
        }
    }

    /// Update agreement score for a peer based on whether their fact matches our own.
    pub fn update_agreement(&mut self, peer_id: &str, agree: bool) {
        if let Some(p) = self.peers.iter_mut().find(|p| p.agent_id == peer_id) {
            let alpha = 0.1;
            let target = if agree { 1.0 } else { 0.0 };
            p.agreement_score = p.agreement_score * (1.0 - alpha) + target * alpha;
        }
    }

    pub fn status(&self) -> String {
        format!(
            "AgentMesh '{}' | peers={} | inbox={} | outbox={} | sent={} received={} processed={}",
            self.agent_id,
            self.peers.len(),
            self.inbox.len(),
            self.outbox.len(),
            self.n_sent,
            self.n_received,
            self.n_processed,
        )
    }

    pub fn report_peers(&self) -> String {
        if self.peers.is_empty() {
            return "Sin peers registrados".to_string();
        }
        let mut out = format!("Peers de '{}':\n", self.agent_id);
        for p in &self.peers {
            out.push_str(&format!(
                "  {} | sent={} received={} | last_heartbeat={} | agreement={:.2}\n",
                p.agent_id,
                p.messages_sent,
                p.messages_received,
                p.last_heartbeat_tick,
                p.agreement_score,
            ));
        }
        out
    }
}

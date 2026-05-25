// paradigms/distributed.rs — UDP gossip decentralized multi-node training
// Each EDEN instance broadcasts ParadigmSignals to peers, forwards received messages,
// and deduplicates via message hash. No master node, fully symmetric.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Gossip message: key paradigm signals broadcast to peers
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GossipMessage {
    pub node_id: String,
    pub cycle: u64,
    pub msg_hash: u64, // for deduplication
    pub edge_trust: Vec<((usize, usize), f32)>,
    pub source_scores: Vec<(String, f32)>,
    pub crawl_recommendations: Vec<(String, f32)>,
    pub novel_edges: Vec<(usize, usize, f32)>,
    pub sleep_recommendation: f32,
    pub prune_threshold: f32,
    pub cooc_boost: f32,
    pub embed_confidence: f32,
    pub model_updates: Vec<(String, Vec<f32>)>,
}

impl GossipMessage {
    pub fn compute_hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.node_id.hash(&mut h);
        self.cycle.hash(&mut h);
        for &((a, b), _c) in &self.edge_trust {
            a.hash(&mut h);
            b.hash(&mut h);
        }
        h.finish()
    }
}

mod gossip_codec {
    use super::GossipMessage;

    const MAGIC: &[u8; 8] = b"WIREV001";
    const MAX_PACKET: usize = 8192;
    const MAX_ITEMS: usize = 128;
    const MAX_STRING: usize = 512;

    pub fn encode(msg: &GossipMessage) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        put_string(&mut out, &msg.node_id)?;
        put_u64(&mut out, msg.cycle);
        put_u64(&mut out, msg.msg_hash);
        put_edge_trust(&mut out, &msg.edge_trust)?;
        put_string_f32_pairs(&mut out, &msg.source_scores)?;
        put_string_f32_pairs(&mut out, &msg.crawl_recommendations)?;
        put_novel_edges(&mut out, &msg.novel_edges)?;
        put_f32(&mut out, msg.sleep_recommendation);
        put_f32(&mut out, msg.prune_threshold);
        put_f32(&mut out, msg.cooc_boost);
        put_f32(&mut out, msg.embed_confidence);
        put_model_updates(&mut out, &msg.model_updates)?;
        if out.len() > MAX_PACKET {
            return Err("wire-v1 packet exceeds maximum size".to_string());
        }
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<GossipMessage, String> {
        if bytes.len() > MAX_PACKET {
            return Err("wire-v1 packet exceeds maximum size".to_string());
        }
        let mut cursor = Cursor { bytes, offset: 0 };
        cursor.take_magic()?;
        let msg = GossipMessage {
            node_id: cursor.string()?,
            cycle: cursor.u64()?,
            msg_hash: cursor.u64()?,
            edge_trust: cursor.edge_trust()?,
            source_scores: cursor.string_f32_pairs()?,
            crawl_recommendations: cursor.string_f32_pairs()?,
            novel_edges: cursor.novel_edges()?,
            sleep_recommendation: cursor.f32()?,
            prune_threshold: cursor.f32()?,
            cooc_boost: cursor.f32()?,
            embed_confidence: cursor.f32()?,
            model_updates: cursor.model_updates()?,
        };
        if cursor.offset != bytes.len() {
            return Err("wire-v1 trailing bytes".to_string());
        }
        Ok(msg)
    }

    fn put_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    fn put_u64(out: &mut Vec<u8>, value: u64) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    fn put_f32(out: &mut Vec<u8>, value: f32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    fn put_len(out: &mut Vec<u8>, len: usize) -> Result<(), String> {
        if len > MAX_ITEMS {
            return Err("wire-v1 item count exceeds maximum".to_string());
        }
        put_u32(out, len as u32);
        Ok(())
    }

    fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), String> {
        if value.len() > MAX_STRING {
            return Err("wire-v1 string exceeds maximum".to_string());
        }
        put_u32(out, value.len() as u32);
        out.extend_from_slice(value.as_bytes());
        Ok(())
    }

    fn put_edge_trust(out: &mut Vec<u8>, values: &[((usize, usize), f32)]) -> Result<(), String> {
        put_len(out, values.len())?;
        for &((a, b), score) in values {
            put_u32(out, checked_usize(a)?);
            put_u32(out, checked_usize(b)?);
            put_f32(out, score);
        }
        Ok(())
    }

    fn put_string_f32_pairs(out: &mut Vec<u8>, values: &[(String, f32)]) -> Result<(), String> {
        put_len(out, values.len())?;
        for (text, score) in values {
            put_string(out, text)?;
            put_f32(out, *score);
        }
        Ok(())
    }

    fn put_novel_edges(out: &mut Vec<u8>, values: &[(usize, usize, f32)]) -> Result<(), String> {
        put_len(out, values.len())?;
        for &(a, b, score) in values {
            put_u32(out, checked_usize(a)?);
            put_u32(out, checked_usize(b)?);
            put_f32(out, score);
        }
        Ok(())
    }

    fn put_model_updates(out: &mut Vec<u8>, values: &[(String, Vec<f32>)]) -> Result<(), String> {
        put_len(out, values.len())?;
        for (name, weights) in values {
            put_string(out, name)?;
            put_len(out, weights.len())?;
            for &weight in weights {
                put_f32(out, weight);
            }
        }
        Ok(())
    }

    fn checked_usize(value: usize) -> Result<u32, String> {
        u32::try_from(value).map_err(|_| "wire-v1 usize exceeds u32".to_string())
    }

    struct Cursor<'a> {
        bytes: &'a [u8],
        offset: usize,
    }

    impl<'a> Cursor<'a> {
        fn take_magic(&mut self) -> Result<(), String> {
            let magic = self.take(MAGIC.len())?;
            if magic != MAGIC {
                return Err("wire-v1 magic mismatch".to_string());
            }
            Ok(())
        }

        fn take(&mut self, len: usize) -> Result<&'a [u8], String> {
            let end = self
                .offset
                .checked_add(len)
                .ok_or_else(|| "wire-v1 cursor overflow".to_string())?;
            if end > self.bytes.len() {
                return Err("wire-v1 truncated payload".to_string());
            }
            let slice = &self.bytes[self.offset..end];
            self.offset = end;
            Ok(slice)
        }

        fn u32(&mut self) -> Result<u32, String> {
            let bytes = self.take(4)?;
            Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }

        fn u64(&mut self) -> Result<u64, String> {
            let bytes = self.take(8)?;
            Ok(u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        }

        fn f32(&mut self) -> Result<f32, String> {
            let bytes = self.take(4)?;
            Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }

        fn len(&mut self) -> Result<usize, String> {
            let len = self.u32()? as usize;
            if len > MAX_ITEMS {
                return Err("wire-v1 item count exceeds maximum".to_string());
            }
            Ok(len)
        }

        fn string(&mut self) -> Result<String, String> {
            let len = self.u32()? as usize;
            if len > MAX_STRING {
                return Err("wire-v1 string exceeds maximum".to_string());
            }
            let bytes = self.take(len)?;
            std::str::from_utf8(bytes)
                .map(|value| value.to_string())
                .map_err(|_| "wire-v1 invalid utf8".to_string())
        }

        fn edge_trust(&mut self) -> Result<Vec<((usize, usize), f32)>, String> {
            let len = self.len()?;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(((self.u32()? as usize, self.u32()? as usize), self.f32()?));
            }
            Ok(values)
        }

        fn string_f32_pairs(&mut self) -> Result<Vec<(String, f32)>, String> {
            let len = self.len()?;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push((self.string()?, self.f32()?));
            }
            Ok(values)
        }

        fn novel_edges(&mut self) -> Result<Vec<(usize, usize, f32)>, String> {
            let len = self.len()?;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push((self.u32()? as usize, self.u32()? as usize, self.f32()?));
            }
            Ok(values)
        }

        fn model_updates(&mut self) -> Result<Vec<(String, Vec<f32>)>, String> {
            let len = self.len()?;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                let name = self.string()?;
                let weights_len = self.len()?;
                let mut weights = Vec::with_capacity(weights_len);
                for _ in 0..weights_len {
                    weights.push(self.f32()?);
                }
                values.push((name, weights));
            }
            Ok(values)
        }
    }
}

/// Inbox of foreign messages, shared between gossip thread and paradigm_tick
pub struct GossipInbox {
    pub messages: Mutex<Vec<GossipMessage>>,
}

impl GossipInbox {
    pub fn new() -> Arc<Self> {
        Arc::new(GossipInbox {
            messages: Mutex::new(Vec::new()),
        })
    }
    pub fn drain(&self) -> Vec<GossipMessage> {
        let mut msgs = self.messages.lock().unwrap();
        std::mem::take(&mut *msgs)
    }
}

/// Start a background gossip thread: receives foreign messages, forwards to other peers
pub fn start_gossip(
    inbox: Arc<GossipInbox>,
    port: u16,
    peers: Vec<String>,
    _node_id: String,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let bind_addr = format!("0.0.0.0:{}", port);
        let socket = match UdpSocket::bind(&bind_addr) {
            Ok(s) => {
                let _ = s.set_read_timeout(Some(Duration::from_millis(200)));
                s
            }
            Err(e) => {
                eprintln!("[GOSSIP] Failed to bind {}: {}", bind_addr, e);
                return;
            }
        };

        let mut seen: HashSet<u64> = HashSet::new();
        loop {
            // ── RECEIVE ──
            let mut buf = [0u8; 8192];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((len, _src)) => {
                        if let Ok(msg) = gossip_codec::decode(&buf[..len]) {
                            let h = msg.compute_hash();
                            if seen.insert(h) {
                                // Forward to all peers
                                let fwd_data = buf[..len].to_vec();
                                for peer in &peers {
                                    let _ = socket.send_to(&fwd_data, peer);
                                }
                                if let Ok(mut msgs) = inbox.messages.lock() {
                                    msgs.push(msg);
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    })
}

/// Broadcast ParadigmSignals to all peers (called from paradigm_tick after signals are built)
pub fn broadcast_signals(
    socket: &UdpSocket,
    peers: &[String],
    node_id: &str,
    cycle: u64,
    edge_trust: Vec<((usize, usize), f32)>,
    source_scores: Vec<(String, f32)>,
    crawl_recommendations: Vec<(String, f32)>,
    novel_edges: Vec<(usize, usize, f32)>,
    sleep_recommendation: f32,
    prune_threshold: f32,
    cooc_boost: f32,
    embed_confidence: f32,
    model_updates: Vec<(String, Vec<f32>)>,
) {
    let mut msg = GossipMessage {
        node_id: node_id.to_string(),
        cycle,
        msg_hash: 0,
        edge_trust,
        source_scores,
        crawl_recommendations,
        novel_edges,
        sleep_recommendation,
        prune_threshold,
        cooc_boost,
        embed_confidence,
        model_updates,
    };
    msg.msg_hash = msg.compute_hash();
    if let Ok(data) = gossip_codec::encode(&msg) {
        for peer in peers {
            let _ = socket.send_to(&data, peer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_gossip_v1_roundtrips() {
        let mut msg = GossipMessage {
            node_id: "eden-a".to_string(),
            cycle: 42,
            msg_hash: 0,
            edge_trust: vec![((1, 2), 0.75)],
            source_scores: vec![("local".to_string(), 0.9)],
            crawl_recommendations: vec![("https://example.org".to_string(), 0.4)],
            novel_edges: vec![(2, 3, 0.8)],
            sleep_recommendation: 0.1,
            prune_threshold: 0.2,
            cooc_boost: 0.3,
            embed_confidence: 0.6,
            model_updates: vec![("edge_scorer".to_string(), vec![0.1, 0.2])],
        };
        msg.msg_hash = msg.compute_hash();

        let data = gossip_codec::encode(&msg).expect("wire-v1 serialization should work");
        assert_eq!(&data[..8], b"WIREV001");
        let restored = gossip_codec::decode(&data).expect("wire-v1 deserialization should work");

        assert_eq!(restored.node_id, msg.node_id);
        assert_eq!(restored.cycle, msg.cycle);
        assert_eq!(restored.msg_hash, msg.msg_hash);
        assert_eq!(restored.edge_trust, msg.edge_trust);
        assert_eq!(restored.model_updates, msg.model_updates);
    }

    #[test]
    fn wire_gossip_v1_rejects_invalid_payloads() {
        assert!(gossip_codec::decode(b"WIREV001\0").is_err());
        let mut oversized = vec![0u8; 8193];
        oversized[..8].copy_from_slice(b"WIREV001");
        assert!(gossip_codec::decode(&oversized).is_err());
        let mut valid = Vec::new();
        valid.extend_from_slice(b"WIREV001");
        valid.extend_from_slice(&0u32.to_le_bytes());
        valid.extend_from_slice(&0u64.to_le_bytes());
        valid.extend_from_slice(&0u64.to_le_bytes());
        for _ in 0..4 {
            valid.extend_from_slice(&0u32.to_le_bytes());
        }
        for _ in 0..4 {
            valid.extend_from_slice(&0f32.to_le_bytes());
        }
        valid.extend_from_slice(&0u32.to_le_bytes());
        valid.push(0);
        assert!(gossip_codec::decode(&valid).is_err());
    }
}

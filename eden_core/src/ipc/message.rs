//! Message types for IPC communication
#![allow(unused_imports)]
#![allow(dead_code)]
use std::sync::Mutex;

use std::io::{self};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum message size (64KB)
pub const MAX_MESSAGE_SIZE: usize = 65536;

/// Message types supported by EDEN IPC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// Control message (start, stop, etc.)
    Control = 0x01,
    /// Data message (binary payload)
    Data = 0x02,
    /// Request message (expecting response)
    Request = 0x03,
    /// Response message (reply to request)
    Response = 0x04,
    /// Error message
    Error = 0x05,
    /// Heartbeat/ping message
    Heartbeat = 0x06,
    /// Genesis pattern message
    Genesis = 0x07,
    /// =====================
    /// EVENTOS (Rust → Python)
    /// =====================
    /// Un Auton nació
    EventNacioAuton = 0x10,
    /// Un Auton murió
    EventMurioAuton = 0x11,
    /// Se detectó un Nomos
    EventNomoFormado = 0x12,
    /// Bifurcación de Auton
    EventBifurcacion = 0x13,
    /// Estado del ecosistema
    EventEcosistemaState = 0x14,
    /// Auton entró en letargo
    EventLetargo = 0x15,
    /// Auton se esporuló
    EventEsporulacion = 0x16,
    /// Espora germinó
    EventGerminacion = 0x17,
    /// =====================
    /// COMANDOS (Python → Rust)
    /// =====================
    /// Inyectar energía en coordenadas
    CmdInyectarEnergon = 0x20,
    /// Aumentar escoria en radio
    CmdAumentarEscoria = 0x21,
    /// Forzar bifurcación
    CmdForzarBifurcacion = 0x22,
    /// Eliminar Auton
    CmdEliminarAuton = 0x23,
    /// Modificar constantes cosmológicas
    CmdModificarConstantes = 0x24,
    /// Pausar/Reanudar simulación
    CmdPausarSimulacion = 0x25,
}

impl MessageType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x01 => Some(Self::Control),
            0x02 => Some(Self::Data),
            0x03 => Some(Self::Request),
            0x04 => Some(Self::Response),
            0x05 => Some(Self::Error),
            0x06 => Some(Self::Heartbeat),
            0x07 => Some(Self::Genesis),
            0x10 => Some(Self::EventNacioAuton),
            0x11 => Some(Self::EventMurioAuton),
            0x12 => Some(Self::EventNomoFormado),
            0x13 => Some(Self::EventBifurcacion),
            0x14 => Some(Self::EventEcosistemaState),
            0x15 => Some(Self::EventLetargo),
            0x16 => Some(Self::EventEsporulacion),
            0x17 => Some(Self::EventGerminacion),
            0x20 => Some(Self::CmdInyectarEnergon),
            0x21 => Some(Self::CmdAumentarEscoria),
            0x22 => Some(Self::CmdForzarBifurcacion),
            0x23 => Some(Self::CmdEliminarAuton),
            0x24 => Some(Self::CmdModificarConstantes),
            0x25 => Some(Self::CmdPausarSimulacion),
            _ => None,
        }
    }

    /// Indica si este tipo es un evento (Rust → Python)
    pub fn is_event(&self) -> bool {
        matches!(
            self,
            Self::EventNacioAuton
                | Self::EventMurioAuton
                | Self::EventNomoFormado
                | Self::EventBifurcacion
                | Self::EventEcosistemaState
                | Self::EventLetargo
                | Self::EventEsporulacion
                | Self::EventGerminacion
        )
    }

    /// Indica si este tipo es un comando (Python → Rust)
    pub fn is_command(&self) -> bool {
        matches!(
            self,
            Self::CmdInyectarEnergon
                | Self::CmdAumentarEscoria
                | Self::CmdForzarBifurcacion
                | Self::CmdEliminarAuton
                | Self::CmdModificarConstantes
                | Self::CmdPausarSimulacion
        )
    }
}

/// Message header (32 bytes fixed size)
#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct MessageHeader {
    /// Magic number for validation
    pub magic: [u8; 4], // 0-3: "EDEN"
    /// Protocol version
    pub version: u8, // 4
    /// Message type
    pub msg_type: u8, // 5
    /// Flags (reserved)
    pub flags: u8, // 6
    /// Payload length (max 64KB)
    pub payload_len: u16, // 7-8
    /// Sequence number
    pub sequence: u64, // 9-16
    /// Timestamp (microseconds since epoch)
    pub timestamp: u64, // 17-24
    /// Checksum (simple XOR of header bytes)
    pub checksum: u8, // 25
    /// Reserved
    _reserved: [u8; 6], // 26-31
}

impl MessageHeader {
    pub const MAGIC: [u8; 4] = [b'E', b'D', b'E', b'N'];
    pub const CURRENT_VERSION: u8 = 1;
    pub const SIZE: usize = 32;

    /// Create a new header
    pub fn new(msg_type: MessageType, payload_len: u16, sequence: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        let mut header = Self {
            magic: Self::MAGIC,
            version: Self::CURRENT_VERSION,
            msg_type: msg_type as u8,
            flags: 0,
            payload_len,
            sequence,
            timestamp,
            checksum: 0,
            _reserved: [0; 6],
        };
        header.checksum = header.calculate_checksum();
        header
    }

    /// Validate header magic and version
    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC && self.version <= Self::CURRENT_VERSION
    }

    /// Calculate header checksum (sum of all bytes except checksum byte)
    pub fn calculate_checksum(&self) -> u8 {
        let bytes = self.as_bytes();
        // Sum all bytes EXCEPT the checksum byte at index 25
        let mut sum = 0u8;
        for (i, &b) in bytes.iter().enumerate() {
            if i != 25 {
                sum = sum.wrapping_add(b);
            }
        }
        sum
    }

    /// Convert header to bytes (little-endian)
    pub fn as_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4] = self.version;
        buf[5] = self.msg_type;
        buf[6] = self.flags;
        buf[7..9].copy_from_slice(&self.payload_len.to_le_bytes());
        buf[9..17].copy_from_slice(&self.sequence.to_le_bytes());
        buf[17..25].copy_from_slice(&self.timestamp.to_le_bytes());
        buf[25] = self.checksum;
        buf
    }

    /// Parse header from bytes
    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> io::Result<Self> {
        let mut header = Self {
            magic: [0; 4],
            version: 0,
            msg_type: 0,
            flags: 0,
            payload_len: 0,
            sequence: 0,
            timestamp: 0,
            checksum: 0,
            _reserved: [0; 6],
        };

        header.magic.copy_from_slice(&bytes[0..4]);
        header.version = bytes[4];
        header.msg_type = bytes[5];
        header.flags = bytes[6];
        header.payload_len = u16::from_le_bytes([bytes[7], bytes[8]]);
        header.sequence = u64::from_le_bytes([
            bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16],
        ]);
        header.timestamp = u64::from_le_bytes([
            bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23], bytes[24],
        ]);
        header.checksum = bytes[25];
        header._reserved.copy_from_slice(&bytes[26..32]);

        // Validate
        if !header.is_valid() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid message header",
            ));
        }

        // Verify checksum
        let mut calc = header.clone();
        calc.checksum = 0;
        if calc.calculate_checksum() != header.checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Checksum mismatch",
            ));
        }

        Ok(header)
    }
}

/// IPC Message with header and payload
#[derive(Debug, Clone)]
pub struct Message {
    pub header: MessageHeader,
    pub payload: Vec<u8>,
}

impl Message {
    /// Create a new message
    pub fn new(msg_type: MessageType, payload: Vec<u8>) -> io::Result<Self> {
        if payload.len() > MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Payload too large: {} > {}",
                    payload.len(),
                    MAX_MESSAGE_SIZE
                ),
            ));
        }

        let mut header = MessageHeader::new(
            msg_type,
            payload.len() as u16,
            0, // Sequence set later
        );
        header.checksum = header.calculate_checksum();

        Ok(Self { header, payload })
    }

    /// Create with sequence number
    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.header.sequence = sequence;
        self.header.checksum = self.header.calculate_checksum();
        self
    }

    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(MessageHeader::SIZE + self.payload.len());
        bytes.extend_from_slice(&self.header.as_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Deserialize message from bytes
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < MessageHeader::SIZE {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Incomplete header",
            ));
        }

        let header = MessageHeader::from_bytes(bytes[0..MessageHeader::SIZE].try_into().unwrap())?;

        let payload_len = header.payload_len as usize;
        if bytes.len() < MessageHeader::SIZE + payload_len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Incomplete payload",
            ));
        }

        let payload = bytes[MessageHeader::SIZE..MessageHeader::SIZE + payload_len].to_vec();

        Ok(Self { header, payload })
    }

    /// Get message type
    pub fn msg_type(&self) -> Option<MessageType> {
        MessageType::from_u8(self.header.msg_type)
    }
}

/// Convenience constructors for common message types
impl Message {
    /// Create a control message
    pub fn control(command: &[u8]) -> io::Result<Self> {
        Self::new(MessageType::Control, command.to_vec())
    }

    /// Create a data message
    pub fn data(payload: Vec<u8>) -> io::Result<Self> {
        Self::new(MessageType::Data, payload)
    }

    /// Create a heartbeat message
    pub fn heartbeat() -> io::Result<Self> {
        Self::new(MessageType::Heartbeat, Vec::new())
    }

    /// Create a genesis pattern message
    pub fn genesis(pattern_id: &[u8]) -> io::Result<Self> {
        Self::new(MessageType::Genesis, pattern_id.to_vec())
    }

    // ==================== EVENTOS (Rust → Python) ====================

    /// Crea evento: NacioAuton { id, posicion_x, posicion_y }
    pub fn evento_nacio_auton(id: u64, pos_x: f64, pos_y: f64) -> io::Result<Self> {
        let payload = format!("{{\"id\":{},\"x\":{},\"y\":{}}}", id, pos_x, pos_y).into_bytes();
        Self::new(MessageType::EventNacioAuton, payload)
    }

    /// Crea evento: MurioAuton { id, causa }
    pub fn evento_murio_auton(id: u64, causa: &str) -> io::Result<Self> {
        let payload = format!("{{\"id\":{},\"causa\":\"{}\"}}", id, causa).into_bytes();
        Self::new(MessageType::EventMurioAuton, payload)
    }

    /// Crea evento: NomoFormado { tipo, centro_x, centro_y }
    pub fn evento_nomo_formado(tipo: &str, centro_x: f64, centro_y: f64) -> io::Result<Self> {
        let payload = format!(
            "{{\"tipo\":\"{}\",\"x\":{},\"y\":{}}}",
            tipo, centro_x, centro_y
        )
        .into_bytes();
        Self::new(MessageType::EventNomoFormado, payload)
    }

    /// Crea evento: Bifurcacion { padre_id, hijo_id }
    pub fn evento_bifurcacion(padre_id: u64, hijo_id: u64) -> io::Result<Self> {
        let payload = format!("{{\"padre\":{},\"hijo\":{}}}", padre_id, hijo_id).into_bytes();
        Self::new(MessageType::EventBifurcacion, payload)
    }

    /// Crea evento: EstadoEcosistema
    pub fn evento_ecosistema_state(
        autons_vivos: u32,
        energia_total: f64,
        escoria_total: f64,
        dens_promedio: f64,
    ) -> io::Result<Self> {
        let payload = format!(
            "{{\"autons\":{},\"energia\":{},\"escoria\":{},\"densidad\":{}}}",
            autons_vivos, energia_total, escoria_total, dens_promedio
        )
        .into_bytes();
        Self::new(MessageType::EventEcosistemaState, payload)
    }

    /// Crea evento: Letargo { id, causa }
    pub fn evento_letargo(id: u64, causa: &str) -> io::Result<Self> {
        let payload = format!("{{\"id\":{},\"causa\":\"{}\"}}", id, causa).into_bytes();
        Self::new(MessageType::EventLetargo, payload)
    }

    /// Crea evento: Esporulacion { id, causa }
    pub fn evento_esporulacion(id: u64, causa: &str) -> io::Result<Self> {
        let payload = format!("{{\"id\":{},\"causa\":\"{}\"}}", id, causa).into_bytes();
        Self::new(MessageType::EventEsporulacion, payload)
    }

    /// Crea evento: Germinacion { id, ciclos, mutacion }
    pub fn evento_germinacion(id: u64, ciclos: u64, mutacion: f64) -> io::Result<Self> {
        let payload = format!(
            "{{\"id\":{},\"ciclos\":{},\"mutacion\":{}}}",
            id, ciclos, mutacion
        )
        .into_bytes();
        Self::new(MessageType::EventGerminacion, payload)
    }

    // ==================== COMANDOS (Python → Rust) ====================

    /// Crea comando: InyectarEnergon { x, y, cantidad }
    pub fn cmd_inyectar_energon(x: f64, y: f64, cantidad: f64) -> io::Result<Self> {
        let payload = format!("{{\"x\":{},\"y\":{},\"cantidad\":{}}}", x, y, cantidad).into_bytes();
        Self::new(MessageType::CmdInyectarEnergon, payload)
    }

    /// Crea comando: AumentarEscoria { x, y, radio, cantidad }
    pub fn cmd_aumentar_escoria(x: f64, y: f64, radio: f64, cantidad: f64) -> io::Result<Self> {
        let payload = format!(
            "{{\"x\":{},\"y\":{},\"radio\":{},\"cantidad\":{}}}",
            x, y, radio, cantidad
        )
        .into_bytes();
        Self::new(MessageType::CmdAumentarEscoria, payload)
    }

    /// Crea comando: ForzarBifurcacion { auton_id }
    pub fn cmd_forzar_bifurcacion(auton_id: u64) -> io::Result<Self> {
        let payload = format!("{{\"id\":{}}}", auton_id).into_bytes();
        Self::new(MessageType::CmdForzarBifurcacion, payload)
    }

    /// Crea comando: EliminarAuton { auton_id }
    pub fn cmd_eliminar_auton(auton_id: u64) -> io::Result<Self> {
        let payload = format!("{{\"id\":{}}}", auton_id).into_bytes();
        Self::new(MessageType::CmdEliminarAuton, payload)
    }

    /// Crea comando: PausarSimulacion { pausar: bool }
    pub fn cmd_pausar_simulacion(pausar: bool) -> io::Result<Self> {
        let payload = format!("{{\"pausar\":{}}}", pausar).into_bytes();
        Self::new(MessageType::CmdPausarSimulacion, payload)
    }

    /// Verifica si el mensaje es un evento
    pub fn is_event(&self) -> bool {
        self.msg_type().map(|t| t.is_event()).unwrap_or(false)
    }

    /// Verifica si el mensaje es un comando
    pub fn is_command(&self) -> bool {
        self.msg_type().map(|t| t.is_command()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_serialization() {
        let header = MessageHeader::new(MessageType::Data, 100, 42);
        let bytes = header.as_bytes();
        let parsed = MessageHeader::from_bytes(&bytes).unwrap();

        // Copy fields to avoid unaligned reference on packed struct
        let (parsed_payload_len, parsed_sequence, header_payload_len, header_sequence) = (
            parsed.payload_len,
            parsed.sequence,
            header.payload_len,
            header.sequence,
        );
        assert_eq!(parsed.magic, header.magic);
        assert_eq!(parsed.version, header.version);
        assert_eq!(parsed.msg_type, header.msg_type);
        assert_eq!(parsed_payload_len, header_payload_len);
        assert_eq!(parsed_sequence, header_sequence);
    }

    #[test]
    fn test_message_roundtrip() {
        let original = Message::new(MessageType::Data, b"EDEN".to_vec()).unwrap();
        let serialized = original.to_bytes();
        let parsed = Message::from_bytes(&serialized).unwrap();

        assert_eq!(parsed.payload, original.payload);
        assert_eq!(parsed.msg_type(), original.msg_type());
    }
}

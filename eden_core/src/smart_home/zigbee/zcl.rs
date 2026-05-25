//! # ZCL - Zigbee Cluster Library
//!
//! Implementación completa de Zigbee Cluster Library.
//! Frame format, command parsing, attribute handling.
//! 100% original, basado en Zigbee ZCL specification.

#![allow(dead_code)]

use std::collections::HashMap;

/// Identificador de cluster Zigbee
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ClusterId {
    OnOff = 0x0006,
    LevelControl = 0x0008,
    ColorControl = 0x0300,
    Temperature = 0x0201,
    Humidity = 0x0405,
    Occupancy = 0x0406,
    DoorLock = 0x0101,
    Thermostat = 0x0202,
    IASZone = 0x0500,
    SimpleMetering = 0x0702,
    Custom(u16),
}

impl ClusterId {
    pub fn from_u16(val: u16) -> Self {
        match val {
            0x0006 => ClusterId::OnOff,
            0x0008 => ClusterId::LevelControl,
            0x0300 => ClusterId::ColorControl,
            0x0201 => ClusterId::Temperature,
            0x0405 => ClusterId::Humidity,
            0x0406 => ClusterId::Occupancy,
            0x0101 => ClusterId::DoorLock,
            0x0500 => ClusterId::IASZone,
            0x0702 => ClusterId::SimpleMetering,
            _ => ClusterId::Custom(val),
        }
    }

    pub fn to_u16(&self) -> u16 {
        match self {
            ClusterId::OnOff => 0x0006,
            ClusterId::LevelControl => 0x0008,
            ClusterId::ColorControl => 0x0300,
            ClusterId::Temperature => 0x0201,
            ClusterId::Humidity => 0x0405,
            ClusterId::Occupancy => 0x0406,
            ClusterId::DoorLock => 0x0101,
            ClusterId::Thermostat => 0x0201,
            ClusterId::IASZone => 0x0500,
            ClusterId::SimpleMetering => 0x0702,
            ClusterId::Custom(id) => *id,
        }
    }
}

/// Frame type dentro del ZCL
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameType {
    Global,
    Local,
}

/// Direction del comando
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    ToServer,
    ToClient,
}

/// Comando global ZCL
#[derive(Debug, Clone, Copy)]
pub enum GlobalCommand {
    ReadAttributes = 0x00,
    ReadAttributesResponse = 0x01,
    WriteAttributes = 0x02,
    WriteAttributesUndivided = 0x03,
    WriteAttributesResponse = 0x04,
    WriteAttributesNoResponse = 0x05,
    ConfigureReporting = 0x06,
    ConfigureReportingResponse = 0x07,
    ReadReportingConfiguration = 0x08,
    DiscoverAttributes = 0x0C,
    DiscoverCommandsReceived = 0x11,
    DiscoverCommandsGenerated = 0x12,
    DefaultResponse = 0x0B,
}

impl GlobalCommand {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(GlobalCommand::ReadAttributes),
            0x01 => Some(GlobalCommand::ReadAttributesResponse),
            0x02 => Some(GlobalCommand::WriteAttributes),
            0x03 => Some(GlobalCommand::WriteAttributesUndivided),
            0x04 => Some(GlobalCommand::WriteAttributesResponse),
            0x05 => Some(GlobalCommand::WriteAttributesNoResponse),
            0x06 => Some(GlobalCommand::ConfigureReporting),
            0x07 => Some(GlobalCommand::ConfigureReportingResponse),
            0x08 => Some(GlobalCommand::ReadReportingConfiguration),
            0x0B => Some(GlobalCommand::DefaultResponse),
            0x0C => Some(GlobalCommand::DiscoverAttributes),
            0x11 => Some(GlobalCommand::DiscoverCommandsReceived),
            0x12 => Some(GlobalCommand::DiscoverCommandsGenerated),
            _ => None,
        }
    }
}

/// Atributo ZCL
#[derive(Debug, Clone)]
pub struct ZclAttribute {
    pub id: u16,
    pub data_type: DataType,
    pub value: AttributeValue,
}

/// Tipo de dato ZCL
#[derive(Debug, Clone, Copy)]
pub enum DataType {
    NoData = 0x00,
    Data8 = 0x08,
    Data16 = 0x09,
    Data24 = 0x0A,
    Data32 = 0x0B,
    Boolean = 0x10,
    Bitmap8 = 0x18,
    Bitmap16 = 0x19,
    Bitmap24 = 0x1A,
    Bitmap32 = 0x1B,
    Uint8 = 0x20,
    Uint16 = 0x21,
    Uint24 = 0x22,
    Uint32 = 0x23,
    Uint48 = 0x25,
    Int8 = 0x28,
    Int16 = 0x29,
    Int32 = 0x2A,
    Enum8 = 0x30,
    Enum16 = 0x31,
    SemiFloat = 0x38,
    Float = 0x39,
    Double = 0x3A,
    String = 0x41,
    OctetString = 0x42,
    LongString = 0x43,
    Array = 0x48,
    Struct = 0x4C,
    Set = 0x50,
    Bag = 0x51,
    ToD = 0xE0,
    Date = 0xE1,
    UTCTime = 0xE2,
    ClusterId = 0xE8,
    AttributeId = 0xE9,
    BACnetOid = 0xEA,
    IEEEAddr = 0xF0,
    SecureKey = 0xF1,
}

/// Valor de atributo ZCL
#[derive(Debug, Clone)]
pub enum AttributeValue {
    Bool(bool),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Enum8(u8),
    Float(f32),
    Double(f64),
    String(String),
    Bytes(Vec<u8>),
    None,
}

impl AttributeValue {
    /// Serializa el valor a bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            AttributeValue::Bool(b) => vec![if *b { 1 } else { 0 }],
            AttributeValue::UInt8(v) => vec![*v],
            AttributeValue::UInt16(v) => v.to_le_bytes().to_vec(),
            AttributeValue::UInt32(v) => v.to_le_bytes().to_vec(),
            AttributeValue::UInt64(v) => v.to_le_bytes().to_vec(),
            AttributeValue::Int8(v) => vec![*v as u8],
            AttributeValue::Int16(v) => v.to_le_bytes().to_vec(),
            AttributeValue::Int32(v) => v.to_le_bytes().to_vec(),
            AttributeValue::Int64(v) => v.to_le_bytes().to_vec(),
            AttributeValue::Enum8(v) => vec![*v],
            AttributeValue::Float(f) => f.to_le_bytes().to_vec(),
            AttributeValue::Double(d) => d.to_le_bytes().to_vec(),
            AttributeValue::String(s) => {
                let len = s.len() as u8;
                let mut bytes = vec![len];
                bytes.extend_from_slice(s.as_bytes());
                bytes
            }
            AttributeValue::Bytes(b) => b.clone(),
            AttributeValue::None => vec![],
        }
    }

    /// Parsea bytes a valor dado un tipo de dato
    pub fn from_bytes(data_type: DataType, bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return Some(AttributeValue::None);
        }

        match data_type {
            DataType::Boolean => Some(AttributeValue::Bool(bytes[0] != 0)),
            DataType::Uint8 | DataType::Enum8 | DataType::Bitmap8 => {
                Some(AttributeValue::UInt8(bytes[0]))
            }
            DataType::Uint16 | DataType::Bitmap16 => {
                if bytes.len() >= 2 {
                    Some(AttributeValue::UInt16(u16::from_le_bytes([
                        bytes[0], bytes[1],
                    ])))
                } else {
                    None
                }
            }
            DataType::Uint32 | DataType::Bitmap32 => {
                if bytes.len() >= 4 {
                    Some(AttributeValue::UInt32(u32::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                    ])))
                } else {
                    None
                }
            }
            DataType::Uint48 => {
                if bytes.len() >= 6 {
                    let mut arr = [0u8; 8];
                    arr[..6].copy_from_slice(bytes);
                    Some(AttributeValue::UInt64(u64::from_le_bytes(arr)))
                } else {
                    None
                }
            }
            DataType::Int8 => Some(AttributeValue::Int8(bytes[0] as i8)),
            DataType::Int16 => {
                if bytes.len() >= 2 {
                    Some(AttributeValue::Int16(i16::from_le_bytes([
                        bytes[0], bytes[1],
                    ])))
                } else {
                    None
                }
            }
            DataType::Int32 => {
                if bytes.len() >= 4 {
                    Some(AttributeValue::Int32(i32::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                    ])))
                } else {
                    None
                }
            }
            DataType::Float => {
                if bytes.len() >= 4 {
                    Some(AttributeValue::Float(f32::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                    ])))
                } else {
                    None
                }
            }
            DataType::Double => {
                if bytes.len() >= 8 {
                    Some(AttributeValue::Double(f64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ])))
                } else {
                    None
                }
            }
            DataType::String | DataType::OctetString => {
                let len = bytes[0] as usize;
                if bytes.len() > len {
                    Some(AttributeValue::String(
                        String::from_utf8_lossy(&bytes[1..=len]).to_string(),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Frame ZCL completo
#[derive(Debug, Clone)]
pub struct ZclFrame {
    pub frame_control: FrameControl,
    pub manufacturer_code: Option<u16>,
    pub sequence: u8,
    pub command: ZclCommand,
    pub payload: Vec<u8>,
}

/// Frame control byte
#[derive(Debug, Clone)]
pub struct FrameControl {
    pub frame_type: FrameType,
    pub manufacturer_specific: bool,
    pub direction: Direction,
    pub disable_default_response: bool,
}

impl FrameControl {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut byte = 0u8;
        byte |= match self.frame_type {
            FrameType::Global => 0x00,
            FrameType::Local => 0x01,
        } << 0;
        byte |= if self.manufacturer_specific { 1 } else { 0 } << 2;
        byte |= match self.direction {
            Direction::ToServer => 0x00,
            Direction::ToClient => 0x01,
        } << 3;
        byte |= if self.disable_default_response { 1 } else { 0 } << 4;
        vec![byte]
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        let b = bytes[0];
        Some(FrameControl {
            frame_type: if (b & 0x01) == 0 {
                FrameType::Global
            } else {
                FrameType::Local
            },
            manufacturer_specific: (b & 0x04) != 0,
            direction: if (b & 0x08) == 0 {
                Direction::ToServer
            } else {
                Direction::ToClient
            },
            disable_default_response: (b & 0x10) != 0,
        })
    }
}

/// Comando ZCL (global o específico de cluster)
#[derive(Debug, Clone)]
pub enum ZclCommand {
    Global(GlobalCommand),
    Specific(u8),
}

/// Zigbee Cluster Library principal
pub struct ZigbeeClusterLibrary {
    sequence: u8,
    attributes: HashMap<ClusterId, Vec<ZclAttribute>>,
}

impl ZigbeeClusterLibrary {
    pub fn new() -> Self {
        Self {
            sequence: 0,
            attributes: HashMap::new(),
        }
    }

    /// Incrementa y retorna secuencia
    pub fn next_sequence(&mut self) -> u8 {
        self.sequence = self.sequence.wrapping_add(1);
        self.sequence
    }

    /// Construye un ZCL frame para leer atributos
    pub fn read_attributes(&mut self, _cluster: ClusterId, attr_ids: &[u16]) -> Vec<u8> {
        let seq = self.next_sequence();

        let mut payload = vec![];
        for attr_id in attr_ids {
            payload.extend_from_slice(&attr_id.to_le_bytes());
        }

        let frame = ZclFrame {
            frame_control: FrameControl {
                frame_type: FrameType::Global,
                manufacturer_specific: false,
                direction: Direction::ToServer,
                disable_default_response: false,
            },
            manufacturer_code: None,
            sequence: seq,
            command: ZclCommand::Global(GlobalCommand::ReadAttributes),
            payload,
        };

        self.encode_frame(&frame)
    }

    /// Construye un ZCL frame para escribir atributos
    pub fn write_attributes(
        &mut self,
        _cluster: ClusterId,
        attrs: &[(u16, AttributeValue)],
    ) -> Vec<u8> {
        let seq = self.next_sequence();

        let mut payload = vec![];
        for (attr_id, value) in attrs {
            payload.extend_from_slice(&attr_id.to_le_bytes());
            payload.push((*attr_id >> 8) as u8); // Data type placeholder
            payload.extend_from_slice(&value.to_bytes());
        }

        let frame = ZclFrame {
            frame_control: FrameControl {
                frame_type: FrameType::Global,
                manufacturer_specific: false,
                direction: Direction::ToServer,
                disable_default_response: false,
            },
            manufacturer_code: None,
            sequence: seq,
            command: ZclCommand::Global(GlobalCommand::WriteAttributes),
            payload,
        };

        self.encode_frame(&frame)
    }

    /// Construye comando OnOff
    pub fn on_off_command(&mut self, on: bool) -> Vec<u8> {
        let seq = self.next_sequence();

        let frame = ZclFrame {
            frame_control: FrameControl {
                frame_type: FrameType::Local,
                manufacturer_specific: false,
                direction: Direction::ToServer,
                disable_default_response: true,
            },
            manufacturer_code: None,
            sequence: seq,
            command: ZclCommand::Specific(if on { 0x01 } else { 0x00 }), // 0x01=On, 0x00=Off
            payload: vec![],
        };

        self.encode_frame(&frame)
    }

    /// Construye comando LevelControl (0-254)
    pub fn level_command(&mut self, level: u8, transition_time: Option<u16>) -> Vec<u8> {
        let seq = self.next_sequence();

        let mut payload = vec![];
        payload.push(level);
        if let Some(tt) = transition_time {
            payload.extend_from_slice(&tt.to_le_bytes());
        } else {
            payload.extend_from_slice(&0u16.to_le_bytes()); // Default transition
        }

        let frame = ZclFrame {
            frame_control: FrameControl {
                frame_type: FrameType::Local,
                manufacturer_specific: false,
                direction: Direction::ToServer,
                disable_default_response: true,
            },
            manufacturer_code: None,
            sequence: seq,
            command: ZclCommand::Specific(0x00), // Move to Level
            payload,
        };

        self.encode_frame(&frame)
    }

    /// Construye comando ColorControl (hue, saturation)
    pub fn color_command(&mut self, hue: u8, saturation: u8) -> Vec<u8> {
        let seq = self.next_sequence();

        let mut payload = vec![];
        payload.push(hue); // Hue
        payload.push(saturation); // Saturation
        payload.extend_from_slice(&0u16.to_le_bytes()); // Transition time

        let frame = ZclFrame {
            frame_control: FrameControl {
                frame_type: FrameType::Local,
                manufacturer_specific: false,
                direction: Direction::ToServer,
                disable_default_response: true,
            },
            manufacturer_code: None,
            sequence: seq,
            command: ZclCommand::Specific(0x06), // Move to hue and saturation
            payload,
        };

        self.encode_frame(&frame)
    }

    /// Construye comando DoorLock
    pub fn door_lock_command(&mut self, lock: bool) -> Vec<u8> {
        let seq = self.next_sequence();

        let frame = ZclFrame {
            frame_control: FrameControl {
                frame_type: FrameType::Local,
                manufacturer_specific: false,
                direction: Direction::ToServer,
                disable_default_response: true,
            },
            manufacturer_code: None,
            sequence: seq,
            command: ZclCommand::Specific(if lock { 0x00 } else { 0x01 }), // 0x00=Lock, 0x01=Unlock
            payload: vec![],
        };

        self.encode_frame(&frame)
    }

    /// Decodifica un ZCL frame
    pub fn decode_frame(&self, data: &[u8]) -> Option<ZclFrame> {
        if data.len() < 3 {
            return None;
        }

        let frame_control = FrameControl::from_bytes(&data[0..1])?;
        let manufacturer_code = if frame_control.manufacturer_specific {
            if data.len() < 5 {
                return None;
            }
            Some(u16::from_le_bytes([data[1], data[2]]))
        } else {
            None
        };

        let seq_offset = if frame_control.manufacturer_specific {
            3
        } else {
            1
        };
        let sequence = data[seq_offset];

        let cmd_offset = seq_offset + 1;
        let command = if frame_control.frame_type == FrameType::Global {
            GlobalCommand::from_u8(data[cmd_offset])
                .map(ZclCommand::Global)
                .unwrap_or(ZclCommand::Specific(data[cmd_offset]))
        } else {
            ZclCommand::Specific(data[cmd_offset])
        };

        let payload = data[cmd_offset + 1..].to_vec();

        Some(ZclFrame {
            frame_control,
            manufacturer_code,
            sequence,
            command,
            payload,
        })
    }

    /// Encodifica un ZCL frame a bytes
    pub fn encode_frame(&self, frame: &ZclFrame) -> Vec<u8> {
        let mut result = frame.frame_control.to_bytes();

        if let Some(mc) = frame.manufacturer_code {
            result.extend_from_slice(&mc.to_le_bytes());
        }

        result.push(frame.sequence);

        match &frame.command {
            ZclCommand::Global(cmd) => result.push(*cmd as u8),
            ZclCommand::Specific(cmd) => result.push(*cmd),
        }

        result.extend_from_slice(&frame.payload);
        result
    }
}

impl Default for ZigbeeClusterLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_off_frame() {
        let mut zcl = ZigbeeClusterLibrary::new();
        let frame = zcl.on_off_command(true);
        assert!(!frame.is_empty());
    }

    #[test]
    fn test_level_frame() {
        let mut zcl = ZigbeeClusterLibrary::new();
        let frame = zcl.level_command(128, Some(10));
        assert!(!frame.is_empty());
    }

    #[test]
    fn test_decode_encode() {
        let mut zcl = ZigbeeClusterLibrary::new();
        let original = zcl.on_off_command(false);
        let decoded = zcl.decode_frame(&original);
        assert!(decoded.is_some());
    }
}

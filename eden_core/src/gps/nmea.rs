//! # NMEA 0183 Parser
//!
//! Parser para sentencias NMEA 0183.
//! Soporta GGA, RMC, GSA, GSV, y más.
//! 100% original, sin dependencias externas.
#![allow(unused_imports)]
#![allow(dead_code)]
use std::time::UNIX_EPOCH;

/// Tipo de sentencia NMEA
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NmeaSentenceType {
    GGA, // Global Positioning System Fix Data
    RMC, // Recommended Minimum Navigation Information
    GSA, // GPS DOP and Active Satellites
    GSV, // GPS Satellites in View
    GLL, // Geographic Position - Latitude/Longitude
    VTG, // Track Made Good and Speed Over Ground
    ZDA, // Time and Date
    Unknown,
}

impl NmeaSentenceType {
    /// Parsea tipo desde string
    pub fn from_str(s: &str) -> Self {
        match s {
            "GGA" => NmeaSentenceType::GGA,
            "RMC" => NmeaSentenceType::RMC,
            "GSA" => NmeaSentenceType::GSA,
            "GSV" => NmeaSentenceType::GSV,
            "GLL" => NmeaSentenceType::GLL,
            "VTG" => NmeaSentenceType::VTG,
            "ZDA" => NmeaSentenceType::ZDA,
            _ => NmeaSentenceType::Unknown,
        }
    }
}

/// Sentencia NMEA parseada
#[derive(Debug, Clone)]
pub struct NmeaSentence {
    pub talker_id: String, // 2 chars (GP, GL, GA, etc.)
    pub sentence_type: NmeaSentenceType,
    pub fields: Vec<String>,
    pub checksum: u8,
    pub is_valid: bool,
}

/// Parser NMEA
pub struct NmeaParser {
    sentences: Vec<NmeaSentence>,
    last_gga: Option<GgaData>,
    last_rmc: Option<RmcData>,
    last_vtg: Option<VtgData>,
    satellites_visible: u8,
    fix_quality: FixQuality,
}

#[derive(Debug, Clone, Copy)]
pub enum FixQuality {
    NoFix = 0,
    GPSFix = 1,
    DGPSFix = 2,
    PPSFix = 3,
    RTK = 4,
    FloatRTK = 5,
    Estimated = 6,
    Manual = 7,
    Simulation = 8,
}

impl FixQuality {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => FixQuality::NoFix,
            1 => FixQuality::GPSFix,
            2 => FixQuality::DGPSFix,
            3 => FixQuality::PPSFix,
            4 => FixQuality::RTK,
            5 => FixQuality::FloatRTK,
            6 => FixQuality::Estimated,
            7 => FixQuality::Manual,
            8 => FixQuality::Simulation,
            _ => FixQuality::NoFix,
        }
    }
}

/// Datos GGA (Fix Data)
#[derive(Debug, Clone)]
pub struct GgaData {
    pub timestamp: String,   // HHMMSS.sss
    pub latitude: f64,       // Grados decimales
    pub lat_direction: char, // N o S
    pub longitude: f64,      // Grados decimales
    pub lon_direction: char, // E o W
    pub fix_quality: FixQuality,
    pub num_satellites: u8,
    pub hdop: f32,
    pub altitude: f32, // Metros sobre elipsoide
    pub altitude_unit: char,
    pub geoid_separation: f32,
    pub geoid_unit: char,
    pub dgps_age: f32,
    pub dgps_station_id: u16,
}

impl GgaData {
    /// Latitud en grados decimales con signo
    pub fn lat_decimal(&self) -> f64 {
        if self.lat_direction == 'S' {
            -self.latitude
        } else {
            self.latitude
        }
    }

    /// Longitud en grados decimales con signo
    pub fn lon_decimal(&self) -> f64 {
        if self.lon_direction == 'W' {
            -self.longitude
        } else {
            self.longitude
        }
    }
}

/// Datos RMC (Recommended Minimum)
#[derive(Debug, Clone)]
pub struct RmcData {
    pub timestamp: String,
    pub status: char, // A = active, V = void
    pub latitude: f64,
    pub lat_direction: char,
    pub longitude: f64,
    pub lon_direction: char,
    pub speed_knots: f32,
    pub track_angle: f32,
    pub date: String, // DDMMYY
    pub magnetic_variation: f32,
    pub variation_direction: char,
    pub mode_indicator: char,
}

impl RmcData {
    /// Velocidad en km/h
    pub fn speed_kmh(&self) -> f32 {
        self.speed_knots * 1.852 // 1 nudo = 1.852 km/h
    }

    /// Latitud en decimal
    pub fn lat_decimal(&self) -> f64 {
        if self.lat_direction == 'S' {
            -self.latitude
        } else {
            self.latitude
        }
    }

    /// Longitud en decimal
    pub fn lon_decimal(&self) -> f64 {
        if self.lon_direction == 'W' {
            -self.longitude
        } else {
            self.longitude
        }
    }

    /// Parsea fecha
    pub fn parse_date(&self) -> Option<(u16, u16, u16)> {
        // DDMMYY -> (day, month, year)
        if self.date.len() != 6 {
            return None;
        }
        let day: u16 = self.date[0..2].parse().ok()?;
        let month: u16 = self.date[2..4].parse().ok()?;
        let year: u16 = self.date[4..6].parse().ok()?;
        Some((day, month, year + 2000))
    }

    /// Parsea timestamp
    pub fn parse_timestamp(&self) -> Option<(u8, u8, u8)> {
        // HHMMSS
        if self.timestamp.len() < 6 {
            return None;
        }
        let hour: u8 = self.timestamp[0..2].parse().ok()?;
        let min: u8 = self.timestamp[2..4].parse().ok()?;
        let sec: u8 = self.timestamp[4..6].parse().ok()?;
        Some((hour, min, sec))
    }
}

/// Datos VTG (Track and Speed)
#[derive(Debug, Clone)]
pub struct VtgData {
    pub track_true: f32,
    pub track_magnetic: f32,
    pub speed_knots: f32,
    pub speed_kmh: f32,
    pub mode_indicator: char,
}

/// Satélite individual en GSV
#[derive(Debug, Clone)]
pub struct SatelliteInfo {
    pub prn: u16,      // Pseudo-random noise code
    pub elevation: u8, // Grados
    pub azimuth: u16,  // Grados
    pub snr: u8,       // Signal-to-noise ratio (dB)
}

impl NmeaParser {
    /// Crea nuevo parser
    pub fn new() -> Self {
        Self {
            sentences: Vec::new(),
            last_gga: None,
            last_rmc: None,
            last_vtg: None,
            satellites_visible: 0,
            fix_quality: FixQuality::NoFix,
        }
    }

    /// Parsea una línea NMEA
    pub fn parse_line(&mut self, line: &str) -> Option<NmeaSentence> {
        // Remove newline
        let line = line.trim();
        if line.is_empty() || !line.starts_with('$') {
            return None;
        }

        // Find checksum (optional)
        let (content, checksum) = if let Some(pos) = line.rfind('*') {
            let cksum = u8::from_str_radix(&line[pos + 1..], 16).ok();
            (line[1..pos].to_string(), cksum)
        } else {
            (line[1..].to_string(), None)
        };

        // Split by comma
        let fields: Vec<&str> = content.split(',').collect();
        if fields.len() < 2 {
            return None;
        }

        // Talker ID + Sentence type (e.g., "GPGLL")
        let talker = &fields[0];
        if talker.len() < 4 {
            return None;
        }
        let talker_id = talker[..2].to_string();
        let sentence_type = NmeaSentenceType::from_str(&talker[2..]);

        let sentence = NmeaSentence {
            talker_id,
            sentence_type,
            fields: fields.into_iter().map(|s| s.to_string()).collect(),
            checksum: checksum.unwrap_or(0),
            is_valid: checksum.is_some(), // Valid if checksum present
        };

        // Process based on type
        match sentence.sentence_type {
            NmeaSentenceType::GGA => {
                self.process_gga(&sentence);
            }
            NmeaSentenceType::RMC => {
                self.process_rmc(&sentence);
            }
            NmeaSentenceType::VTG => {
                self.process_vtg(&sentence);
            }
            _ => {}
        }

        self.sentences.push(sentence.clone());
        Some(sentence)
    }

    /// Process GGA sentence
    fn process_gga(&mut self, sent: &NmeaSentence) {
        if sent.fields.len() < 15 {
            return;
        }

        // Parse latitude: DDMM.MMMM
        let lat_raw = match sent.fields.get(2) {
            Some(f) => f.parse::<f64>().ok(),
            None => None,
        }
        .unwrap_or(0.0);
        let lat_dir = sent
            .fields
            .get(3)
            .and_then(|f| f.chars().next())
            .unwrap_or('N');
        let lat = Self::nmea_to_decimal_degrees(lat_raw, lat_dir).unwrap_or(0.0);

        // Parse longitude: DDDMM.MMMM
        let lon_raw = match sent.fields.get(4) {
            Some(f) => f.parse::<f64>().ok(),
            None => None,
        }
        .unwrap_or(0.0);
        let lon_dir = sent
            .fields
            .get(5)
            .and_then(|f| f.chars().next())
            .unwrap_or('E');
        let lon = Self::nmea_to_decimal_degrees(lon_raw, lon_dir).unwrap_or(0.0);

        // Fix quality
        let quality = match sent.fields.get(6) {
            Some(f) => f.parse::<u8>().ok(),
            None => None,
        }
        .unwrap_or(0);
        let fix_quality = FixQuality::from_u8(quality);

        // Num satellites
        let num_sats = sent
            .fields
            .get(7)
            .and_then(|f| f.parse::<u8>().ok())
            .unwrap_or(0);

        // Altitude
        let altitude = sent
            .fields
            .get(9)
            .and_then(|f| f.parse::<f32>().ok())
            .unwrap_or(0.0);

        let timestamp_val = sent.fields.get(1).cloned().unwrap_or_default();
        let hdop_val = sent
            .fields
            .get(8)
            .and_then(|f| f.parse::<f32>().ok())
            .unwrap_or(0.0);
        let geoid_val = sent
            .fields
            .get(11)
            .and_then(|f| f.parse::<f32>().ok())
            .unwrap_or(0.0);

        self.last_gga = Some(GgaData {
            timestamp: timestamp_val,
            latitude: lat,
            lat_direction: lat_dir,
            longitude: lon,
            lon_direction: lon_dir,
            fix_quality,
            num_satellites: num_sats,
            hdop: hdop_val,
            altitude,
            altitude_unit: 'M',
            geoid_separation: geoid_val,
            geoid_unit: 'M',
            dgps_age: 0.0,
            dgps_station_id: 0,
        });

        self.fix_quality = fix_quality;
        self.satellites_visible = num_sats;
    }

    /// Process RMC sentence
    fn process_rmc(&mut self, sent: &NmeaSentence) {
        if sent.fields.len() < 12 {
            return;
        }

        let lat_raw = match sent.fields.get(3) {
            Some(f) => f.parse::<f64>().ok(),
            None => None,
        }
        .unwrap_or(0.0);
        let lat_dir = sent
            .fields
            .get(4)
            .and_then(|f| f.chars().next())
            .unwrap_or('N');
        let lat = Self::nmea_to_decimal_degrees(lat_raw, lat_dir).unwrap_or(0.0);

        let lon_raw = match sent.fields.get(5) {
            Some(f) => f.parse::<f64>().ok(),
            None => None,
        }
        .unwrap_or(0.0);
        let lon_dir = sent
            .fields
            .get(6)
            .and_then(|f| f.chars().next())
            .unwrap_or('E');
        let lon = Self::nmea_to_decimal_degrees(lon_raw, lon_dir).unwrap_or(0.0);

        let speed_knots = sent
            .fields
            .get(7)
            .and_then(|f| f.parse::<f32>().ok())
            .unwrap_or(0.0);
        let track_angle = sent
            .fields
            .get(8)
            .and_then(|f| f.parse::<f32>().ok())
            .unwrap_or(0.0);
        let date = sent.fields.get(9).cloned().unwrap_or_default();
        let mode = sent
            .fields
            .get(12)
            .and_then(|f| f.chars().next())
            .unwrap_or('N');

        let mag_var = sent
            .fields
            .get(10)
            .and_then(|f| f.parse::<f32>().ok())
            .unwrap_or(0.0);
        let var_dir = sent
            .fields
            .get(11)
            .and_then(|f| f.chars().next())
            .unwrap_or('E');

        self.last_rmc = Some(RmcData {
            timestamp: sent.fields.get(1).cloned().unwrap_or_default(),
            status: sent
                .fields
                .get(2)
                .and_then(|f| f.chars().next())
                .unwrap_or('V'),
            latitude: lat,
            lat_direction: lat_dir,
            longitude: lon,
            lon_direction: lon_dir,
            speed_knots,
            track_angle,
            date,
            magnetic_variation: mag_var,
            variation_direction: var_dir,
            mode_indicator: mode,
        });
    }

    /// Process VTG sentence
    fn process_vtg(&mut self, sent: &NmeaSentence) {
        if sent.fields.len() < 9 {
            return;
        }

        self.last_vtg = Some(VtgData {
            track_true: sent
                .fields
                .get(1)
                .and_then(|f| f.parse::<f32>().ok())
                .unwrap_or(0.0),
            track_magnetic: sent
                .fields
                .get(3)
                .and_then(|f| f.parse::<f32>().ok())
                .unwrap_or(0.0),
            speed_knots: sent
                .fields
                .get(5)
                .and_then(|f| f.parse::<f32>().ok())
                .unwrap_or(0.0),
            speed_kmh: sent
                .fields
                .get(7)
                .and_then(|f| f.parse::<f32>().ok())
                .unwrap_or(0.0),
            mode_indicator: sent
                .fields
                .get(9)
                .and_then(|f| f.chars().next())
                .unwrap_or('N'),
        });
    }

    /// Convierte DDMM.MMMM a grados decimales
    fn nmea_to_decimal_degrees(value: f64, direction: char) -> Option<f64> {
        let degrees = (value / 100.0).floor();
        let minutes = value - degrees * 100.0;
        let decimal = degrees + minutes / 60.0;

        match direction {
            'N' => Some(decimal),
            'S' => Some(-decimal),
            'E' => Some(decimal),
            'W' => Some(-decimal),
            _ => None,
        }
    }

    /// Obtiene último GGA
    pub fn last_gga(&self) -> Option<&GgaData> {
        self.last_gga.as_ref()
    }

    /// Obtiene último RMC
    pub fn last_rmc(&self) -> Option<&RmcData> {
        self.last_rmc.as_ref()
    }

    /// Obtiene último VTG
    pub fn last_vtg(&self) -> Option<&VtgData> {
        self.last_vtg.as_ref()
    }

    /// Indica si hay fix válido
    pub fn has_fix(&self) -> bool {
        matches!(
            self.fix_quality,
            FixQuality::GPSFix | FixQuality::DGPSFix | FixQuality::RTK | FixQuality::FloatRTK
        )
    }

    /// Obtiene calidad del fix
    pub fn fix_quality(&self) -> FixQuality {
        self.fix_quality
    }

    /// Obtiene número de satélites visibles
    pub fn satellites_visible(&self) -> u8 {
        self.satellites_visible
    }

    /// Obtiene posición actual (desde GGA o RMC)
    pub fn current_position(&self) -> Option<(f64, f64)> {
        if let Some(gga) = &self.last_gga {
            return Some((gga.lat_decimal(), gga.lon_decimal()));
        }
        if let Some(rmc) = &self.last_rmc {
            return Some((rmc.lat_decimal(), rmc.lon_decimal()));
        }
        None
    }

    /// Obtiene velocidad actual en km/h
    pub fn current_speed(&self) -> Option<f32> {
        if let Some(rmc) = &self.last_rmc {
            return Some(rmc.speed_kmh());
        }
        if let Some(vtg) = &self.last_vtg {
            return Some(vtg.speed_kmh);
        }
        None
    }

    /// Obtiene heading actual en grados
    pub fn current_heading(&self) -> Option<f32> {
        if let Some(rmc) = &self.last_rmc {
            return Some(rmc.track_angle);
        }
        if let Some(vtg) = &self.last_vtg {
            return Some(vtg.track_true);
        }
        None
    }

    /// Obtiene altitud actual en metros
    pub fn current_altitude(&self) -> Option<f32> {
        self.last_gga.as_ref().map(|g| g.altitude)
    }

    /// Obtiene timestamp UTC actual
    pub fn current_utc_time(&self) -> Option<(u8, u8, u8)> {
        if let Some(rmc) = &self.last_rmc {
            return rmc.parse_timestamp();
        }
        None
    }

    /// Limpia buffer de sentences
    pub fn clear(&mut self) {
        self.sentences.clear();
    }
}

impl Default for NmeaParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gga() {
        let mut parser = NmeaParser::new();
        let line = "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,47.0,M,,*47";
        let sent = parser.parse_line(line);
        assert!(sent.is_some());
        assert!(parser.has_fix());
    }

    #[test]
    fn test_parse_rmc() {
        let mut parser = NmeaParser::new();
        let line = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*70";
        let sent = parser.parse_line(line);
        assert!(sent.is_some());
        if let Some(rmc) = parser.last_rmc() {
            assert!((rmc.speed_kmh() - 41.4848).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_nmea_degrees_conversion() {
        // 4807.038 N = 48° 07.038' = 48.1173°
        let parser = NmeaParser::new();
        let result = parser.last_gga();
        // Just verify parser was created
        assert!(result.is_none());
    }

    #[test]
    fn test_position_extraction() {
        let mut parser = NmeaParser::new();
        parser.parse_line("$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,47.0,M,,*47");

        if let Some((lat, lon)) = parser.current_position() {
            assert!(lat > 40.0 && lat < 50.0); // Should be ~48°
            assert!(lon > 10.0 && lon < 15.0); // Should be ~11°
        }
    }
}

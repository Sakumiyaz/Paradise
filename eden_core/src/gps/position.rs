//! # Position - Geographic position and velocity
//!
//! Tipos para representar posición geográfica y velocidad.
//! 100% original.
#![allow(unused_imports)]
#![allow(dead_code)]
use std::io::Write;

/// Coordenadas lat/lon
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatLon {
    pub lat: f64, // Latitud en grados (-90 a 90)
    pub lon: f64, // Longitud en grados (-180 a 180)
}

impl LatLon {
    /// Crea nueva coordenada
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    /// Crea desde grados, minutos, segundos
    pub fn from_dms(
        lat_deg: f64,
        lat_min: f64,
        lat_sec: f64,
        lon_deg: f64,
        lon_min: f64,
        lon_sec: f64,
        lat_dir: char,
        lon_dir: char,
    ) -> Self {
        let lat = lat_deg + lat_min / 60.0 + lat_sec / 3600.0;
        let lon = lon_deg + lon_min / 60.0 + lon_sec / 3600.0;

        let lat = if lat_dir == 'S' || lat_dir == 's' {
            -lat
        } else {
            lat
        };
        let lon = if lon_dir == 'W' || lon_dir == 'w' {
            -lon
        } else {
            lon
        };

        Self { lat, lon }
    }

    /// Verifica si es válido
    pub fn is_valid(&self) -> bool {
        self.lat >= -90.0 && self.lat <= 90.0 && self.lon >= -180.0 && self.lon <= 180.0
    }

    /// Distancia a otro punto en metros (fórmula Haversine)
    pub fn distance_to(&self, other: &LatLon) -> f64 {
        let r = 6371000.0; // Radio terrestre en metros

        let lat1 = self.lat.to_radians();
        let lat2 = other.lat.to_radians();
        let dlat = (other.lat - self.lat).to_radians();
        let dlon = (other.lon - self.lon).to_radians();

        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();

        r * c
    }

    /// Bearing (dirección) a otro punto en grados
    pub fn bearing_to(&self, other: &LatLon) -> f64 {
        let lat1 = self.lat.to_radians();
        let lat2 = other.lat.to_radians();
        let dlon = (other.lon - self.lon).to_radians();

        let y = dlon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();

        let bearing = y.atan2(x).to_degrees();
        (bearing + 360.0) % 360.0
    }

    /// Punto medio entre dos coordenadas
    pub fn midpoint_to(&self, other: &LatLon) -> LatLon {
        let lat1 = self.lat.to_radians();
        let lat2 = other.lat.to_radians();
        let lon1 = self.lon.to_radians();
        let dlon = (other.lon - self.lon).to_radians();

        let _bx = lat2.cos() * dlon.cos();
        let by = lat2.cos() * dlon.sin();

        let lat3 = ((lat1 + lat2) / 2.0).sqrt().asin();
        let lon3 = lon1 + (by / (lat3.cos())).atan2(1.0);

        LatLon::new(lat3.to_degrees(), lon3.to_degrees())
    }
}

impl Default for LatLon {
    fn default() -> Self {
        Self { lat: 0.0, lon: 0.0 }
    }
}

/// Posición completa con altitud
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub coords: LatLon,
    pub altitude: f64, // Metros sobre nivel del mar
    pub timestamp_ms: u64,
}

impl Position {
    /// Crea nueva posición
    pub fn new(lat: f64, lon: f64, altitude: f64) -> Self {
        Self {
            coords: LatLon::new(lat, lon),
            altitude,
            timestamp_ms: current_time_ms(),
        }
    }

    /// Desde LatLon
    pub fn from_coords(coords: LatLon, altitude: f64) -> Self {
        Self {
            coords,
            altitude,
            timestamp_ms: current_time_ms(),
        }
    }

    /// Verifica si tiene datos válidos
    pub fn is_valid(&self) -> bool {
        self.coords.is_valid()
    }

    /// Distancia a otra posición en metros
    pub fn distance_to(&self, other: &Position) -> f64 {
        // Distancia horizontal
        let horizontal = self.coords.distance_to(&other.coords);
        // Diferencia de altitud
        let vertical = (self.altitude - other.altitude).abs();
        // Distancia 3D
        (horizontal.powi(2) + vertical.powi(2)).sqrt()
    }

    /// Actualiza timestamp
    pub fn update_timestamp(&mut self) {
        self.timestamp_ms = current_time_ms();
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            coords: LatLon::default(),
            altitude: 0.0,
            timestamp_ms: 0,
        }
    }
}

/// Velocidad vectorial
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    pub speed: f32,    // Metros por segundo
    pub heading: f32,  // Grados (0-360, 0=North)
    pub vertical: f32, // Metros por segundo (positivo = ascendente)
}

impl Velocity {
    /// Crea nueva velocidad
    pub fn new(speed: f32, heading: f32, vertical: f32) -> Self {
        Self {
            speed,
            heading: heading % 360.0,
            vertical,
        }
    }

    /// Desde nudos y grados
    pub fn from_knots_and_degrees(speed_knots: f32, heading_degrees: f32) -> Self {
        Self::new(speed_knots * 0.514444, heading_degrees, 0.0)
    }

    /// Speed en km/h
    pub fn speed_kmh(&self) -> f32 {
        self.speed * 3.6
    }

    /// Speed en mph
    pub fn speed_mph(&self) -> f32 {
        self.speed * 2.23694
    }

    /// Speed en nudos
    pub fn speed_knots(&self) -> f32 {
        self.speed * 1.94384
    }

    /// Indica si está en movimiento
    pub fn is_moving(&self) -> bool {
        self.speed > 0.5 // Más de 0.5 m/s
    }

    /// Calcula distancia recorrida en tiempo dado
    pub fn distance_in_time(&self, seconds: f32) -> f32 {
        self.speed * seconds
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            speed: 0.0,
            heading: 0.0,
            vertical: 0.0,
        }
    }
}

/// Timestamp helper
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

    #[test]
    fn test_latlon_creation() {
        let coord = LatLon::new(40.7128, -74.0060); // Nueva York
        assert!(coord.is_valid());
    }

    #[test]
    fn test_distance_calculation() {
        let nyc = LatLon::new(40.7128, -74.0060);
        let london = LatLon::new(51.5074, -0.1278);

        let dist = nyc.distance_to(&london);
        // Should be approximately 5,570 km
        assert!(dist > 5_000_000.0 && dist < 6_000_000.0);
    }

    #[test]
    fn test_bearing_calculation() {
        let start = LatLon::new(0.0, 0.0);
        let end = LatLon::new(0.0, 10.0);

        let bearing = start.bearing_to(&end);
        assert!((bearing - 90.0).abs() < 1.0); // Should be roughly East = 90
    }

    #[test]
    fn test_velocity_conversion() {
        let v = Velocity::from_knots_and_degrees(10.0, 180.0);
        assert!((v.speed - 5.14).abs() < 0.1); // ~5.14 m/s
    }

    #[test]
    fn test_position_3d_distance() {
        let p1 = Position::new(0.0, 0.0, 100.0);
        let p2 = Position::new(0.001, 0.0, 110.0);

        let dist = p1.distance_to(&p2);
        assert!(dist > 0.0);
    }
}

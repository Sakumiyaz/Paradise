//! # GPS Utils - Helper functions for GPS calculations
//!
//! Funciones utilitarias para navegación y geodesia.
//! 100% original.

#![allow(dead_code)]

use super::position::LatLon;

/// Calcula distancia entre dos puntos usando Haversine
pub fn calculate_distance(p1: &LatLon, p2: &LatLon) -> f64 {
    p1.distance_to(p2)
}

/// Calcula bearing (dirección inicial) entre dos puntos
pub fn bearing_to(p1: &LatLon, p2: &LatLon) -> f64 {
    p1.bearing_to(p2)
}

/// Calcula punto de destino dado una distancia y dirección
pub fn destination_point(start: &LatLon, distance_meters: f64, bearing_degrees: f64) -> LatLon {
    let earth_radius = 6371000.0; // metros

    let lat1 = start.lat.to_radians();
    let lon1 = start.lon.to_radians();
    let bearing = bearing_degrees.to_radians();

    let lat2 = (lat1.sin() * (distance_meters / earth_radius).cos()
        + lat1.cos() * (distance_meters / earth_radius).sin() * bearing.cos())
    .asin();

    let lon2 =
        lon1 + (bearing.sin() * (distance_meters / earth_radius).sin() / lat2.cos()).atan2(1.0);

    LatLon::new(lat2.to_degrees(), lon2.to_degrees())
}

/// Calcula intersección de dos líneas (cruce de caminos)
pub fn line_intersection(p1: &LatLon, bearing1: f64, p2: &LatLon, bearing2: f64) -> Option<LatLon> {
    let lat1 = p1.lat.to_radians();
    let lon1 = p1.lon.to_radians();
    let lat2 = p2.lat.to_radians();
    let lon2 = p2.lon.to_radians();

    let b1 = bearing1.to_radians();
    let b2 = bearing2.to_radians();

    let dlon = lon2 - lon1;

    let la = (dlon.sin() * b2.cos()).asin();
    let lb = lat2.sin() - lat1.sin() * la.cos();
    let lc = b1.sin() * dlon.cos();

    let lad = (la.sin() * lc - lb.cos() * b2.sin() * dlon.sin())
        .atan2(lat1.cos() * b2.cos() + lat1.sin() * la.sin() * b2.sin());

    let lon3 = lon1 + (lb.sin() * dlon.cos() - lc.sin()).atan2(b1.cos());

    Some(LatLon::new(lad.to_degrees(), lon3.to_degrees()))
}

/// Convierte coordenadas a formato DMS (Degrees-Minutes-Seconds)
pub fn to_dms(decimal_degrees: f64) -> (i32, i32, f32, char) {
    let absolute = decimal_degrees.abs();
    let degrees = absolute.floor() as i32;
    let minutes_float = (absolute - degrees as f64) * 60.0;
    let minutes = minutes_float.floor() as i32;
    let seconds = ((minutes_float - minutes as f64) * 60.0) as f32;
    let direction = if decimal_degrees >= 0.0 { 'N' } else { 'S' };
    (degrees, minutes, seconds, direction)
}

/// Convierte longitud a DMS
pub fn lon_to_dms(decimal_degrees: f64) -> (i32, i32, f32, char) {
    let absolute = decimal_degrees.abs();
    let degrees = absolute.floor() as i32;
    let minutes_float = (absolute - degrees as f64) * 60.0;
    let minutes = minutes_float.floor() as i32;
    let seconds = ((minutes_float - minutes as f64) * 60.0) as f32;
    let direction = if decimal_degrees >= 0.0 { 'E' } else { 'W' };
    (degrees, minutes, seconds, direction)
}

/// Formatea LatLon como string legible
pub fn format_coordinates(lat: f64, lon: f64) -> String {
    let (lat_d, lat_m, lat_s, lat_dir) = to_dms(lat);
    let (lon_d, lon_m, lon_s, lon_dir) = lon_to_dms(lon);
    format!(
        "{:02}°{:02}'{:06.3}\"{} {:03}°{:02}'{:06.3}\"{}",
        lat_d, lat_m, lat_s, lat_dir, lon_d, lon_m, lon_s, lon_dir
    )
}

/// Verifica si un punto está dentro de un círculo geográfico
pub fn point_in_circle(center: &LatLon, radius_meters: f64, point: &LatLon) -> bool {
    center.distance_to(point) <= radius_meters
}

/// Calcula perpendicular distance desde un punto a una línea
pub fn perpendicular_distance(line_start: &LatLon, line_end: &LatLon, point: &LatLon) -> f64 {
    let d = line_start.distance_to(line_end);
    if d == 0.0 {
        return point.distance_to(line_start);
    }

    let lat1 = line_start.lat.to_radians();
    let lon1 = line_start.lon.to_radians();
    let _lat2 = line_end.lat.to_radians();
    let _lon2 = line_end.lon.to_radians();
    let lat3 = point.lat.to_radians();
    let lon3 = point.lon.to_radians();

    let d13 = ((lat3 - lat1).powi(2) + (lon3 - lon1).powi(2)).sqrt();
    let d12 = d / 6371000.0; // Distancia en radians

    let _bearing12 = line_start.bearing_to(line_end).to_radians();
    let _bearing13 = line_start.bearing_to(point).to_radians();

    let cross_track = d13.sin().asin() * 6371000.0;

    let _along_track = (d13.cos().asin() * 6371000.0 - d12.cos().asin() * 6371000.0).abs();

    cross_track.abs()
}

/// Normaliza ángulo a rango 0-360
pub fn normalize_angle(angle: f64) -> f64 {
    let mut a = angle % 360.0;
    if a < 0.0 {
        a += 360.0;
    }
    a
}

/// Diferencia angular más corta
pub fn angle_difference(a: f64, b: f64) -> f64 {
    let diff = (b - a) % 360.0;
    if diff > 180.0 {
        diff - 360.0
    } else if diff < -180.0 {
        diff + 360.0
    } else {
        diff
    }
}

/// Tiempo estimado de llegada dado velocidad y distancia
pub fn eta_seconds(distance_meters: f64, speed_mps: f32) -> f64 {
    if speed_mps <= 0.0 {
        f64::INFINITY
    } else {
        (distance_meters / speed_mps as f64).round()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destination_point() {
        let start = LatLon::new(0.0, 0.0);
        let dest = destination_point(&start, 1000000.0, 90.0); // 1000km East
        assert!(dest.lat.abs() < 1.0);
        assert!(dest.lon > 0.0);
    }

    #[test]
    fn test_format_coordinates() {
        let formatted = format_coordinates(40.7128, -74.0060);
        assert!(formatted.contains("N"));
        assert!(formatted.contains("W"));
    }

    #[test]
    fn test_point_in_circle() {
        let center = LatLon::new(0.0, 0.0);
        let point = LatLon::new(0.01, 0.01);
        assert!(point_in_circle(&center, 2000.0, &point)); // ~1.5km
    }

    #[test]
    fn test_normalize_angle() {
        assert!((normalize_angle(450.0) - 90.0).abs() < 0.001);
        assert!((normalize_angle(-90.0) - 270.0).abs() < 0.001);
    }

    #[test]
    fn test_angle_difference() {
        assert!((angle_difference(10.0, 350.0) - (-20.0)).abs() < 0.001);
    }
}

//! EdenTime - Sistema de tiempo propio, sin dependencias externas
//! Implementa timestamps UNIX sin usar chrono
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdenTime {
    pub unix_secs: u64,
    pub unix_nanos: u64,
}

impl EdenTime {
    /// Obtener tiempo actual
    pub fn now() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();
        Self {
            unix_secs: now.as_secs(),
            unix_nanos: now.subsec_nanos() as u64,
        }
    }

    /// Crear desde segundos UNIX
    pub fn from_secs(secs: u64) -> Self {
        Self {
            unix_secs: secs,
            unix_nanos: 0,
        }
    }

    /// Convertir a RFC3339 string (formato estándar)
    pub fn to_rfc3339(&self) -> String {
        // Simplified RFC3339: 2024-01-15T10:30:00.123Z
        let secs = self.unix_secs;
        let nanos = self.unix_nanos;

        // Calculate date components manually (no chrono!)
        let days = secs / 86400;
        let secs_in_day = secs % 86400;
        let hours = secs_in_day / 3600;
        let mins = (secs_in_day % 3600) / 60;
        let secs_in_min = secs_in_day % 60;

        // Reference: 1970-01-01 was a Thursday (day 0)
        let mut year = 1970;
        let mut remaining_days = days as i64;

        // Calculate year
        loop {
            let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            year += 1;
        }

        // Calculate month
        let is_leap = Self::is_leap_year(year);
        let days_in_months = if is_leap {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        let mut month = 1;
        for &days_in_month in &days_in_months {
            if remaining_days < days_in_month as i64 {
                break;
            }
            remaining_days -= days_in_month as i64;
            month += 1;
        }

        let day = remaining_days + 1;

        // Format with nanoseconds if present
        if nanos > 0 {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}Z",
                year, month, day, hours, mins, secs_in_min, nanos
            )
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                year, month, day, hours, mins, secs_in_min
            )
        }
    }

    /// Parse RFC3339 string (parsing from stored timestamps)
    pub fn parse_rfc3339(s: &str) -> Option<Self> {
        // Expected format: 2024-01-15T10:30:00.123Z or 2024-01-15T10:30:00Z
        let s = s.trim();

        // Find the date/time separator
        let t_pos = s.find('T')?;
        let z_pos = s.find('Z').or_else(|| s.find('z'))?;

        let date_part = &s[..t_pos];
        let time_part = &s[t_pos + 1..z_pos];

        // Parse date: YYYY-MM-DD
        let date_parts: Vec<u64> = date_part.split('-')
            .filter_map(|p| p.parse().ok())
            .collect();

        if date_parts.len() != 3 {
            return None;
        }
        let (year, month, day) = (date_parts[0], date_parts[1], date_parts[2]);

        // Parse time: HH:MM:SS[.nnn]
        let time_parts: Vec<&str> = time_part.split(':').collect();
        if time_parts.len() != 3 {
            return None;
        }

        let hours: u64 = time_parts[0].parse().ok()?;
        let mins: u64 = time_parts[1].parse().ok()?;

        // Handle optional nanoseconds
        let (secs, nanos) = if time_parts[2].contains('.') {
            let parts: Vec<&str> = time_parts[2].split('.').collect();
            let secs: u64 = parts[0].parse().ok()?;
            let nanos: u64 = if parts.len() > 1 {
                let n = parts[1].trim_end_matches('Z');
                let padded = format!("{:<9}", n).chars().take(9).collect::<String>();
                padded.parse().unwrap_or(0)
            } else {
                0
            };
            (secs, nanos)
        } else {
            let secs: u64 = time_parts[2].trim_end_matches('Z').parse().ok()?;
            (secs, 0)
        };

        // Calculate UNIX timestamp
        let total_days = Self::days_since_epoch(year, month, day);
        let total_secs = total_days * 86400 + hours * 3600 + mins * 60 + secs;

        Some(Self {
            unix_secs: total_secs,
            unix_nanos: nanos,
        })
    }

    fn is_leap_year(year: u64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    fn days_since_epoch(year: u64, month: u64, day: u64) -> u64 {
        let mut total = 0;

        // Add days for full years since 1970
        for y in 1970..year {
            total += if Self::is_leap_year(y) { 366 } else { 365 };
        }

        // Add days for full months this year
        let days_in_months = if Self::is_leap_year(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        for m in 1..month {
            total += days_in_months[(m - 1) as usize] as u64;
        }

        // Add days
        total += day - 1;

        total
    }

    /// Diferencia en segundos entre dos tiempos
    pub fn diff_secs(&self, other: &EdenTime) -> i64 {
        let diff = (self.unix_secs as i64) - (other.unix_secs as i64);
        diff
    }

    /// Verificar si es más reciente que otro tiempo
    pub fn is_newer_than(&self, other: &EdenTime) -> bool {
        self.unix_secs > other.unix_secs ||
        (self.unix_secs == other.unix_secs && self.unix_nanos > other.unix_nanos)
    }
}

impl Default for EdenTime {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for EdenTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_rfc3339())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        let t = EdenTime::now();
        assert!(t.unix_secs > 0);
    }

    #[test]
    fn test_rfc3339_roundtrip() {
        let t = EdenTime::now();
        let s = t.to_rfc3339();
        let parsed = EdenTime::parse_rfc3339(&s);
        assert!(parsed.is_some());
        let p = parsed.unwrap();
        assert_eq!(t.unix_secs, p.unix_secs);
    }

    #[test]
    fn test_leap_year() {
        assert!(!EdenTime::is_leap_year(2023));
        assert!(EdenTime::is_leap_year(2024));
        assert!(!EdenTime::is_leap_year(2100)); // not leap - divisible by 100
        assert!(EdenTime::is_leap_year(2000)); // is leap - divisible by 400
    }

    #[test]
    fn test_known_date() {
        // Jan 1, 1970 should be day 0
        let t = EdenTime::from_secs(0);
        let s = t.to_rfc3339();
        assert!(s.starts_with("1970-01-01"));
    }
}

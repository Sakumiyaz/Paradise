//! # Serial - RS232/USB Serial communication
//!
//! Comunicación serial via /dev/tty*.

#![allow(dead_code)]

use std::fs::{File, OpenOptions};
use std::io::{Read, Result as IoResult, Write};
use std::path::Path;

/// Configuración de puerto serial
#[derive(Debug, Clone)]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub data_bits: u8, // 5, 6, 7, 8
    pub stop_bits: u8, // 1, 2
    pub parity: Parity,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            baud_rate: 9600,
            data_bits: 8,
            stop_bits: 1,
            parity: Parity::None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Parity {
    None,
    Even,
    Odd,
}

/// Puerto serial abierto
pub struct SerialPort {
    file: File,
    config: SerialConfig,
}

impl SerialPort {
    /// Abre un puerto serial
    pub fn open(path: &Path, config: SerialConfig) -> IoResult<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;

        let mut port = Self { file, config };
        port.apply_config()?;
        Ok(port)
    }

    /// Aplica configuración termios
    fn apply_config(&mut self) -> IoResult<()> {
        use std::os::unix::io::AsRawFd;

        // Get current config
        let fd = self.file.as_raw_fd();

        // En Linux, usar termios2 para baud rates arbitrarios
        // Por ahora solo establecemos flags básicos

        // Configuración simple: 8N1, raw mode
        let _options = std::fs::OpenOptions::new();

        // Usar libc para configuraciones reales
        unsafe {
            use libc::termios;
            let mut term: termios = std::mem::zeroed();

            // tcgetattr
            if libc::tcgetattr(fd, &mut term) == 0 {
                // Set raw mode
                libc::cfmakeraw(&mut term);

                // Set baud rate
                let baud = self.config.baud_rate;
                libc::cfsetispeed(&mut term, baud);
                libc::cfsetospeed(&mut term, baud);

                // Set attributes
                libc::tcsetattr(fd, libc::TCSANOW, &mut term);
            }
        }

        Ok(())
    }

    /// Lee datos disponibles
    pub fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.file.read(buf)
    }

    /// Escribe datos
    pub fn write(&mut self, data: &[u8]) -> IoResult<usize> {
        self.file.write(data)
    }

    /// Lee hasta encontrar un delimitador o timeout
    pub fn read_until(&mut self, delimiter: u8, timeout_ms: u64) -> IoResult<Vec<u8>> {
        let mut result = Vec::new();
        let start = std::time::Instant::now();

        while start.elapsed().as_millis() < timeout_ms as u128 {
            let mut byte = [0u8];
            match self.file.read(&mut byte) {
                Ok(0) => break,
                Ok(1) => {
                    result.push(byte[0]);
                    if byte[0] == delimiter {
                        break;
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_micros(100));
                }
                Err(e) => return Err(e),
            }
        }

        Ok(result)
    }

    /// Flush de buffers
    pub fn flush(&mut self) -> IoResult<()> {
        self.file.flush()
    }
}

/// Lista puertos seriales disponibles
pub fn list_ports() -> Vec<String> {
    let mut ports = Vec::new();

    // Common Linux serial ports
    let patterns = [
        "/dev/ttyUSB",
        "/dev/ttyACM",
        "/dev/ttyS",
        "/dev/ttyUSB",
        "/dev/cu.",
    ];

    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            for pattern in &patterns {
                if name.starts_with(pattern) || name.contains("usb") || name.contains("acm") {
                    if !ports.contains(&name) {
                        ports.push(format!("/dev/{}", name));
                    }
                }
            }
        }
    }

    ports.sort();
    ports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_ports() {
        let ports = list_ports();
        println!("Available ports: {:?}", ports);
    }
}

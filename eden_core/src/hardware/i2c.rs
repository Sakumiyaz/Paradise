//! # I2C - Inter-Integrated Circuit
//!
//! Comunicación I2C via /dev/i2c-* o /dev/i2c/

#![allow(dead_code)]

use std::fs::OpenOptions;
use std::io::{Read, Result as IoResult, Write};

/// Bus I2C
pub struct I2CBus {
    bus: u8,
    file: std::fs::File,
}

impl I2CBus {
    /// Abre un bus I2C
    pub fn new(bus: u8) -> IoResult<Self> {
        let path = format!("/dev/i2c-{}", bus);
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        Ok(Self { bus, file })
    }

    /// Escribe a un dispositivo
    pub fn write(&mut self, addr: u8, data: &[u8]) -> IoResult<()> {
        use std::os::unix::io::AsRawFd;

        let fd = self.file.as_raw_fd();

        // Set slave address
        unsafe {
            libc::ioctl(fd, I2C_SLAVE, addr as u64);
        }

        // Write data
        self.file.write_all(data)
    }

    /// Lee de un dispositivo
    pub fn read(&mut self, addr: u8, buf: &mut [u8]) -> IoResult<usize> {
        use std::os::unix::io::AsRawFd;

        let fd = self.file.as_raw_fd();

        // Set slave address
        unsafe {
            libc::ioctl(fd, I2C_SLAVE, addr as u64);
        }

        self.file.read(buf)
    }

    /// Lee un registro específico
    pub fn read_register(&mut self, addr: u8, reg: u8, buf: &mut [u8]) -> IoResult<()> {
        self.write(addr, &[reg])?;
        self.read(addr, buf)?;
        Ok(())
    }

    /// Escribe a un registro específico
    pub fn write_register(&mut self, addr: u8, reg: u8, data: u8) -> IoResult<()> {
        self.write(addr, &[reg, data])
    }

    /// Scan del bus - busca dispositivos
    pub fn scan(&mut self) -> Vec<u8> {
        Self::scan_with_probe(|addr| self.probe(addr))
    }

    /// Ejecuta el rango estándar de scan usando una función de probe inyectada.
    ///
    /// La ruta real usa `ioctl`; esta ruta pura permite validar el contrato de
    /// descubrimiento sin requerir `/dev/i2c-*`.
    pub fn scan_with_probe<F>(mut probe: F) -> Vec<u8>
    where
        F: FnMut(u8) -> bool,
    {
        let mut devices = Vec::new();

        for addr in 0x03..0x78 {
            if probe(addr) {
                devices.push(addr);
            }
        }

        devices
    }

    fn probe(&mut self, addr: u8) -> bool {
        let _buf = [0u8];
        use std::os::unix::io::AsRawFd;

        let fd = self.file.as_raw_fd();

        unsafe {
            // Try to set address
            if libc::ioctl(fd, I2C_SLAVE, addr as u64) < 0 {
                return false;
            }

            // Try to write nothing (ping)
            let result = libc::write(fd, std::ptr::null(), 0);
            result >= 0
        }
    }
}

// I2C ioctl codes
const I2C_SLAVE: u64 = 0x0703;
const I2C_FUNCS: u64 = 0x0705;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_mock_probe_reports_expected_devices() {
        let devices = I2CBus::scan_with_probe(|addr| matches!(addr, 0x1d | 0x68));

        assert_eq!(devices, vec![0x1d, 0x68]);
    }

    #[test]
    fn test_scan_mock_probe_respects_i2c_address_range() {
        let mut seen = Vec::new();
        let devices = I2CBus::scan_with_probe(|addr| {
            seen.push(addr);
            false
        });

        assert!(devices.is_empty());
        assert_eq!(seen.first(), Some(&0x03));
        assert_eq!(seen.last(), Some(&0x77));
    }

    #[test]
    #[cfg_attr(not(feature = "external-tests"), ignore)]
    fn test_scan() {
        if let Ok(mut bus) = I2CBus::new(1) {
            let devices = bus.scan();
            println!("Found I2C devices: {:?}", devices);
        }
    }
}

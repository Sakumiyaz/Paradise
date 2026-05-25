//! # SPI - Serial Peripheral Interface
//!
//! Comunicación SPI via /dev/spidev*
#![allow(unused_imports)]
#![allow(dead_code)]
use std::io::Write;

use std::fs::File;
use std::io::Result as IoResult;
use std::path::Path;

/// Configuración SPI
#[derive(Debug, Clone)]
pub struct SpiConfig {
    pub mode: SpiMode,
    pub bits_per_word: u8,
    pub max_speed: u32,
    pub delay_us: u16,
}

impl Default for SpiConfig {
    fn default() -> Self {
        Self {
            mode: SpiMode::Mode0,
            bits_per_word: 8,
            max_speed: 1_000_000, // 1 MHz
            delay_us: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpiMode {
    Mode0, // CPOL=0, CPHA=0
    Mode1, // CPOL=0, CPHA=1
    Mode2, // CPOL=1, CPHA=0
    Mode3, // CPOL=1, CPHA=1
}

impl SpiMode {
    fn to_u8(&self) -> u8 {
        match self {
            SpiMode::Mode0 => 0,
            SpiMode::Mode1 => 1,
            SpiMode::Mode2 => 2,
            SpiMode::Mode3 => 3,
        }
    }
}

/// Estructura SPI para ioctl (definida en kernel)
#[repr(C)]
struct SpiIocTransfer {
    tx_buf: *mut libc::c_void,
    rx_buf: *mut libc::c_void,
    len: u32,
    speed_hz: u32,
    delay_usecs: u16,
    bits_per_word: u8,
    cs_change: u8,
    tx_nbits: u8,
    rx_nbits: u8,
    pad: u32,
}

impl Default for SpiIocTransfer {
    fn default() -> Self {
        Self {
            tx_buf: std::ptr::null_mut(),
            rx_buf: std::ptr::null_mut(),
            len: 0,
            speed_hz: 0,
            delay_usecs: 0,
            bits_per_word: 0,
            cs_change: 0,
            tx_nbits: 0,
            rx_nbits: 0,
            pad: 0,
        }
    }
}

/// Bus SPI
pub struct SpiBus {
    file: File,
    config: SpiConfig,
}

impl SpiBus {
    /// Abre un dispositivo SPI
    pub fn open(path: &Path, config: SpiConfig) -> IoResult<Self> {
        let file = std::fs::File::open(path)?;
        let mut bus = Self { file, config };
        bus.apply_config()?;
        Ok(bus)
    }

    fn apply_config(&mut self) -> IoResult<()> {
        use std::os::unix::io::AsRawFd;

        let fd = self.file.as_raw_fd();

        unsafe {
            // Set SPI mode
            let mode = self.config.mode.to_u8() as u32;
            libc::ioctl(fd, SPI_IOC_WR_MODE, &mode);

            // Set bits per word
            let bits = self.config.bits_per_word as u32;
            libc::ioctl(fd, SPI_IOC_WR_BITS_PER_WORD, &bits);

            // Set max speed
            let speed = self.config.max_speed as u32;
            libc::ioctl(fd, SPI_IOC_WR_MAX_SPEED_HZ, &speed);
        }

        Ok(())
    }

    /// Transferencia full duplex
    pub fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> IoResult<()> {
        use std::os::unix::io::AsRawFd;

        let fd = self.file.as_raw_fd();

        // Build iovec
        let tx_ptr = tx.as_ptr() as *mut libc::c_void;
        let rx_ptr = rx.as_mut_ptr() as *mut libc::c_void;

        let msg = SpiIocTransfer {
            tx_buf: tx_ptr,
            rx_buf: rx_ptr,
            len: tx.len() as u32,
            speed_hz: self.config.max_speed,
            delay_usecs: self.config.delay_us,
            bits_per_word: self.config.bits_per_word,
            ..Default::default()
        };

        unsafe {
            libc::ioctl(fd, 0x40206b00, &msg);
        }

        Ok(())
    }

    /// Lee bytes
    pub fn read(&mut self, buf: &mut [u8]) -> IoResult<()> {
        let tx = vec![0u8; buf.len()];
        self.transfer(&tx, buf)
    }

    /// Escribe bytes
    pub fn write(&mut self, data: &[u8]) -> IoResult<()> {
        let mut rx = vec![0u8; data.len()];
        self.transfer(data, &mut rx)
    }
}

// SPI ioctl codes
const SPI_IOC_WR_MODE: u64 = 0x40016b01;
const SPI_IOC_RD_MODE: u64 = 0x80016b01;
const SPI_IOC_WR_BITS_PER_WORD: u64 = 0x40016b03;
const SPI_IOC_RD_BITS_PER_WORD: u64 = 0x80016b03;
const SPI_IOC_WR_MAX_SPEED_HZ: u64 = 0x40016b04;
const SPI_IOC_RD_MAX_SPEED_HZ: u64 = 0x80016b04;

// SPI_IOC_MESSAGE(n) macro expansion: creates ioctl number for n messages
#[allow(unused_macros)]
macro_rules! SPI_IOC_MESSAGE {
    ($n:expr) => {
        0x40206b00 | ($n as u32)
    };
}

/// Lista dispositivos SPI disponibles
pub fn list_devices() -> Vec<String> {
    let mut devices = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("spidev") {
                devices.push(format!("/dev/{}", name));
            }
        }
    }

    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() {
        let devices = list_devices();
        println!("SPI devices: {:?}", devices);
    }
}

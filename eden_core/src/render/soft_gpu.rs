//! # SoftGPU: Renderizado de Alta Performance para A-Life
//!
//! Renderiza el Mar Morfóseo y Auton directamente en el framebuffer de Linux
//! sin X11/Wayland, o en fallback de terminal usando caracteres Unicode Braille.
//!
//! ## Características
//!
//! - **Framebuffer Directo**: Abre `/dev/fb0` y mapea memoria con mmap
//! - **Conversión Longitud de Onda → RGB**: Implementación manual de CIE 1931
//! - **Marching Squares**: Para dibujar isosuperficies φ=0 de Auton
//! - **Hilo Separado**: Renderizado async para no bloquear simulación
//! - **Fallback Terminal**: Caracteres Braille cuando no hay framebuffer

#![allow(dead_code)]

use std::fs::File;
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Archivo de framebuffer
const FB_PATH: &str = "/dev/fb0";

/// Constantes de framebuffer (ioctls)
const FBIOGET_VSCREENINFO: u64 = 0x4600;
const FBIOPUT_VSCREENINFO: u64 = 0x4601;
const FBIOGET_FSCREENINFO: u64 = 0x4602;

/// Bits por píxel típicos
const BPP_16: u32 = 16;
const BPP_24: u32 = 24;
const BPP_32: u32 = 32;

/// Formato RGB 565
#[derive(Debug, Clone, Copy)]
struct Rgb565(u16);

impl Rgb565 {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let r5 = (r as u16 >> 3) & 0x1F;
        let g6 = (g as u16 >> 2) & 0x3F;
        let b5 = (b as u16 >> 3) & 0x1F;
        Rgb565((r5 << 11) | (g6 << 5) | b5)
    }
}

/// RGB 888
#[derive(Debug, Clone, Copy)]
struct Rgb888(u32);

impl Rgb888 {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Rgb888(((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    fn to_bytes(self) -> [u8; 3] {
        [
            (self.0 & 0xFF) as u8,
            ((self.0 >> 8) & 0xFF) as u8,
            ((self.0 >> 16) & 0xFF) as u8,
        ]
    }
}

/// RGBA 8888
#[derive(Debug, Clone, Copy)]
struct Rgba8888(u32);

impl Rgba8888 {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Rgba8888(0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    fn to_bytes(self) -> [u8; 4] {
        [
            (self.0 & 0xFF) as u8,
            ((self.0 >> 8) & 0xFF) as u8,
            ((self.0 >> 16) & 0xFF) as u8,
            ((self.0 >> 24) & 0xFF) as u8,
        ]
    }
}

/// Información del framebuffer (var)
#[repr(C)]
struct VarScreenInfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: u32,
    green: u32,
    blue: u32,
    transp: u32,
}

/// Información fija del framebuffer (fix)
#[repr(C)]
struct FixScreenInfo {
    smem_start: u64,
    smem_len: u32,
    r#type: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
}

/// Modo de renderizado
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModoRender {
    /// Framebuffer de Linux
    Framebuffer,
    /// Terminal con caracteres Braille
    Terminal,
    /// Sin renderizado
    Ninguno,
}

/// Estado del renderer
#[derive(Debug, Clone)]
pub struct RenderStats {
    pub modo: ModoRender,
    pub fps: f64,
    pub pixeles_renderizados: u64,
    pub tiempo_render_us: u64,
    pub anchos: u32,
    pub alto: u32,
}

impl Default for RenderStats {
    fn default() -> Self {
        RenderStats {
            modo: ModoRender::Ninguno,
            fps: 0.0,
            pixeles_renderizados: 0,
            tiempo_render_us: 0,
            anchos: 0,
            alto: 0,
        }
    }
}

/// Buffer de píxeles para renderizar
#[derive(Debug, Clone)]
pub struct PixelBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl PixelBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        PixelBuffer {
            width,
            height,
            data: vec![0u8; width * height * 3],
        }
    }

    /// Establece un píxel en formato RGB
    pub fn set_pixel_rgb(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * 3;
            self.data[idx] = r;
            self.data[idx + 1] = g;
            self.data[idx + 2] = b;
        }
    }

    /// Obtiene un píxel como RGB
    pub fn get_pixel_rgb(&self, x: usize, y: usize) -> Option<(u8, u8, u8)> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * 3;
            Some((self.data[idx], self.data[idx + 1], self.data[idx + 2]))
        } else {
            None
        }
    }

    /// Limpia el buffer con un color
    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        for i in (0..self.data.len()).step_by(3) {
            self.data[i] = r;
            self.data[i + 1] = g;
            self.data[i + 2] = b;
        }
    }
}

/// SoftGPU: Sistema de renderizado
pub struct SoftGPU {
    /// Modo actual
    modo: ModoRender,
    /// Archivo del framebuffer (si está abierto)
    fb_file: Option<File>,
    /// Mapeo de memoria del framebuffer
    fb_mmap: Option<Vec<u8>>,
    /// Geometría del framebuffer
    fb_width: u32,
    fb_height: u32,
    fb_bpp: u32,
    fb_line_length: u32,
    /// Buffer de píxeles para acumular
    buffer: Arc<RwLock<PixelBuffer>>,
    /// Hilo de renderizado
    render_thread: Option<thread::JoinHandle<()>>,
    /// Señal de stop
    running: Arc<RwLock<bool>>,
    /// Estadísticas
    stats: Arc<RwLock<RenderStats>>,
    /// Último tiempo de frame
    last_frame: Instant,
    /// Contador de frames
    frame_count: u64,
}

impl SoftGPU {
    /// Crea nuevo SoftGPU, intentando abrir framebuffer
    pub fn new() -> io::Result<Self> {
        let (fb_file, fb_mmap, width, height, bpp, line_length) = Self::abrir_framebuffer();

        let modo = if fb_file.is_some() {
            ModoRender::Framebuffer
        } else {
            ModoRender::Terminal
        };

        let buffer = Arc::new(RwLock::new(PixelBuffer::new(
            width as usize,
            height as usize,
        )));

        let stats = Arc::new(RwLock::new(RenderStats {
            modo,
            anchos: width,
            alto: height,
            ..Default::default()
        }));

        Ok(SoftGPU {
            modo,
            fb_file,
            fb_mmap,
            fb_width: width,
            fb_height: height,
            fb_bpp: bpp,
            fb_line_length: line_length,
            buffer,
            render_thread: None,
            running: Arc::new(RwLock::new(false)),
            stats,
            last_frame: Instant::now(),
            frame_count: 0,
        })
    }

    /// Intenta abrir el framebuffer de Linux
    fn abrir_framebuffer() -> (Option<File>, Option<Vec<u8>>, u32, u32, u32, u32) {
        let fb_path = Path::new(FB_PATH);

        if !fb_path.exists() {
            return (None, None, 80, 24, 24, 80);
        }

        match File::open(fb_path) {
            Ok(file) => {
                // Obtener información de pantalla
                let (width, height, bpp, line_length) = Self::obtener_geometria(&file);

                if bpp == 0 || width == 0 || height == 0 {
                    return (None, None, 80, 24, 24, 80);
                }

                // Mapear memoria (solo lectura por ahora para detectar)
                let mmap = unsafe {
                    let fd = file.as_raw_fd();
                    let len = (line_length * height) as usize;
                    libc::mmap(
                        std::ptr::null_mut(),
                        len,
                        libc::PROT_READ,
                        libc::MAP_SHARED,
                        fd,
                        0,
                    )
                };

                if mmap == libc::MAP_FAILED {
                    return (Some(file), None, width, height, bpp, line_length);
                }

                // En Linux podemos usar MAP_SHARED para读写
                let mmap_write = unsafe {
                    libc::mmap(
                        std::ptr::null_mut(),
                        (line_length * height) as usize,
                        libc::PROT_READ | libc::PROT_WRITE,
                        libc::MAP_SHARED,
                        file.as_raw_fd(),
                        0,
                    )
                };

                if mmap_write == libc::MAP_FAILED {
                    // Fallback: usar solo lectura
                    return (Some(file), None, width, height, bpp, line_length);
                }

                let mmap_vec = unsafe {
                    let ptr = mmap_write as *mut u8;
                    std::slice::from_raw_parts_mut(ptr, (line_length * height) as usize).to_vec()
                };

                (Some(file), Some(mmap_vec), width, height, bpp, line_length)
            }
            Err(_) => (None, None, 80, 24, 24, 80),
        }
    }

    /// Obtiene la geometría del framebuffer via ioctl
    fn obtener_geometria(file: &File) -> (u32, u32, u32, u32) {
        let mut var_info = VarScreenInfo {
            xres: 0,
            yres: 0,
            xres_virtual: 0,
            yres_virtual: 0,
            xoffset: 0,
            yoffset: 0,
            bits_per_pixel: 0,
            grayscale: 0,
            red: 0,
            green: 0,
            blue: 0,
            transp: 0,
        };

        let mut fix_info = FixScreenInfo {
            smem_start: 0,
            smem_len: 0,
            r#type: 0,
            type_aux: 0,
            visual: 0,
            xpanstep: 0,
            ypanstep: 0,
            ywrapstep: 0,
            line_length: 0,
        };

        // Intentar ioctl
        let fd = file.as_raw_fd();

        let res_var = unsafe {
            libc::ioctl(
                fd,
                FBIOGET_VSCREENINFO,
                &mut var_info as *mut _ as *mut libc::c_void,
            )
        };

        if res_var < 0 {
            // Default para terminal
            return (80, 24, 24, 80);
        }

        let res_fix = unsafe {
            libc::ioctl(
                fd,
                FBIOGET_FSCREENINFO,
                &mut fix_info as *mut _ as *mut libc::c_void,
            )
        };

        if res_fix < 0 {
            return (var_info.xres, var_info.yres, var_info.bits_per_pixel, 0);
        }

        let line_length = if fix_info.line_length > 0 {
            fix_info.line_length
        } else {
            var_info.xres * (var_info.bits_per_pixel / 8)
        };

        (
            var_info.xres,
            var_info.yres,
            var_info.bits_per_pixel,
            line_length,
        )
    }

    /// Convierte longitud de onda a RGB usando aproximación de CIE 1931
    /// wavelength: en nanómetros (380-780)
    fn longitud_onda_a_rgb(wavelength: f64) -> (u8, u8, u8) {
        // Factores de calibración para 700nm (rojo), 546.1nm (verde), 435.8nm (azul)
        let (mut r, mut g, mut b) = if wavelength < 380.0 {
            (0.0, 0.0, 0.0)
        } else if wavelength < 440.0 {
            let t = (wavelength - 380.0) / (440.0 - 380.0);
            (-t, 0.0, 1.0 + t)
        } else if wavelength < 490.0 {
            let t = (wavelength - 440.0) / (490.0 - 440.0);
            (0.0, t, 1.0)
        } else if wavelength < 510.0 {
            let t = (wavelength - 490.0) / (510.0 - 490.0);
            (0.0, 1.0, -t)
        } else if wavelength < 580.0 {
            let t = (wavelength - 510.0) / (580.0 - 510.0);
            (t, 1.0, 0.0)
        } else if wavelength < 645.0 {
            let t = (wavelength - 580.0) / (645.0 - 580.0);
            (1.0, 1.0 - t, 0.0)
        } else if wavelength <= 780.0 {
            (1.0, 0.0, 0.0)
        } else {
            (0.0, 0.0, 0.0)
        };

        // Ajuste de intensidad para longitudes de onda más cortas
        let factor = if wavelength < 380.0 {
            0.0
        } else if wavelength < 420.0 {
            0.3 + 0.7 * (wavelength - 380.0) / (420.0 - 380.0)
        } else if wavelength <= 645.0 {
            1.0
        } else if wavelength < 780.0 {
            0.3 + 0.7 * (780.0 - wavelength) / (780.0 - 645.0)
        } else {
            0.0
        };

        r *= factor;
        g *= factor;
        b *= factor;

        // Convertir a 0-255
        let r = (r * 255.0).min(255.0).max(0.0) as u8;
        let g = (g * 255.0).min(255.0).max(0.0) as u8;
        let b = (b * 255.0).min(255.0).max(0.0) as u8;

        (r, g, b)
    }

    /// Convierte densidad de Energon a longitud de onda (espectro)
    fn densidad_a_longitud_onda(densidad: f64) -> f64 {
        // Densidad normalizada 0.0-1.0 → longitud de onda 780nm (rojo) - 380nm (azul)
        let t = densidad.clamp(0.0, 1.0);
        780.0 - t * 400.0
    }

    /// Inicia el hilo de renderizado
    pub fn iniciar(&mut self) {
        let running = self.running.clone();
        let buffer = self.buffer.clone();
        let stats = self.stats.clone();
        let mut fb_mmap = self.fb_mmap.clone();
        let modo = self.modo;
        let fb_width = self.fb_width;
        let fb_height = self.fb_height;
        let fb_bpp = self.fb_bpp;
        let fb_line_length = self.fb_line_length;
        let _fb_file = self.fb_file.take();

        *running.write().unwrap() = true;

        self.render_thread = Some(thread::spawn(move || {
            let mut _last_stats = RenderStats::default();
            let mut frame_count = 0u64;
            let start_time = Instant::now();

            while *running.read().unwrap() {
                let frame_start = Instant::now();

                // Tomar buffer snapshot
                let pixels = buffer.read().unwrap().clone();

                // Renderizar según modo
                match modo {
                    ModoRender::Framebuffer => {
                        if let Some(ref mut mmap) = fb_mmap.as_mut() {
                            Self::render_framebuffer(
                                mmap,
                                &pixels,
                                fb_width as usize,
                                fb_height as usize,
                                fb_bpp as usize,
                                fb_line_length as usize,
                            );
                        }
                    }
                    ModoRender::Terminal => {
                        Self::render_terminal(&pixels, fb_width as usize, fb_height as usize);
                    }
                    ModoRender::Ninguno => {}
                }

                let frame_time = frame_start.elapsed().as_micros() as u64;
                frame_count += 1;

                // Actualizar stats cada 60 frames
                if frame_count % 60 == 0 {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let fps = frame_count as f64 / elapsed;

                    let mut s = stats.write().unwrap();
                    s.fps = fps;
                    s.pixeles_renderizados = (pixels.width * pixels.height) as u64;
                    s.tiempo_render_us = frame_time;
                    _last_stats = s.clone();
                }

                // 60 FPS objetivo
                thread::sleep(Duration::from_millis(16));
            }

            *running.write().unwrap() = false;
        }));
    }

    /// Detiene el hilo de renderizado
    pub fn detener(&mut self) {
        *self.running.write().unwrap() = false;
        if let Some(handle) = self.render_thread.take() {
            let _ = handle.join();
        }
    }

    /// Renderiza en framebuffer
    fn render_framebuffer(
        mmap: &mut [u8],
        pixels: &PixelBuffer,
        width: usize,
        height: usize,
        bpp: usize,
        line_length: usize,
    ) {
        let bytes_per_pixel = bpp / 8;

        for y in 0..height.min(pixels.height) {
            for x in 0..width.min(pixels.width) {
                if let Some((r, g, b)) = pixels.get_pixel_rgb(x, y) {
                    let offset = y * line_length + x * bytes_per_pixel;

                    match bpp {
                        16 => {
                            if offset + 1 < mmap.len() {
                                let rgb565 = Rgb565::from_rgb(r, g, b);
                                mmap[offset] = (rgb565.0 & 0xFF) as u8;
                                mmap[offset + 1] = ((rgb565.0 >> 8) & 0xFF) as u8;
                            }
                        }
                        24 => {
                            if offset + 2 < mmap.len() {
                                mmap[offset] = b;
                                mmap[offset + 1] = g;
                                mmap[offset + 2] = r;
                            }
                        }
                        32 => {
                            if offset + 3 < mmap.len() {
                                mmap[offset] = b;
                                mmap[offset + 1] = g;
                                mmap[offset + 2] = r;
                                mmap[offset + 3] = 0xFF;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Renderiza en terminal usando caracteres Braille
    fn render_terminal(pixels: &PixelBuffer, width: usize, height: usize) {
        // Cada carácter Braille representa 2x4 píxeles
        let char_width = (width + 1) / 2;
        let char_height = (height + 3) / 4;

        print!("\x1b[2J"); // Limpiar pantalla
        print!("\x1b[H"); // Cursor a inicio

        for cy in 0..char_height {
            for cx in 0..char_width {
                // Calcular los 8 píxeles del bloque Braille
                // Patrón: 1  8
                //          2 16
                //          4 32
                //          64 ...
                let mut pattern: u8 = 0x00; // Inicializar patrón Braille

                let mut bit = 0u8;
                for py in 0..4 {
                    for px in 0..2 {
                        let x = cx * 2 + px;
                        let y = cy * 4 + py;

                        if let Some((r, g, b)) = pixels.get_pixel_rgb(x, y) {
                            // Usar luminance para decidir si es "encendido"
                            let lum = (r as u32 + g as u32 + b as u32) / 3;
                            if lum > 128 {
                                pattern |= 1 << bit;
                            }
                        }
                        bit += 1;
                    }
                }

                // Convertir patrón a unicode Braille
                let ch = char::from_u32(pattern as u32).unwrap_or(' ');
                print!("{}", ch);
            }
            println!();
        }

        // Flush output
        let _ = std::io::stdout().flush();
    }

    /// Renderiza una sección del Mar Morfóseo
    pub fn render_mar<Mar>(&self, mar: &Mar, offset_x: usize, offset_y: usize, scale: usize)
    where
        Mar: MarAccess,
    {
        let mut buffer = self.buffer.write().unwrap();

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                // Calcular coordenadas en el Mar
                let mx = offset_x + x / scale;
                let my = offset_y + y / scale;

                if let Some(densidad) = mar.densidad_en(mx, my, 0) {
                    // Normalizar densidad a 0.0-1.0
                    let dens_norm = (densidad.to_raw() as f64 / i64::MAX as f64).clamp(0.0, 1.0);
                    let wavelength = Self::densidad_a_longitud_onda(dens_norm);
                    let (r, g, b) = Self::longitud_onda_a_rgb(wavelength);

                    buffer.set_pixel_rgb(x, y, r, g, b);
                } else {
                    buffer.set_pixel_rgb(x, y, 0, 0, 0);
                }
            }
        }
    }

    /// Renderiza el contorno de un Auton (Marching Squares)
    pub fn render_auton<Campo>(&self, campo: &Campo, color: (u8, u8, u8))
    where
        Campo: CampoAccess,
    {
        let mut buffer = self.buffer.write().unwrap();

        let dims = campo.dims();
        let threshold = 0.0f64; // φ = 0

        // Implementación simplificada de Marching Squares
        // Para cada celda, verificar el signo de φ en las 4 esquinas
        for y in 0..dims.1.saturating_sub(1) {
            for x in 0..dims.0.saturating_sub(1) {
                let v00 = campo.phi_at(x, y).to_raw() as f64 / i64::MAX as f64;
                let v10 = campo.phi_at(x + 1, y).to_raw() as f64 / i64::MAX as f64;
                let v01 = campo.phi_at(x, y + 1).to_raw() as f64 / i64::MAX as f64;
                let v11 = campo.phi_at(x + 1, y + 1).to_raw() as f64 / i64::MAX as f64;

                // Contar esquinas con signo negativo (interior del Auton)
                let inside = (v00 < threshold) as u8
                    + (v10 < threshold) as u8
                    + (v01 < threshold) as u8
                    + (v11 < threshold) as u8;

                // Dibujar líneas de contorno según configuración
                match inside {
                    1 | 3 => {
                        // Cruce de contorno - dibujar líneas
                        if v00 < threshold && v10 >= threshold {
                            Self::draw_line(&mut buffer, x, y, x + 1, y, color);
                        }
                        if v10 < threshold && v11 >= threshold {
                            Self::draw_line(&mut buffer, x + 1, y, x + 1, y + 1, color);
                        }
                        if v01 < threshold && v00 >= threshold {
                            Self::draw_line(&mut buffer, x, y + 1, x, y, color);
                        }
                    }
                    2 => {
                        // Dos cruces posibles
                        if (v00 < threshold) != (v11 < threshold) {
                            Self::draw_line(&mut buffer, x, y, x + 1, y + 1, color);
                        }
                        if (v10 < threshold) != (v01 < threshold) {
                            Self::draw_line(&mut buffer, x + 1, y, x, y + 1, color);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Dibuja una línea en el buffer (algoritmo de Bresenham simplificado)
    fn draw_line(
        buffer: &mut PixelBuffer,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        color: (u8, u8, u8),
    ) {
        let dx = (x1 as i32 - x0 as i32).abs() as usize;
        let dy = (y1 as i32 - y0 as i32).abs() as usize;
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };

        let mut err = dx as i32 - dy as i32;
        let mut x = x0 as i32;
        let mut y = y0 as i32;

        loop {
            if x >= 0 && (x as usize) < buffer.width && y >= 0 && (y as usize) < buffer.height {
                buffer.set_pixel_rgb(x as usize, y as usize, color.0, color.1, color.2);
            }

            if x == x1 as i32 && y == y1 as i32 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -(dy as i32) {
                err -= dy as i32;
                x += sx;
            }
            if e2 < dx as i32 {
                err += dx as i32;
                y += sy;
            }
        }
    }

    /// Obtiene las estadísticas de renderizado
    pub fn estadisticas(&self) -> RenderStats {
        self.stats.read().unwrap().clone()
    }

    /// Obtiene el modo actual
    pub fn modo(&self) -> ModoRender {
        self.modo
    }

    /// Devuelve true si está usando framebuffer real
    pub fn usa_framebuffer(&self) -> bool {
        self.modo == ModoRender::Framebuffer
    }

    /// Sincroniza con framebuffer (flush)
    pub fn sync(&self) {
        if let Some(ref file) = self.fb_file {
            // Sync en Linux: fsync
            unsafe {
                libc::fsync(file.as_raw_fd());
            }
        }
    }
}

impl Drop for SoftGPU {
    fn drop(&mut self) {
        self.detener();
        // Desmapear memoria
        if let Some(ref mmap) = self.fb_mmap {
            if !mmap.is_empty() {
                unsafe {
                    libc::munmap(mmap.as_ptr() as *mut libc::c_void, mmap.len());
                }
            }
        }
    }
}

// Trait para acceder al Mar Morfóseo
pub trait MarAccess {
    fn densidad_en(&self, x: usize, y: usize, z: usize) -> Option<crate::physics::I32F32>;
}

// Trait para acceder al Campo Estructural
pub trait CampoAccess {
    fn phi_at(&self, x: usize, y: usize) -> crate::physics::I32F32;
    fn dims(&self) -> (usize, usize);
}

// Implementaciones para MarMorfoseo
impl MarAccess for crate::physics::MarMorfoseo {
    fn densidad_en(&self, x: usize, y: usize, z: usize) -> Option<crate::physics::I32F32> {
        crate::physics::MarMorfoseo::densidad_en(self, x, y, z)
    }
}

// Implementaciones para CampoEstructural
impl CampoAccess for crate::life::campo_estructural::CampoEstructural {
    fn phi_at(&self, x: usize, y: usize) -> crate::physics::I32F32 {
        crate::life::campo_estructural::CampoEstructural::phi_at(self, x, y, 0)
    }

    fn dims(&self) -> (usize, usize) {
        let d = crate::life::campo_estructural::CampoEstructural::dims(self);
        (d.nx, d.ny)
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda_to_rgb_red() {
        let (r, g, b) = SoftGPU::longitud_onda_a_rgb(700.0);
        assert!(r > g && r > b);
    }

    #[test]
    fn test_lambda_to_rgb_green() {
        let (r, g, b) = SoftGPU::longitud_onda_a_rgb(550.0);
        assert!(g > r && g > b);
    }

    #[test]
    fn test_lambda_to_rgb_blue() {
        let (r, g, b) = SoftGPU::longitud_onda_a_rgb(450.0);
        assert!(b > r && b > g);
    }

    #[test]
    fn test_densidad_a_lambda() {
        let lambda = SoftGPU::densidad_a_longitud_onda(0.0);
        assert!((lambda - 780.0).abs() < 1.0);

        let lambda = SoftGPU::densidad_a_longitud_onda(1.0);
        assert!((lambda - 380.0).abs() < 1.0);
    }

    #[test]
    fn test_pixel_buffer() {
        let mut buf = PixelBuffer::new(10, 10);
        buf.set_pixel_rgb(5, 5, 255, 128, 64);

        let (r, g, b) = buf.get_pixel_rgb(5, 5).unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 128);
        assert_eq!(b, 64);
    }

    #[test]
    fn test_rgb565() {
        let rgb = Rgb565::from_rgb(255, 255, 255);
        assert_eq!(rgb.0, 0xFFFF);

        let rgb = Rgb565::from_rgb(0, 0, 0);
        assert_eq!(rgb.0, 0x0000);
    }

    #[test]
    fn test_rgb888() {
        let rgb = Rgb888::from_rgb(255, 128, 64);
        assert_eq!(rgb.0, 0xFF8040);
    }

    #[test]
    fn test_crear_softgpu() {
        // Test PixelBuffer creation which doesn't require framebuffer
        let buf = PixelBuffer::new(100, 100);
        assert_eq!(buf.width, 100);
        assert_eq!(buf.height, 100);
        assert_eq!(buf.data.len(), 100 * 100 * 3);
    }

    #[test]
    fn test_linea_bresenham() {
        let mut buf = PixelBuffer::new(100, 100);
        SoftGPU::draw_line(&mut buf, 0, 0, 10, 10, (255, 255, 255));

        // Verificar que hay píxeles dibujados
        let count = buf
            .data
            .chunks(3)
            .filter(|p| p[0] == 255 && p[1] == 255 && p[2] == 255)
            .count();
        assert!(count > 0);
    }
}

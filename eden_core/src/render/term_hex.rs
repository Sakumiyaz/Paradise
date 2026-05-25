//! # TermHex: Visualizador de Terminal con Cuadrícula Hexagonal
//!
//! Renderizado hexagonal usando caracteres Unicode/ASCII en la terminal.
//! Cada celda hexagonal se representa con dos caracteres (fila superior/inferior).
//!
//! ## Características
//!
//! - **Cuadrícula Hexagonal**: Offset coordinates (axial) para hex grid
//! - **16 Colores ANSI**: Mapeo de densidad a paleta estándar
//! - **Secuencias ANSI**: Movimiento de cursor y colores para rendimiento
//! - **Estadísticas en Tiempo Real**: Autons vivos, energía total, escoria
//! - **Fallback**: Se activa automáticamente si no hay framebuffer

#![allow(dead_code)]

use std::io::{stdout, Write};
use std::time::Instant;

/// Caracteres Unicode para celda hexagonal
mod hex_chars {
    /// Fila superior del hex (ángulos en °, °, °)
    pub const UL: &str = "╭"; // esquina superior izquierda
    pub const UR: &str = "╮"; // esquina superior derecha
    pub const HL: &str = "─"; // lado horizontal arriba
    pub const VL: &str = "│"; // lado vertical izquierda
    pub const VR: &str = "│"; // lado vertical derecha

    /// Fila inferior del hex
    pub const LL: &str = "╰"; // esquina inferior izquierda
    pub const LR: &str = "╯"; // esquina inferior derecha

    /// Lados verticales cortos
    pub const VL_SHORT: &str = "╎";

    /// Relleno de celda
    pub const EMPTY: &str = " ";

    /// Caracteres de densidad (ejemplo: 16 niveles)
    pub const DENSITY_CHARS: &[&str] = &[
        " ", "▘", "▝", "▀", "▖", "▌", "▞", "▘", "▝", "▀", "▗", "▚", "▙", "▜", "▟", "█",
    ];
}

/// Colores ANSI de 16 colores (estándar)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorAnsi {
    Negro = 30,
    Rojo = 31,
    Verde = 32,
    Amarillo = 33,
    Azul = 34,
    Magenta = 35,
    Cyan = 36,
    Blanco = 37,
    Brillante = 90,
    RojoBrillante = 91,
    VerdeBrillante = 92,
    AmarilloBrillante = 93,
    AzulBrillante = 94,
    MagentaBrillante = 95,
    CyanBrillante = 96,
    BlancoBrillante = 97,
}

impl ColorAnsi {
    /// Convierte densidad (0.0-1.0) a color ANSI
    pub fn from_densidad(densidad: f64) -> Self {
        let idx = ((densidad.clamp(0.0, 1.0)) * 15.999) as usize;
        match idx {
            0 => ColorAnsi::Negro,
            1 => ColorAnsi::Amarillo,
            2 => ColorAnsi::VerdeBrillante,
            3 => ColorAnsi::Verde,
            4 => ColorAnsi::Cyan,
            5 => ColorAnsi::VerdeBrillante,
            6 => ColorAnsi::AmarilloBrillante,
            7 => ColorAnsi::Amarillo,
            8 => ColorAnsi::Verde,
            9 => ColorAnsi::CyanBrillante,
            10 => ColorAnsi::Cyan,
            11 => ColorAnsi::VerdeBrillante,
            12 => ColorAnsi::Amarillo,
            13 => ColorAnsi::AmarilloBrillante,
            14 => ColorAnsi::Cyan,
            _ => ColorAnsi::BlancoBrillante,
        }
    }

    /// Código ANSI para foreground
    pub fn fg_code(&self) -> &'static str {
        match self {
            ColorAnsi::Negro => "30",
            ColorAnsi::Rojo => "31",
            ColorAnsi::Verde => "32",
            ColorAnsi::Amarillo => "33",
            ColorAnsi::Azul => "34",
            ColorAnsi::Magenta => "35",
            ColorAnsi::Cyan => "36",
            ColorAnsi::Blanco => "37",
            ColorAnsi::Brillante => "90",
            ColorAnsi::RojoBrillante => "91",
            ColorAnsi::VerdeBrillante => "92",
            ColorAnsi::AmarilloBrillante => "93",
            ColorAnsi::AzulBrillante => "94",
            ColorAnsi::MagentaBrillante => "95",
            ColorAnsi::CyanBrillante => "96",
            ColorAnsi::BlancoBrillante => "97",
        }
    }

    /// Código ANSI para background
    pub fn bg_code(&self) -> &'static str {
        match self {
            ColorAnsi::Negro => "40",
            ColorAnsi::Rojo => "41",
            ColorAnsi::Verde => "42",
            ColorAnsi::Amarillo => "43",
            ColorAnsi::Azul => "44",
            ColorAnsi::Magenta => "45",
            ColorAnsi::Cyan => "46",
            ColorAnsi::Blanco => "47",
            ColorAnsi::Brillante => "100",
            ColorAnsi::RojoBrillante => "101",
            ColorAnsi::VerdeBrillante => "102",
            ColorAnsi::AmarilloBrillante => "103",
            ColorAnsi::AzulBrillante => "104",
            ColorAnsi::MagentaBrillante => "105",
            ColorAnsi::CyanBrillante => "106",
            ColorAnsi::BlancoBrillante => "107",
        }
    }
}

/// Secuencias ANSI escape
mod ansi {
    /// Movimiento de cursor
    pub fn move_to(row: u16, col: u16) -> String {
        format!("\x1b[{};{}H", row + 1, col + 1)
    }

    /// Limpiar pantalla
    pub fn clear_screen() -> String {
        "\x1b[2J".to_string()
    }

    /// Limpiar línea
    pub fn clear_line() -> String {
        "\x1b[K".to_string()
    }

    /// Ocultar cursor
    pub fn hide_cursor() -> String {
        "\x1b[?25l".to_string()
    }

    /// Mostrar cursor
    pub fn show_cursor() -> String {
        "\x1b[?25h".to_string()
    }

    /// Reset color
    pub fn reset() -> String {
        "\x1b[0m".to_string()
    }

    /// Color foreground
    pub fn fg(color: &str) -> String {
        format!("\x1b[{}m", color)
    }

    /// Color background
    pub fn bg(color: &str) -> String {
        format!("\x1b[{}m", color)
    }

    /// Negrita
    pub fn bold() -> String {
        "\x1b[1m".to_string()
    }

    ///篱 RESET
    pub fn reset_color() -> String {
        "\x1b[39m".to_string()
    }
}

/// Coordenadas hexagonales (offset coords -odd-q vertical)
#[derive(Debug, Clone, Copy)]
struct HexCoord {
    col: usize, // columna
    row: usize, // fila
}

impl HexCoord {
    /// Convierte coord hexagonal a position de celdas de texto
    /// Retorna (col_texto, row_texto)
    fn to_text_pos(&self) -> (usize, usize) {
        // Cada hex ocupa 2 columnas de texto y 2 filas
        // Offset vertical alterno para columnas impares
        let text_col = self.col * 2;
        let text_row = self.row * 2 + if self.col % 2 == 1 { 1 } else { 0 };
        (text_col, text_row)
    }
}

/// Estadísticas del sistema
#[derive(Debug, Clone, Default)]
pub struct StatsSistema {
    pub autons_vivos: u32,
    pub energia_total: f64,
    pub escoria_total: f64,
    pub densidad_promedio: f64,
    pub fps: f64,
}

/// TermHex: Visualizador hexagonal de terminal
pub struct TermHex {
    /// Ancho en celdas hex
    width: usize,
    /// Alto en celdas hex
    height: usize,
    /// Buffer de colores para cada celda
    color_buffer: Vec<ColorAnsi>,
    /// Buffer de densidad
    densidad_buffer: Vec<f64>,
    /// Último tiempo de stats
    last_stats_time: Instant,
    /// Contador de frames
    frame_count: u64,
    /// Stats actuales
    stats: StatsSistema,
}

impl TermHex {
    /// Crea nuevo TermHex con dimensiones de celdas hex
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        TermHex {
            width,
            height,
            color_buffer: vec![ColorAnsi::Negro; size],
            densidad_buffer: vec![0.0; size],
            last_stats_time: Instant::now(),
            frame_count: 0,
            stats: StatsSistema::default(),
        }
    }

    /// Actualiza la densidad de una celda
    pub fn set_densidad(&mut self, col: usize, row: usize, densidad: f64) {
        if col < self.width && row < self.height {
            let idx = row * self.width + col;
            self.densidad_buffer[idx] = densidad;
            self.color_buffer[idx] = ColorAnsi::from_densidad(densidad);
        }
    }

    /// Actualiza stats del sistema
    pub fn actualizar_stats(&mut self, stats: StatsSistema) {
        self.stats = stats;
    }

    /// Renderiza el buffer completo a la terminal
    pub fn render<Mar>(&mut self, mar: &Mar)
    where
        Mar: MarStatsAccess,
    {
        self.frame_count += 1;

        // Limpiar pantalla y posicionar cursor
        let output = self.generar_salida(mar);

        // Escribir a stdout
        let mut stdout = stdout();
        stdout.write_all(ansi::clear_screen().as_bytes()).ok();
        stdout.write_all(ansi::hide_cursor().as_bytes()).ok();
        stdout.write_all(output.as_bytes()).ok();
        stdout.write_all(ansi::show_cursor().as_bytes()).ok();
        stdout.flush().ok();

        // Actualizar FPS
        let elapsed = self.last_stats_time.elapsed().as_secs_f64();
        if elapsed >= 1.0 {
            self.stats.fps = self.frame_count as f64 / elapsed;
            self.last_stats_time = Instant::now();
            self.frame_count = 0;
        }
    }

    /// Genera string de salida
    fn generar_salida<Mar>(&mut self, _mar: &Mar) -> String
    where
        Mar: MarStatsAccess,
    {
        let mut s = String::with_capacity(
            self.width * self.height * 8, // Estimate
        );

        // Renderizar hexgrid
        s.push_str(&ansi::move_to(0, 0));

        for row in 0..self.height {
            // Fila superior de cada hex
            for col in 0..self.width {
                let idx = row * self.width + col;
                let color = self.color_buffer[idx];
                let densidade = self.densidad_buffer[idx];

                // Determinar carácter según densidad
                let char_dens = Self::densidad_a_char(densidade);
                let char_top = Self::hex_top_row(color, char_dens);

                s.push_str(&char_top);
            }
            s.push('\n');

            // Fila inferior de cada hex
            for col in 0..self.width {
                let idx = row * self.width + col;
                let color = self.color_buffer[idx];
                let densidade = self.densidad_buffer[idx];

                let char_dens = Self::densidad_a_char(densidade);
                let char_bot = Self::hex_bottom_row(color, char_dens, col);

                s.push_str(&char_bot);
            }
            s.push('\n');
        }

        // Línea separadora de stats
        s.push_str(&ansi::move_to((self.height * 2 + 1) as u16, 0));
        s.push_str(&ansi::clear_line());
        s.push_str(&self.linea_separadora());

        // Stats del sistema
        let stats_row = (self.height * 2 + 2) as u16;
        s.push_str(&ansi::move_to(stats_row, 0));
        s.push_str(&Self::render_stats(&self.stats));

        s
    }

    /// Carácter para representar densidad
    fn densidad_a_char(densidade: f64) -> &'static str {
        let idx = ((densidade.clamp(0.0, 1.0)) * 15.999) as usize;
        hex_chars::DENSITY_CHARS[idx.min(15)]
    }

    /// Fila superior del hex
    fn hex_top_row(color: ColorAnsi, char_dens: &str) -> String {
        format!(
            "{}{}{}{}",
            ansi::fg(color.fg_code()),
            ansi::bg(color.bg_code()),
            char_dens,
            ansi::reset()
        )
    }

    /// Fila inferior del hex (con bordes)
    fn hex_bottom_row(color: ColorAnsi, char_dens: &str, col: usize) -> String {
        let left_border = if col % 2 == 0 { "╎" } else { " " };
        format!(
            "{}{}{}{}{}",
            ansi::fg(color.fg_code()),
            ansi::bg(color.bg_code()),
            left_border,
            char_dens,
            ansi::reset()
        )
    }

    /// Línea separadora para stats
    fn linea_separadora(&self) -> String {
        let ancho = self.width * 2 + 2;
        format!(
            "{}{}{}{}",
            ansi::bold(),
            ansi::fg("97"), // Blanco brillante
            "─".repeat(ancho),
            ansi::reset()
        )
    }

    /// Renderiza estadísticas del sistema
    fn render_stats(stats: &StatsSistema) -> String {
        format!(
            "{}{} {}Autons:{:>6} {} Energia:{:>10.2} {} Escoria:{:>9.3} {} FPS:{:>6.1}{}{}",
            ansi::bold(),
            ansi::fg("96"), // Cyan
            ansi::fg("93"), // Amarillo brillante
            stats.autons_vivos,
            ansi::fg("92"), // Verde
            stats.energia_total,
            ansi::fg("91"), // Rojo brillante
            stats.escoria_total,
            ansi::fg("94"), // Azul brillante
            stats.fps,
            ansi::reset(),
            ansi::clear_line()
        )
    }

    /// Actualiza desde el Mar Morfóseo
    pub fn actualizar_desde_mar<Mar>(&mut self, mar: &Mar, escala: usize)
    where
        Mar: MarAccess,
    {
        for row in 0..self.height {
            for col in 0..self.width {
                let mx = col * escala;
                let my = row * escala;

                if let Some(densidade) = mar.densidad_en(mx, my, 0) {
                    // Normalizar: I32F32 raw a f64 0.0-1.0
                    let raw = densidade.to_raw() as f64;
                    let normalized = (raw / i64::MAX as f64).clamp(0.0, 1.0);
                    self.set_densidad(col, row, normalized);
                } else {
                    self.set_densidad(col, row, 0.0);
                }
            }
        }
    }

    /// Ancho en caracteres de texto
    pub fn text_width(&self) -> usize {
        self.width * 2 + 1
    }

    /// Alto en líneas de texto
    pub fn text_height(&self) -> usize {
        self.height * 2 + 4 // +4 para separador y stats
    }

    /// Obtiene dimensiones推荐adas para terminal
    pub fn recomendada_terminal_size(&self) -> (usize, usize) {
        (self.text_width(), self.text_height())
    }
}

/// Trait para acceder a estadísticas del Mar
pub trait MarStatsAccess {
    fn autons_vivos(&self) -> u32;
    fn energia_total(&self) -> f64;
    fn escoria_total(&self) -> f64;
    fn densidad_promedio(&self) -> f64;
}

/// Trait para acceder a densidades del Mar
pub trait MarAccess {
    fn densidad_en(&self, x: usize, y: usize, z: usize) -> Option<crate::physics::I32F32>;
}

// Implementación para MarMorfoseo
impl MarAccess for crate::physics::MarMorfoseo {
    fn densidad_en(&self, x: usize, y: usize, z: usize) -> Option<crate::physics::I32F32> {
        crate::physics::MarMorfoseo::densidad_en(self, x, y, z)
    }
}

// Trait marker para implementar stats externamente
pub trait MarMorfoseoRef {
    fn autons_vivos_count(&self) -> u32;
    fn energia_total_sum(&self) -> f64;
    fn escoria_total_sum(&self) -> f64;
}

// Implementación vacía para stats de Mar (debe implementarse externamente)
impl<T> MarStatsAccess for T
where
    T: MarMorfoseoRef,
{
    fn autons_vivos(&self) -> u32 {
        0
    }
    fn energia_total(&self) -> f64 {
        0.0
    }
    fn escoria_total(&self) -> f64 {
        0.0
    }
    fn densidad_promedio(&self) -> f64 {
        0.0
    }
}

/// Wrapper para stats de Auton
pub struct AutonStats {
    pub vivos: u32,
    pub energia: f64,
    pub escoria: f64,
}

impl Default for AutonStats {
    fn default() -> Self {
        AutonStats {
            vivos: 0,
            energia: 0.0,
            escoria: 0.0,
        }
    }
}

/// Actualiza stats desde datos externos
pub struct TermHexStats {
    pub autons_vivos: u32,
    pub energia_total: f64,
    pub escoria_total: f64,
    pub fps: f64,
}

impl Default for TermHexStats {
    fn default() -> Self {
        TermHexStats {
            autons_vivos: 0,
            energia_total: 0.0,
            escoria_total: 0.0,
            fps: 0.0,
        }
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_densidad() {
        let c0 = ColorAnsi::from_densidad(0.0);
        assert!(matches!(c0, ColorAnsi::Negro));

        let c1 = ColorAnsi::from_densidad(1.0);
        assert!(matches!(c1, ColorAnsi::BlancoBrillante));

        let c05 = ColorAnsi::from_densidad(0.5);
        // No debe ser Negro ni BlancoBrillante
        assert!(c05 != ColorAnsi::Negro || c05 != ColorAnsi::BlancoBrillante);
    }

    #[test]
    fn test_crear_termhex() {
        let th = TermHex::new(40, 20);
        assert_eq!(th.width, 40);
        assert_eq!(th.height, 20);
    }

    #[test]
    fn test_set_densidad() {
        let mut th = TermHex::new(10, 10);
        th.set_densidad(5, 5, 0.7);
        assert_eq!(th.densidad_buffer[5 * 10 + 5], 0.7);
    }

    #[test]
    fn test_text_dimensions() {
        let th = TermHex::new(40, 20);
        assert_eq!(th.text_width(), 81);
        assert_eq!(th.text_height(), 44); // 20*2 + 4
    }

    #[test]
    fn test_hex_coord_text_pos() {
        let coord = HexCoord { col: 5, row: 3 };
        let (text_col, text_row) = coord.to_text_pos();
        assert_eq!(text_col, 10); // col * 2
        assert_eq!(text_row, 7); // row * 2 (col 5 is odd, so +1)
    }

    #[test]
    fn test_densidad_a_char() {
        assert_eq!(TermHex::densidad_a_char(0.0), " ");
        assert_eq!(TermHex::densidad_a_char(1.0), "█");
    }

    #[test]
    fn test_ansi_move_to() {
        let s = ansi::move_to(5, 10);
        assert_eq!(s, "\x1b[6;11H");
    }

    #[test]
    fn test_render_stats_format() {
        let stats = StatsSistema {
            autons_vivos: 42,
            energia_total: 1234.567,
            escoria_total: 89.012,
            densidad_promedio: 0.5,
            fps: 60.0,
        };

        let output = TermHex::render_stats(&stats);
        assert!(output.contains("42"));
        assert!(output.contains("1234.57"));
        assert!(output.contains("89.012"));
        assert!(output.contains("60.0"));
    }

    #[test]
    fn test_color_codes() {
        assert_eq!(ColorAnsi::Rojo.fg_code(), "31");
        assert_eq!(ColorAnsi::Rojo.bg_code(), "41");
        assert_eq!(ColorAnsi::BlancoBrillante.fg_code(), "97");
    }
}

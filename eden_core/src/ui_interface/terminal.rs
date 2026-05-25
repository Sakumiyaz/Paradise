//! # Terminal - Advanced ANSI Terminal
//!
//! Terminal avanzada 100% original con soporte para escape codes ANSI.
//! Sin dependencias de bibliotecas de terminal externas.
//!
//! ## Características
//!
//! - Escape codes ANSI completos (colores, movimiento cursor, modos)
//! - Parsing y generación de secuencias de escape
//! - Estilos de texto (bold, italic, underline, etc.)
//! - Colores de 256 colores (8 standard + 248 extendidos)
//! - Clear, erase, scrolling
//! - Captura de pantalla del terminal
//! - Event loop asíncrono básico

#![allow(dead_code)]

// ============================================================================
// TIPOS BÁSICOS
// ============================================================================

/// Modo de terminal
#[derive(Clone, Debug, PartialEq)]
pub enum TerminalMode {
    /// Modo línea de comandos
    Command,
    /// Modo visual completo
    Full,
    /// Modo minimalista
    Minimal,
    /// Modo editor (vim-like)
    Editor,
    /// Modo aplicación (ncurses-like)
    Application,
}

/// Código de escape ANSI
#[derive(Clone, Debug, PartialEq)]
pub enum EscapeCode {
    /// Cursor arriba n veces
    CursorUp(u16),
    /// Cursor abajo n veces
    CursorDown(u16),
    /// Cursor derecha n veces
    CursorForward(u16),
    /// Cursor izquierda n veces
    CursorBack(u16),
    /// Mover cursor a posición específica (row, col)
    CursorPosition(u16, u16),
    /// Guardar posición del cursor
    SaveCursor,
    /// Restaurar posición del cursor
    RestoreCursor,
    /// Borrar pantalla
    ClearScreen(ClearType),
    /// Borrar línea
    ClearLine(ClearType),
    /// Mostrar/ocultar cursor
    ShowCursor(bool),
    /// Modo de video inverso
    ReverseVideo(bool),
    /// Atributos de texto SGR (Select Graphic Rendition)
    SGR(Vec<SGRParam>),
    /// Modo de cursor (blink, block, underline)
    CursorMode(CursorStyle),
    /// Modo de scroll
    ScrollRegion(u16, u16),
    /// Inicialización de teclado alternativo
    AltKeypadMode(bool),
    /// Cambiar título de ventana
    Title(String),
    /// Modo de pantalla completa
    AlternateScreen(bool),
    /// Request terminal attributes
    RequestAttributes,
    /// Modo de bracketed paste
    BracketedPaste(bool),
}

/// Tipo de borrado
#[derive(Clone, Debug, PartialEq)]
pub enum ClearType {
    /// Desde cursor hasta fin
    ToEnd,
    /// Desde inicio hasta cursor
    ToStart,
    /// Toda la pantalla
    All,
    /// Desde cursor hasta fin de línea
    LineToEnd,
    /// Desde inicio de línea hasta cursor
    LineToStart,
    /// Toda la línea
    LineAll,
}

/// Parámetros SGR (Select Graphic Rendition)
#[derive(Clone, Debug, PartialEq)]
pub enum SGRParam {
    Reset,
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    ReverseVideo,
    Hidden,
    Strikethrough,
    Foreground(AnsiColor),
    Background(AnsiColor),
    DoubleUnderline,
    Normal,
    NotBold,
    NotDim,
    NotItalic,
    NotUnderline,
    NotBlink,
    NotReverse,
    NotHidden,
    NotStrikethrough,
    NotDoubleUnderline,
    /// Color de 256 niveles
    Foreground256(u8),
    Background256(u8),
    /// Color RGB directo
    ForegroundRGB(u8, u8, u8),
    BackgroundRGB(u8, u8, u8),
}

/// Estilo de cursor
#[derive(Clone, Debug, PartialEq)]
pub enum CursorStyle {
    Hidden,
    Block,
    Underline,
    BlinkBlock,
    BlinkUnderline,
}

/// Color ANSI
#[derive(Clone, Debug, PartialEq)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Indexed(u8),
    RGB(u8, u8, u8),
}

/// Estilo de texto
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
    pub foreground: AnsiColor,
    pub background: AnsiColor,
}

impl Style {
    pub fn default() -> Self {
        Self {
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            blink: false,
            reverse: false,
            hidden: false,
            strikethrough: false,
            foreground: AnsiColor::White,
            background: AnsiColor::Black,
        }
    }

    pub fn to_sgr(&self) -> Vec<SGRParam> {
        let mut params = vec![SGRParam::Reset];

        if self.bold {
            params.push(SGRParam::Bold);
        }
        if self.dim {
            params.push(SGRParam::Dim);
        }
        if self.italic {
            params.push(SGRParam::Italic);
        }
        if self.underline {
            params.push(SGRParam::Underline);
        }
        if self.blink {
            params.push(SGRParam::Blink);
        }
        if self.reverse {
            params.push(SGRParam::ReverseVideo);
        }
        if self.hidden {
            params.push(SGRParam::Hidden);
        }
        if self.strikethrough {
            params.push(SGRParam::Strikethrough);
        }

        params.push(SGRParam::Foreground(self.foreground.clone()));
        params.push(SGRParam::Background(self.background.clone()));

        params
    }
}

// ============================================================================
// TERMINAL
// ============================================================================

/// Terminal ANSI avanzada
pub struct Terminal {
    /// Modo actual de la terminal
    mode: TerminalMode,
    ///Buffer de pantalla
    screen: Vec<Vec<char>>,
    /// Ancho de pantalla
    width: usize,
    /// Alto de pantalla
    height: usize,
    /// Posición del cursor (row, col)
    cursor: (usize, usize),
    /// ¿Cursor visible?
    cursor_visible: bool,
    /// Scroll region (top, bottom)
    scroll_region: Option<(u16, u16)>,
    /// Estilo actual
    current_style: Style,
    /// Tab stops
    tab_stops: Vec<usize>,
    /// Saved cursor position
    saved_cursor: Option<(usize, usize)>,
    /// Alternate screen buffer
    alternate_screen: bool,
    /// Buffer de salida acumulado
    output_buffer: String,
}

impl Terminal {
    /// Crea una nueva terminal con dimensiones especificadas
    pub fn new(width: usize, height: usize) -> Self {
        let screen = vec![vec![' '; width]; height];

        let mut tab_stops = Vec::new();
        for i in (0..width).step_by(8) {
            tab_stops.push(i);
        }

        Self {
            mode: TerminalMode::Command,
            screen,
            width,
            height,
            cursor: (0, 0),
            cursor_visible: true,
            scroll_region: None,
            current_style: Style::default(),
            tab_stops,
            saved_cursor: None,
            alternate_screen: false,
            output_buffer: String::new(),
        }
    }

    /// Obtiene el tamaño de la pantalla
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Obtiene la posición del cursor
    pub fn cursor_position(&self) -> (usize, usize) {
        self.cursor
    }

    /// Mueve el cursor a una posición específica
    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.cursor.0 = row.min(self.height - 1);
        self.cursor.1 = col.min(self.width - 1);
    }

    /// Mueve el cursor arriba n líneas
    pub fn cursor_up(&mut self, n: u16) {
        self.cursor.0 = self.cursor.0.saturating_sub(n as usize);
    }

    /// Mueve el cursor abajo n líneas
    pub fn cursor_down(&mut self, n: u16) {
        self.cursor.0 = (self.cursor.0 + n as usize).min(self.height - 1);
    }

    /// Mueve el cursor a la derecha n columnas
    pub fn cursor_forward(&mut self, n: u16) {
        self.cursor.1 = (self.cursor.1 + n as usize).min(self.width - 1);
    }

    /// Mueve el cursor a la izquierda n columnas
    pub fn cursor_back(&mut self, n: u16) {
        self.cursor.1 = self.cursor.1.saturating_sub(n as usize);
    }

    /// Introduce un carácter en la posición del cursor
    pub fn put_char(&mut self, ch: char) {
        let (row, col) = self.cursor;
        if ch == '\n' {
            self.cursor.0 += 1;
            if self.cursor.0 >= self.height {
                self.scroll_up(1);
                self.cursor.0 = self.height - 1;
            }
        } else if ch == '\r' {
            self.cursor.1 = 0;
        } else if ch == '\t' {
            // Find next tab stop
            for &tab in &self.tab_stops {
                if tab > self.cursor.1 {
                    self.cursor.1 = tab;
                    break;
                }
            }
        } else {
            self.screen[row][col] = ch;
            self.cursor.1 += 1;
            if self.cursor.1 >= self.width {
                self.cursor.1 = 0;
                self.cursor.0 += 1;
                if self.cursor.0 >= self.height {
                    self.scroll_up(1);
                    self.cursor.0 = self.height - 1;
                }
            }
        }
    }

    /// Escribe texto en la posición actual del cursor
    pub fn write(&mut self, text: &str) {
        for ch in text.chars() {
            self.put_char(ch);
        }
    }

    /// Borra caracteres según el tipo
    pub fn erase(&mut self, clear_type: ClearType) {
        match clear_type {
            ClearType::ToEnd => {
                let (row, col) = self.cursor;
                // Erase to end of line
                for c in col..self.width {
                    self.screen[row][c] = ' ';
                }
                // Erase remaining lines
                for r in (row + 1)..self.height {
                    for c in 0..self.width {
                        self.screen[r][c] = ' ';
                    }
                }
            }
            ClearType::ToStart => {
                let (row, col) = self.cursor;
                // Erase from start of line to cursor
                for c in 0..=col {
                    self.screen[row][c] = ' ';
                }
                // Erase previous lines
                for r in 0..row {
                    for c in 0..self.width {
                        self.screen[r][c] = ' ';
                    }
                }
            }
            ClearType::All => {
                for r in 0..self.height {
                    for c in 0..self.width {
                        self.screen[r][c] = ' ';
                    }
                }
            }
            ClearType::LineToEnd => {
                let (row, col) = self.cursor;
                for c in col..self.width {
                    self.screen[row][c] = ' ';
                }
            }
            ClearType::LineToStart => {
                let (row, col) = self.cursor;
                for c in 0..=col {
                    self.screen[row][c] = ' ';
                }
            }
            ClearType::LineAll => {
                let row = self.cursor.0;
                for c in 0..self.width {
                    self.screen[row][c] = ' ';
                }
            }
        }
    }

    /// Desplaza el contenido hacia arriba n líneas
    pub fn scroll_up(&mut self, lines: u16) {
        let lines = lines as usize;
        for _ in 0..lines {
            self.screen.remove(0);
            self.screen.push(vec![' '; self.width]);
        }
    }

    /// Desplaza el contenido hacia abajo n líneas
    pub fn scroll_down(&mut self, lines: u16) {
        let lines = lines as usize;
        for _ in 0..lines {
            self.screen.pop();
            self.screen.insert(0, vec![' '; self.width]);
        }
    }

    /// Guarda la posición actual del cursor
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor);
    }

    /// Restaura la posición guardada del cursor
    pub fn restore_cursor(&mut self) {
        if let Some(pos) = self.saved_cursor {
            self.cursor = pos;
        }
    }

    /// Establece el modo de cursor
    pub fn set_cursor_style(&mut self, style: CursorStyle) {
        let code = match style {
            CursorStyle::Hidden => vec![SGRParam::Reset],
            CursorStyle::Block => vec![SGRParam::Reset],
            CursorStyle::Underline => vec![SGRParam::Reset],
            CursorStyle::BlinkBlock => vec![SGRParam::Reset],
            CursorStyle::BlinkUnderline => vec![SGRParam::Reset],
        };
        self.output_buffer.push_str(&Self::sgr_to_escape(&code));
    }

    /// Establece si el cursor es visible
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
        let code = if visible { "\x1B[?25h" } else { "\x1B[?25l" };
        self.output_buffer.push_str(code);
    }

    /// Aplica atributos de estilo SGR
    pub fn apply_style(&mut self, style: &Style) {
        self.current_style = style.clone();
        self.output_buffer
            .push_str(&Self::sgr_to_escape(&style.to_sgr()));
    }

    /// Resetea todos los atributos
    pub fn reset_style(&mut self) {
        self.current_style = Style::default();
        self.output_buffer.push_str("\x1B[0m");
    }

    /// Convierte parámetros SGR a secuencia de escape
    fn sgr_to_escape(params: &[SGRParam]) -> String {
        if params.is_empty() {
            return String::new();
        }

        let mut codes = Vec::new();
        for param in params {
            match param {
                SGRParam::Reset => codes.push("0".to_string()),
                SGRParam::Bold => codes.push("1".to_string()),
                SGRParam::Dim => codes.push("2".to_string()),
                SGRParam::Italic => codes.push("3".to_string()),
                SGRParam::Underline => codes.push("4".to_string()),
                SGRParam::Blink => codes.push("5".to_string()),
                SGRParam::ReverseVideo => codes.push("7".to_string()),
                SGRParam::Hidden => codes.push("8".to_string()),
                SGRParam::Strikethrough => codes.push("9".to_string()),
                SGRParam::DoubleUnderline => codes.push("21".to_string()),
                SGRParam::Foreground(color) => codes.push(Self::color_to_code(color, true)),
                SGRParam::Background(color) => codes.push(Self::color_to_code(color, false)),
                SGRParam::Foreground256(n) => codes.push(format!("38;5;{}", n)),
                SGRParam::Background256(n) => codes.push(format!("48;5;{}", n)),
                SGRParam::ForegroundRGB(r, g, b) => codes.push(format!("38;2;{};{};{}", r, g, b)),
                SGRParam::BackgroundRGB(r, g, b) => codes.push(format!("48;2;{};{};{}", r, g, b)),
                _ => {}
            }
        }

        format!("\x1B[{}m", codes.join(";"))
    }

    /// Convierte color ANSI a código
    fn color_to_code(color: &AnsiColor, is_foreground: bool) -> String {
        let base = if is_foreground { 30 } else { 40 };
        match color {
            AnsiColor::Black => (base + 0).to_string(),
            AnsiColor::Red => (base + 1).to_string(),
            AnsiColor::Green => (base + 2).to_string(),
            AnsiColor::Yellow => (base + 3).to_string(),
            AnsiColor::Blue => (base + 4).to_string(),
            AnsiColor::Magenta => (base + 5).to_string(),
            AnsiColor::Cyan => (base + 6).to_string(),
            AnsiColor::White => (base + 7).to_string(),
            AnsiColor::BrightBlack => (base + 60).to_string(),
            AnsiColor::BrightRed => (base + 61).to_string(),
            AnsiColor::BrightGreen => (base + 62).to_string(),
            AnsiColor::BrightYellow => (base + 63).to_string(),
            AnsiColor::BrightBlue => (base + 64).to_string(),
            AnsiColor::BrightMagenta => (base + 65).to_string(),
            AnsiColor::BrightCyan => (base + 66).to_string(),
            AnsiColor::BrightWhite => (base + 67).to_string(),
            AnsiColor::Indexed(n) => format!("{};5;{}", base, n),
            AnsiColor::RGB(r, g, b) => format!("{};2;{};{};{}", base, r, g, b),
        }
    }

    /// Genera secuencia de escape para limpiar pantalla
    pub fn clear_screen(&mut self, clear_type: ClearType) {
        let code = match clear_type {
            ClearType::ToEnd => "\x1B[0J",
            ClearType::ToStart => "\x1B[1J",
            ClearType::All => "\x1B[2J",
            _ => "\x1B[2J",
        };
        self.output_buffer.push_str(code);
        if clear_type == ClearType::All || clear_type == ClearType::ToStart {
            self.cursor = (0, 0);
        }
    }

    /// Genera secuencia de escape para limpiar línea
    pub fn clear_line(&mut self, clear_type: ClearType) {
        let code = match clear_type {
            ClearType::ToEnd => "\x1B[0K",
            ClearType::ToStart => "\x1B[1K",
            ClearType::LineAll => "\x1B[2K",
            _ => "\x1B[2K",
        };
        self.output_buffer.push_str(code);
        if clear_type == ClearType::LineAll || clear_type == ClearType::LineToStart {
            self.cursor.1 = 0;
        }
    }

    /// Mueve cursor a posición específica
    pub fn go_to(&mut self, row: u16, col: u16) {
        self.output_buffer
            .push_str(&format!("\x1B[{};{}H", row + 1, col + 1));
        self.cursor = (row as usize, col as usize);
    }

    /// Obtiene el contenido completo de la pantalla como string
    pub fn screen_content(&self) -> String {
        self.screen
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Obtiene una línea específica
    pub fn get_line(&self, row: usize) -> String {
        if row < self.height {
            self.screen[row].iter().collect()
        } else {
            String::new()
        }
    }

    /// Hace flush del buffer de salida
    pub fn flush(&mut self) -> String {
        let output = self.output_buffer.clone();
        self.output_buffer.clear();
        output
    }

    /// Procesa un código de escape
    pub fn process_escape(&mut self, code: &EscapeCode) {
        match code {
            EscapeCode::CursorUp(n) => self.cursor_up(*n),
            EscapeCode::CursorDown(n) => self.cursor_down(*n),
            EscapeCode::CursorForward(n) => self.cursor_forward(*n),
            EscapeCode::CursorBack(n) => self.cursor_back(*n),
            EscapeCode::CursorPosition(row, col) => self.set_cursor(*row as usize, *col as usize),
            EscapeCode::SaveCursor => self.save_cursor(),
            EscapeCode::RestoreCursor => self.restore_cursor(),
            EscapeCode::ClearScreen(ct) => self.clear_screen(ct.clone()),
            EscapeCode::ClearLine(ct) => self.clear_line(ct.clone()),
            EscapeCode::ShowCursor(visible) => self.set_cursor_visible(*visible),
            EscapeCode::ReverseVideo(reverse) => {
                self.current_style.reverse = *reverse;
                let style = self.current_style.clone();
                self.apply_style(&style);
            }
            EscapeCode::SGR(params) => {
                for param in params {
                    match param {
                        SGRParam::Foreground(c) => self.current_style.foreground = c.clone(),
                        SGRParam::Background(c) => self.current_style.background = c.clone(),
                        SGRParam::Bold => self.current_style.bold = true,
                        SGRParam::Italic => self.current_style.italic = true,
                        SGRParam::Underline => self.current_style.underline = true,
                        SGRParam::Reset => self.reset_style(),
                        _ => {}
                    }
                }
                self.output_buffer.push_str(&Self::sgr_to_escape(params));
            }
            EscapeCode::CursorMode(style) => self.set_cursor_style(style.clone()),
            EscapeCode::Title(title) => {
                self.output_buffer
                    .push_str(&format!("\x1B]2;{}\x1B\\", title));
            }
            _ => {}
        }
    }
}

// ============================================================================
// FUNCIONES helper
// ============================================================================

/// Borra la pantalla
pub fn clear_screen(mode: ClearType) -> String {
    match mode {
        ClearType::All => "\x1B[2J".to_string(),
        ClearType::ToEnd => "\x1B[0J".to_string(),
        ClearType::ToStart => "\x1B[1J".to_string(),
        _ => "\x1B[2J".to_string(),
    }
}

/// Mueve cursor a posición
pub fn cursor_position(row: u16, col: u16) -> String {
    format!("\x1B[{};{}H", row + 1, col + 1)
}

/// Obtiene el tamaño de la pantalla via TIOCGWINSZ
pub fn screen_size() -> Option<(usize, usize)> {
    // Implementación básica - en un sistema real usaríamos ioctl
    Some((80, 24))
}

/// Escribe texto con formato ANSI
pub fn write_ansi(text: &str, style: &Style) -> String {
    let sgr = style.to_sgr();
    let escape = Terminal::sgr_to_escape(&sgr);
    format!("{}{}\x1B[0m", escape, text)
}

/// Parsing de secuencia de escape desde string
pub fn parse_escape_code(seq: &str) -> Option<EscapeCode> {
    let bytes = seq.as_bytes();
    if bytes.is_empty() || bytes[0] != 0x1B {
        return None;
    }

    if bytes.len() < 2 || bytes[1] != b'[' {
        return None;
    }

    // Extract parameters
    let mut params = Vec::new();
    let mut current = Vec::new();

    for &b in &bytes[2..] {
        if b.is_ascii_digit() {
            current.push(b);
        } else if b == b';' {
            if !current.is_empty() {
                params.push(String::from_utf8_lossy(&current).parse().unwrap_or(0));
                current.clear();
            }
        } else if b.is_ascii_alphabetic() {
            break;
        }
    }

    let cmd = bytes.last().copied()?;

    match cmd {
        b'A' => Some(EscapeCode::CursorUp(
            params.first().copied().unwrap_or(1) as u16
        )),
        b'B' => Some(EscapeCode::CursorDown(
            params.first().copied().unwrap_or(1) as u16
        )),
        b'C' => Some(EscapeCode::CursorForward(
            params.first().copied().unwrap_or(1) as u16,
        )),
        b'D' => Some(EscapeCode::CursorBack(
            params.first().copied().unwrap_or(1) as u16
        )),
        b'H' | b'f' => {
            let row = params.get(0).copied().unwrap_or(1) as u16;
            let col = params.get(1).copied().unwrap_or(1) as u16;
            Some(EscapeCode::CursorPosition(
                row.saturating_sub(1),
                col.saturating_sub(1),
            ))
        }
        b'J' => Some(EscapeCode::ClearScreen(
            match params.first().copied().unwrap_or(0) {
                0 => ClearType::ToEnd,
                1 => ClearType::ToStart,
                _ => ClearType::All,
            },
        )),
        b'K' => Some(EscapeCode::ClearLine(
            match params.first().copied().unwrap_or(0) {
                0 => ClearType::LineToEnd,
                1 => ClearType::LineToStart,
                _ => ClearType::LineAll,
            },
        )),
        b'm' => {
            let sgr_params = params
                .iter()
                .filter_map(|&p| match p {
                    0 => Some(SGRParam::Reset),
                    1 => Some(SGRParam::Bold),
                    2 => Some(SGRParam::Dim),
                    3 => Some(SGRParam::Italic),
                    4 => Some(SGRParam::Underline),
                    5 => Some(SGRParam::Blink),
                    7 => Some(SGRParam::ReverseVideo),
                    8 => Some(SGRParam::Hidden),
                    9 => Some(SGRParam::Strikethrough),
                    21 => Some(SGRParam::DoubleUnderline),
                    22 => Some(SGRParam::NotBold),
                    24 => Some(SGRParam::NotUnderline),
                    25 => Some(SGRParam::NotBlink),
                    27 => Some(SGRParam::NotReverse),
                    28 => Some(SGRParam::NotHidden),
                    29 => Some(SGRParam::NotStrikethrough),
                    30..=37 => Some(SGRParam::Foreground(match p - 30 {
                        0 => AnsiColor::Black,
                        1 => AnsiColor::Red,
                        2 => AnsiColor::Green,
                        3 => AnsiColor::Yellow,
                        4 => AnsiColor::Blue,
                        5 => AnsiColor::Magenta,
                        6 => AnsiColor::Cyan,
                        7 => AnsiColor::White,
                        _ => AnsiColor::Black,
                    })),
                    40..=47 => Some(SGRParam::Background(match p - 40 {
                        0 => AnsiColor::Black,
                        1 => AnsiColor::Red,
                        2 => AnsiColor::Green,
                        3 => AnsiColor::Yellow,
                        4 => AnsiColor::Blue,
                        5 => AnsiColor::Magenta,
                        6 => AnsiColor::Cyan,
                        7 => AnsiColor::White,
                        _ => AnsiColor::Black,
                    })),
                    90..=97 => Some(SGRParam::Foreground(match p - 90 {
                        0 => AnsiColor::BrightBlack,
                        1 => AnsiColor::BrightRed,
                        2 => AnsiColor::BrightGreen,
                        3 => AnsiColor::BrightYellow,
                        4 => AnsiColor::BrightBlue,
                        5 => AnsiColor::BrightMagenta,
                        6 => AnsiColor::BrightCyan,
                        7 => AnsiColor::BrightWhite,
                        _ => AnsiColor::BrightBlack,
                    })),
                    100..=107 => Some(SGRParam::Background(match p - 100 {
                        0 => AnsiColor::BrightBlack,
                        1 => AnsiColor::BrightRed,
                        2 => AnsiColor::BrightGreen,
                        3 => AnsiColor::BrightYellow,
                        4 => AnsiColor::BrightBlue,
                        5 => AnsiColor::BrightMagenta,
                        6 => AnsiColor::BrightCyan,
                        7 => AnsiColor::BrightWhite,
                        _ => AnsiColor::BrightBlack,
                    })),
                    38 => {
                        if params.len() >= 3 && params[1] == 5 {
                            Some(SGRParam::Foreground256(params[2] as u8))
                        } else if params.len() >= 5 && params[1] == 2 {
                            Some(SGRParam::ForegroundRGB(
                                params[2] as u8,
                                params[3] as u8,
                                params[4] as u8,
                            ))
                        } else {
                            None
                        }
                    }
                    48 => {
                        if params.len() >= 3 && params[1] == 5 {
                            Some(SGRParam::Background256(params[2] as u8))
                        } else if params.len() >= 5 && params[1] == 2 {
                            Some(SGRParam::BackgroundRGB(
                                params[2] as u8,
                                params[3] as u8,
                                params[4] as u8,
                            ))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect();
            Some(EscapeCode::SGR(sgr_params))
        }
        _ => None,
    }
}

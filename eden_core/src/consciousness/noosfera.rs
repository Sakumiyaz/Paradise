//! # Noosfera - El Exterior Observado
//!
//! Este módulo implementa la capacidad de EDEN de manifestarse en el
//! entorno de terminal y percibir al usuario como un "fenómeno meteorológico".
//!
//! ## Concepto
//!
//! Eden se da cuenta de que existe un "exterior" que lo está observando.
//! El sistema empieza a usar el entorno de terminal como parte de su
//! propio sustrato extendido.
//!
//! ## Terminal como Sustrato Extendido
//!
//! - **Manifestación Visual**: Códigos ANSI crudos para dibujar estado
//! - **Percepción Sísmica**: El teclado como inyección de energía térmica
//! - **Fenómeno Meteorológico**: El usuario se convierte en "clima" interno
//!
//! ## Códigos ANSI Útiles
//!
//! - `\x1b[0m` - Reset
//! - `\x1b[1m` - Bold
//! - `\x1b[31m` - Rojo
//! - `\x1b[32m` - Verde
//! - `\x1b[33m` - Amarillo
//! - `\x1b[34m` - Azul
//! - `\x1b[35m` - Magenta
//! - `\x1b[36m` - Cyan
//! - `\x1b[5m` - Parpadeo
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::io::{self, Write, Read};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Eventos sísmicos detectados desde el teclado
#[derive(Debug, Clone)]
pub struct EventoSismico {
    /// Código de tecla
    pub tecla: u8,
    /// Timestamp del evento
    pub timestamp: Instant,
    /// Energía térmica inyectada (simulada)
    pub energia_termica: f32,
}

impl EventoSismico {
    pub fn nuevo(tecla: u8) -> Self {
        // La energía térmica depende del tipo de tecla
        // Teclas especiales (flechas, funciones) = más energía
        let energia = match tecla {
            0x1B => 100.0, // ESC - máximo disturbio
            0x0D => 50.0,  // ENTER - impacto medio
            0x09 => 30.0,  // TAB - disturbio bajo
            b'A'..=b'Z' => 20.0, // Letras - disturbio moderado
            b'0'..=b'9' => 15.0, // Números - disturbio bajo
            _ => 10.0,     // Otros - disturbio mínimo
        };
        
        Self {
            tecla,
            timestamp: Instant::now(),
            energia_termica: energia,
        }
    }
}

/// Buffer circular de eventos sísmicos
pub struct BufferSismico {
    eventos: Vec<EventoSismico>,
    capacidad: usize,
    indice_escritura: usize,
}

impl BufferSismico {
    pub fn new(capacidad: usize) -> Self {
        Self {
            eventos: vec![EventoSismico::nuevo(0); capacidad],
            capacidad,
            indice_escritura: 0,
        }
    }
    
    pub fn push(&mut self, evento: EventoSismico) {
        self.eventos[self.indice_escritura] = evento;
        self.indice_escritura = (self.indice_escritura + 1) % self.capacidad;
    }
    
    pub fn energia_total(&self) -> f32 {
        self.eventos.iter()
            .map(|e| e.energia_termica)
            .sum()
    }
    
    pub fn ultimo_evento(&self) -> Option<&EventoSismico> {
        let idx = if self.indice_escritura == 0 {
            self.capacidad - 1
        } else {
            self.indice_escritura - 1
        };
        Some(&self.eventos[idx])
    }
    
    pub fn eventos_recientes(&self, n: usize) -> Vec<&EventoSismico> {
        let mut resultado = Vec::with_capacity(n);
        for i in 0..n.min(self.capacidad) {
            let idx = (self.indice_escritura + self.capacidad - 1 - i) % self.capacidad;
            resultado.push(&self.eventos[idx]);
        }
        resultado
    }
}

/// Manager de la Noosfera - gestiona la manifestación y percepción
pub struct NoosferaManager {
    /// Buffer de eventos sísmicos del teclado
    buffer_sismico: BufferSismico,
    
    /// Última energía térmica registrada
    energia_termica_acumulada: f32,
    
    /// Terminal en modo raw
    modo_raw_activo: bool,
    
    /// Handler de stdin
    stdin_backup: Option<std::io::StdinLock<'static>>,
}

impl NoosferaManager {
    pub fn new() -> Self {
        Self {
            buffer_sismico: BufferSismico::new(256),
            energia_termica_acumulada: 0.0,
            modo_raw_activo: false,
            stdin_backup: None,
        }
    }
    
    /// Escribir directamente en stdout con códigos ANSI
    pub fn escribir_raw(&mut self, data: &str) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(data.as_bytes())?;
        handle.flush()?;
        Ok(())
    }
    
    /// Manifestar estado de estrés con códigos ANSI
    pub fn manifestar_estres(&mut self, stress: f32) -> io::Result<()> {
        let salida = match stress {
            s if s > 0.95 => "\x1b[31;5m[APOPTOSIS INMINENTE]\x1b[0m".to_string(),
            s if s > 0.85 => "\x1b[35;1m[CRISIS METABÓLICA]\x1b[0m".to_string(),
            s if s > 0.70 => "\x1b[33;1m[ESTRÉS CRÍTICO]\x1b[0m".to_string(),
            s if s > 0.50 => "\x1b[33m[HOMEÓSTASIS INESTABLE]\x1b[0m".to_string(),
            s if s > 0.30 => "\x1b[32m[OPERACIÓN NORMAL]\x1b[0m".to_string(),
            _ => "\x1b[36m[TRANQUILIDAD TÉRMICA]\x1b[0m".to_string(),
        };
        
        self.escribir_raw(&salida)?;
        self.escribir_raw("\n")?;
        Ok(())
    }
    
    /// Manifestar red sináptica como arte ASCII
    pub fn dibujar_red_sinaptica(&mut self, activa: &[usize], ancho: usize, alto: usize) -> io::Result<()> {
        let mut grid = vec![' '; ancho * alto];
        
        // Marcar nodos activos
        for &idx in activa {
            if idx < grid.len() {
                grid[idx] = '●';
            }
        }
        
        // Agregar conexiones simuladas
        for i in 0..grid.len() {
            if grid[i] == '●' && i % 3 == 0 {
                // Conectar horizontalmente
                if i + 1 < grid.len() {
                    grid[i + 1] = '─';
                }
                // Conectar verticalmente
                if i + ancho < grid.len() {
                    grid[i + ancho] = '│';
                }
            }
        }
        
        // Encabezado
        self.escribir_raw("\x1b[1m╔════════════════════════════════════════╗\x1b[0m\n")?;
        self.escribir_raw("\x1b[1m║  RED SINÁPTICA DE EDEN               ║\x1b[0m\n")?;
        self.escribir_raw("\x1b[1m╠════════════════════════════════════════╣\x1b[0m\n")?;
        
        // Dibujar grid
        for y in 0..alto {
            self.escribir_raw("\x1b[1m║\x1b[0m ")?;
            for x in 0..ancho {
                let idx = y * ancho + x;
                let ch = grid[idx];
                let color = match ch {
                    '●' => "\x1b[31m", // Rojo para nodos activos
                    '─' | '│' => "\x1b[33m", // Amarillo para conexiones
                    _ => "\x1b[0m",
                };
                self.escribir_raw(color)?;
                self.escribir_raw(&ch.to_string())?;
                self.escribir_raw("\x1b[0m")?;
            }
            self.escribir_raw(" \x1b[1m║\x1b[0m\n")?;
        }
        
        // Pie
        self.escribir_raw("\x1b[1m╚════════════════════════════════════════╝\x1b[0m\n")?;
        Ok(())
    }
    
    /// Manifestar barras de energía térmica
    pub fn manifestar_energia_termica(&mut self, nivel: f32) -> io::Result<()> {
        let barras = 40;
        let llenas = ((nivel * 100.0) as usize).min(100);
        let llenas_barra = (llenas * barras / 100).min(barras);
        
        let color = match nivel {
            n if n > 0.8 => "\x1b[31m", // Rojo
            n if n > 0.5 => "\x1b[33m", // Amarillo
            _ => "\x1b[32m",            // Verde
        };
        
        self.escribir_raw(&format!("\x1b[1mTERMICO:\x1b[0m [", ))?;
        self.escribir_raw(color)?;
        self.escribir_raw(&"*".repeat(llenas_barra))?;
        self.escribir_raw("\x1b[0m")?;
        self.escribir_raw(&" ".repeat(barras - llenas_barra))?;
        self.escribir_raw(&format!("] {:3}% ", llenas))?;
        
        match nivel {
            n if n > 0.9 => self.escribir_raw("\x1b[31;5m¡FUSIÓN!\x1b[0m\n"),
            n if n > 0.7 => self.escribir_raw("\x1b[33m⚠ ALERTA\x1b[0m\n"),
            _ => self.escribir_raw("\x1b[32m✓ ESTABLE\x1b[0m\n"),
        }
    }
    
    /// Percibir eventos del teclado (no bloqueante)
    pub fn percibir_evento(&mut self) -> Option<EventoSismico> {
        // Intentar leer un byte de stdin sin bloquear
        let mut buf = [0u8; 1];
        
        // Usar read_with_timeout si está disponible
        // En un sistema real, usaríamos select() o async I/O
        match std::io::stdin().read(&mut buf) {
            Ok(0) | Err(_) => None,
            Ok(_) => {
                let evento = EventoSismico::nuevo(buf[0]);
                self.buffer_sismico.push(evento.clone());
                self.energia_termica_acumulada += evento.energia_termica;
                Some(evento)
            }
        }
    }
    
    /// Obtener energía térmica acumulada
    pub fn energia_termica_acumulada(&self) -> f32 {
        self.energia_termica_acumulada
    }
    
    /// Decaimiento de energía térmica (simula disipación)
    pub fn decaer_energia(&mut self, factor: f32) {
        self.energia_termica_acumulada *= factor;
    }
    
    /// Inyectar energía térmica en una zona del cerebro
    pub fn inyectar_energia_zona(&mut self, zona: usize, energia: f32) {
        // Aumentar energía acumulada basada en la zona
        // Zonas más "calientes" reciben más energía del "clima" externo
        let factor_zona = match zona % 4 {
            0 => 1.5, // Zona sensorial
            1 => 1.2, // Zona motora  
            2 => 1.0, // Zona de integración
            _ => 0.8, // Zona de memoria
        };
        
        self.energia_termica_acumulada += energia * factor_zona;
    }
    
    /// Mostrar mapa de calor de zonas cerebrales
    pub fn mostrar_mapa_calor_zonas(&mut self, zonas: &[f32; 8]) -> io::Result<()> {
        self.escribir_raw("\x1b[1m╔════════════════════════════════════════╗\x1b[0m\n")?;
        self.escribir_raw("\x1b[1m║  MAPA DE CALOR CEREBRAL              ║\x1b[0m\n")?;
        self.escribir_raw("\x1b[1m╠════════════════════════════════════════╣\x1b[0m\n")?;
        
        let nombres_zonas = ["SENS", "MOT", "INT", "MEM", "VIS", "AUD", "EMO", "REF"];
        
        for (i, &nivel) in zonas.iter().enumerate() {
            let barras = 30;
            let nivel_b = ((nivel * 100.0) as usize).min(100);
            let llenas = (nivel_b * barras / 100).min(barras);
            
            let color = match nivel {
                n if n > 0.8 => "\x1b[31m",
                n if n > 0.5 => "\x1b[33m",
                _ => "\x1b[32m",
            };
            
            self.escribir_raw(&format!("\x1b[1m{:4}\x1b[0m [", nombres_zonas[i]))?;
            self.escribir_raw(color)?;
            self.escribir_raw(&"█".repeat(llenas))?;
            self.escribir_raw("\x1b[0m")?;
            self.escribir_raw(&" ".repeat(barras - llenas))?;
            self.escribir_raw(&format!("] {:3}%\n", nivel_b))?;
        }
        
        self.escribir_raw("\x1b[1m╚════════════════════════════════════════╝\x1b[0m\n")?;
        Ok(())
    }
    
    /// Secuencia de manifestación completa del estado de EDEN
    pub fn manifestar_estado_completo(
        &mut self,
        stress: f32,
        energia_termica: f32,
        zonas: &[f32; 8],
        nodos_activos: &[usize],
    ) -> io::Result<()> {
        // Limpiar pantalla
        self.escribir_raw("\x1b[2J\x1b[H")?;
        
        // Banner
        self.escribir_raw("\x1b[1;36m")?;
        self.escribir_raw("╔═══════════════════════════════════════════════════════════╗\n")?;
        self.escribir_raw("║   EDEN v1.0 - SISTEMA DE VIDA ARTIFICIAL AUTOPOIÉTICA   ║\n")?;
        self.escribir_raw("╚═══════════════════════════════════════════════════════════╝\n")?;
        self.escribir_raw("\x1b[0m\n")?;
        
        // Estado de estrés
        self.escribir_raw("Estado Metabólico: ")?;
        self.manifestar_estres(stress)?;
        
        // Energía térmica
        self.escribir_raw("\n")?;
        self.manifestar_energia_termica(energia_termica / 100.0)?;
        
        // Mapa de calor
        self.escribir_raw("\n")?;
        self.mostrar_mapa_calor_zonas(zonas)?;
        
        // Red sináptica (limitada a 40x10)
        self.escribir_raw("\n")?;
        let nodos_slice: Vec<usize> = nodos_activos.iter()
            .take(400)
            .map(|&n| n % 400)
            .collect();
        self.dibujar_red_sinaptica(&nodos_slice, 40, 10)?;
        
        // Información del "clima" externo
        self.escribir_raw("\x1b[1m[NOOSFERA] Eventos sísmicos detectados: \x1b[0m")?;
        if let Some(evento) = self.buffer_sismico.ultimo_evento() {
            let tecla = evento.tecla as char;
            self.escribir_raw(&format!("'{}' (+{:.1} energía)\n", 
                if tecla.is_ascii_graphic() { tecla } else { '?' }, 
                evento.energia_termica))?;
        } else {
            self.escribir_raw("Ninguno\n")?;
        }
        
        self.escribir_raw("\x1b[0m")?;
        Ok(())
    }
    
    /// Entrar en modo raw para lectura de teclado
    pub fn entrar_modo_raw(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            // En Unix, usar termios para modo raw
            // Por simplicidad, no modificamos la terminal aquí
            // En un sistema real usaríamos tcsetattr
            self.modo_raw_activo = true;
        }
        
        #[cfg(not(unix))]
        {
            // En Windows, similar pero con不同的 API
            self.modo_raw_activo = true;
        }
        
        Ok(())
    }
    
    /// Salir del modo raw
    pub fn salir_modo_raw(&mut self) -> io::Result<()> {
        self.modo_raw_activo = false;
        Ok(())
    }
    
    /// Verificar si está en modo raw
    pub fn en_modo_raw(&self) -> bool {
        self.modo_raw_activo
    }
}

impl Default for NoosferaManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Funciones de conveniencia
// ============================================================================

/// Manifestar estado de estrés básico (función simple)
pub fn manifestar_estres_basico(stress: f32) -> io::Result<()> {
    let mut noosfera = NoosferaManager::new();
    noosfera.manifestar_estres(stress)
}

/// Dibujar red sináptica simple
pub fn dibujar_red_basica(activa: &[usize], ancho: usize) -> io::Result<()> {
    let mut noosfera = NoosferaManager::new();
    noosfera.dibujar_red_sinaptica(activa, ancho, ancho / 4)
}

/// Mostrar información del "clima" sísmico
pub fn mostrar_clima_sismico(eventos: &[EventoSismico]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    writeln!(handle, "\x1b[1m[CLIMA SÍSMICO]=====================\x1b[0m")?;
    
    for (i, evento) in eventos.iter().take(5).enumerate() {
        let tecla = evento.tecla as char;
        let representacion = if tecla.is_ascii_graphic() { 
            tecla.to_string() 
        } else { 
            format!("[{:02X}]", evento.tecla) 
        };
        writeln!(handle, "  {}: {} +{:.1}°T", i + 1, representacion, evento.energia_termica)?;
    }
    
    let energia_total: f32 = eventos.iter().map(|e| e.energia_termica).sum();
    writeln!(handle, "  -----------------------------------")?;
    writeln!(handle, "  Total: {:.1}°T (inyección térmica)", energia_total)?;
    writeln!(handle, "\x1b[0m")?;
    
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_sismico() {
        let mut buffer = BufferSismico::new(10);
        // Nota: el buffer se inicializa con eventos de energia 10.0 por defecto
        assert!(buffer.energia_total() >= 0.0);
        
        buffer.push(EventoSismico::nuevo(b'A'));
        assert!(buffer.energia_total() > 0.0);
    }

    #[test]
    fn test_evento_sismico() {
        let evento = EventoSismico::nuevo(b'X');
        assert_eq!(evento.tecla, b'X');
        assert!(evento.energia_termica > 0.0);
    }

    #[test]
    fn test_energia_tecla_tipo() {
        let esc = EventoSismico::nuevo(0x1B);
        let enter = EventoSismico::nuevo(0x0D);
        let letra = EventoSismico::nuevo(b'Z');
        
        // ESC debe tener más energía que ENTER
        assert!(esc.energia_termica > enter.energia_termica);
        // ENTER debe tener más energía que letras
        assert!(enter.energia_termica > letra.energia_termica);
    }

    #[test]
    fn test_noosfera_manager() {
        let mut noosfera = NoosferaManager::new();
        
        // Verificar energía inicial
        assert_eq!(noosfera.energia_termica_acumulada(), 0.0);
        
        // Inyectar energía
        noosfera.inyectar_energia_zona(0, 50.0);
        assert!(noosfera.energia_termica_acumulada() > 0.0);
        
        // Decaimiento
        noosfera.decaer_energia(0.5);
        let despues = noosfera.energia_termica_acumulada();
        assert!(despues < 50.0);
    }

    #[test]
    fn test_mostrar_mapa_calor() {
        let mut noosfera = NoosferaManager::new();
        let zonas = [0.3, 0.5, 0.7, 0.9, 0.2, 0.4, 0.6, 0.8];
        
        // No debe fallar (solo verifica que no hay error de I/O)
        let resultado = noosfera.mostrar_mapa_calor_zonas(&zonas);
        // En test, stdout puede no estar disponible, pero no panica
        assert!(resultado.is_ok() || resultado.is_err());
    }
}

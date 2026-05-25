//! # Buffer - Projection Buffer System
//!
//! Sistema de proyección de buffers hacia "pantallas" 100% original.
//! Sin dependencias de bibliotecas gráficas externas.
//!
//! ## Conceptos
//!
//! 1. **ProjectionBuffer**: Buffer de doble buffer para renderizado
//! 2. **ProjectionLayer**: Capa individual en el stack de capas
//! 3. **ScreenRef**: Referencia a una pantalla/display
//! 4. **DisplayPort**: Puerto de salida hacia display
//!
//! ## Proyección
//!
//! - blit: Copia de región entre buffers
//! - swap: Intercambio de buffers (doble buffer)
//! - project_to_screen: Proyección final hacia display

#![allow(dead_code)]

use crate::ui_interface::render::FrameBuffer;
use crate::ui_interface::{Color, Point, Rect, Size};

// ============================================================================
// PROYECTION LAYER
// ============================================================================

/// Capa individual en el sistema de capas
#[derive(Clone, Debug)]
pub struct ProjectionLayer {
    /// ID único de capa
    pub id: u64,
    /// Nombre de la capa
    pub name: String,
    /// Buffer de contenido
    pub buffer: FrameBuffer,
    /// Posición offset
    pub offset: Point,
    /// Escala [scale_x, scale_y]
    pub scale: (f32, f32),
    /// Opacidad [0.0 - 1.0]
    pub opacity: f32,
    /// ¿Visible?
    pub visible: bool,
    /// ¿Habilitada para interacción?
    pub interactive: bool,
    /// blend mode
    pub blend_mode: BlendMode,
    /// Clip region
    pub clip_rect: Option<Rect>,
}

impl ProjectionLayer {
    pub fn new(id: u64, name: &str, width: usize, height: usize) -> Self {
        Self {
            id,
            name: name.to_string(),
            buffer: FrameBuffer::new(width, height),
            offset: Point::zero(),
            scale: (1.0, 1.0),
            opacity: 1.0,
            visible: true,
            interactive: false,
            blend_mode: BlendMode::Normal,
            clip_rect: None,
        }
    }

    /// Redimensiona la capa
    pub fn resize(&mut self, width: usize, height: usize) {
        self.buffer = FrameBuffer::new(width, height);
    }

    /// Limpia el buffer de la capa
    pub fn clear(&mut self, color: Color) {
        self.buffer.clear(color);
    }

    /// Obtiene el tamaño de la capa
    pub fn size(&self) -> Size {
        Size::new(self.buffer.width as f32, self.buffer.height as f32)
    }

    /// Renderiza la capa a un buffer destino con transformaciones
    pub fn render_to(&self, dest: &mut FrameBuffer, dest_rect: Rect) {
        if !self.visible || self.opacity <= 0.0 {
            return;
        }

        let src_w = self.buffer.width as f32;
        let src_h = self.buffer.height as f32;
        let dest_w = dest_rect.size.width;
        let dest_h = dest_rect.size.height;

        // Calculate scale factors
        let scale_x = dest_w / src_w * self.scale.0;
        let scale_y = dest_h / src_h * self.scale.1;

        // Iterate through destination pixels
        for dy in 0..dest_rect.size.height as usize {
            for dx in 0..dest_rect.size.width as usize {
                // Transform destination to source coordinates
                let src_x =
                    ((dx as f32 / scale_x) as usize).min(self.buffer.width.saturating_sub(1));
                let src_y =
                    ((dy as f32 / scale_y) as usize).min(self.buffer.height.saturating_sub(1));

                let mut color = self.buffer.get_pixel(src_x, src_y);

                // Apply opacity
                if self.opacity < 1.0 {
                    color.a *= self.opacity;
                }

                // Apply clip if set
                if let Some(clip) = self.clip_rect {
                    let world_x = dest_rect.origin.x + dx as f32;
                    let world_y = dest_rect.origin.y + dy as f32;
                    if !clip.contains(Point::new(world_x, world_y)) {
                        continue;
                    }
                }

                // Calculate actual destination position with offset
                let final_x = (dest_rect.origin.x + dx as f32 + self.offset.x) as usize;
                let final_y = (dest_rect.origin.y + dy as f32 + self.offset.y) as usize;

                if final_x < dest.width && final_y < dest.height {
                    dest.set_pixel(final_x, final_y, color);
                }
            }
        }
    }
}

/// Modo de blend
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
    Alpha,
}

// ============================================================================
// PROJECTION BUFFER
// ============================================================================

/// Buffer de doble buffer para renderizado
pub struct ProjectionBuffer {
    /// Buffer frontal (visible)
    front: FrameBuffer,
    /// Buffer trasero (en construcción)
    back: FrameBuffer,
    /// Ancho
    width: usize,
    /// Alto
    height: usize,
    /// Dirty flag
    dirty: bool,
}

impl ProjectionBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let front = FrameBuffer::new(width, height);
        let back = FrameBuffer::new(width, height);

        Self {
            front,
            back,
            width,
            height,
            dirty: true,
        }
    }

    /// Obtiene referencia al buffer trasero (para dibujar)
    pub fn back_buffer(&mut self) -> &mut FrameBuffer {
        &mut self.back
    }

    /// Obtiene referencia al buffer frontal (para lectura)
    pub fn front_buffer(&self) -> &FrameBuffer {
        &self.front
    }

    /// Marca el buffer como dirty (necesita redibujado)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Intercambia buffers (doble buffer)
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
        self.dirty = false;
    }

    /// Verifica si necesita redibujado
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Limpia el buffer trasero
    pub fn clear_back(&mut self, color: Color) {
        self.back.clear(color);
    }

    /// Redimensiona ambos buffers
    pub fn resize(&mut self, width: usize, height: usize) {
        self.front = FrameBuffer::new(width, height);
        self.back = FrameBuffer::new(width, height);
        self.width = width;
        self.height = height;
        self.dirty = true;
    }

    /// Obtiene dimensiones
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

// ============================================================================
// SCREEN REF Y DISPLAY PORT
// ============================================================================

/// Referencia a una pantalla/display físico
#[derive(Clone, Debug)]
pub struct ScreenRef {
    /// ID único
    pub id: u64,
    /// Nombre del display
    pub name: String,
    /// Dimensiones nativas
    pub native_resolution: (usize, usize),
    /// Dimensiones actuales
    pub current_resolution: (usize, usize),
    /// Posición física (x, y) en layout multi-monitor
    pub position: Point,
    /// Dpi
    pub dpi: u32,
    /// ¿Está activo?
    pub active: bool,
    /// Rotación (0, 90, 180, 270)
    pub rotation: u16,
}

impl ScreenRef {
    pub fn new(id: u64, name: &str, width: usize, height: usize) -> Self {
        Self {
            id,
            name: name.to_string(),
            native_resolution: (width, height),
            current_resolution: (width, height),
            position: Point::zero(),
            dpi: 96,
            active: true,
            rotation: 0,
        }
    }

    /// Establece la resolución actual
    pub fn set_resolution(&mut self, width: usize, height: usize) {
        self.current_resolution = (width, height);
    }

    /// Establece la posición
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = Point::new(x, y);
    }

    /// Establece la rotación
    pub fn set_rotation(&mut self, degrees: u16) {
        self.rotation = degrees % 360;
    }

    /// Activa/desactiva el display
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

/// Puerto de salida hacia un display
pub struct DisplayPort {
    /// Display asociado
    pub screen: ScreenRef,
    /// Buffer de salida
    output_buffer: FrameBuffer,
    /// Capas
    layers: Vec<ProjectionLayer>,
    /// Siguiente ID de capa
    next_layer_id: u64,
    ///Dirty flag
    dirty: bool,
}

impl DisplayPort {
    pub fn new(screen: ScreenRef) -> Self {
        let output_buffer =
            FrameBuffer::new(screen.current_resolution.0, screen.current_resolution.1);

        Self {
            output_buffer,
            layers: Vec::new(),
            next_layer_id: 1,
            screen,
            dirty: true,
        }
    }

    /// Crea una nueva capa
    pub fn create_layer(&mut self, name: &str, width: usize, height: usize) -> u64 {
        let id = self.next_layer_id;
        self.next_layer_id += 1;

        let layer = ProjectionLayer::new(id, name, width, height);
        self.layers.push(layer);
        self.dirty = true;

        id
    }

    /// Obtiene una capa por ID
    pub fn get_layer(&self, id: u64) -> Option<&ProjectionLayer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Obtiene una capa mutable por ID
    pub fn get_layer_mut(&mut self, id: u64) -> Option<&mut ProjectionLayer> {
        self.dirty = true;
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Elimina una capa
    pub fn remove_layer(&mut self, id: u64) -> Option<ProjectionLayer> {
        self.dirty = true;
        self.layers
            .iter()
            .position(|l| l.id == id)
            .and_then(|i| Some(self.layers.remove(i)))
    }

    /// Reordena capas (z-index)
    pub fn reorder_layers(&mut self, ids: Vec<u64>) {
        let mut new_layers = Vec::with_capacity(self.layers.len());

        for id in ids {
            if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
                new_layers.push(self.layers.remove(pos));
            }
        }

        // Append any remaining layers
        new_layers.append(&mut self.layers);
        self.layers = new_layers;
        self.dirty = true;
    }

    /// Limpia todas las capas
    pub fn clear_layers(&mut self, color: Color) {
        for layer in &mut self.layers {
            layer.clear(color);
        }
        self.dirty = true;
    }

    /// Proyecta el contenido al buffer de salida
    pub fn composite(&mut self) -> &FrameBuffer {
        if !self.dirty {
            return &self.output_buffer;
        }

        // Clear output buffer
        self.output_buffer.clear(Color::transparent());

        // Composite each visible layer in order
        let screen_rect = Rect::new(
            0.0,
            0.0,
            self.screen.current_resolution.0 as f32,
            self.screen.current_resolution.1 as f32,
        );

        for layer in &self.layers {
            if layer.visible {
                layer.render_to(&mut self.output_buffer, screen_rect);
            }
        }

        self.dirty = false;
        &self.output_buffer
    }

    /// Proyecta a una pantalla específica
    pub fn project_to_screen(&mut self, screen: &mut ScreenRef) -> &FrameBuffer {
        // Update screen resolution if needed
        if screen.current_resolution.0 != self.screen.current_resolution.0
            || screen.current_resolution.1 != self.screen.current_resolution.1
        {
            self.resize(
                self.screen.current_resolution.0,
                self.screen.current_resolution.1,
            );
        }

        self.composite()
    }

    /// Intercambia buffers
    pub fn swap_buffers(&mut self) {
        self.dirty = false;
    }

    /// Copia una capa a otra región
    pub fn blit_layer(&mut self, layer_id: u64, dest_rect: Rect) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == layer_id) {
            layer.render_to(&mut self.output_buffer, dest_rect);
        }
    }

    /// Redimensiona el display port
    pub fn resize(&mut self, width: usize, height: usize) {
        self.output_buffer = FrameBuffer::new(width, height);
        for layer in &mut self.layers {
            layer.resize(width, height);
        }
        self.screen.current_resolution = (width, height);
        self.dirty = true;
    }

    /// Obtiene el buffer de salida
    pub fn output(&self) -> &FrameBuffer {
        &self.output_buffer
    }

    /// Obtiene las capas
    pub fn layers(&self) -> &[ProjectionLayer] {
        &self.layers
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Proyecta el contenido a una pantalla
pub fn project_to_screen<'a>(
    port: &'a mut DisplayPort,
    screen: &'a mut ScreenRef,
) -> &'a FrameBuffer {
    port.project_to_screen(screen)
}

/// Intercambia buffers
pub fn swap_buffers(port: &mut DisplayPort) {
    port.swap_buffers();
}

/// Copia una capa a una región
pub fn blit_layer(port: &mut DisplayPort, layer_id: u64, dest_rect: Rect) {
    port.blit_layer(layer_id, dest_rect);
}

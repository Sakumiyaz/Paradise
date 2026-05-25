//! # Widgets - Mental Widget System
//!
//! Sistema de widgets mentales 100% original para EDEN.
//! Sin dependencias de bibliotecas UI externas.
//!
//! ## Tipos de Widgets
//!
//! - Container: Contenedor con children
//! - Button: Botón clickeable
//! - Label: Texto estático
//! - TextInput: Campo de texto
//! - Slider: Deslizador de valor
//! - ProgressBar: Barra de progreso
//! - Image: Imagen renderizada
//! - Canvas: Lienzo para dibujo libre
//! - Scrollable: Contenedor con scroll
//! - Splitter: Divisor de paneles
//!
//! ## Arquitectura
//!
//! 1. **WidgetTree**: Árbol jerárquico de widgets
//! 2. **WidgetEvent**: Eventos específicos de widget
//! 3. **LayoutConstraint**: Restricciones de layout
//! 4. **SizeHint**: Sugerencias de tamaño

#![allow(dead_code)]

use crate::ui_interface::{Color, Point, Rect, Size, VectorRenderer};
use std::collections::HashMap;

// ============================================================================
// TIPOS BASE
// ============================================================================

/// Identificador único de widget
pub type WidgetId = u64;

/// Estado de un widget
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WidgetState {
    /// Widget normal
    Normal,
    /// Widget deshabilitado
    Disabled,
    /// Widget enfocado
    Focused,
    /// Widget siendo presionado
    Pressed,
    /// Widget seleccionado
    Selected,
    /// Widget en hover
    Hover,
}

/// Tipo de widget
#[derive(Clone, Debug, PartialEq)]
pub enum WidgetKind {
    Container,
    Button,
    Label,
    TextInput,
    Slider,
    ProgressBar,
    Image,
    Canvas,
    Scrollable,
    Splitter,
}

/// Evento de widget
#[derive(Clone, Debug, PartialEq)]
pub enum WidgetEvent {
    Click { x: f32, y: f32 },
    DoubleClick { x: f32, y: f32 },
    Hover { x: f32, y: f32 },
    Press { x: f32, y: f32 },
    Release,
    Focus,
    Blur,
    Change { value: f32 },
    Submit { text: String },
    Scroll { delta: f32 },
}

/// Widget base
#[derive(Clone, Debug)]
pub struct Widget {
    /// ID único
    pub id: WidgetId,
    /// Tipo de widget
    pub kind: WidgetKind,
    /// Rectángulo bounds
    pub bounds: Rect,
    /// Estado actual
    pub state: WidgetState,
    /// Estilo visual
    pub style: WidgetStyle,
    /// ¿Visible?
    pub visible: bool,
    /// ¿Habilitado?
    pub enabled: bool,
    /// Children (para contenedores)
    pub children: Vec<WidgetId>,
    /// Parent ID
    pub parent: Option<WidgetId>,
    /// Propiedades específicas del tipo
    pub props: WidgetProps,
}

/// Propiedades específicas por tipo
#[derive(Clone, Debug, PartialEq)]
pub enum WidgetProps {
    Empty,
    Label {
        text: String,
        font_size: f32,
    },
    Button {
        text: String,
        icon: Option<String>,
    },
    TextInput {
        text: String,
        placeholder: String,
        multiline: bool,
    },
    Slider {
        value: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    ProgressBar {
        progress: f32,
        show_text: bool,
    },
    Image {
        data: Vec<u8>,
        path: Option<String>,
    },
    Canvas {
        buffer: Vec<u8>,
    },
    Scrollable {
        content_size: Size,
        scroll_pos: Point,
    },
    Splitter {
        orientation: Orientation,
        split_ratio: f32,
    },
}

/// Orientación
#[derive(Clone, Debug, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Estilo visual de widget
#[derive(Clone, Debug)]
pub struct WidgetStyle {
    /// Color de fondo
    pub background: Color,
    /// Color del borde
    pub border_color: Color,
    /// Ancho del borde
    pub border_width: f32,
    /// Color del texto
    pub text_color: Color,
    /// Color de fondo según estado
    pub state_colors: HashMap<WidgetState, Color>,
    /// Padding interno
    pub padding: f32,
    /// Margen externo
    pub margin: f32,
    /// Border radius
    pub border_radius: f32,
    /// Font size (para texto)
    pub font_size: f32,
    /// Font family
    pub font_family: String,
}

impl WidgetStyle {
    pub fn default_for(kind: &WidgetKind) -> Self {
        let (bg, text) = match kind {
            WidgetKind::Label => (Color::transparent(), Color::white()),
            _ => (Color::rgb(40, 40, 40), Color::white()),
        };

        let mut state_colors = HashMap::new();
        state_colors.insert(WidgetState::Hover, Color::rgb(60, 60, 60));
        state_colors.insert(WidgetState::Pressed, Color::rgb(80, 80, 80));
        state_colors.insert(WidgetState::Focused, Color::rgb(50, 50, 80));

        Self {
            background: bg,
            border_color: Color::rgb(80, 80, 80),
            border_width: 1.0,
            text_color: text,
            state_colors,
            padding: 8.0,
            margin: 4.0,
            border_radius: 4.0,
            font_size: 14.0,
            font_family: "monospace".to_string(),
        }
    }
}

/// Hint de tamaño para layout
#[derive(Clone, Debug)]
pub struct SizeHint {
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
    pub preferred_width: f32,
    pub preferred_height: f32,
    pub stretch_factor: f32,
}

impl SizeHint {
    pub fn for_widget(widget: &Widget) -> Self {
        match &widget.props {
            WidgetProps::Label { text, font_size } => {
                let w = text.len() as f32 * font_size * 0.6;
                Self {
                    min_width: w,
                    min_height: font_size * 1.2,
                    max_width: f32::MAX,
                    max_height: font_size * 1.2,
                    preferred_width: w,
                    preferred_height: font_size * 1.2,
                    stretch_factor: 0.0,
                }
            }
            WidgetProps::Button { text, .. } => Self {
                min_width: 80.0,
                min_height: 32.0,
                max_width: f32::MAX,
                max_height: 32.0,
                preferred_width: text.len() as f32 * 10.0 + 20.0,
                preferred_height: 32.0,
                stretch_factor: 0.0,
            },
            WidgetProps::TextInput { multiline, .. } => Self {
                min_width: 100.0,
                min_height: if *multiline { 80.0 } else { 32.0 },
                max_width: f32::MAX,
                max_height: if *multiline { 200.0 } else { 32.0 },
                preferred_width: 200.0,
                preferred_height: if *multiline { 100.0 } else { 32.0 },
                stretch_factor: 1.0,
            },
            WidgetProps::Slider { .. } => Self {
                min_width: 100.0,
                min_height: 24.0,
                max_width: f32::MAX,
                max_height: 24.0,
                preferred_width: 200.0,
                preferred_height: 24.0,
                stretch_factor: 1.0,
            },
            WidgetProps::ProgressBar { .. } => Self {
                min_width: 100.0,
                min_height: 16.0,
                max_width: f32::MAX,
                max_height: 16.0,
                preferred_width: 200.0,
                preferred_height: 16.0,
                stretch_factor: 1.0,
            },
            _ => Self {
                min_width: 50.0,
                min_height: 50.0,
                max_width: f32::MAX,
                max_height: f32::MAX,
                preferred_width: 100.0,
                preferred_height: 100.0,
                stretch_factor: 1.0,
            },
        }
    }
}

/// Restricción de layout
#[derive(Clone, Debug)]
pub struct LayoutConstraint {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub exact_width: Option<f32>,
    pub exact_height: Option<f32>,
    pub stretch_horizontal: f32,
    pub stretch_vertical: f32,
}

impl LayoutConstraint {
    pub fn new() -> Self {
        Self {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            exact_width: None,
            exact_height: None,
            stretch_horizontal: 1.0,
            stretch_vertical: 1.0,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.exact_width = Some(width);
        self.exact_height = Some(height);
        self
    }

    pub fn apply_to(&self, hint: &mut SizeHint) {
        if let Some(min) = self.min_width {
            hint.min_width = min;
        }
        if let Some(max) = self.max_width {
            hint.max_width = max;
        }
        if let Some(min) = self.min_height {
            hint.min_height = min;
        }
        if let Some(max) = self.max_height {
            hint.max_height = max;
        }
        if let Some(w) = self.exact_width {
            hint.preferred_width = w;
            hint.min_width = w;
            hint.max_width = w;
        }
        if let Some(h) = self.exact_height {
            hint.preferred_height = h;
            hint.min_height = h;
            hint.max_height = h;
        }
        hint.stretch_factor = self.stretch_horizontal.max(self.stretch_vertical);
    }
}

// ============================================================================
// WIDGET TREE
// ============================================================================

/// Árbol de widgets
pub struct WidgetTree {
    ///Widgets por ID
    widgets: HashMap<WidgetId, Widget>,
    /// Orden Z (profundidad)
    z_order: Vec<WidgetId>,
    /// Siguiente ID disponible
    next_id: WidgetId,
    /// Widget enfocado actual
    focused: Option<WidgetId>,
    /// Widget en hover actual
    hovered: Option<WidgetId>,
    /// Ratón position
    mouse_pos: Point,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
            z_order: Vec::new(),
            next_id: 1,
            focused: None,
            hovered: None,
            mouse_pos: Point::zero(),
        }
    }

    /// Añade un widget al árbol
    pub fn insert(&mut self, mut widget: Widget) -> WidgetId {
        let id = self.next_id;
        self.next_id += 1;
        widget.id = id;

        // Add to parent if exists
        if let Some(parent_id) = widget.parent {
            if let Some(parent) = self.widgets.get_mut(&parent_id) {
                parent.children.push(id);
            }
        }

        self.widgets.insert(id, widget);
        self.z_order.push(id);

        id
    }

    /// Elimina un widget del árbol
    pub fn remove(&mut self, id: WidgetId) -> Option<Widget> {
        // Get widget info first
        let (parent_id, children_ids) = if let Some(widget) = self.widgets.get(&id) {
            (widget.parent, widget.children.clone())
        } else {
            return None;
        };

        // Remove from parent's children
        if let Some(pid) = parent_id {
            if let Some(parent) = self.widgets.get_mut(&pid) {
                parent.children.retain(|&c| c != id);
            }
        }

        // Remove children recursively
        for child_id in children_ids {
            self.remove(child_id);
        }

        self.z_order.retain(|&z| z != id);
        if self.focused == Some(id) {
            self.focused = None;
        }
        if self.hovered == Some(id) {
            self.hovered = None;
        }

        self.widgets.remove(&id)
    }

    /// Busca un widget por ID
    pub fn find(&self, id: WidgetId) -> Option<&Widget> {
        self.widgets.get(&id)
    }

    /// Busca un widget mutable por ID
    pub fn find_mut(&mut self, id: WidgetId) -> Option<&mut Widget> {
        self.widgets.get_mut(&id)
    }

    /// Encuentra widget en posición específica
    pub fn find_at(&self, pos: Point) -> Option<WidgetId> {
        // Iterate in reverse Z order (top to bottom)
        for &id in self.z_order.iter().rev() {
            if let Some(widget) = self.widgets.get(&id) {
                if widget.visible && widget.bounds.contains(pos) {
                    return Some(id);
                }
            }
        }
        None
    }

    /// Obtiene widget en la posición del mouse
    pub fn widget_at_mouse(&self) -> Option<WidgetId> {
        self.find_at(self.mouse_pos)
    }

    /// Actualiza la posición del mouse
    pub fn set_mouse_pos(&mut self, pos: Point) {
        let old_hovered = self.hovered;
        self.mouse_pos = pos;

        let new_hovered = self.widget_at_mouse();

        if old_hovered != new_hovered {
            if let Some(id) = old_hovered {
                if let Some(widget) = self.widgets.get_mut(&id) {
                    widget.state = WidgetState::Normal;
                }
            }
            if let Some(id) = new_hovered {
                if let Some(widget) = self.widgets.get_mut(&id) {
                    widget.state = WidgetState::Hover;
                }
            }
            self.hovered = new_hovered;
        }
    }

    /// Maneja un evento de click
    pub fn handle_click(&mut self, pos: Point, double: bool) -> Option<WidgetEvent> {
        let widget_id = self.find_at(pos)?;

        if let Some(widget) = self.widgets.get_mut(&widget_id) {
            if !widget.enabled {
                return None;
            }

            widget.state = WidgetState::Pressed;

            let event = if double {
                WidgetEvent::DoubleClick { x: pos.x, y: pos.y }
            } else {
                WidgetEvent::Click { x: pos.x, y: pos.y }
            };

            // Handle based on widget type
            match &mut widget.props {
                WidgetProps::Button { .. } => {
                    widget.state = WidgetState::Normal;
                }
                _ => {}
            }

            return Some(event);
        }

        None
    }

    /// Maneja un evento de release
    pub fn handle_release(&mut self) -> Option<WidgetEvent> {
        if let Some(pressed) = self.z_order.iter().rev().find_map(|&id| {
            let widget = self.widgets.get(&id)?;
            if widget.state == WidgetState::Pressed {
                Some(id)
            } else {
                None
            }
        }) {
            if let Some(widget) = self.widgets.get_mut(&pressed) {
                widget.state = WidgetState::Normal;
            }
            return Some(WidgetEvent::Release);
        }
        None
    }

    /// Maneja scroll
    pub fn handle_scroll(&mut self, delta: f32) -> Option<WidgetEvent> {
        if let Some(id) = self.hovered {
            if let Some(widget) = self.widgets.get_mut(&id) {
                if matches!(widget.props, WidgetProps::Scrollable { .. }) {
                    return Some(WidgetEvent::Scroll { delta });
                }
            }
        }
        None
    }

    /// Renderiza el árbol completo
    pub fn render(&self, renderer: &mut VectorRenderer) {
        for &id in &self.z_order {
            if let Some(widget) = self.widgets.get(&id) {
                if widget.visible {
                    widget_render(self, renderer, id);
                }
            }
        }
    }

    /// Obtiene el widget enfocado
    pub fn focused(&self) -> Option<WidgetId> {
        self.focused
    }

    /// Establece el widget enfocado
    pub fn set_focused(&mut self, id: Option<WidgetId>) {
        // Blur old focused
        if let Some(old) = self.focused {
            if let Some(widget) = self.widgets.get_mut(&old) {
                widget.state = WidgetState::Normal;
            }
        }

        // Focus new
        if let Some(new) = id {
            if let Some(widget) = self.widgets.get_mut(&new) {
                widget.state = WidgetState::Focused;
            }
        }

        self.focused = id;
    }
}

// ============================================================================
// FUNCIONES helper
// ============================================================================

/// Renderiza un widget específico
pub fn widget_render(tree: &WidgetTree, renderer: &mut VectorRenderer, id: WidgetId) {
    let Some(widget) = tree.find(id) else {
        return;
    };

    let style = &widget.style;
    let state = &widget.state;

    // Get background color based on state
    let bg = style
        .state_colors
        .get(state)
        .copied()
        .unwrap_or(style.background);

    // Draw background
    if bg.a > 0.0 {
        renderer.set_color(bg);
        renderer.draw_rect(widget.bounds);
    }

    // Draw border
    if style.border_width > 0.0 {
        renderer.set_color(style.border_color);
        renderer.draw_rect(widget.bounds);
    }

    // Draw content based on type
    match &widget.props {
        WidgetProps::Label { text, font_size } => {
            renderer.set_color(style.text_color);
            renderer.draw_text(widget.bounds.origin, text, *font_size);
        }
        WidgetProps::Button { text, .. } => {
            renderer.set_color(style.text_color);
            let center_x =
                widget.bounds.origin.x + widget.bounds.size.width / 2.0 - (text.len() as f32 * 5.0);
            let center_y = widget.bounds.origin.y + widget.bounds.size.height / 2.0 - 6.0;
            renderer.draw_text(Point::new(center_x, center_y), text, style.font_size);
        }
        WidgetProps::ProgressBar {
            progress,
            show_text,
        } => {
            let inner = Rect::new(
                widget.bounds.origin.x + 2.0,
                widget.bounds.origin.y + 2.0,
                (widget.bounds.size.width - 4.0) * (*progress as f32 / 100.0),
                widget.bounds.size.height - 4.0,
            );
            renderer.set_color(Color::rgb(0, 150, 255));
            renderer.draw_rect(inner);

            if *show_text {
                let text = format!("{:.0}%", progress);
                renderer.set_color(style.text_color);
                renderer.draw_text(
                    Point::new(
                        widget.bounds.origin.x + widget.bounds.size.width / 2.0 - 15.0,
                        widget.bounds.origin.y + 2.0,
                    ),
                    &text,
                    12.0,
                );
            }
        }
        WidgetProps::Slider {
            value, min, max, ..
        } => {
            let track_y = widget.bounds.origin.y + widget.bounds.size.height / 2.0 - 2.0;
            let track_rect = Rect::new(
                widget.bounds.origin.x,
                track_y,
                widget.bounds.size.width,
                4.0,
            );
            renderer.set_color(Color::rgb(60, 60, 60));
            renderer.draw_rect(track_rect);

            let ratio = (*value as f32 - *min) / (*max - *min);
            let thumb_x = widget.bounds.origin.x + widget.bounds.size.width * ratio - 6.0;
            let thumb_rect = Rect::new(thumb_x, track_y - 4.0, 12.0, 12.0);
            renderer.set_color(Color::rgb(0, 150, 255));
            renderer.draw_rect(thumb_rect);
        }
        _ => {}
    }
}

// Widget tree helper functions
pub fn widget_tree_insert(tree: &mut WidgetTree, widget: Widget) -> WidgetId {
    tree.insert(widget)
}

pub fn widget_tree_remove(tree: &mut WidgetTree, id: WidgetId) -> Option<Widget> {
    tree.remove(id)
}

pub fn widget_tree_find(tree: &WidgetTree, id: WidgetId) -> Option<&Widget> {
    tree.find(id)
}

pub fn widget_handle_click(
    tree: &mut WidgetTree,
    x: f32,
    y: f32,
    double: bool,
) -> Option<WidgetEvent> {
    tree.handle_click(Point::new(x, y), double)
}

pub fn widget_handle_release(tree: &mut WidgetTree) -> Option<WidgetEvent> {
    tree.handle_release()
}

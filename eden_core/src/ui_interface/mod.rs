//! # UI Interface Module
//!
//! Sistema de interfaz de usuario 100% original para EDEN.
//! Sin dependencias externas - renderizado vectorial, widgets mentales,
//! proyección de buffers, gestión de eventos, terminal avanzada.
//!
//! ## Componentes
//!
//! - `render`: Motor de renderizado vectorial 2D/3D
//! - `terminal`: Terminal avanzada con escape codes ANSI
//! - `widgets`: Sistema de widgets mentales
//! - `events`: Gestión de eventos de entrada
//! - `layout`: Layout engine (grid, flex, stack)
//! - `buffer`: Sistema de proyección de buffers
//!
//! ## Conceptos
//!
//! 1. **FrameBuffer**: Buffer de píxeles conceptual para renderizado
//! 2. **VectorRender**: Renderizado de primitivas geométricas desde cero
//! 3. **WidgetTree**: Árbol de widgets con jerarquía
//! 4. **EventBubble**: Sistema de eventos en burbuja
//! 5. **LayoutEngine**: Motor de layout flex/grid/stack
//! 6. **ProjectionLayer**: Capa de proyección hacia "pantallas"

#![allow(dead_code)]

pub mod buffer;
pub mod events;
pub mod layout;
pub mod render;
pub mod terminal;
pub mod widgets;

// ============================================================================
// RE-EXPORTS
// ============================================================================

pub use buffer::{
    blit_layer, project_to_screen, swap_buffers, DisplayPort, ProjectionBuffer, ProjectionLayer,
    ScreenRef,
};
pub use events::{
    create_button_event, create_key_event, create_mouse_event, create_resize_event, dispatch_event,
    poll_events, Event, EventBubble, EventHandler, EventPhase, EventType,
};
pub use layout::{
    compute_layout, flex_layout, grid_layout, invalidate_layout, stack_layout, Constraint,
    FlexParams, GridParams, LayoutDirection, LayoutEngine, LayoutNode, LayoutType, StackParams,
};
pub use render::{
    vector_render_circle, vector_render_line, vector_render_path, vector_render_point,
    vector_render_rect, vector_render_text, BlendMode, Color, Glyph, PathCommand, Point, Primitive,
    Rect, Size, Transform2D, VectorRenderer,
};
pub use terminal::{
    clear_screen, cursor_position, parse_escape_code, screen_size, write_ansi, AnsiColor,
    EscapeCode, Style, Terminal, TerminalMode,
};
pub use widgets::{
    widget_handle_click, widget_handle_release, widget_tree_find, widget_tree_insert,
    widget_tree_remove, LayoutConstraint, SizeHint, Widget, WidgetEvent, WidgetId, WidgetKind,
    WidgetState, WidgetTree,
};

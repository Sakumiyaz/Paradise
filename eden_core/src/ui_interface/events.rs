//! # Events - Event Handling System
//!
//! Sistema de gestión de eventos de entrada 100% original.
//! Sin dependencias de bibliotecas externas.
//!
//! ## Tipos de Eventos
//!
//! - Keyboard: Teclado (key press, release)
//! - Mouse: Ratón (move, click, scroll, drag)
//! - Resize: Cambio de tamaño de ventana
//! - Focus: Ganar/perder foco
//! - Custom: Eventos personalizados
//!
//! ## Arquitectura
//!
//! 1. **Event**: Tipo unificado de evento
//! 2. **EventHandler**: Closure que maneja eventos
//! 3. **EventBubble**: Sistema de burbujeo de eventos
//! 4. **EventPhase**: Fase del evento (captura, objetivo, burbuja)

#![allow(dead_code)]

use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

/// Tipo de evento
#[derive(Clone, Debug, PartialEq)]
pub enum EventType {
    /// Evento de teclado
    Keyboard(KeyboardEvent),
    /// Evento de ratón
    Mouse(MouseEvent),
    /// Evento de redimensionamiento
    Resize(ResizeEvent),
    /// Evento de foco
    Focus(FocusEvent),
    /// Evento de scroll
    Scroll(ScrollEvent),
    /// Evento de toque (touch)
    Touch(TouchEvent),
    /// Evento personalizado
    Custom,
}

/// Evento de teclado
#[derive(Clone, Debug, PartialEq)]
pub struct KeyboardEvent {
    /// Código de tecla virtual
    pub key_code: KeyCode,
    /// Carácter producido
    pub key_char: Option<char>,
    /// Modificadores activos
    pub modifiers: Modifiers,
    /// ¿Es repetición?
    pub repeat: bool,
    /// Timestamp
    pub timestamp_ms: u64,
}

/// Código de tecla
#[derive(Clone, Debug, PartialEq)]
pub enum KeyCode {
    // Teclas especiales
    Escape,
    Enter,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    // Teclas de control
    Ctrl,
    Alt,
    Shift,
    Meta,
    CapsLock,
    // Caracteres imprimibles
    Char(char),
    // Desconocido
    Unknown(u32),
}

/// Modificadores de teclado
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

/// Evento de ratón
#[derive(Clone, Debug, PartialEq)]
pub struct MouseEvent {
    /// Tipo de evento de ratón
    pub kind: MouseEventKind,
    /// Posición X
    pub x: f32,
    /// Posición Y
    pub y: f32,
    /// Botón involucrado
    pub button: MouseButton,
    /// Modificadores
    pub modifiers: Modifiers,
    /// Timestamp
    pub timestamp_ms: u64,
}

/// Tipo de evento de ratón
#[derive(Clone, Debug, PartialEq)]
pub enum MouseEventKind {
    Move,
    Press,
    Release,
    Drag,
    Drop,
    Enter,
    Leave,
}

/// Botón de ratón
#[derive(Clone, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    None,
}

/// Evento de redimensionamiento
#[derive(Clone, Debug, PartialEq)]
pub struct ResizeEvent {
    /// Nuevo ancho
    pub width: u32,
    /// Nuevo alto
    pub height: u32,
    /// Timestamp
    pub timestamp_ms: u64,
}

/// Evento de foco
#[derive(Clone, Debug, PartialEq)]
pub struct FocusEvent {
    /// ¿Ganó foco?
    pub gained: bool,
    /// Timestamp
    pub timestamp_ms: u64,
}

/// Evento de scroll
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollEvent {
    /// Delta X
    pub delta_x: f32,
    /// Delta Y
    pub delta_y: f32,
    /// Timestamp
    pub timestamp_ms: u64,
}

/// Evento de toque (touch)
#[derive(Clone, Debug, PartialEq)]
pub struct TouchEvent {
    /// Tipo de touch
    pub kind: TouchEventKind,
    /// ID del toque
    pub touch_id: u64,
    /// Posición X
    pub x: f32,
    /// Posición Y
    pub y: f32,
    /// Presión (0.0 - 1.0)
    pub pressure: f32,
    /// Timestamp
    pub timestamp_ms: u64,
}

/// Tipo de evento de touch
#[derive(Clone, Debug, PartialEq)]
pub enum TouchEventKind {
    Start,
    Move,
    End,
    Cancel,
}

/// Evento unificado
#[derive(Clone, Debug)]
pub struct Event {
    /// Tipo de evento
    pub event_type: EventType,
    /// Timestamp del sistema
    pub timestamp_ms: u64,
    /// Origen del evento (widget ID, etc)
    pub target: Option<u64>,
    /// ¿Handled?
    pub handled: bool,
    /// Fase de propagación
    pub phase: EventPhase,
}

impl Event {
    pub fn keyboard(
        key_code: KeyCode,
        key_char: Option<char>,
        modifiers: Modifiers,
        repeat: bool,
    ) -> Self {
        Self {
            event_type: EventType::Keyboard(KeyboardEvent {
                key_code,
                key_char,
                modifiers,
                repeat,
                timestamp_ms: current_time_ms(),
            }),
            timestamp_ms: current_time_ms(),
            target: None,
            handled: false,
            phase: EventPhase::Target,
        }
    }

    pub fn mouse(
        kind: MouseEventKind,
        x: f32,
        y: f32,
        button: MouseButton,
        modifiers: Modifiers,
    ) -> Self {
        Self {
            event_type: EventType::Mouse(MouseEvent {
                kind,
                x,
                y,
                button,
                modifiers,
                timestamp_ms: current_time_ms(),
            }),
            timestamp_ms: current_time_ms(),
            target: None,
            handled: false,
            phase: EventPhase::Target,
        }
    }

    pub fn resize(width: u32, height: u32) -> Self {
        Self {
            event_type: EventType::Resize(ResizeEvent {
                width,
                height,
                timestamp_ms: current_time_ms(),
            }),
            timestamp_ms: current_time_ms(),
            target: None,
            handled: false,
            phase: EventPhase::Target,
        }
    }

    pub fn focus(gained: bool) -> Self {
        Self {
            event_type: EventType::Focus(FocusEvent {
                gained,
                timestamp_ms: current_time_ms(),
            }),
            timestamp_ms: current_time_ms(),
            target: None,
            handled: false,
            phase: EventPhase::Target,
        }
    }

    pub fn scroll(delta_x: f32, delta_y: f32) -> Self {
        Self {
            event_type: EventType::Scroll(ScrollEvent {
                delta_x,
                delta_y,
                timestamp_ms: current_time_ms(),
            }),
            timestamp_ms: current_time_ms(),
            target: None,
            handled: false,
            phase: EventPhase::Target,
        }
    }

    pub fn touch(kind: TouchEventKind, touch_id: u64, x: f32, y: f32, pressure: f32) -> Self {
        Self {
            event_type: EventType::Touch(TouchEvent {
                kind,
                touch_id,
                x,
                y,
                pressure,
                timestamp_ms: current_time_ms(),
            }),
            timestamp_ms: current_time_ms(),
            target: None,
            handled: false,
            phase: EventPhase::Target,
        }
    }

    /// Marca el evento como manejado
    pub fn mark_handled(&mut self) {
        self.handled = true;
    }

    /// Verifica si el evento es de un tipo específico
    pub fn is<T: EventTypeTrait>(&self) -> bool {
        T::matches(&self.event_type)
    }
}

/// Trait para verificar tipo de evento
pub trait EventTypeTrait: 'static {
    fn matches(event_type: &EventType) -> bool;
}

impl EventTypeTrait for KeyboardEvent {
    fn matches(event_type: &EventType) -> bool {
        matches!(event_type, EventType::Keyboard(_))
    }
}

impl EventTypeTrait for MouseEvent {
    fn matches(event_type: &EventType) -> bool {
        matches!(event_type, EventType::Mouse(_))
    }
}

impl EventTypeTrait for ResizeEvent {
    fn matches(event_type: &EventType) -> bool {
        matches!(event_type, EventType::Resize(_))
    }
}

/// Fase de propagación de evento
#[derive(Clone, Debug, PartialEq)]
pub enum EventPhase {
    /// Fase de captura (top-down)
    Capture,
    /// Fase en el objetivo
    Target,
    /// Fase de burbuja (bottom-up)
    Bubble,
}

/// Handler de evento
pub type EventHandler = Arc<RwLock<dyn Fn(&mut Event) -> bool + Send + Sync>>;

/// Event bubble system
pub struct EventBubble {
    /// Event queue
    queue: VecDeque<Event>,
    /// Handlers por tipo de evento
    handlers: HashMap<TypeId, Vec<EventHandler>>,
    /// ¿Está propagando?
    is_propagating: bool,
    /// Current event (durante propagación)
    current_event: Option<Event>,
}

impl EventBubble {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            handlers: HashMap::new(),
            is_propagating: false,
            current_event: None,
        }
    }

    /// Registra un handler para un tipo de evento
    pub fn add_handler<E: EventTypeTrait + Clone + 'static>(&mut self, handler: EventHandler) {
        let type_id = TypeId::of::<E>();
        self.handlers
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Emite un evento a la cola
    pub fn emit(&mut self, mut event: Event) {
        event.target = None;
        self.queue.push_back(event);
    }

    /// Procesa un solo evento
    pub fn process_one(&mut self, event: &mut Event) -> bool {
        let type_id = {
            let evt = &*event;
            evt.type_id()
        };
        let handlers = match self.handlers.get(&type_id) {
            Some(h) => h.clone(),
            None => return false,
        };
        self.current_event = Some(event.clone());

        // Get handlers for this event type
        // Propagation: Capture phase
        event.phase = EventPhase::Capture;
        for handler in &handlers {
            let handler = handler.read().unwrap();
            if handler(event) {
                if event.handled {
                    return true;
                }
            }
        }

        // Target phase
        event.phase = EventPhase::Target;
        for handler in &handlers {
            let handler = handler.read().unwrap();
            if handler(event) {
                if event.handled {
                    return true;
                }
            }
        }
        // Bubble phase
        event.phase = EventPhase::Bubble;
        for handler in handlers {
            let handler = handler.read().unwrap();
            if handler(event) {
                if event.handled {
                    return true;
                }
            }
        }

        false
    }

    /// Procesa todos los eventos en cola
    pub fn process_all(&mut self) {
        while let Some(mut event) = self.queue.pop_front() {
            self.process_one(&mut event);
        }
    }

    /// Procesa eventos hasta que la cola esté vacía o un handler lo maneje
    pub fn poll_events(&mut self) -> bool {
        let mut handled = false;
        while let Some(mut event) = self.queue.pop_front() {
            if self.process_one(&mut event) {
                handled = true;
                // Clear remaining events if handled
                self.queue.clear();
                break;
            }
        }
        handled
    }

    /// Despatcha un evento directamente
    pub fn dispatch_event(&mut self, mut event: Event) -> bool {
        event.target = None;
        self.process_one(&mut event)
    }
}

impl Event {
    fn type_id(&self) -> TypeId {
        match &self.event_type {
            EventType::Keyboard(_) => TypeId::of::<KeyboardEvent>(),
            EventType::Mouse(_) => TypeId::of::<MouseEvent>(),
            EventType::Resize(_) => TypeId::of::<ResizeEvent>(),
            EventType::Focus(_) => TypeId::of::<FocusEvent>(),
            EventType::Scroll(_) => TypeId::of::<ScrollEvent>(),
            EventType::Touch(_) => TypeId::of::<TouchEvent>(),
            EventType::Custom => TypeId::of::<Event>(),
        }
    }
}

// Helper functions

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Crea un evento de botón
pub fn create_button_event(x: f32, y: f32, button: MouseButton, pressed: bool) -> Event {
    Event::mouse(
        if pressed {
            MouseEventKind::Press
        } else {
            MouseEventKind::Release
        },
        x,
        y,
        button,
        Modifiers::default(),
    )
}

/// Crea un evento de teclado
pub fn create_key_event(
    key_code: KeyCode,
    key_char: Option<char>,
    modifiers: Modifiers,
    repeat: bool,
) -> Event {
    Event::keyboard(key_code, key_char, modifiers, repeat)
}

/// Crea un evento de ratón
pub fn create_mouse_event(kind: MouseEventKind, x: f32, y: f32, button: MouseButton) -> Event {
    Event::mouse(kind, x, y, button, Modifiers::default())
}

/// Crea un evento de resize
pub fn create_resize_event(width: u32, height: u32) -> Event {
    Event::resize(width, height)
}

/// Process events from event queue
pub fn poll_events(bubble: &mut EventBubble) -> bool {
    bubble.poll_events()
}

/// Dispatch a single event
pub fn dispatch_event(bubble: &mut EventBubble, event: Event) -> bool {
    bubble.dispatch_event(event)
}

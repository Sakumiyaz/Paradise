//! # Layout - Layout Engine
//!
//! Motor de layout 100% original: flex, grid, stack.
//! Sin dependencias de bibliotecas CSS/layout externas.
//!
//! ## Algoritmos
//!
//! 1. **Flex Layout**: Algoritmo similar a CSS Flexbox
//! 2. **Grid Layout**: Grid de filas y columnas
//! 3. **Stack Layout**: Apilamiento vertical/horizontal
//!
//! ## Conceptos
//!
//! - Constraint: Restricciones de tamaño
//! - FlexParams: Parámetros flex (grow, shrink, basis)
//! - GridParams: Parámetros de grid (rows, cols, gap)
//! - StackParams: Parámetros de stack (direction, spacing)

#![allow(dead_code)]

use crate::ui_interface::{Rect, Size};

// ============================================================================
// TIPOS BASE
// ============================================================================

/// Tipo de layout
#[derive(Clone, Debug, PartialEq)]
pub enum LayoutType {
    Flex,
    Grid,
    Stack,
}

/// Dirección del layout
#[derive(Clone, Debug, PartialEq)]
pub enum LayoutDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Modo de wrap
#[derive(Clone, Debug, PartialEq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Justify content
#[derive(Clone, Debug, PartialEq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Align items
#[derive(Clone, Debug, PartialEq)]
pub enum AlignItems {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

/// Align content
#[derive(Clone, Debug, PartialEq)]
pub enum AlignContent {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
}

/// Parámetros de layout flex
#[derive(Clone, Debug)]
pub struct FlexParams {
    pub grow: f32,
    pub shrink: f32,
    pub basis: f32,
    pub align_self: Option<AlignItems>,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub direction: LayoutDirection,
    pub gap: f32,
}

impl Default for FlexParams {
    fn default() -> Self {
        Self {
            grow: 0.0,
            shrink: 1.0,
            basis: 0.0,
            align_self: None,
            flex_wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            align_content: AlignContent::Stretch,
            direction: LayoutDirection::Row,
            gap: 0.0,
        }
    }
}

/// Parámetros de layout grid
#[derive(Clone, Debug)]
pub struct GridParams {
    pub rows: u32,
    pub cols: u32,
    pub row_gap: f32,
    pub col_gap: f32,
    pub row_height: f32,
    pub col_width: f32,
    pub justify_items: AlignItems,
    pub align_items: AlignItems,
}

impl Default for GridParams {
    fn default() -> Self {
        Self {
            rows: 1,
            cols: 1,
            row_gap: 0.0,
            col_gap: 0.0,
            row_height: 0.0,
            col_width: 0.0,
            justify_items: AlignItems::Stretch,
            align_items: AlignItems::Stretch,
        }
    }
}

/// Parámetros de layout stack
#[derive(Clone, Debug)]
pub struct StackParams {
    pub direction: LayoutDirection,
    pub spacing: f32,
    pub alignment: AlignItems,
}

impl Default for StackParams {
    fn default() -> Self {
        Self {
            direction: LayoutDirection::Column,
            spacing: 0.0,
            alignment: AlignItems::Center,
        }
    }
}

/// Nodo de layout
#[derive(Clone, Debug)]
pub struct LayoutNode {
    /// ID del widget asociado
    pub widget_id: u64,
    /// Tipo de layout
    pub layout_type: LayoutType,
    /// Bounds calculados
    pub bounds: Rect,
    /// Restricciones
    pub constraint: Constraint,
    /// Parámetros flex (si es flex)
    pub flex_params: Option<FlexParams>,
    /// Parámetros grid (si es grid)
    pub grid_params: Option<GridParams>,
    /// Parámetros stack (si es stack)
    pub stack_params: Option<StackParams>,
    /// Children IDs
    pub children: Vec<u64>,
    /// Tamaño hint
    pub size_hint: Size,
    /// ¿Necesita recalcular layout?
    pub dirty: bool,
}

impl LayoutNode {
    pub fn new(widget_id: u64, layout_type: LayoutType) -> Self {
        Self {
            widget_id,
            layout_type,
            bounds: Rect::zero(),
            constraint: Constraint::new(),
            flex_params: None,
            grid_params: None,
            stack_params: None,
            children: Vec::new(),
            size_hint: Size::zero(),
            dirty: true,
        }
    }

    pub fn with_flex(mut self, params: FlexParams) -> Self {
        self.layout_type = LayoutType::Flex;
        self.flex_params = Some(params);
        self
    }

    pub fn with_grid(mut self, params: GridParams) -> Self {
        self.layout_type = LayoutType::Grid;
        self.grid_params = Some(params);
        self
    }

    pub fn with_stack(mut self, params: StackParams) -> Self {
        self.layout_type = LayoutType::Stack;
        self.stack_params = Some(params);
        self
    }
}

// ============================================================================
// CONSTRAINT
// ============================================================================

/// Restricciones de layout
#[derive(Clone, Debug, Default)]
pub struct Constraint {
    /// Ancho mínimo
    pub min_width: f32,
    /// Ancho máximo
    pub max_width: f32,
    /// Alto mínimo
    pub min_height: f32,
    /// Alto máximo
    pub max_height: f32,
}

impl Constraint {
    pub fn new() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::MAX,
            min_height: 0.0,
            max_height: f32::MAX,
        }
    }

    pub fn with_width(mut self, min: f32, max: f32) -> Self {
        self.min_width = min;
        self.max_width = max;
        self
    }

    pub fn with_height(mut self, min: f32, max: f32) -> Self {
        self.min_height = min;
        self.max_height = max;
        self
    }

    pub fn with_size(mut self, width: (f32, f32), height: (f32, f32)) -> Self {
        self.min_width = width.0;
        self.max_width = width.1;
        self.min_height = height.0;
        self.max_height = height.1;
        self
    }

    /// Clamp a size within constraints
    pub fn clamp(&self, size: Size) -> Size {
        Size::new(
            size.width.max(self.min_width).min(self.max_width),
            size.height.max(self.min_height).min(self.max_height),
        )
    }

    /// Verifica si un tamaño es válido
    pub fn is_valid(&self) -> bool {
        self.min_width <= self.max_width && self.min_height <= self.max_height
    }
}

// ============================================================================
// LAYOUT ENGINE
// ============================================================================

/// Motor de layout
pub struct LayoutEngine {
    /// Nodos de layout
    nodes: Vec<LayoutNode>,
    /// Dirty flag
    dirty: bool,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            dirty: true,
        }
    }

    /// Añade un nodo al engine
    pub fn add_node(&mut self, node: LayoutNode) -> usize {
        self.dirty = true;
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// Obtiene un nodo
    pub fn get_node(&self, index: usize) -> Option<&LayoutNode> {
        self.nodes.get(index)
    }

    /// Obtiene un nodo mutable
    pub fn get_node_mut(&mut self, index: usize) -> Option<&mut LayoutNode> {
        self.dirty = true;
        self.nodes.get_mut(index)
    }

    /// Establece dirty flag
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    /// Calcula el layout completo
    pub fn compute_layout(&mut self, container_bounds: Rect) {
        if !self.dirty && self.nodes.is_empty() {
            return;
        }

        // Find root node (node with no parent in our list)
        let root_index = self.find_root_index();

        if let Some(idx) = root_index {
            self.compute_node_layout(idx, container_bounds);
        }

        self.dirty = false;
    }

    fn find_root_index(&self) -> Option<usize> {
        // Asumimos que el primer nodo es el root
        self.nodes.first().map(|_| 0)
    }

    fn compute_node_layout(&mut self, index: usize, bounds: Rect) {
        let Some(node) = self.nodes.get(index) else {
            return;
        };

        match node.layout_type {
            LayoutType::Flex => {
                self.compute_flex_layout(index, bounds);
            }
            LayoutType::Grid => {
                self.compute_grid_layout(index, bounds);
            }
            LayoutType::Stack => {
                self.compute_stack_layout(index, bounds);
            }
        }
    }

    fn compute_flex_layout(&mut self, index: usize, container: Rect) {
        let children: Vec<usize> = {
            let node = self.nodes.get(index).unwrap();
            (0..node.children.len()).collect()
        };

        if children.is_empty() {
            return;
        }

        let flex_direction = {
            let node = self.nodes.get(index).unwrap();
            node.flex_params.as_ref().map(|p| p.direction.clone())
        };
        let flex_gap = {
            let node = self.nodes.get(index).unwrap();
            node.flex_params.as_ref().map(|p| p.gap).unwrap_or(0.0)
        };

        let is_horizontal = matches!(
            flex_direction.unwrap_or(LayoutDirection::Row),
            LayoutDirection::Row | LayoutDirection::RowReverse
        );

        // Calculate total flex grow/shrink
        let total_grow: f32 = children
            .iter()
            .filter_map(|&i| self.nodes.get(i).and_then(|n| n.flex_params.as_ref()))
            .map(|p| p.grow)
            .sum();

        let _total_shrink: f32 = children
            .iter()
            .filter_map(|&i| self.nodes.get(i).and_then(|n| n.flex_params.as_ref()))
            .map(|p| p.shrink)
            .sum();

        // Main axis size
        let container_size = if is_horizontal {
            container.size.width
        } else {
            container.size.height
        };

        // Layout children - collect updates first
        let mut updates = Vec::new();
        let mut offset: f32 = 0.0;
        let mut cross_offset: f32 = 0.0;

        for (_i, &child_idx) in children.iter().enumerate() {
            let child = self.nodes.get(child_idx).unwrap();
            let child_flex = child.flex_params.as_ref();

            // Calculate main size based on flex grow/shrink
            let flex_size = if total_grow > 0.0 && child_flex.map(|p| p.grow > 0.0).unwrap_or(false)
            {
                (container_size * child_flex.unwrap().grow) / total_grow
            } else {
                child.size_hint.width
            };

            // Clamp to constraints
            let size = child.constraint.clamp(Size::new(
                if is_horizontal {
                    flex_size
                } else {
                    container.size.width
                },
                if is_horizontal {
                    container.size.height
                } else {
                    flex_size
                },
            ));

            // Calculate position
            let (x, y) = if is_horizontal {
                (container.origin.x + offset, container.origin.y)
            } else {
                (container.origin.x, container.origin.y + offset)
            };

            updates.push((child_idx, Rect::new(x, y, size.width, size.height)));

            offset += if is_horizontal {
                size.width
            } else {
                size.height
            };
            offset += flex_gap;

            let cross_size: f32 = if is_horizontal {
                size.height
            } else {
                size.width
            };
            cross_offset = cross_offset.max(cross_size);
        }

        // Apply all updates
        for (child_idx, bounds) in updates {
            if let Some(node) = self.nodes.get_mut(child_idx) {
                node.bounds = bounds;
                node.dirty = false;
            }
        }
    }

    fn compute_grid_layout(&mut self, index: usize, container: Rect) {
        let (cols, col_gap, col_width, rows, row_gap, row_height, children) = {
            let node = self.nodes.get(index).unwrap();
            let grid = node.grid_params.as_ref().unwrap();
            (
                grid.cols,
                grid.col_gap,
                grid.col_width,
                grid.rows,
                grid.row_gap,
                grid.row_height,
                node.children.clone(),
            )
        };

        let cell_width = if col_width > 0.0 {
            col_width
        } else {
            (container.size.width - (cols as f32 - 1.0) * col_gap) / cols as f32
        };

        let cell_height = if row_height > 0.0 {
            row_height
        } else {
            (container.size.height - (rows as f32 - 1.0) * row_gap) / rows as f32
        };

        let mut updates = Vec::new();
        for (i, &child_idx) in children.iter().enumerate() {
            let col = (i as u32) % cols;
            let row = (i as u32) / cols;

            let x = container.origin.x + col as f32 * (cell_width + col_gap);
            let y = container.origin.y + row as f32 * (cell_height + row_gap);

            updates.push((child_idx as usize, Rect::new(x, y, cell_width, cell_height)));
        }

        for (child_idx, bounds) in updates {
            if let Some(node) = self.nodes.get_mut(child_idx) {
                node.bounds = bounds;
                node.dirty = false;
            }
        }
    }

    fn compute_stack_layout(&mut self, index: usize, container: Rect) {
        let (direction, spacing, children) = {
            let node = self.nodes.get(index).unwrap();
            let stack = node.stack_params.as_ref().unwrap();
            (
                stack.direction.clone(),
                stack.spacing,
                node.children.clone(),
            )
        };

        let is_vertical = matches!(
            direction,
            LayoutDirection::Column | LayoutDirection::ColumnReverse
        );

        let mut offset = 0.0;
        let mut updates = Vec::new();

        for &child_idx in children.iter() {
            let (size, _child_bounds) = {
                let child = self.nodes.get(child_idx as usize).unwrap();
                (child.size_hint, child.bounds)
            };

            let (x, y) = if is_vertical {
                (container.origin.x, container.origin.y + offset)
            } else {
                (container.origin.x + offset, container.origin.y)
            };

            updates.push((child_idx as usize, Rect::new(x, y, size.width, size.height)));
            offset += if is_vertical { size.height } else { size.width };
            offset += spacing;
        }

        for (child_idx, bounds) in updates {
            if let Some(node) = self.nodes.get_mut(child_idx) {
                node.bounds = bounds;
                node.dirty = false;
            }
        }
    }
}

// ============================================================================
// LAYOUT FUNCTIONS
// ============================================================================

/// Calcula layout completo
pub fn compute_layout(engine: &mut LayoutEngine, container_bounds: Rect) {
    engine.compute_layout(container_bounds);
}

/// Invalida el layout
pub fn invalidate_layout(engine: &mut LayoutEngine) {
    engine.invalidate();
}

/// Layout flex helper
pub fn flex_layout(
    direction: LayoutDirection,
    justify: JustifyContent,
    align: AlignItems,
    gap: f32,
) -> FlexParams {
    FlexParams {
        direction,
        justify_content: justify,
        align_items: align,
        gap,
        ..Default::default()
    }
}

/// Layout grid helper
pub fn grid_layout(rows: u32, cols: u32, row_gap: f32, col_gap: f32) -> GridParams {
    GridParams {
        rows,
        cols,
        row_gap,
        col_gap,
        ..Default::default()
    }
}

/// Layout stack helper
pub fn stack_layout(
    direction: LayoutDirection,
    spacing: f32,
    alignment: AlignItems,
) -> StackParams {
    StackParams {
        direction,
        spacing,
        alignment,
    }
}

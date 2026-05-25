// EDEN GARM GridWorld — Mundo simulado para embodiment.
// 100% Rust puro, 0 LLM, 0 red.
//
// Grid 2D donde EDEN tiene un agente que puede moverse, agarrar y soltar
// objetos. Las acciones tienen consecuencias fisicas observables, lo que
// crea un loop genuino de feedback embodied (vs simulacion abstracta).
//
// Cell types: Empty, Wall, Object, Agent, Goal
// Actions: MoveUp, MoveDown, MoveLeft, MoveRight, Pickup, Drop, Wait

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum CellType {
    Empty,
    Wall,
    Object(u64), // object id
    Goal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Pickup,
    Drop,
    Wait,
}

#[derive(Clone, Debug)]
pub struct ObjectData {
    pub id: u64,
    pub label: String,
    pub mass: f32,
}

#[derive(Clone, Debug)]
pub struct ActionResult {
    pub action: GridAction,
    pub success: bool,
    pub reward: f32,
    pub note: String,
    pub goal_reached: bool,
}

#[derive(Clone, Debug)]
pub struct GridWorld {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<CellType>>,
    pub agent_x: usize,
    pub agent_y: usize,
    pub holding: Option<u64>,
    pub objects: HashMap<u64, ObjectData>,
    pub goal_x: usize,
    pub goal_y: usize,
    pub n_steps: u64,
    pub n_pickups: u64,
    pub n_drops: u64,
    pub n_collisions: u64,
    pub n_goal_reached: u64,
    pub history: Vec<ActionResult>,
    pub max_history: usize,
    pub next_obj_id: u64,
}

impl GridWorld {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![CellType::Empty; width]; height];
        let mut gw = GridWorld {
            width,
            height,
            cells,
            agent_x: 0,
            agent_y: 0,
            holding: None,
            objects: HashMap::new(),
            goal_x: width - 1,
            goal_y: height - 1,
            n_steps: 0,
            n_pickups: 0,
            n_drops: 0,
            n_collisions: 0,
            n_goal_reached: 0,
            history: Vec::new(),
            max_history: 100,
            next_obj_id: 1,
        };
        gw.cells[gw.goal_y][gw.goal_x] = CellType::Goal;
        gw
    }

    /// Initialize a basic scene with walls and a few objects.
    pub fn init_basic_scene(&mut self) {
        // border walls (only top/bottom and side rows)
        for x in 0..self.width {
            self.cells[0][x] = CellType::Wall;
            self.cells[self.height - 1][x] = CellType::Wall;
        }
        for y in 0..self.height {
            self.cells[y][0] = CellType::Wall;
            self.cells[y][self.width - 1] = CellType::Wall;
        }
        // Re-set goal
        self.cells[self.goal_y][self.goal_x] = CellType::Goal;
        // Agent in (1,1)
        self.agent_x = 1;
        self.agent_y = 1;
        self.cells[1][1] = CellType::Empty;
        // Add 3 objects
        for (label, x, y) in [("rock", 3, 2), ("ball", 4, 4), ("box", 2, 5)].iter() {
            let id = self.next_obj_id;
            self.next_obj_id += 1;
            self.objects.insert(
                id,
                ObjectData {
                    id,
                    label: label.to_string(),
                    mass: 1.0,
                },
            );
            if *y < self.height && *x < self.width {
                self.cells[*y][*x] = CellType::Object(id);
            }
        }
    }

    /// Take an action; returns the result.
    pub fn step(&mut self, action: GridAction) -> ActionResult {
        self.n_steps += 1;
        let (dx, dy): (i32, i32) = match action {
            GridAction::MoveUp => (0, -1),
            GridAction::MoveDown => (0, 1),
            GridAction::MoveLeft => (-1, 0),
            GridAction::MoveRight => (1, 0),
            _ => (0, 0),
        };
        let result = match action.clone() {
            GridAction::MoveUp
            | GridAction::MoveDown
            | GridAction::MoveLeft
            | GridAction::MoveRight => {
                let nx = self.agent_x as i32 + dx;
                let ny = self.agent_y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
                    self.n_collisions += 1;
                    ActionResult {
                        action: action.clone(),
                        success: false,
                        reward: -0.1,
                        note: "out of bounds".to_string(),
                        goal_reached: false,
                    }
                } else {
                    let target = &self.cells[ny as usize][nx as usize];
                    match target {
                        CellType::Wall => {
                            self.n_collisions += 1;
                            ActionResult {
                                action: action.clone(),
                                success: false,
                                reward: -0.1,
                                note: "hit wall".to_string(),
                                goal_reached: false,
                            }
                        }
                        CellType::Object(_) => {
                            // can't walk through object; could push (simplified: blocked)
                            self.n_collisions += 1;
                            ActionResult {
                                action: action.clone(),
                                success: false,
                                reward: -0.05,
                                note: "blocked by object".to_string(),
                                goal_reached: false,
                            }
                        }
                        CellType::Goal => {
                            self.agent_x = nx as usize;
                            self.agent_y = ny as usize;
                            self.n_goal_reached += 1;
                            ActionResult {
                                action: action.clone(),
                                success: true,
                                reward: 1.0,
                                note: "reached goal!".to_string(),
                                goal_reached: true,
                            }
                        }
                        CellType::Empty => {
                            self.agent_x = nx as usize;
                            self.agent_y = ny as usize;
                            ActionResult {
                                action: action.clone(),
                                success: true,
                                reward: -0.01,
                                note: "moved".to_string(),
                                goal_reached: false,
                            }
                        }
                    }
                }
            }
            GridAction::Pickup => {
                if self.holding.is_some() {
                    ActionResult {
                        action: action.clone(),
                        success: false,
                        reward: -0.05,
                        note: "already holding".to_string(),
                        goal_reached: false,
                    }
                } else {
                    // Look for adjacent object
                    let mut found = None;
                    for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)].iter() {
                        let cx = self.agent_x as i32 + dx;
                        let cy = self.agent_y as i32 + dy;
                        if cx >= 0 && cy >= 0 && cx < self.width as i32 && cy < self.height as i32 {
                            if let CellType::Object(oid) = &self.cells[cy as usize][cx as usize] {
                                found = Some((*oid, cx as usize, cy as usize));
                                break;
                            }
                        }
                    }
                    if let Some((oid, cx, cy)) = found {
                        self.holding = Some(oid);
                        self.cells[cy][cx] = CellType::Empty;
                        self.n_pickups += 1;
                        let label = self
                            .objects
                            .get(&oid)
                            .map(|o| o.label.clone())
                            .unwrap_or_default();
                        ActionResult {
                            action: action.clone(),
                            success: true,
                            reward: 0.2,
                            note: format!("picked up {}", label),
                            goal_reached: false,
                        }
                    } else {
                        ActionResult {
                            action: action.clone(),
                            success: false,
                            reward: -0.05,
                            note: "no object adjacent".to_string(),
                            goal_reached: false,
                        }
                    }
                }
            }
            GridAction::Drop => {
                if let Some(oid) = self.holding {
                    // Drop in front of agent (any empty adjacent cell)
                    let mut placed = false;
                    for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)].iter() {
                        let cx = self.agent_x as i32 + dx;
                        let cy = self.agent_y as i32 + dy;
                        if cx >= 0 && cy >= 0 && cx < self.width as i32 && cy < self.height as i32 {
                            if self.cells[cy as usize][cx as usize] == CellType::Empty {
                                self.cells[cy as usize][cx as usize] = CellType::Object(oid);
                                placed = true;
                                break;
                            }
                        }
                    }
                    if placed {
                        self.holding = None;
                        self.n_drops += 1;
                        ActionResult {
                            action: action.clone(),
                            success: true,
                            reward: 0.1,
                            note: "dropped".to_string(),
                            goal_reached: false,
                        }
                    } else {
                        ActionResult {
                            action: action.clone(),
                            success: false,
                            reward: -0.05,
                            note: "no empty space".to_string(),
                            goal_reached: false,
                        }
                    }
                } else {
                    ActionResult {
                        action: action.clone(),
                        success: false,
                        reward: -0.05,
                        note: "not holding anything".to_string(),
                        goal_reached: false,
                    }
                }
            }
            GridAction::Wait => ActionResult {
                action: action.clone(),
                success: true,
                reward: 0.0,
                note: "waited".to_string(),
                goal_reached: false,
            },
        };
        self.history.push(result.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        result
    }

    /// Render the grid as ASCII.
    pub fn render(&self) -> String {
        let mut out = format!(
            "GridWorld {}x{} | agent=({},{}) | holding={:?}\n",
            self.width, self.height, self.agent_x, self.agent_y, self.holding
        );
        for y in 0..self.height {
            for x in 0..self.width {
                if x == self.agent_x && y == self.agent_y {
                    out.push('A');
                } else {
                    let ch = match &self.cells[y][x] {
                        CellType::Empty => '.',
                        CellType::Wall => '#',
                        CellType::Object(_) => 'O',
                        CellType::Goal => 'G',
                    };
                    out.push(ch);
                }
            }
            out.push('\n');
        }
        out
    }

    pub fn status(&self) -> String {
        format!(
            "GridWorld | {}x{} | steps={} pickups={} drops={} collisions={} goal_reached={}",
            self.width,
            self.height,
            self.n_steps,
            self.n_pickups,
            self.n_drops,
            self.n_collisions,
            self.n_goal_reached,
        )
    }
}

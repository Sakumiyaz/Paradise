// EDEN GARM Program — Lenguaje de control interno
// El transformer genera texto en pseudo-ensamblador; este parser lo convierte
// a instrucciones ejecutables por el sistema.
//
// Ejemplo de programa generado:
//   INVOKE planner
//   TOOLCALL calculator 2+2
//   SETGOAL explore 0.8
//   TRAIN_TF 5
//   READ_BUS motivation
//   WAIT 10
//   HALT

use crate::eden_garm::capabilities::GarmCapability;

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Halt,
    /// Invoke a cognitive capability (e.g., planner, morphogenesis)
    Invoke(GarmCapability),
    /// Call a tool by name with a single string argument
    ToolCall {
        name: String,
        arg: String,
    },
    /// Set a goal with label and priority
    SetGoal {
        label: String,
        priority: f32,
    },
    /// Train transformer on N sentences from corpus
    TrainTf {
        n_sentences: usize,
    },
    /// Read a slot from the unified bus
    ReadBus {
        slot: String,
    },
    /// Write a value to a bus slot (value encoded as string)
    WriteBus {
        slot: String,
        value: String,
    },
    /// Wait N ticks
    Wait {
        ticks: u64,
    },
    /// Conditional: if bus slot activity > threshold, jump N instructions forward
    If {
        slot: String,
        threshold: f32,
        skip: usize,
    },
    /// Log a message
    Log {
        message: String,
    },
}

#[derive(Clone, Debug)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub pc: usize, // program counter
    pub source_text: String,
}

impl Program {
    pub fn new() -> Self {
        Program {
            instructions: Vec::new(),
            pc: 0,
            source_text: String::new(),
        }
    }

    pub fn from_text(text: &str) -> Self {
        let instructions = parse_program(text);
        Program {
            instructions,
            pc: 0,
            source_text: text.to_string(),
        }
    }

    pub fn is_halted(&self) -> bool {
        self.pc >= self.instructions.len()
    }

    pub fn current(&self) -> Option<&Instruction> {
        self.instructions.get(self.pc)
    }

    pub fn step(&mut self) {
        self.pc += 1;
    }

    pub fn jump(&mut self, offset: usize) {
        self.pc += offset;
    }

    pub fn reset(&mut self) {
        self.pc = 0;
    }
}

/// Parse a simple assembly-like language into instructions.
/// Zero LLM. Pure string matching.
pub fn parse_program(text: &str) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let instr = match parts[0].to_uppercase().as_str() {
            "HALT" => Some(Instruction::Halt),
            "INVOKE" => {
                if parts.len() >= 2 {
                    let cap = parse_capability(parts[1]);
                    Some(Instruction::Invoke(cap))
                } else {
                    None
                }
            }
            "TOOLCALL" => {
                if parts.len() >= 3 {
                    Some(Instruction::ToolCall {
                        name: parts[1].to_string(),
                        arg: parts[2..].join(" "),
                    })
                } else {
                    None
                }
            }
            "SETGOAL" => {
                if parts.len() >= 3 {
                    if let Ok(priority) = parts[2].parse::<f32>() {
                        Some(Instruction::SetGoal {
                            label: parts[1].to_string(),
                            priority,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "TRAIN_TF" | "TRAINTF" => {
                if parts.len() >= 2 {
                    if let Ok(n) = parts[1].parse::<usize>() {
                        Some(Instruction::TrainTf { n_sentences: n })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "READ_BUS" | "READBUS" => {
                if parts.len() >= 2 {
                    Some(Instruction::ReadBus {
                        slot: parts[1].to_string(),
                    })
                } else {
                    None
                }
            }
            "WRITE_BUS" | "WRITEBUS" => {
                if parts.len() >= 3 {
                    Some(Instruction::WriteBus {
                        slot: parts[1].to_string(),
                        value: parts[2..].join(" "),
                    })
                } else {
                    None
                }
            }
            "WAIT" => {
                if parts.len() >= 2 {
                    if let Ok(ticks) = parts[1].parse::<u64>() {
                        Some(Instruction::Wait { ticks })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "IF" => {
                if parts.len() >= 4 {
                    if let Ok(threshold) = parts[2].parse::<f32>() {
                        if let Ok(skip) = parts[3].parse::<usize>() {
                            Some(Instruction::If {
                                slot: parts[1].to_string(),
                                threshold,
                                skip,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "LOG" => Some(Instruction::Log {
                message: parts[1..].join(" "),
            }),
            _ => None,
        };
        if let Some(i) = instr {
            instructions.push(i);
        }
    }
    instructions
}

fn parse_capability(s: &str) -> GarmCapability {
    match s.to_lowercase().as_str() {
        "planner" => GarmCapability::PlannerSystem,
        "morphogenesis" => GarmCapability::Morphogenesis,
        "selfmodel" => GarmCapability::SelfModel,
        "metacognition" => GarmCapability::Metacognition,
        "causality" => GarmCapability::Causality,
        "motivation" => GarmCapability::Motivation,
        "mood" => GarmCapability::Mood,
        "worldmodel" => GarmCapability::WorldModel,
        "autonomy" => GarmCapability::Autonomy,
        "tools" => GarmCapability::ToolCalling,
        "bigtransformer" => GarmCapability::BigTransformer,
        "vision" => GarmCapability::Vision,
        _ => GarmCapability::NaturalLanguage,
    }
}

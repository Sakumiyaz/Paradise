//! IPC - Inter-Process Communication via Unix Datagram Sockets
//! Uses only std library (Unix-specific)
//!
//! ## Protocolo Extendido
//!
//! ### Eventos (Rust → Python)
//! - `EventNacioAuton`: Un Auton nació
//! - `EventMurioAuton`: Un Auton murió
//! - `EventNomoFormado`: Se detectó un Nomos
//! - `EventBifurcacion`: Un Auton se bifurc贸
//! - `EventEcosistemaState`: Estado actual del ecosistema
//!
//! ### Comandos (Python → Rust)
//! - `CmdInyectarEnergon`: Inyectar energía en coordenadas
//! - `CmdAumentarEscoria`: Aumentar escoria en radio
//! - `CmdForzarBifurcacion`: Forzar bifurcación de Auton
//! - `CmdEliminarAuton`: Eliminar un Auton
//! - `CmdPausarSimulacion`: Pausar/reanudar simulación

#![allow(dead_code)]

pub mod channel;
pub mod command;
pub mod message;
pub mod socket;

pub use channel::{Channel, ChannelEndpoint};
pub use command::{Comando, EdenCommand, EdenCommandBuilder, Evento};
pub use message::Message;
pub use socket::{SocketAddr, UnixDatagram};

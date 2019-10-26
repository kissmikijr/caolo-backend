//! The public API of the game.
//! Wraps the Engine functions in a more managable API
//!
//! (De)Serialization between WASM and the Engine is done via [MessagePack](https://msgpack.org/index.html)
//! When using the API for writing bots this does not effect you.
//!
#[macro_use]
extern crate serde_derive;

pub mod bots;
pub mod pathfinding;
pub mod point;
pub mod resources;
pub mod structures;
pub mod terrain;
pub mod user;

pub use cao_lang::prelude::{AstNode, CompilationUnit, CompiledProgram};

pub type EntityId = u64;
pub type UserId = uuid::Uuid;

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ScriptId(pub uuid::Uuid);

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(i32)]
pub enum OperationResult {
    Ok = 0,
    NotOwner = -1,
    InvalidInput = -2,
    OperationFailed = -3,
    NotInRange = -4,
    InvalidTarget = -5,
    Empty = -6,
    Full = -7,
}

impl From<i32> for OperationResult {
    fn from(i: i32) -> OperationResult {
        match i {
            0 => OperationResult::Ok,
            -1 => OperationResult::NotOwner,
            -2 => OperationResult::InvalidInput,
            -3 => OperationResult::OperationFailed,
            -4 => OperationResult::NotInRange,
            -5 => OperationResult::InvalidTarget,
            -6 => OperationResult::Empty,
            -7 => OperationResult::Full,
            _ => panic!("Got an unexpected return code {}", i),
        }
    }
}

impl cao_lang::traits::AutoByteEncodeProperties for OperationResult {}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Script {
    pub compiled: Option<CompiledProgram>,
    pub script: CompilationUnit,
}

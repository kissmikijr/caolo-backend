//! Structs intended to be used as table indices.
//!
use crate::tables::SerialId;
use cao_lang::{prelude::Scalar, traits::AutoByteEncodeProperties};
use serde_derive::Serialize;
use std::convert::TryFrom;

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize)]
pub struct EntityTime(pub EntityId, pub u64);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize)]
pub struct EntityId(pub u32);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize)]
pub struct ScriptId(pub uuid::Uuid);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize)]
pub struct UserId(pub uuid::Uuid);

impl SerialId for EntityId {
    fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl AutoByteEncodeProperties for EntityId {}
impl TryFrom<Scalar> for EntityId {
    type Error = Scalar;
    fn try_from(s: Scalar) -> Result<EntityId, Scalar> {
        match s {
            Scalar::Integer(i) => {
                if i < 0 {
                    return Err(s);
                }
                return Ok(EntityId(i as u32));
            }
            _ => Err(s),
        }
    }
}

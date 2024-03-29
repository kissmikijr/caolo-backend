//! Represents a bot's intent to move to a new location.
//! Currently only bots are allowed to move!
//!
use crate::components::{self, EntityComponent, PositionComponent};
use crate::indices::{EntityId, UserId, WorldPosition};
use crate::scripting_api::OperationResult;
use crate::storage::views::View;
use crate::tables::traits::Table;
use crate::terrain;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MoveIntent {
    pub bot: EntityId,
    pub position: WorldPosition,
}

type CheckInput<'a> = (
    View<'a, EntityId, components::OwnedEntity>,
    View<'a, EntityId, PositionComponent>,
    View<'a, EntityId, components::Bot>,
    View<'a, WorldPosition, components::TerrainComponent>,
    View<'a, WorldPosition, EntityComponent>,
);

pub fn check_move_intent(
    intent: &MoveIntent,
    user_id: UserId,
    (owner_ids, positions, bots, terrain, entity_positions): CheckInput,
) -> OperationResult {
    let id = intent.bot;
    match bots.get(id) {
        Some(_) => {
            let owner_id = owner_ids.get(id);
            if owner_id.map(|id| id.owner_id != user_id).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
        }
        None => return OperationResult::InvalidInput,
    };

    let pos = match positions.get(id) {
        Some(pos) => pos,
        None => {
            debug!("Bot has no position");
            return OperationResult::InvalidInput;
        }
    };

    // TODO: bot speed component?
    if 1 < pos.0.pos.hex_distance(intent.position.pos) || pos.0.room != intent.position.room {
        trace!(
            "Bot move target {:?} is out of range of bot position {:?} and velocity {:?}",
            intent.position,
            pos,
            1
        );
        return OperationResult::InvalidInput;
    }

    if let Some(components::TerrainComponent(terrain::TileTerrainType::Wall)) =
        terrain.get(intent.position)
    {
        debug!("Position is occupied by terrain");
        return OperationResult::InvalidInput;
    }
    if let Some(entity) = entity_positions.get(intent.position) {
        debug!("Position is occupied by another entity {:?}", entity);
        return OperationResult::InvalidInput;
    }
    OperationResult::Ok
}

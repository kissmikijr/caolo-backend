use super::*;
use crate::model::{self, EntityComponent, PositionComponent};
use crate::prelude::*;
use caolo_api::OperationResult;

#[derive(Debug, Clone)]
pub struct MoveIntent {
    pub bot: EntityId,
    pub position: Point,
}

impl MoveIntent {
    pub fn execute(&self, storage: &mut Storage) -> IntentResult {
        debug!("Moving bot[{:?}] to {:?}", self.bot, self.position);

        let table = storage.point_table::<EntityComponent>();

        if storage
            .entity_table::<model::Bot>()
            .get_by_id(&self.bot)
            .is_none()
        {
            debug!("Bot by id {:?} does not exist", self.bot);
            return Err("Bot not found".into());
        }

        if table.get_by_id(&self.position).is_some() {
            debug!("Occupied {:?} ", self);
            return Err("Occupied".into());
        }
        if !table.intersects(&self.position) {
            debug!("PositionTable insert failed {:?}", self);
            return Err("Out of bounds".into());
        }

        let table = storage.entity_table_mut::<PositionComponent>();
        table.insert(self.bot, PositionComponent(self.position));

        debug!("Move successful");

        Ok(())
    }
}

pub fn check_move_intent(
    intent: &caolo_api::bots::MoveIntent,
    userid: model::UserId,
    storage: &crate::storage::Storage,
) -> OperationResult {
    let bots = storage.entity_table::<model::Bot>();
    let terrain = storage.point_table::<model::TerrainComponent>();

    let id = model::EntityId(intent.id);
    match bots.get_by_id(&id) {
        Some(_) => {
            let owner_id = storage.entity_table::<model::OwnedEntity>().get_by_id(&id);
            if owner_id.map(|id| id.owner_id != userid).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
        }
        None => return OperationResult::InvalidInput,
    };

    let pos = match storage.entity_table::<PositionComponent>().get_by_id(&id) {
        Some(pos) => pos,
        None => {
            debug!("Bot has no position");
            return OperationResult::InvalidInput;
        }
    };

    // TODO: bot speed component?
    if 1 < pos.0.hex_distance(intent.position) {
        return OperationResult::InvalidInput;
    }

    match terrain.get_by_id(&intent.position) {
        Some(model::TerrainComponent(model::TileTerrainType::Wall)) => {
            OperationResult::InvalidInput
        }
        _ => OperationResult::Ok,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Bot, EntityComponent, Point, PositionComponent};
    use crate::storage::Storage;
    use crate::tables::{BTreeTable, QuadtreeTable};

    #[test]
    fn test_move_intent_fails_if_node_is_occupied() {
        let mut storage = Storage::new();
        storage.add_entity_table::<Bot>(BTreeTable::new());
        storage.add_entity_table::<PositionComponent>(BTreeTable::new());
        storage.add_point_table::<EntityComponent>(QuadtreeTable::new(Point::new(0, 0), 30, 8));

        let id = storage.insert_entity();

        storage.entity_table_mut::<Bot>().insert(id, Bot {});

        storage
            .entity_table_mut::<PositionComponent>()
            .insert(id, PositionComponent(Point::new(12, 13)));

        let intent = MoveIntent {
            bot: EntityId(69),
            position: Point::new(42, 42),
        };

        intent
            .execute(&mut storage)
            .expect_err("Expected move to fail");
    }
}

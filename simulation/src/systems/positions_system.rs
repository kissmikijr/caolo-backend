use super::System;
use crate::model::{self, EntityId, Point};
use crate::storage::views::{UnsafeView, View};

pub struct PositionSystem;

impl<'a> System<'a> for PositionSystem {
    type Mut = UnsafeView<Point, model::EntityComponent>;
    type Const = View<'a, EntityId, model::PositionComponent>;

    /// Reset the entity positions table
    fn update(&mut self, mut position_entities: Self::Mut, positions: Self::Const) {
        debug!("update positions system called");

        unsafe {
            position_entities.as_mut().clear();

            positions
                .iter()
                .map(|(id, pos)| (pos.0, model::EntityComponent(id)))
                .for_each(|(point, entity)| {
                    position_entities.as_mut().insert(point, entity);
                });
        }

        debug!("update positions system done");
    }
}

use crate::components::{Bot, EntityComponent, PositionComponent, TerrainComponent};
use crate::indices::{EmptyKey, EntityId, WorldPosition};
use crate::intents::{Intents, MoveIntent};
use crate::profile;
use crate::storage::views::{UnsafeView, UnwrapViewMut, View};
use crate::tables::traits::Table;
use tracing::trace;

type Mut = (
    UnsafeView<EntityId, PositionComponent>,
    UnwrapViewMut<EmptyKey, Intents<MoveIntent>>,
);
type Const<'a> = (
    View<'a, EntityId, Bot>,
    View<'a, WorldPosition, EntityComponent>,
    View<'a, WorldPosition, TerrainComponent>,
);

pub fn move_intents_update(
    (mut positions, mut intents): Mut,
    (bots, pos_entities, _terrain): Const,
) {
    profile!(" MoveSystem update");

    pre_process_move_intents(&mut intents.0);
    for intent in intents.iter() {
        trace!("Moving bot[{:?}] to {:?}", intent.bot, intent.position);

        debug_assert!(_terrain
            .at(intent.position)
            .expect("Failed to get the terrain under bot")
            .0
            .is_walkable());

        if bots.get(intent.bot).is_none() {
            trace!("Bot by id {:?} does not exist", intent.bot);
            continue;
        }

        if pos_entities.get(intent.position).is_some() {
            trace!("Occupied {:?} ", intent.position);
            continue;
        }

        positions.insert(intent.bot, PositionComponent(intent.position));

        trace!("Move successful");
    }
}

/// Remove duplicate positions.
/// We assume that there are no duplicated entities
fn pre_process_move_intents(move_intents: &mut Vec<MoveIntent>) {
    profile!("pre_process_move_intents");

    let len = move_intents.len();
    if len < 2 {
        // 0 and 1 long vectors do not have duplicates
        return;
    }
    move_intents.sort_unstable_by_key(|intent| intent.position);
    // move in reverse order because we want to remove invalid intents as we move,
    // swap_remove would change the last position, screwing with the ordering
    for current in (0..=len - 2).rev() {
        let last = current + 1;
        let a = &move_intents[last];
        let b = &move_intents[current];
        if a.position == b.position {
            trace!("Duplicated position in move intents, removing {:?}", a);
            move_intents.swap_remove(last);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Axial;
    use crate::indices::EntityId;
    use crate::indices::WorldPosition;

    #[test]
    fn pre_process_move_intents_removes_last_dupe() {
        let mut intents = vec![
            MoveIntent {
                bot: EntityId::new(42, 0),
                position: WorldPosition {
                    room: Default::default(),
                    pos: Axial::new(42, 69),
                },
            },
            MoveIntent {
                bot: EntityId::new(123, 21),
                position: WorldPosition {
                    room: Default::default(),
                    pos: Axial::new(42, 69),
                },
            },
            MoveIntent {
                bot: EntityId::new(64, 21),
                position: WorldPosition {
                    room: Default::default(),
                    pos: Axial::new(43, 69),
                },
            },
            MoveIntent {
                bot: EntityId::new(69, 123),
                position: WorldPosition {
                    room: Default::default(),
                    pos: Axial::new(42, 69),
                },
            },
        ];

        pre_process_move_intents(&mut intents);
        assert_eq!(intents.len(), 2);
        assert_ne!(intents[0].position, intents[1].position);
    }
}

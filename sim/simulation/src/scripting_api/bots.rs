use super::*;
use crate::{
    components::{self, Resource},
    indices::{EntityId, UserId, WorldPosition},
    intents::{
        check_dropoff_intent, check_melee_intent, check_mine_intent, check_move_intent,
        CachePathIntent, DropoffIntent, MeleeIntent, MineIntent, MoveIntent, MutPathCacheIntent,
        PathCacheIntentAction,
    },
    pathfinding, profile,
    storage::views::FromWorld,
};
use crate::{prelude::World, terrain::TileTerrainType};
use std::convert::{TryFrom, TryInto};
use tracing::{debug, error, trace, warn};

pub fn melee_attack(vm: &mut Vm<ScriptExecutionData>, target: i64) -> Result<(), ExecutionError> {
    profile!("melee-attack");

    let aux = vm.get_aux();
    trace!("melee_attack");

    let target: u64 = target.try_into().map_err(|_| {
        warn!("melee_attack called without a valid target");
        ExecutionError::invalid_argument("melee_attack called without valid a target".to_owned())
    })?;
    let target: EntityId = EntityId::from(target);

    let storage = aux.storage();
    let entity_id = aux.entity_id;
    let user_id = aux.user_id.expect("user_id to be set");

    let intent = MeleeIntent {
        attacker: entity_id,
        defender: target,
    };

    let res = check_melee_intent(&intent, user_id, FromWorld::from_world(storage));

    if let OperationResult::Ok = res {
        vm.get_aux_mut().intents.melee_attack_intent = Some(intent);
    }
    vm.stack_push(res)?;
    Ok(())
}

pub fn unload(
    vm: &mut Vm<ScriptExecutionData>,
    amount: i64,
    ty: Resource,
    target: i64,
) -> Result<(), ExecutionError> {
    profile!("unload");
    let aux = vm.get_aux();

    trace!("unload");

    let amount = TryFrom::try_from(amount).map_err(|e| {
        ExecutionError::invalid_argument(format!("unload called with invalid amount: {}", e))
    })?;
    let target: u64 = target.try_into().map_err(|_| {
        warn!("melee_attack called without a valid target");
        ExecutionError::invalid_argument("melee_attack called without valid a target".to_owned())
    })?;
    let target: EntityId = EntityId::from(target);

    trace!(
        "unload: amount: {} type: {:?} target: {:?}, {}",
        amount,
        ty,
        target,
        aux
    );

    let storage = aux.storage();
    let entity_id = aux.entity_id;
    let user_id = aux.user_id.expect("user_id to be set");

    let dropoff_intent = DropoffIntent {
        bot: entity_id,
        amount,
        ty,
        structure: target,
    };

    let checkresult =
        check_dropoff_intent(&dropoff_intent, user_id, FromWorld::from_world(storage));
    if let OperationResult::Ok = checkresult {
        vm.get_aux_mut().intents.dropoff_intent = Some(dropoff_intent);
    }
    vm.stack_push(checkresult)?;
    Ok(())
}

pub fn mine_resource(vm: &mut Vm<ScriptExecutionData>, target: i64) -> Result<(), ExecutionError> {
    profile!("mine_resource");

    let aux = vm.get_aux();

    let target: u64 = target.try_into().map_err(|_| {
        warn!("melee_attack called without a valid target");
        ExecutionError::invalid_argument("melee_attack called without valid a target".to_owned())
    })?;
    let target: EntityId = EntityId::from(target);

    let s = tracing::trace_span!(
        "mine_resource",
        entity_id = aux.entity_id.to_string().as_str()
    );
    let _e = s.enter();

    trace!("target: {:?}, {}", target, aux);

    let storage = aux.storage();
    let user_id = aux.user_id.expect("user_id to be set");

    let intent = MineIntent {
        bot: aux.entity_id,
        resource: target,
    };

    let checkresult = check_mine_intent(&intent, user_id, FromWorld::from_world(storage));
    vm.stack_push(checkresult)?;
    trace!("result: {:?}", checkresult);
    if let OperationResult::Ok = checkresult {
        vm.get_aux_mut().intents.mine_intent = Some(intent);
    }
    Ok(())
}

pub fn approach_entity(
    vm: &mut Vm<ScriptExecutionData>,
    target: i64,
) -> Result<(), ExecutionError> {
    profile!("approach_entity");

    let aux = vm.get_aux();
    let target: u64 = target.try_into().map_err(|_| {
        warn!("melee_attack called without a valid target");
        ExecutionError::invalid_argument("melee_attack called without valid a target".to_owned())
    })?;
    let target: EntityId = EntityId::from(target);

    trace!("approach_entity: target: {:?}", target);

    let entity = aux.entity_id;
    let storage = aux.storage();
    let user_id = aux.user_id.expect("user_id to be set");

    let targetpos = match storage
        .view::<EntityId, components::PositionComponent>()
        .reborrow()
        .get(target)
    {
        Some(x) => x,
        None => {
            warn!("entity {:?} does not have position component!", target);
            vm.stack_push(OperationResult::InvalidInput)?;
            return Ok(());
        }
    };

    let checkresult = match move_to_pos(entity, targetpos.0, user_id, storage) {
        Ok(Some((move_intent, pop_cache_intent, update_cache_intent))) => {
            let intents = &mut vm.get_aux_mut().intents;
            intents.move_intent = Some(move_intent);
            if let Some(pop_cache_intent) = pop_cache_intent {
                intents.mut_path_cache_intent = Some(pop_cache_intent);
            }
            if let Some(update_cache_intent) = update_cache_intent {
                intents.update_path_cache_intent = Some(update_cache_intent);
            }

            OperationResult::Ok
        }
        Ok(None) => {
            trace!("Bot {:?} approach_entity: nothing to do", entity);
            OperationResult::Ok
        }
        Err(e) => e,
    };
    vm.stack_push(checkresult)?;
    Ok(())
}

pub fn move_bot_to_position(
    vm: &mut Vm<ScriptExecutionData>,
    point: &FieldTable,
) -> Result<(), ExecutionError> {
    profile!("move_bot_to_position");

    let aux = vm.get_aux();

    trace!("move_bot_to_position");

    let entity = aux.entity_id;
    let storage = aux.storage();
    let user_id = aux.user_id.expect("user_id to be set");

    let point: WorldPosition = parse_world_pos(point)?;

    let checkresult = match move_to_pos(entity, point, user_id, storage) {
        Ok(Some((move_intent, pop_cache_intent, update_cache_intent))) => {
            let intents = &mut vm.get_aux_mut().intents;
            intents.move_intent = Some(move_intent);
            if let Some(pop_cache_intent) = pop_cache_intent {
                intents.mut_path_cache_intent = Some(pop_cache_intent);
            }
            if let Some(update_cache_intent) = update_cache_intent {
                intents.update_path_cache_intent = Some(update_cache_intent);
            }
            OperationResult::Ok
        }
        Ok(None) => {
            trace!("{:?} move_to_pos nothing to do", entity);
            OperationResult::Ok
        }
        Err(e) => e,
    };
    vm.stack_push(checkresult)?;
    Ok(())
}

type MoveToPosIntent = (
    MoveIntent,
    Option<MutPathCacheIntent>,
    Option<CachePathIntent>,
);

fn move_to_pos(
    bot: EntityId,
    to: WorldPosition,
    user_id: UserId,
    storage: &World,
) -> Result<Option<MoveToPosIntent>, OperationResult> {
    use crate::prelude::*;

    profile!("move_to_pos");

    let botpos = storage
        .view::<EntityId, components::PositionComponent>()
        .reborrow()
        .get(bot)
        .ok_or_else(|| {
            warn!("entity does not have position component!");
            OperationResult::InvalidInput
        })?;

    // attempt to use the cached path
    // which requires non-empty cache with a valid next step
    match storage
        .view::<EntityId, PathCacheComponent>()
        .reborrow()
        .get(bot)
    {
        Some(cache) if cache.target == to => {
            if let Some(position) = cache.path.last().cloned() {
                let intent = MoveIntent {
                    bot,
                    position: WorldPosition {
                        room: botpos.0.room,
                        pos: position.0,
                    },
                };
                if let OperationResult::Ok =
                    check_move_intent(&intent, user_id, FromWorld::from_world(storage))
                {
                    trace!("Bot {:?} path cache hit", bot);
                    let result = (
                        intent,
                        Some(MutPathCacheIntent {
                            bot,
                            action: PathCacheIntentAction::Pop,
                        }),
                        None,
                    );
                    return Ok(Some(result));
                }
            }
        }
        _ => {}
    }
    trace!("Bot path cache miss");

    let conf = UnwrapView::<ConfigKey, GameConfig>::from_world(storage);
    let max_pathfinding_iter = conf.path_finding_limit;

    let mut path = Vec::with_capacity(max_pathfinding_iter as usize);
    let mut next_room = None;
    if let Err(e) = pathfinding::find_path(
        botpos.0,
        to,
        1,
        FromWorld::from_world(storage),
        max_pathfinding_iter,
        &mut path,
        &mut next_room,
    ) {
        trace!("pathfinding failed {:?}", e);
        return Err(OperationResult::InvalidTarget);
    }

    match path.pop() {
        Some(position) => {
            let intent = MoveIntent {
                bot,
                position: WorldPosition {
                    room: botpos.0.room,
                    pos: position.0,
                },
            };

            let checkresult = check_move_intent(&intent, user_id, FromWorld::from_world(storage));
            match checkresult {
                OperationResult::Ok => {
                    let cache_intent = if !path.is_empty() {
                        // skip >= 0
                        let skip = path.len().max(PATH_CACHE_LEN) - PATH_CACHE_LEN;

                        let cache_intent = CachePathIntent {
                            bot,
                            cache: PathCacheComponent {
                                target: to,
                                path: path.into_iter().skip(skip).take(PATH_CACHE_LEN).collect(),
                            },
                        };
                        Some(cache_intent)
                    } else {
                        None
                    };

                    Ok(Some((intent, None, cache_intent)))
                }
                _ => Err(checkresult),
            }
        }
        None => {
            trace!("Entity is trying to move to its own position");
            match next_room {
                Some(to_room) => {
                    let is_bridge = storage
                        .view::<WorldPosition, TerrainComponent>()
                        .get(botpos.0)
                        .map(|TerrainComponent(t)| *t == TileTerrainType::Bridge)
                        .unwrap_or_else(|| {
                            error!("Bot is not standing on terrain {:?}", botpos);
                            false
                        });
                    if !is_bridge {
                        return Err(OperationResult::InvalidTarget);
                    }
                    let target_pos = match pathfinding::get_valid_transits(
                        botpos.0,
                        to_room,
                        FromWorld::from_world(storage),
                    ) {
                        Ok(candidates) => candidates[0],
                        Err(pathfinding::TransitError::NotFound) => {
                            return Err(OperationResult::PathNotFound)
                        }
                        Err(e) => {
                            error!("Transit failed {:?}", e);
                            return Err(OperationResult::OperationFailed);
                        }
                    };
                    let intent = MoveIntent {
                        bot,
                        position: target_pos,
                    };
                    Ok(Some((
                        intent,
                        Some(MutPathCacheIntent {
                            bot,
                            action: PathCacheIntentAction::Del,
                        }),
                        None,
                    )))
                }
                None => {
                    debug!("Entity is trying to move to its own position, but no next room was returned");

                    Ok(None)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map_generation::room::iter_edge;
    use crate::prelude::*;
    use crate::query;
    use crate::terrain::TileTerrainType;

    #[test]
    fn can_move_to_another_room() {
        let mut storage = World::new();

        let bot_id = storage.insert_entity();
        let room_radius = 3;
        let room_center = Axial::new(room_radius, room_radius);

        let mut from = WorldPosition {
            room: Axial::new(0, 0),
            pos: Axial::default(),
        };
        let to = WorldPosition {
            room: Axial::new(0, 2),
            pos: Axial::new(2, 1),
        };

        let next_room = Axial::new(0, 1);

        from.pos = iter_edge(
            room_center,
            room_radius as u32,
            &RoomConnection {
                direction: next_room,
                offset_end: 1,
                offset_start: 1,
            },
        )
        .unwrap()
        .next()
        .unwrap();

        let user_id = UserId::default();

        query!(
            mutate
            storage
            {
                EntityId, Bot,
                    .insert(bot_id);
                EntityId, PositionComponent,
                    .insert(bot_id, PositionComponent(from));
                EntityId, OwnedEntity,
                    .insert(bot_id, OwnedEntity{owner_id:user_id});
                ConfigKey, RoomProperties,
                    .update(Some(RoomProperties{radius:room_radius as u32, center: room_center}));

                WorldPosition, EntityComponent,
                    .extend_rooms([Room(from.room),Room(Axial::new(0,1)), Room(to.room)].iter().cloned())
                    .expect("Failed to add rooms");


                WorldPosition, TerrainComponent,
                    .extend_rooms([Room(from.room),Room(Axial::new(0,1)), Room(to.room)].iter().cloned())
                    .expect("Failed to add rooms");
                WorldPosition, TerrainComponent,
                    .iter_rooms_mut().for_each(|(_, room)|room.resize(3));
                WorldPosition, TerrainComponent,
                    .extend_from_slice(&mut [
                        ( from, TerrainComponent(TileTerrainType::Bridge) ),
                        ( WorldPosition{room: Axial::new(0,1), pos: Axial::new(5,0)}
                          , TerrainComponent(TileTerrainType::Bridge) ),
                    ])
                    .expect("Failed to insert terrain");
        });

        let mut init_connections = |room| {
            // init connections...
            let mut connections = RoomConnections::default();
            let neighbour = next_room;
            connections.0[Axial::neighbour_index(neighbour).expect("Bad neighbour")] =
                Some(RoomConnection {
                    direction: neighbour,
                    offset_end: 0,
                    offset_start: 0,
                });
            query!(
                mutate
                storage
                {
                    Axial, RoomConnections,
                        .insert(from.room, connections )
                        .expect("Failed to add room connections");
                }
            );
            let mut connections = RoomConnections::default();
            let neighbour = next_room;
            connections.0[Axial::neighbour_index(neighbour).expect("Bad neighbour")] =
                Some(RoomConnection {
                    direction: neighbour,
                    offset_end: 0,
                    offset_start: 0,
                });
            query!(
                mutate
                storage
                {
                Axial, RoomConnections,
                    .insert( room, connections )
                    .expect("Failed to add room connections");
                }
            );
        };
        init_connections(next_room);
        init_connections(to.room);

        let (MoveIntent { bot, position }, ..) = move_to_pos(bot_id, to, user_id, &storage)
            .expect("Expected move to succeed")
            .expect("Expected a move intent");

        assert_eq!(bot, bot_id);
        assert_eq!(position.room, next_room);
    }
}

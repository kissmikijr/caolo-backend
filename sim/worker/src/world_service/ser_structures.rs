use std::collections::HashMap;

use super::util::push_room_pl;
use crate::protos::cao_common;
use crate::protos::cao_world;
use caolo_sim::prelude::*;

type StructureTables<'a> = (
    View<'a, WorldPosition, EntityComponent>,
    View<'a, Axial, RoomComponent>,
    View<'a, EntityId, Structure>,
    View<'a, EntityId, HpComponent>,
    View<'a, EntityId, OwnedEntity>,
    View<'a, EntityId, EnergyComponent>,
    View<'a, EntityId, EnergyRegenComponent>,
    View<'a, EntityId, SpawnComponent>,
    View<'a, EntityId, SpawnQueueComponent>,
    WorldTime,
);

pub fn structure_payload(
    out: &mut HashMap<Axial, cao_world::RoomEntities>,
    (
        room_entities,
        rooms,
        structures,
        hp,
        owner,
        energy,
        energy_regen,
        spawn,
        spawn_q,
        WorldTime(time),
    ): StructureTables,
) {
    let room_entities = room_entities.iter_rooms();

    let mut room = None;
    let mut offset = None;
    let mut accumulator = Vec::with_capacity(128);

    for (next_room, entities) in room_entities {
        // push the accumulator
        if Some(next_room) != room {
            if !accumulator.is_empty() {
                debug_assert!(room.is_some());
                push_room_pl(
                    out,
                    room.unwrap().0,
                    |pl| &mut pl.structures,
                    std::mem::take(&mut accumulator),
                    time as i64,
                );
            }
            room = Some(next_room);
            offset = rooms.get(next_room.0).map(|x| x.offset);
            accumulator.clear();
        }
        for (pos, EntityComponent(entity_id)) in entities.iter() {
            if structures.contains(entity_id) {
                let entity_id = *entity_id;
                let pl = cao_world::Structure {
                    id: entity_id.into(),
                    pos: Some(cao_common::WorldPosition {
                        pos: Some(pos.into()),
                        room: room.map(|x| x.0.into()),
                        offset: offset.map(|x| x.into()),
                    }),

                    hp: hp
                        .get(entity_id)
                        .copied()
                        .map(|HpComponent { hp, hp_max }| cao_world::Bounded {
                            value: hp.into(),
                            value_max: hp_max.into(),
                        }),
                    energy: energy.get(entity_id).copied().map(
                        |EnergyComponent { energy, energy_max }| cao_world::Bounded {
                            value: energy.into(),
                            value_max: energy_max.into(),
                        },
                    ),
                    energy_regen: energy_regen
                        .get(entity_id)
                        .copied()
                        .map(|EnergyRegenComponent { amount }| amount.into())
                        .unwrap_or(0),
                    owner: owner.get(entity_id).map(
                        |OwnedEntity {
                             owner_id: UserId(owner_id),
                         }| {
                            cao_common::Uuid {
                                data: owner_id.as_bytes().to_vec(),
                            }
                        },
                    ),
                    structure_body: {
                        if let Some(spawn) = spawn.get(entity_id) {
                            Some(cao_world::structure::StructureBody::Spawn(
                                cao_world::structure::Spawn {
                                    spawning: spawn
                                        .spawning
                                        .map(|id| id.into())
                                        .unwrap_or(u64::MAX),
                                    time_to_spawn: spawn.time_to_spawn.into(),
                                    spawn_queue: spawn_q
                                        .get(entity_id)
                                        .map(|SpawnQueueComponent { queue }| {
                                            queue.iter().copied().map(|id| id.into()).collect()
                                        })
                                        .unwrap_or_default(),
                                },
                            ))
                        } else {
                            None
                        }
                    },
                };
                accumulator.push(pl);
            }
        }
    }
    // push the last accumulator
    if let Some(room) = (!accumulator.is_empty()).then(|| ()).and(room) {
        push_room_pl(
            out,
            room.0,
            |pl| &mut pl.structures,
            accumulator,
            time as i64,
        );
    }
}

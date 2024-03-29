use std::collections::HashMap;

use crate::protos::cao_common;
use crate::protos::cao_intents;
use crate::protos::cao_world;
use caolo_sim::prelude::*;

use super::util::push_room_pl;

type BotTables<'a> = (
    View<'a, WorldPosition, EntityComponent>,
    View<'a, Axial, RoomComponent>,
    View<'a, EntityId, Bot>,
    View<'a, EntityId, CarryComponent>,
    View<'a, EntityId, HpComponent>,
    View<'a, EntityId, MeleeAttackComponent>,
    View<'a, EntityId, DecayComponent>,
    View<'a, EntityId, OwnedEntity>,
    View<'a, EntityId, EntityScript>,
    View<'a, EntityId, SayComponent>,
    View<'a, EntityId, DropoffEventComponent>,
    View<'a, EntityId, MineEventComponent>,
    View<'a, EntityTime, LogEntry>,
    WorldTime,
);

pub fn bot_payload(
    out: &mut HashMap<Axial, cao_world::RoomEntities>,
    (
        room_entities,
        rooms,
        bots,
        carry,
        hp,
        melee,
        decay,
        owner,
        script,
        say,
        dropoff,
        mine,
        logs,
        WorldTime(time),
    ): BotTables,
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
                    |pl| &mut pl.bots,
                    std::mem::take(&mut accumulator),
                    time as i64,
                );
            }
            room = Some(next_room);
            offset = rooms.get(next_room.0).map(|x| x.offset);
            accumulator.clear();
        }
        for (pos, EntityComponent(entity_id)) in entities.iter() {
            if bots.contains(entity_id) {
                let entity_id = *entity_id;
                accumulator.push(cao_world::Bot {
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
                    carry: carry.get(entity_id).copied().map(
                        |CarryComponent { carry, carry_max }| cao_world::Bounded {
                            value: carry.into(),
                            value_max: carry_max.into(),
                        },
                    ),
                    decay: decay.get(entity_id).copied().map(
                        |DecayComponent {
                             hp_amount,
                             interval,
                             time_remaining,
                         }| cao_world::bot::Decay {
                            hp_amount: hp_amount.into(),
                            interval: interval.into(),
                            time_remaining: time_remaining.into(),
                        },
                    ),
                    melee_strength: melee
                        .get(entity_id)
                        .copied()
                        .map(|MeleeAttackComponent { strength }| strength.into())
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
                    script: script
                        .get(entity_id)
                        .map(|EntityScript(ScriptId(script_id))| cao_common::Uuid {
                            data: script_id.as_bytes().to_vec(),
                        }),
                    logs: logs
                        .get(EntityTime(entity_id, time - 1)) // send the logs of the last tick
                        .map(|logs| logs.payload.clone())
                        .unwrap_or_default(),
                    say: say
                        .get(entity_id)
                        .map(|SayComponent(pl)| pl.to_string())
                        .unwrap_or_default(),
                    dropoff_intent: dropoff.get(entity_id).copied().map(
                        |DropoffEventComponent(pl)| cao_intents::DropoffIntent {
                            target_id: pl.into(),
                        },
                    ),
                    mine_intent: mine.get(entity_id).copied().map(|MineEventComponent(pl)| {
                        cao_intents::MineIntent {
                            target_id: pl.into(),
                        }
                    }),
                });
            }
        }
    }
    // push the last accumulator
    if let Some(room) = (!accumulator.is_empty()).then(|| ()).and(room) {
        push_room_pl(out, room.0, |pl| &mut pl.bots, accumulator, time as i64);
    }
}

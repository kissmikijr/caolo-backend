use crate::components::{
    CarryComponent, EnergyComponent, MineEventComponent, Resource, ResourceComponent,
};
use crate::indices::*;
use crate::intents::{Intents, MineIntent};
use crate::profile;
use crate::storage::views::{UnsafeView, UnwrapView, View};
use tracing::{trace, warn};

pub const MINE_AMOUNT: u16 = 10; // TODO: get from bot body

type Mut = (
    UnsafeView<EntityId, EnergyComponent>,
    UnsafeView<EntityId, CarryComponent>,
    UnsafeView<EntityId, MineEventComponent>,
);
type Const<'a> = (
    View<'a, EntityId, ResourceComponent>,
    UnwrapView<'a, EmptyKey, Intents<MineIntent>>,
);

pub fn mine_intents_update(
    (mut energy_table, mut carry_table, mut event): Mut,
    (resource_table, intents): Const,
) {
    profile!("MineSystem update");

    event.clear();

    for intent in intents.iter() {
        trace!("Bot {:?} is mining [{:?}]", intent.bot, intent.resource);
        match resource_table.get(intent.resource) {
            Some(ResourceComponent(Resource::Energy)) => {
                let resource_energy = match energy_table.get_mut(intent.resource) {
                    Some(resource_energy) => {
                        if resource_energy.energy == 0 {
                            trace!("Mineral is empty!");
                            continue;
                        }
                        resource_energy
                    }
                    None => {
                        warn!("MineIntent resource has no energy component!");
                        continue;
                    }
                };
                let carry = match carry_table.get_mut(intent.bot) {
                    Some(x) => x,
                    None => {
                        warn!("MineIntent bot {:?} has no carry component", intent.bot);
                        continue;
                    }
                };

                let mined = resource_energy.energy.min(MINE_AMOUNT); // Max amount that can be mined
                let mined = (carry.carry_max - carry.carry).min(mined); // Max amount the bot can carry

                carry.carry += mined;
                resource_energy.energy -= mined;

                event.insert(intent.bot, MineEventComponent(intent.resource));

                trace!(
                    "Mine succeeded new bot carry {:?} new resource energy {:?}",
                    carry,
                    resource_energy
                );
            }
            Some(ResourceComponent(Resource::Empty)) | None => {
                warn!("Resource ({:?}) not found", intent.resource)
            }
        }
    }
}

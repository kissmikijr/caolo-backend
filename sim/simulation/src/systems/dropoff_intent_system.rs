use crate::components::{CarryComponent, DropoffEventComponent, EnergyComponent};
use crate::indices::*;
use crate::intents::*;
use crate::profile;
use crate::storage::views::{UnsafeView, UnwrapView};
use tracing::{trace, warn};

type Mut = (
    UnsafeView<EntityId, EnergyComponent>,
    UnsafeView<EntityId, CarryComponent>,
    UnsafeView<EntityId, DropoffEventComponent>,
);
type Const<'a> = (UnwrapView<'a, EmptyKey, Intents<DropoffIntent>>,);

pub fn dropoff_intents_update(
    (mut energy_table, mut carry_table, mut events): Mut,
    (intents,): Const,
) {
    profile!("DropoffSystem update");

    events.clear();
    for intent in intents.iter() {
        trace!("Executing dropoff intent {:?}", intent);
        // dropoff amount = min(bot carry , amount , structure capacity)
        let carry_component = match carry_table.get_mut(intent.bot) {
            Some(x) => x,
            None => {
                warn!("Bot has no carry");
                continue;
            }
        };
        let store_component = match energy_table.get_mut(intent.structure) {
            Some(x) => x,
            None => {
                warn!("Structure has no energy");
                continue;
            }
        };
        let dropoff = intent
            .amount
            .min(carry_component.carry)
            .min(store_component.energy_max - store_component.energy);

        store_component.energy += dropoff;
        carry_component.carry -= dropoff;

        events.insert(intent.bot, DropoffEventComponent(intent.structure));
    }
}

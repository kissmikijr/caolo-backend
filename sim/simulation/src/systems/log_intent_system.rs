use crate::components::LogEntry;
use crate::indices::*;
use crate::intents::{Intents, LogIntent};
use crate::profile;
use crate::storage::views::{UnsafeView, UnwrapViewMut};
use crate::tables::Table;
use std::mem::take;
use tracing::trace;

type Mut = (
    UnsafeView<EntityTime, LogEntry>,
    UnwrapViewMut<EmptyKey, Intents<LogIntent>>,
);

pub fn log_intents_update((mut log_table, mut intents): Mut, (): ()) {
    profile!("LogIntentSystem update");

    let intents = take(&mut intents.0);

    for intent in intents {
        trace!("inserting log entry {:?}", intent);
        let id = EntityTime(intent.entity, intent.time);
        // use delete to move out of the data structure, then we'll move it back in
        // this should be cheaper than cloning all the time, because of the inner vectors
        match log_table.delete(id) {
            Some(mut entry) => {
                entry.payload.push_str(&intent.payload);
                log_table.insert(id, entry);
            }
            None => {
                let entry = LogEntry {
                    payload: intent.payload,
                };
                log_table.insert(id, entry);
            }
        };
    }
}

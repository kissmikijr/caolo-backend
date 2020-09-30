use crate::RedisPool;
use anyhow::Context;
use cao_messages::WorldState;
use redis::Commands;
use slog::{debug, error, o, Logger};
use std::sync::{Arc, RwLock};
use tokio::time::{self, Duration};

pub async fn load_state(key: &str, pool: RedisPool, logger: Logger) -> anyhow::Result<WorldState> {
    let mut connection = pool.get().unwrap();
    // TODO: async pls
    connection
        .get::<_, Vec<u8>>(key)
        .map(|bytes| {
            rmp_serde::from_read_ref(bytes.as_slice()).expect("WorldState deserialization error")
        })
        .map(|mut state: WorldState| {
            // make sure history is sorted
            // the worker should do this for us, so report an error if this assumption is broken
            let history = &mut state.script_history;
            if history.len() < 2 {
                return state;
            }
            let mut last = &history[0];
            let mut needs_sort = false;
            for entry in history[1..].iter() {
                if last.entity_id > entry.entity_id {
                    error!(logger, "scrip_history was not sorted!");
                    needs_sort = true;
                    break;
                }
                last = entry;
            }
            if needs_sort {
                history.sort_unstable_by_key(|entry| entry.entity_id);
            }
            state
        })
        .map_err(|err| {
            error!(logger, "Failed to read world state {:?}", err);
            err
        })
        .with_context(|| "Failed to read world state")
}

pub async fn refresh_state_job(
    key: &str,
    pool: RedisPool,
    logger: Logger,
    state: Arc<RwLock<WorldState>>,
    interval: Duration,
) -> anyhow::Result<()> {
    let mut interval = time::interval(interval);

    let logger = logger.new(o!("key"=>key.to_owned()));

    loop {
        interval.tick().await;
        debug!(logger, "Reading world state");

        let new_state = load_state(key, pool.clone(), logger.clone()).await?;

        let mut state = state.write().unwrap();
        *state = new_state;

        debug!(logger, "Reading world state - done");
    }
}

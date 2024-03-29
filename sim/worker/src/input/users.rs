use crate::protos::cao_users::RegisterUserMsg;
use caolo_sim::{prelude::*, query};
use std::{convert::TryFrom, num::TryFromIntError};
use thiserror::Error;
use tracing::{info, trace};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("User by id {0} has been registered already")]
    AlreadyRegistered(Uuid),
    #[error("{0} is not a valid level")]
    BadLevel(TryFromIntError),
    #[error("Failed to parse uuid {0}")]
    UuidError(anyhow::Error),
    #[error("Missing expected field {0}")]
    MissingField(&'static str),
}

pub fn register_user(world: &mut World, msg: &RegisterUserMsg) -> Result<(), RegisterUserError> {
    trace!("Register user");

    let user_id = msg
        .user_id
        .as_ref()
        .ok_or(RegisterUserError::MissingField("user_id"))?
        .data
        .as_slice();
    let user_id =
        uuid::Uuid::from_slice(user_id).map_err(|err| RegisterUserError::UuidError(err.into()))?;

    let level = msg.level;
    let level = u16::try_from(level).map_err(RegisterUserError::BadLevel)?;

    info!("Registering new user. Id: {} level: {}", user_id, level);

    if world
        .view::<UserId, UserProperties>()
        .reborrow()
        .contains(UserId(user_id))
    {
        return Err(RegisterUserError::AlreadyRegistered(user_id));
    }

    let user_id = UserId(user_id);

    query!(
        mutate
        world
        {
            UserId, UserComponent,
                .insert(user_id);
            UserId, Rooms,
                .insert(user_id, Rooms::default());
            UserId, UserProperties,
                .insert(user_id, UserProperties{level});
        }
    );

    Ok(())
}

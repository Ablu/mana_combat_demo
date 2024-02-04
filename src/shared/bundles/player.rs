use crate::shared::{
    components::*,
    protocol::{Replicate, REPLICATION_GROUP},
};
use bevy::prelude::*;
use lightyear::netcode::ClientId;

#[derive(Bundle)]
pub struct Player {
    id: PlayerId,
    position: Position,
    replicate: Replicate,
}

impl Player {
    pub(crate) fn new(id: ClientId, position: Position) -> Self {
        Self {
            id: PlayerId(id),
            position: position,

            replicate: Replicate {
                // NOTE (important): all entities that are being predicted need to be part of the same replication-group
                //  so that all their updates are sent as a single message and are consistent (on the same tick)
                replication_group: REPLICATION_GROUP,
                ..default()
            },
        }
    }
}

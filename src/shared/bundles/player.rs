use crate::shared::{
    components::*,
    protocol::{PlayerActions, Replicate, REPLICATION_GROUP},
};
use bevy::prelude::*;
use bevy_aseprite::{anim::AsepriteAnimation, AsepriteBundle};
use leafwing_input_manager::prelude::*;
use lightyear::netcode::ClientId;

#[derive(Bundle)]
pub struct Player {
    id: PlayerId,
    position: Position,
    replicate: Replicate,
    inputs: InputManagerBundle<PlayerActions>,
    sprite: AsepriteBundle,
}

pub mod sprites {
    use bevy_aseprite::aseprite;
    aseprite!(pub Player, "player.aseprite");
}

impl Player {
    pub(crate) fn new(
        id: ClientId,
        position: Position,
        input_map: InputMap<PlayerActions>,
        asset_server: &AssetServer,
    ) -> Self {
        Self {
            id: PlayerId(id),
            position: position,

            replicate: Replicate {
                // NOTE (important): all entities that are being predicted need to be part of the same replication-group
                //  so that all their updates are sent as a single message and are consistent (on the same tick)
                replication_group: REPLICATION_GROUP,
                ..default()
            },

            inputs: InputManagerBundle::<PlayerActions> {
                action_state: ActionState::default(),
                input_map,
            },
            sprite: AsepriteBundle {
                aseprite: asset_server.load(sprites::Player::PATH),
                animation: AsepriteAnimation::from(sprites::Player::tags::DOWN_STAND),
                transform: Transform::from_xyz(100.0, 100.0, 10.0),
                ..Default::default()
            },
        }
    }
}

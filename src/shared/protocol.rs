use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::client::components::LerpFn;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, Mul},
    time::Duration,
};

use crate::shared::components::ability::Ability;
use crate::shared::components::facing_direction::FacingDirection;
use crate::shared::components::player_id::PlayerId;
use crate::shared::components::position::Position;

pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::Group(1);

pub const LINK_CONDITIONER: LinkConditionerConfig = LinkConditionerConfig {
    incoming_latency: Duration::from_millis(150),
    incoming_jitter: Duration::from_millis(25),
    incoming_loss: 0.02,
};

#[message_protocol(protocol = "ManaProtocol")]
pub enum Messages {}

#[component_protocol(protocol = "ManaProtocol")]
pub enum Components {
    #[sync(once)]
    PlayerId(PlayerId),
    #[sync(full)]
    Position(Position),
    #[sync(full)]
    FacingDirection(FacingDirection),
    #[sync(once)]
    Ability(Ability),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum PlayerActions {
    Move,
    Slash,
}

impl LeafwingUserAction for PlayerActions {}

#[derive(Channel)]
pub struct Channel1;

protocolize! {
    Self = ManaProtocol,
    Message = Messages,
    Component = Components,
    Input = (),
    LeafwingInput1 = PlayerActions,
    LeafwingInput2 = NoAction2,
}

pub fn protocol() -> ManaProtocol {
    let mut protocol = ManaProtocol::default();
    protocol.add_channel::<Channel1>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        direction: ChannelDirection::Bidirectional,
    });
    protocol
}

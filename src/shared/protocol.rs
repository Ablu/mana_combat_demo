use std::ops::{Add, Mul};

use bevy::prelude::*;

use lightyear::prelude::*;

use crate::shared::components::*;

pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::Group(1);

#[message_protocol(protocol = "ManaProtocol")]
pub enum Messages {}

#[component_protocol(protocol = "ManaProtocol")]
pub enum Components {
    #[sync(once)]
    PlayerId(PlayerId),
    #[sync(full)]
    Position(Position),
}

#[derive(Channel)]
pub struct Channel1;

protocolize! {
    Self = ManaProtocol,
    Message = Messages,
    Component = Components,
    Input = (),
}

pub fn protocol() -> ManaProtocol {
    let mut protocol = ManaProtocol::default();
    protocol.add_channel::<Channel1>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        direction: ChannelDirection::Bidirectional,
    });
    protocol
}

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::client::components::LerpFn;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, Mul},
    time::Duration,
};

use crate::shared::components::*;

pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::Group(1);

pub const LINK_CONDITIONER: LinkConditionerConfig = LinkConditionerConfig {
    incoming_latency: Duration::from_millis(150),
    incoming_jitter: Duration::from_millis(25),
    incoming_loss: 0.02,
};

#[message_protocol(protocol = "ManaProtocol")]
pub enum Messages {}

pub struct PositionInterpolator;
impl LerpFn<Position> for PositionInterpolator {
    fn lerp(start: Position, other: Position, t: f32) -> Position {
        Position {
            x: start.x + (other.x - start.x) * t,
            y: start.y + (other.y - start.y) * t,
        }
    }
}

#[component_protocol(protocol = "ManaProtocol")]
pub enum Components {
    #[sync(once)]
    PlayerId(PlayerId),
    #[sync(
        full,
        lerp = "PositionInterpolator",
        corrector = "InterpolatedCorrector"
    )]
    Position(Position),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum PlayerActions {
    Up,
    Down,
    Left,
    Right,
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

use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Ability {
    pub start_tick: Tick,
    pub end_tick: Tick,
    pub position: Vec2,
    pub direction: Vec2,
}

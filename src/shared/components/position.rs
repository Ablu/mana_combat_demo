use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};

#[derive(Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Position(pub Vec2);

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Mul<f32> for Position {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

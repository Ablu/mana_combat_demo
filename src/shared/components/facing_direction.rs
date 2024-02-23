use std::ops::{Add, Mul};

use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct FacingDirection(pub Vec2);

impl Add for FacingDirection {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        FacingDirection(self.0 + rhs.0)
    }
}

impl Mul<f32> for FacingDirection {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

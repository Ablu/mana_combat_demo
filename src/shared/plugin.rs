use crate::shared::protocol::PlayerActions;

use super::components::*;
use bevy::{prelude::*, render::RenderPlugin};
use leafwing_input_manager::prelude::*;
use lightyear::{
    client::{
        components::Confirmed,
        interpolation::{plugin::InterpolationSet, Interpolated},
        prediction::{plugin::PredictionSet, Predicted},
    },
    shared::{sets::FixedUpdateSet, tick_manager::TickManager},
};

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        if app.is_plugin_added::<RenderPlugin>() {}

        // registry types for reflection
        app.register_type::<PlayerId>();
        app.register_type::<Position>();
    }
}

pub(crate) fn draw_elements(
    mut gizmos: Gizmos,
    players: Query<&Position, (With<Interpolated>, With<PlayerId>)>,
) {
    for position in &players {
        gizmos.rect_2d(
            Vec2::new(position.x, position.y),
            0.0,
            Vec2::new(64.0, 64.0),
            Color::RED,
        );
    }
}

pub(crate) fn shared_movement_behaviour(
    mut pos: Mut<Position>,
    action: &ActionState<PlayerActions>,
) {
    const MOVE_SPEED: f32 = 3.0;
    if action.pressed(PlayerActions::Up) {
        pos.y += MOVE_SPEED;
    }
    if action.pressed(PlayerActions::Down) {
        pos.y -= MOVE_SPEED;
    }
    if action.pressed(PlayerActions::Left) {
        pos.x -= MOVE_SPEED;
    }
    if action.pressed(PlayerActions::Right) {
        pos.x += MOVE_SPEED;
    }
}

pub fn draw_sprite(gizmos: &mut Gizmos, position: &Position) {
    gizmos.rect_2d(
        Vec2::new(position.x, position.y),
        0.0,
        Vec2::new(64.0, 64.0),
        Color::GRAY,
    );
}

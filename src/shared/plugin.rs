use std::time::Duration;

use crate::shared::{
    components::ability::Ability, config::duration_to_ticks, protocol::PlayerActions,
};

use self::{facing_direction::FacingDirection, player_id::PlayerId, position::Position};

use super::components::*;
use bevy::{prelude::*, render::RenderPlugin};
use leafwing_input_manager::prelude::*;
use lightyear::{
    client::{
        components::Confirmed,
        interpolation::{plugin::InterpolationSet, Interpolated},
        prediction::{plugin::PredictionSet, Predicted},
    },
    shared::{
        sets::FixedUpdateSet,
        tick_manager::{Tick, TickManager},
    },
};

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (update_abilities).in_set(FixedUpdateSet::Main));

        // registry types for reflection
        app.register_type::<PlayerId>();
        app.register_type::<Position>();
    }
}

fn update_abilities(
    mut commands: Commands,
    tick_manager: Res<TickManager>,
    abilities: Query<(Entity, &Ability)>,
) {
    for (entity, ability) in abilities.iter() {
        if ability.end_tick == tick_manager.tick() {
            commands.entity(entity).remove::<Ability>();
        }
    }
}

pub fn shared_player_input(
    commands: &mut Commands,
    entity: Entity,
    mut pos: Mut<Position>,
    mut direction: Mut<FacingDirection>,
    ability: Option<&Ability>,
    tick_manager: &TickManager,
    action: &ActionState<PlayerActions>,
) {
    // no movement while an attack is happening!
    if ability.is_some() {
        return;
    }

    if action.pressed(PlayerActions::Slash) {
        let current_tick = tick_manager.tick();
        let duration = Duration::from_millis(300);
        commands.entity(entity).insert(Ability {
            position: pos.0,
            direction: Vec2::ZERO,
            start_tick: current_tick,
            end_tick: current_tick + duration_to_ticks(duration),
        });
    }

    const WALK_SPEED: f32 = 3.0;
    if action.pressed(PlayerActions::Move) {
        let input = action.clamped_axis_pair(PlayerActions::Move).unwrap().xy();
        let dir = input.clamp_length_max(1.0);
        *direction = FacingDirection(dir);
        let delta = dir * WALK_SPEED;
        pos.0 += delta;
    }
}

pub fn draw_sprite(gizmos: &mut Gizmos, position: &Position) {
    gizmos.rect_2d(position.0, 0.0, Vec2::new(64.0, 64.0), Color::GRAY);
}

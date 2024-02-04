use super::components::*;
use bevy::{prelude::*, render::RenderPlugin};
use lightyear::client::{
    components::Confirmed, interpolation::plugin::InterpolationSet,
    prediction::plugin::PredictionSet,
};

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        if app.is_plugin_added::<RenderPlugin>() {
            app.add_systems(
                PostUpdate,
                draw_elements
                    .after(InterpolationSet::Interpolate)
                    .after(PredictionSet::VisualCorrection),
            );
        }

        // registry types for reflection
        app.register_type::<PlayerId>();
    }
}

pub(crate) fn draw_elements(
    mut gizmos: Gizmos,
    players: Query<&Position, (Without<Confirmed>, With<PlayerId>)>,
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

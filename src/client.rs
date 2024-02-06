use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use bevy::{app::PluginGroupBuilder, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_aseprite::{anim::AsepriteAnimation, AsepriteBundle, AsepritePlugin};
use leafwing_input_manager::prelude::*;
use lightyear::{
    client::{
        components::Confirmed,
        config::{ClientConfig, NetcodeConfig},
        input_leafwing::{LeafwingInputConfig, LeafwingInputPlugin},
        interpolation::{
            plugin::{InterpolationConfig, InterpolationDelay, InterpolationSet},
            Interpolated,
        },
        plugin::{ClientPlugin, PluginConfig},
        prediction::{
            plugin::{PredictionConfig, PredictionSet},
            Predicted,
        },
        resource::Authentication,
    },
    netcode::ClientId,
    prelude::LinkConditionerConfig,
    shared::{sets::FixedUpdateSet, tick_manager::TickManager},
    transport::io::{Io, IoConfig, TransportConfig},
};
use rand::Rng;

use crate::shared::{
    bundles::{self, player},
    components::{PlayerId, Position},
    config::{shared_config, KEY, PROTOCOL_ID},
    plugin::{draw_sprite, shared_movement_behaviour, SharedPlugin},
    protocol::{protocol, ClientMut, ManaProtocol, PlayerActions, LINK_CONDITIONER},
};

pub const INPUT_DELAY_TICKS: u16 = 0;
pub const CORRECTION_TICKS_FACTOR: f32 = 1.5;
pub struct ClientPluginGroup {
    client_id: ClientId,
    lightyear: ClientPlugin<ManaProtocol>,
}

impl ClientPluginGroup {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let client_id = rng.gen_range(0..1000000);
        let auth = Authentication::Manual {
            server_addr: "127.0.0.1:8888"
                .parse()
                .expect("should be valid SocketAddr"),
            client_id,
            private_key: KEY,
            protocol_id: PROTOCOL_ID,
        };
        let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
        let transport_config = TransportConfig::UdpSocket(client_addr);

        let io = Io::from_config(
            IoConfig::from_transport(transport_config).with_conditioner(LINK_CONDITIONER),
        );
        let config = ClientConfig {
            shared: shared_config(),
            netcode: NetcodeConfig::default(),
            prediction: PredictionConfig {
                input_delay_ticks: INPUT_DELAY_TICKS,
                correction_ticks_factor: CORRECTION_TICKS_FACTOR,
                ..Default::default()
            },
            interpolation: InterpolationConfig::default()
                .with_delay(InterpolationDelay::default().with_send_interval_ratio(2.0)),
            ..Default::default()
        };
        let plugin_config = PluginConfig::new(config, io, protocol(), auth);
        ClientPluginGroup {
            client_id,
            lightyear: ClientPlugin::new(plugin_config),
        }
    }
}

impl PluginGroup for ClientPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(self.lightyear)
            .add(ManaClientPlugin {
                client_id: self.client_id,
            })
            .add(SharedPlugin)
            .add(LeafwingInputPlugin::<ManaProtocol, PlayerActions>::new(
                LeafwingInputConfig::<PlayerActions> {
                    send_diffs_only: true,
                    ..Default::default()
                },
            ))
            .add(AsepritePlugin)
    }
}

pub struct ManaClientPlugin {
    client_id: ClientId,
}

#[derive(Resource)]
pub struct Global {
    client_id: ClientId,
}

impl Plugin for ManaClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Global {
            client_id: self.client_id,
        });
        app.add_systems(Startup, init);
        app.add_systems(FixedUpdate, (predict_movement).in_set(FixedUpdateSet::Main));
        app.add_systems(
            Update,
            (
                draw_own_player,
                update_sprite_positions,
                upate_sprite_direction,
            ),
        );
        app.add_systems(
            PostUpdate,
            (draw_confirmed, draw_predicted, draw_interpolated)
                .after(InterpolationSet::Interpolate)
                .after(PredictionSet::VisualCorrection),
        );
    }
}

pub(crate) fn init(
    mut commands: Commands,
    mut client: ClientMut,
    global: Res<Global>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(
        TextBundle::from_section(
            format!("Client {}", global.client_id),
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..Default::default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::End,
            ..Default::default()
        }),
    );
    commands.spawn(bundles::player::Player::new(
        global.client_id,
        Position { x: 100.0, y: 100.0 },
        InputMap::new([
            (KeyCode::W, PlayerActions::Up),
            (KeyCode::S, PlayerActions::Down),
            (KeyCode::A, PlayerActions::Left),
            (KeyCode::D, PlayerActions::Right),
        ]),
        &asset_server,
    ));

    let _ = client.connect();
}

pub(crate) fn predict_movement(
    tick_manager: Res<TickManager>,
    mut action_query: Query<(Entity, &mut Position, &ActionState<PlayerActions>)>,
) {
    for (entity, position, action) in action_query.iter_mut() {
        shared_movement_behaviour(position, action);
    }
}

pub(crate) fn draw_confirmed(
    mut gizmos: Gizmos,
    players: Query<&Position, (With<Confirmed>, With<PlayerId>)>,
) {
    for position in &players {
        gizmos.rect_2d(
            Vec2::new(position.x, position.y),
            0.0,
            Vec2::new(64.0, 64.0),
            Color::GREEN,
        );
    }
}

pub(crate) fn draw_predicted(
    mut gizmos: Gizmos,
    players: Query<&Position, (With<Predicted>, With<PlayerId>)>,
) {
    for position in &players {
        gizmos.rect_2d(
            Vec2::new(position.x, position.y),
            0.0,
            Vec2::new(64.0, 64.0),
            Color::YELLOW,
        );
    }
}

pub(crate) fn draw_interpolated(
    mut gizmos: Gizmos,
    players: Query<&Position, (With<Interpolated>, With<PlayerId>)>,
) {
    for position in &players {
        gizmos.rect_2d(
            Vec2::new(position.x, position.y),
            0.0,
            Vec2::new(64.0, 64.0),
            Color::BLUE,
        );
    }
}

fn update_sprite_positions(mut players: Query<(&Position, &mut Transform), With<PlayerId>>) {
    for (pos, mut transform) in &mut players {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

fn upate_sprite_direction(
    mut players: Query<(&ActionState<PlayerActions>, &mut AsepriteAnimation), With<PlayerId>>,
) {
    for (input, mut animation) in &mut players {
        let tag = if input.just_pressed(PlayerActions::Down) {
            player::sprites::Player::tags::DOWN
        } else if input.just_pressed(PlayerActions::Up) {
            player::sprites::Player::tags::UP
        } else if input.just_pressed(PlayerActions::Left) {
            player::sprites::Player::tags::LEFT
        } else if input.just_pressed(PlayerActions::Right) {
            player::sprites::Player::tags::RIGHT
        } else {
            if input.get_pressed().is_empty() {
                if input.just_released(PlayerActions::Down) {
                    player::sprites::Player::tags::DOWN_STAND
                } else if input.just_released(PlayerActions::Up) {
                    player::sprites::Player::tags::UP_STAND
                } else if input.just_released(PlayerActions::Left) {
                    player::sprites::Player::tags::LEFT_STAND
                } else if input.just_released(PlayerActions::Right) {
                    player::sprites::Player::tags::RIGHT_STAND
                } else {
                    continue;
                }
            } else {
                continue;
            }
        };

        *animation = AsepriteAnimation::from(tag);
        animation.is_playing = true;
    }
}

pub(crate) fn draw_own_player(
    mut gizmos: Gizmos,
    global: Res<Global>,
    players: Query<
        (&PlayerId, &Position),
        (With<PlayerId>, Without<Predicted>, Without<Interpolated>),
    >,
) {
    let player: Vec<_> = players
        .iter()
        .filter(|(id, _)| id.0 == global.client_id)
        .collect();
    assert_eq!(player.len(), 1);
    let (_id, pos) = player[0];
    draw_sprite(&mut gizmos, pos);
}

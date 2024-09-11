use std::{
    net::{Ipv6Addr, SocketAddr},
    time::Duration,
};

use bevy::{
    app::PluginGroupBuilder, diagnostic::FrameTimeDiagnosticsPlugin, ecs::entity, prelude::*,
};
use bevy_asepritesheet::{
    animator::{AnimTimestamp, AnimatedSpriteBundle, SpriteAnimator},
    core::{load_spritesheet, AsepritesheetPlugin},
    sprite::{AnimHandle, Spritesheet},
};
use leafwing_input_manager::prelude::*;
use lightyear::{
    client::{
        components::Confirmed,
        config::{ClientConfig, NetcodeConfig},
        events::ComponentInsertEvent,
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
    shared::{
        sets::{FixedUpdateSet, MainSet},
        tick_manager::{self, TickManager},
    },
    transport::io::{Io, IoConfig, TransportConfig},
};
use rand::Rng;

use crate::shared::{
    bundles::{
        self,
        player::{self, Player},
    },
    components::{
        ability::{self, Ability},
        facing_direction::FacingDirection,
        player_id::PlayerId,
        position::Position,
    },
    config::{shared_config, ticks_to_duration, KEY, PROTOCOL_ID},
    plugin::{draw_sprite, shared_player_input, SharedPlugin},
    protocol::{protocol, ClientMut, ManaProtocol, PlayerActions, Replicate, LINK_CONDITIONER},
};

pub const INPUT_DELAY_TICKS: u16 = 0;
pub const CORRECTION_TICKS_FACTOR: f32 = 1.5;
pub struct ClientPluginGroup {
    client_id: ClientId,
    lightyear: ClientPlugin<ManaProtocol>,
}

impl ClientPluginGroup {
    pub fn new(ip: String) -> Self {
        let mut rng = rand::thread_rng();
        let client_id = rng.gen_range(0..1000000);
        let auth = Authentication::Manual {
            server_addr: ip.parse().expect("should be valid SocketAddr"),
            client_id,
            private_key: KEY,
            protocol_id: PROTOCOL_ID,
        };
        let client_addr = SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0);
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
            .add(AsepritesheetPlugin::new(&["sprite.json"]))
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
    }
}

pub struct ManaClientPlugin {
    client_id: ClientId,
}

#[derive(Resource)]
pub struct Global {
    client_id: ClientId,
}

#[derive(Resource)]
struct PlayerSprite(Handle<Spritesheet>);

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
                center_camera_on_own_player,
            ),
        );
        app.add_systems(
            PostUpdate,
            (draw_confirmed, draw_predicted, draw_interpolated)
                .after(InterpolationSet::Interpolate)
                .after(PredictionSet::VisualCorrection),
        );
        app.add_systems(
            PreUpdate,
            (add_new_player_sprites, add_own_player_sprite).in_set(MainSet::ClientReplication),
        );
    }
}

pub(crate) fn init(
    mut commands: Commands,
    mut client: ClientMut,
    global: Res<Global>,
    sprite_sheets: Res<Assets<Spritesheet>>,
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

    let player_sprite = load_spritesheet(
        &mut commands,
        &asset_server,
        "player.sprite.json",
        bevy::sprite::Anchor::Center,
    );

    commands.insert_resource(PlayerSprite(player_sprite.clone()));

    let mut input = InputMap::default();
    input.insert(
        VirtualDPad {
            up: KeyCode::W.into(),
            down: KeyCode::S.into(),
            left: KeyCode::A.into(),
            right: KeyCode::D.into(),
        },
        PlayerActions::Move,
    );
    input.insert(KeyCode::Left, PlayerActions::Slash);

    let mut player = commands.spawn(bundles::player::Player::new(
        global.client_id,
        Position(Vec2::new(100.0, 100.0)),
        input,
        create_player_sprite(player_sprite),
    ));

    let _ = client.connect();
}

fn create_player_sprite(sprite: Handle<Spritesheet>) -> AnimatedSpriteBundle {
    AnimatedSpriteBundle {
        animator: SpriteAnimator::from_anim(AnimHandle::from_index(1)),
        spritesheet: sprite,
        ..Default::default()
    }
}

fn add_new_player_sprites(
    mut commands: Commands,
    player_sprite: Res<PlayerSprite>,
    mut new_players: Query<Entity, (Added<PlayerId>, With<Interpolated>)>,
) {
    for entity in &mut new_players {
        commands
            .entity(entity)
            .insert(create_player_sprite(player_sprite.0.clone()));
    }
}

fn add_own_player_sprite(
    mut commands: Commands,
    player_sprite: Res<PlayerSprite>,
    mut new_players: Query<Entity, (With<PlayerId>, Added<Predicted>)>,
) {
    for entity in &mut new_players {
        commands
            .entity(entity)
            .insert(create_player_sprite(player_sprite.0.clone()));
    }
}

pub(crate) fn predict_movement(
    mut commands: Commands,
    tick_manager: Res<TickManager>,
    mut action_query: Query<
        (
            Entity,
            &mut Position,
            &mut FacingDirection,
            Option<&Ability>,
            &ActionState<PlayerActions>,
        ),
        With<Predicted>,
    >,
) {
    for (entity, position, direction, ability, action) in action_query.iter_mut() {
        shared_player_input(
            &mut commands,
            entity,
            position,
            direction,
            ability,
            &tick_manager,
            action,
        );
    }
}

pub(crate) fn draw_confirmed(
    mut gizmos: Gizmos,
    players: Query<&Position, (With<Confirmed>, With<PlayerId>)>,
) {
    for position in &players {
        gizmos.rect_2d(position.0, 0.0, Vec2::new(64.0, 64.0), Color::GREEN);
    }
}

pub(crate) fn draw_predicted(
    mut gizmos: Gizmos,
    players: Query<&Position, (With<Predicted>, With<PlayerId>)>,
) {
    for position in &players {
        gizmos.rect_2d(position.0, 0.0, Vec2::new(64.0, 64.0), Color::YELLOW);
    }
}

pub(crate) fn draw_interpolated(
    mut gizmos: Gizmos,
    players: Query<&Position, (With<Interpolated>, With<PlayerId>)>,
) {
    for position in &players {
        gizmos.rect_2d(position.0, 0.0, Vec2::new(64.0, 64.0), Color::BLUE);
    }
}

fn update_sprite_positions(mut players: Query<(&Position, &mut Transform), With<PlayerId>>) {
    for (pos, mut transform) in &mut players {
        transform.translation = pos.0.extend(0.0);
    }
}

#[derive(Debug)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

fn vec2_to_direction(v: Vec2) -> Direction {
    if v.x.abs() > v.y.abs() {
        if v.x.is_sign_negative() {
            Direction::Left
        } else {
            Direction::Right
        }
    } else {
        if v.y.is_sign_negative() {
            Direction::Down
        } else {
            Direction::Up
        }
    }
}

#[test]
fn vec2_to_direction_test() {
    // default to down
    matches!(vec2_to_direction(Vec2::new(0.0, 0.0)), Direction::Down);
    // simple left
    matches!(vec2_to_direction(Vec2::new(-1.0, 0.0)), Direction::Left);
    // simple right
    matches!(vec2_to_direction(Vec2::new(1.0, 0.0)), Direction::Right);
    // simple down
    matches!(vec2_to_direction(Vec2::new(0.0, 1.0)), Direction::Down);
    // simple up
    matches!(vec2_to_direction(Vec2::new(0.0, -1.0)), Direction::Up);

    // left dominates
    matches!(vec2_to_direction(Vec2::new(-0.9, -0.1)), Direction::Left);
    // right dominates
    matches!(vec2_to_direction(Vec2::new(0.9, -0.1)), Direction::Right);
    // down wins on equal
    matches!(vec2_to_direction(Vec2::new(1.0, 1.0)), Direction::Down);
    // up wins on equal
    matches!(vec2_to_direction(Vec2::new(1.0, -1.0)), Direction::Down);
}

fn upate_sprite_direction(
    player_sprite: Res<PlayerSprite>,
    sprite_sheets: Res<Assets<Spritesheet>>,
    tick_manager: Res<TickManager>,
    mut players: Query<
        (
            &ActionState<PlayerActions>,
            &FacingDirection,
            Option<&Ability>,
            &mut SpriteAnimator,
        ),
        With<PlayerId>,
    >,
) {
    let spritesheet = sprite_sheets
        .get(player_sprite.0.clone())
        .expect("Expected player spritesheet to be loaded!");

    for (input, direction, ability, mut animator) in &mut players {
        let dir = vec2_to_direction(direction.0);
        let anim_handle = if ability.is_some() {
            match dir {
                Direction::Down => "down_slash",
                Direction::Up => "up_slash",
                Direction::Left => "left_slash",
                Direction::Right => "right_slash",
            }
        } else if input.pressed(PlayerActions::Move) {
            match dir {
                Direction::Down => "down",
                Direction::Up => "up",
                Direction::Left => "left",
                Direction::Right => "right",
            }
        } else {
            match dir {
                Direction::Down => "down_stand",
                Direction::Up => "up_stand",
                Direction::Left => "left_stand",
                Direction::Right => "right_stand",
            }
        };

        let anim_handle = spritesheet.get_anim_handle(anim_handle);

        if let Some(current) = animator.cur_anim() {
            if *current == anim_handle {
                continue;
            }
        }
        animator.set_anim(anim_handle);
        if let Some(ability) = ability {
            let elasped_ticks = u16::try_from(tick_manager.tick() - ability.start_tick)
                .expect("tick diff should be positive!");
            animator.set_cur_time(AnimTimestamp::Seconds(
                ticks_to_duration(elasped_ticks).as_secs_f32(),
            ));
        }
    }
}

pub(crate) fn draw_own_player(
    mut gizmos: Gizmos,
    global: Res<Global>,
    players: Query<(&PlayerId, &Position), (With<PlayerId>, With<Predicted>)>,
) {
    for (player, pos) in players.iter().filter(|(id, _)| id.0 == global.client_id) {
        draw_sprite(&mut gizmos, pos);
    }
}

fn center_camera_on_own_player(
    mut camera: Query<&mut Transform, With<Camera>>,
    player_pos: Query<&Position, (With<PlayerId>, With<Predicted>)>,
) {
    if let Ok(pos) = player_pos.get_single() {
        camera.single_mut().translation = pos.0.extend(0.0);
    }
}

use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use bevy::{
    app::{App, Plugin, PluginGroup, PluginGroupBuilder, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res, Resource},
    render::color::Color,
    text::TextStyle,
    ui::{node_bundles::TextBundle, AlignSelf, Style},
};

use lightyear::{
    client::{
        config::{ClientConfig, NetcodeConfig},
        interpolation::plugin::{InterpolationConfig, InterpolationDelay},
        plugin::{ClientPlugin, PluginConfig},
        prediction::plugin::PredictionConfig,
        resource::Authentication,
    },
    netcode::ClientId,
    prelude::LinkConditionerConfig,
    transport::io::{Io, IoConfig, TransportConfig},
};
use rand::Rng;

use crate::shared::{
    bundles,
    components::Position,
    config::{shared_config, KEY, PROTOCOL_ID},
    plugin::SharedPlugin,
    protocol::{protocol, ClientMut, ManaProtocol},
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
        let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(75),
            incoming_jitter: Duration::from_millis(10),
            incoming_loss: 0.02,
        };
        let io = Io::from_config(
            IoConfig::from_transport(transport_config).with_conditioner(link_conditioner),
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
    }
}

pub(crate) fn init(mut commands: Commands, mut client: ClientMut, global: Res<Global>) {
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
    commands.spawn(bundles::Player::new(
        global.client_id,
        Position { x: 100.0, y: 100.0 },
    ));
    let _ = client.connect();
}

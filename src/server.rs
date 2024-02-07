use std::net::Ipv4Addr;
use std::{net::SocketAddr, time::Duration};

use bevy::app::{App, FixedUpdate, Plugin, PluginGroup, PluginGroupBuilder};
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Query, Res};
use bevy::log::info;
use bevy::prelude::*;
use bevy_aseprite::anim::AsepriteAnimation;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::LinkConditionerConfig;
use lightyear::server::config::{NetcodeConfig, ServerConfig};
use lightyear::server::events::ComponentInsertEvent;
use lightyear::server::input_leafwing::LeafwingInputPlugin;
use lightyear::server::plugin::{PluginConfig, ServerPlugin};
use lightyear::shared::replication::components::NetworkTarget;
use lightyear::shared::sets::{FixedUpdateSet, MainSet};
use lightyear::shared::tick_manager::TickManager;
use lightyear::transport::io::{Io, IoConfig, TransportConfig};

use crate::shared::components::{PlayerId, Position};
use crate::shared::config::{shared_config, KEY, PROTOCOL_ID};
use crate::shared::plugin::{shared_movement_behaviour, SharedPlugin};
use crate::shared::protocol::{
    protocol, ManaProtocol, PlayerActions, Replicate, LINK_CONDITIONER, REPLICATION_GROUP,
};

pub const SERVER_PORT: u16 = 5000;

pub struct ServerPluginGroup {
    pub lightyear: ServerPlugin<ManaProtocol>,
}

impl ServerPluginGroup {
    pub fn new() -> Self {
        let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 8888);
        let transport_config = TransportConfig::UdpSocket(server_addr);
        let io = Io::from_config(
            IoConfig::from_transport(transport_config).with_conditioner(LINK_CONDITIONER),
        );

        let config = ServerConfig {
            shared: shared_config().clone(),
            netcode: NetcodeConfig {
                protocol_id: PROTOCOL_ID,
                private_key: Some(KEY),
                ..Default::default()
            },
            ..Default::default()
        };

        let plugin_config = PluginConfig::new(config, io, protocol());
        Self {
            lightyear: ServerPlugin::new(plugin_config),
        }
    }
}

impl PluginGroup for ServerPluginGroup {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(self.lightyear)
            .add(LeafwingInputPlugin::<ManaProtocol, PlayerActions>::default())
            .add(SharedPlugin)
            .add(ManaServerPlugin)
    }
}

pub struct ManaServerPlugin;

impl Plugin for ManaServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (movement).in_set(FixedUpdateSet::Main));
        app.add_systems(
            PreUpdate,
            (replicate_players).in_set(MainSet::ClientReplication),
        );
    }
}

pub(crate) fn movement(
    tick_manager: Res<TickManager>,
    mut action_query: Query<(Entity, &mut Position, &ActionState<PlayerActions>)>,
) {
    for (entity, position, action) in action_query.iter_mut() {
        shared_movement_behaviour(position, action);
        dbg!(tick_manager.tick(), entity);
    }
}

pub(crate) fn replicate_players(
    mut commands: Commands,
    mut player_spawn_reader: EventReader<ComponentInsertEvent<PlayerId>>,
) {
    for event in player_spawn_reader.read() {
        debug!("received player spawn event: {:?}", event);
        let client_id = event.context();
        let entity = event.entity();

        // for all cursors we have received, add a Replicate component so that we can start replicating it
        // to other clients

        if let Some(mut e) = commands.get_entity(entity) {
            let mut replicate = Replicate {
                // replication_target: NetworkTarget::AllExcept(vec![*client_id]),
                replication_target: NetworkTarget::All,
                interpolation_target: NetworkTarget::All,
                prediction_target: NetworkTarget::All,
                // interpolation_target: NetworkTarget::AllExcept(vec![*client_id]),
                // prediction_target: NetworkTarget::AllExcept(vec![*client_id]),
                replication_group: REPLICATION_GROUP,
                ..default()
            };
            replicate.add_target::<ActionState<PlayerActions>>(NetworkTarget::AllExcept(vec![
                *client_id,
            ]));

            e.insert((
                replicate,
                // not all physics components are replicated over the network, so add them on the server as well
                // PhysicsBundle::player(),
            ));
        }
    }
}

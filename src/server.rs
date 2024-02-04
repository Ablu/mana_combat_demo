use std::net::Ipv4Addr;
use std::{net::SocketAddr, time::Duration};

use bevy::app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use lightyear::prelude::LinkConditionerConfig;
use lightyear::server::config::{NetcodeConfig, ServerConfig};
use lightyear::server::plugin::{PluginConfig, ServerPlugin};
use lightyear::transport::io::{Io, IoConfig, TransportConfig};

use crate::shared::config::{shared_config, KEY, PROTOCOL_ID};
use crate::shared::plugin::SharedPlugin;
use crate::shared::protocol::{protocol, ManaProtocol};

pub const SERVER_PORT: u16 = 5000;

pub struct ServerPluginGroup {
    pub lightyear: ServerPlugin<ManaProtocol>,
}

impl ServerPluginGroup {
    pub fn new() -> Self {
        let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 8888);
        let transport_config = TransportConfig::UdpSocket(server_addr);
        let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(0),
            incoming_jitter: Duration::from_millis(0),
            incoming_loss: 0.0,
        };
        let io = Io::from_config(
            IoConfig::from_transport(transport_config).with_conditioner(link_conditioner),
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
            .add(SharedPlugin)
    }
}

use std::time::Duration;

use bevy::log::Level;
use lightyear::prelude::*;

const FRAME_HZ: f64 = 60.0;
const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub const PROTOCOL_ID: u64 = 0;
pub const KEY: Key = [0; 32];

pub fn shared_config() -> SharedConfig {
    SharedConfig {
        enable_replication: true,
        client_send_interval: Duration::default(),
        server_send_interval: Duration::from_secs_f64(1.0 / 32.0),
        // server_send_interval: Duration::from_millis(100),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        },
        log: LogConfig {
            level: Level::WARN,
            filter: "wgpu=error,wgpu_hal=error,naga=warn,bevy_app=info,bevy_render=warn,quinn=warn"
                .to_string(),
        },
    }
}

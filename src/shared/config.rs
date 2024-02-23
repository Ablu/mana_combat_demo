use std::time::Duration;

use bevy::log::Level;
use lightyear::prelude::*;

const FRAME_HZ: f64 = 60.0;
const TICK_DURATION_MILLISECONDS: u64 = 1000 / 64;

pub const PROTOCOL_ID: u64 = 0;
pub const KEY: Key = [0; 32];

pub fn duration_to_ticks(duration: Duration) -> Tick {
    Tick(
        u16::try_from(duration.as_millis() / u128::from(TICK_DURATION_MILLISECONDS))
            .expect("duration in ticks should not overflow Tick"),
    )
}

pub fn ticks_to_duration(ticks: u16) -> Duration {
    Duration::from_millis(u64::from(ticks) * TICK_DURATION_MILLISECONDS)
}

pub fn shared_config() -> SharedConfig {
    SharedConfig {
        enable_replication: true,
        client_send_interval: Duration::default(),
        server_send_interval: Duration::from_secs_f64(1.0 / 32.0),
        // server_send_interval: Duration::from_millis(100),
        tick: TickConfig {
            tick_duration: Duration::from_millis(TICK_DURATION_MILLISECONDS),
        },
        log: LogConfig {
            level: Level::WARN,
            filter: "wgpu=error,wgpu_hal=error,naga=warn,bevy_app=info,bevy_render=warn,quinn=warn"
                .to_string(),
        },
    }
}

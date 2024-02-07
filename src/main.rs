use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use clap::Parser;

mod client;
mod helpers;
mod server;
mod shared;

use client::ClientPluginGroup;
use server::ServerPluginGroup;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<helpers::tiled::TiledMap> = asset_server.load("main.tmx");

    commands.spawn(helpers::tiled::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

#[derive(Parser, PartialEq, Debug)]
enum Cli {
    Server {},
    Client {},
}

fn main() {
    let cli = Cli::parse();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Tiled Map Editor Example"),
                    ..Default::default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    );

    // app.add_plugins(WorldInspectorPlugin::new());
    match cli {
        Cli::Server {} => app.add_plugins(ServerPluginGroup::new()),
        Cli::Client {} => app.add_plugins(ClientPluginGroup::new()),
    };

    app.add_plugins(TilemapPlugin)
        .add_plugins(helpers::tiled::TiledMapPlugin)
        .add_systems(Startup, startup)
        .run();
}

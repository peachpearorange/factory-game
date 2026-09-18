mod catalog;
mod construction;
mod machine;
mod ore;
mod player;
mod sdf;
mod store;
mod ui;
mod world;

use avian3d::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Factory Game".into(),
                    resolution: (1440, 810).into(),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default().with_collision_hooks::<machine::ConveyorHooks>(),
        ))
        .add_plugins((
            world::plugin,
            ore::plugin,
            catalog::plugin,
            machine::plugin,
            construction::plugin,
            player::plugin,
            store::plugin,
            ui::plugin,
        ))
        .run();
}

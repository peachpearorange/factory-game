mod block;
mod catalog;
mod construction;
mod icon;
mod machine;
mod menu;
mod ore;
mod player;
mod sdf;
mod showcase;
mod store;
mod style;
mod texture;
mod ui;
mod world;

use {avian3d::prelude::*, bevy::prelude::*};

pub fn env_secs(key: &str) -> Option<f32> {
  std::env::var(key).ok().and_then(|value| value.parse().ok())
}

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
      PhysicsPlugins::default().with_collision_hooks::<machine::ConveyorHooks>()
    ))
    .add_plugins((
      style::plugin,
      world::plugin,
      ore::plugin,
      catalog::plugin,
      machine::plugin,
      construction::plugin,
      player::plugin,
      store::plugin,
      ui::plugin,
      menu::plugin,
      showcase::plugin
    ))
    .run();
}

mod airdrop;
mod boat;
mod catalog;
mod construction;
mod icon;
mod machine;
mod material;
mod menu;
mod opts;
mod ore;
mod part;
mod player;
mod sdf;
mod showcase;
mod store;
mod style;
mod texture;
mod ui;
mod world;

use {avian3d::prelude::*, bevy::prelude::*};
#[cfg(target_arch = "wasm32")]
use {bevy::winit::{UpdateMode, WinitSettings},
     std::time::Duration};

fn main() {
  let mut app = App::new();
  app
    .add_plugins((
      DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
          title: "Factory Game".into(),
          resolution: (1440, 810).into(),
          canvas: Some("#game".into()),
          fit_canvas_to_parent: true,
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
      boat::plugin,
      airdrop::plugin,
      ui::plugin,
      menu::plugin,
      showcase::plugin
    ));

  #[cfg(target_arch = "wasm32")]
  app.insert_resource(WinitSettings {
    focused_mode: UpdateMode::Reactive {
      wait: Duration::from_micros(16_667),
      react_to_device_events: false,
      react_to_user_events: false,
      react_to_window_events: false
    },
    unfocused_mode: UpdateMode::reactive_low_power(Duration::from_millis(200))
  });

  app.run();
}

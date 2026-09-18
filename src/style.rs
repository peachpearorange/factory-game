use bevy::prelude::*;

pub const TEXT: Color = Color::srgb(0.93, 0.94, 0.96);
pub const TEXT_DIM: Color = Color::srgb(0.55, 0.58, 0.63);
pub const INK: Color = Color::srgb(0.06, 0.06, 0.07);

pub const PANEL: Color = Color::srgba(0.03, 0.04, 0.06, 0.95);
pub const POPUP: Color = Color::srgba(0.02, 0.03, 0.05, 0.9);
pub const TAG: Color = Color::srgba(0.02, 0.03, 0.05, 0.66);
pub const SHADE: Color = Color::srgba(0.01, 0.02, 0.04, 0.72);
pub const SLOT: Color = Color::srgb(0.10, 0.11, 0.13);
pub const BUTTON: Color = Color::srgb(0.07, 0.08, 0.11);
pub const BUTTON_ARMED: Color = Color::srgb(0.32, 0.07, 0.08);
pub const MUTED: Color = Color::srgb(0.42, 0.44, 0.48);
pub const VEIL: Color = Color::srgba(0.08, 0.08, 0.10, 0.85);

pub const ORE: Color = Color::srgb(0.98, 0.70, 0.16);
pub const CASH: Color = Color::srgb(0.58, 0.99, 0.62);
pub const FIRE: Color = Color::srgb(1.0, 0.45, 0.15);
pub const WATER: Color = Color::srgb(0.35, 0.65, 1.0);
pub const DECAY: Color = Color::srgb(0.45, 0.95, 0.35);

pub const ITEMS: Color = Color::srgb(0.36, 0.86, 0.99);
pub const STORE: Color = Color::srgb(0.46, 0.93, 0.46);
pub const DELETE: Color = Color::srgb(0.98, 0.31, 0.29);
pub const SHOT: Color = Color::srgb(0.99, 0.78, 0.34);
pub const GRANTED: Color = Color::srgb(0.48, 0.96, 0.52);
pub const DENIED: Color = Color::srgb(1.0, 0.56, 0.30);
pub const SEALED: Color = Color::srgb(0.66, 0.72, 0.86);

pub const TITLE: f32 = 23.0;
pub const READOUT: f32 = 26.0;
pub const BODY: f32 = 15.0;
pub const SMALL: f32 = 13.0;
pub const TINY: f32 = 12.0;

const SCALE: f32 = 1.14;

#[derive(Resource)]
pub struct Bold(pub Handle<Font>);

fn load_bold(mut commands: Commands, assets: Res<AssetServer>) {
  commands.insert_resource(Bold(assets.load("fonts/NotoSans-Bold.ttf")));
}

pub fn tinted(text: &str, size: f32, color: Color) -> impl Bundle {
  (
    Text::new(text),
    TextFont { font_size: FontSize::Px(size * SCALE), ..default() },
    TextColor(color)
  )
}

pub fn label(text: &str, size: f32) -> impl Bundle { tinted(text, size, TEXT) }

pub fn heavy(text: &str, size: f32, color: Color, bold: &Bold) -> impl Bundle {
  (
    Text::new(text),
    TextFont {
      font: FontSource::Handle(bold.0.clone()),
      font_size: FontSize::Px(size * SCALE),
      ..default()
    },
    TextColor(color)
  )
}

pub fn plugin(app: &mut App) { app.add_systems(PreStartup, load_bold); }

#![allow(dead_code)]

use {crate::texture,
     bevy::{math::Affine2, prelude::*, render::render_resource::Face},
     enum_assoc::Assoc};

#[derive(Clone, Copy, PartialEq, Eq, Assoc)]
#[func(pub const fn roughness(self) -> f32)]
#[func(pub const fn metallic(self) -> f32)]
#[func(pub const fn reflectance(self) -> f32)]
#[func(pub const fn grain(self) -> Option<Grain>)]
#[func(pub const fn tiling(self) -> Vec2 { Vec2::ONE })]
#[func(pub const fn clear(self) -> bool { false })]
#[func(pub const fn limp(self) -> bool { false })]
pub enum Surface {
  #[assoc(roughness = 0.6, metallic = 0.35, reflectance = 0.5)]
  Painted,
  #[assoc(roughness = 0.42, metallic = 0.0, reflectance = 0.38)]
  Plastic,
  #[assoc(roughness = 0.95, metallic = 0.3, reflectance = 0.15)]
  Rock,
  #[assoc(
    roughness = 0.88,
    metallic = 0.0,
    reflectance = 0.14,
    grain = Grain::Wood,
    tiling = texture::GRAIN
  )]
  Wood,
  #[assoc(
    roughness = 0.80,
    metallic = 0.0,
    reflectance = 0.20,
    grain = Grain::Plank,
    tiling = texture::PLANK
  )]
  Planked,
  #[assoc(roughness = 0.24, metallic = 0.95, reflectance = 0.72)]
  Metal,
  #[assoc(roughness = 0.08, metallic = 0.0, reflectance = 0.9, clear = true)]
  Glass,
  #[assoc(roughness = 0.95, metallic = 0.0, reflectance = 0.05, limp = true)]
  Cloth
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Grain {
  Wood,
  Plank
}

#[derive(Clone, Copy, PartialEq)]
pub struct Coat {
  pub surface: Surface,
  pub glow: LinearRgba
}

#[derive(Clone, Copy, PartialEq)]
pub struct Finish {
  pub surface: Surface,
  pub color: LinearRgba,
  pub glow: LinearRgba
}

impl Finish {
  pub const fn new(surface: Surface, color: LinearRgba) -> Self {
    Self { surface, color, glow: LinearRgba::BLACK }
  }

  pub const fn tinted(self, color: LinearRgba) -> Self { Self { color, ..self } }

  pub const fn lit(self, glow: LinearRgba) -> Self { Self { glow, ..self } }

  pub const fn shaded(self, amount: f32) -> Self {
    let LinearRgba { red, green, blue, alpha } = self.color;
    self.tinted(LinearRgba::new(red * amount, green * amount, blue * amount, alpha))
  }

  pub const fn veiled(self, alpha: f32) -> Self {
    let LinearRgba { red, green, blue, .. } = self.color;
    self.tinted(LinearRgba::new(red, green, blue, alpha))
  }

  pub const fn coat(self) -> Coat { Coat { surface: self.surface, glow: self.glow } }
}

const fn rgb(red: f32, green: f32, blue: f32) -> LinearRgba {
  LinearRgba::rgb(red, green, blue)
}

pub const PLAIN: Finish = Finish::new(Surface::Painted, rgb(1.0, 1.0, 1.0));
pub const SHELL: Finish = Finish::new(Surface::Plastic, rgb(1.0, 1.0, 1.0));
pub const PLATE: Finish = Finish::new(Surface::Metal, rgb(1.0, 1.0, 1.0));
pub const SANDED: Finish = Finish::new(Surface::Wood, rgb(1.0, 1.0, 1.0));
pub const CASING: Finish = Finish::new(Surface::Painted, rgb(0.86, 0.87, 0.90));
pub const TIMBER: Finish = Finish::new(Surface::Wood, rgb(0.58, 0.40, 0.24));
pub const BOARD: Finish = Finish::new(Surface::Planked, rgb(0.58, 0.42, 0.26));
pub const PINE: Finish = Finish::new(Surface::Wood, rgb(0.74, 0.58, 0.36));
pub const STONE: Finish = Finish::new(Surface::Rock, rgb(0.33, 0.31, 0.29));
pub const GRIT: Finish = Finish::new(Surface::Rock, rgb(0.44, 0.42, 0.39));
pub const SHADOW: Finish = Finish::new(Surface::Rock, rgb(0.06, 0.05, 0.05));
pub const STEEL: Finish = Finish::new(Surface::Metal, rgb(0.44, 0.46, 0.50));
pub const IRON: Finish = Finish::new(Surface::Metal, rgb(0.30, 0.31, 0.34));
pub const BRASS: Finish = Finish::new(Surface::Metal, rgb(0.74, 0.55, 0.20));
pub const COPPER: Finish = Finish::new(Surface::Metal, rgb(0.72, 0.36, 0.18));
pub const GOLD: Finish = Finish::new(Surface::Metal, rgb(0.96, 0.74, 0.16));
pub const SOOT: Finish = Finish::new(Surface::Painted, rgb(0.11, 0.11, 0.12));
pub const CINDER: Finish = Finish::new(Surface::Rock, rgb(1.0, 0.42, 0.07));
pub const RUBBER: Finish = Finish::new(Surface::Plastic, rgb(0.13, 0.14, 0.16));
pub const BARN: Finish = Finish::new(Surface::Painted, rgb(0.92, 0.24, 0.17));
pub const SHINGLE: Finish = Finish::new(Surface::Painted, rgb(0.96, 0.96, 0.94));
pub const STRAW: Finish = Finish::new(Surface::Painted, rgb(0.92, 0.76, 0.36));
pub const GLASS: Finish =
  Finish::new(Surface::Glass, LinearRgba::new(0.62, 0.78, 0.86, 0.34));
pub const CANVAS: Finish = Finish::new(Surface::Cloth, rgb(0.90, 0.88, 0.82));
pub const SILK: Finish = Finish::new(Surface::Cloth, rgb(0.94, 0.94, 0.90));
pub const SIGNAL: Finish = Finish::new(Surface::Cloth, rgb(0.96, 0.38, 0.10));
pub const ROPE: Finish = Finish::new(Surface::Cloth, rgb(0.58, 0.50, 0.34));
pub const EMBERS: Finish =
  Finish::new(Surface::Painted, rgb(1.0, 0.42, 0.07)).lit(rgb(2.6, 0.72, 0.10));

#[derive(Resource)]
pub struct Coats {
  wood: Handle<Image>,
  planks: Handle<Image>,
  made: Vec<(Coat, Handle<StandardMaterial>)>
}

impl Coats {
  pub fn new(images: &mut Assets<Image>) -> Self {
    Self {
      wood: images.add(texture::wood()),
      planks: images.add(texture::planks()),
      made: Vec::new()
    }
  }

  pub fn of(
    &mut self,
    coat: Coat,
    materials: &mut Assets<StandardMaterial>
  ) -> Handle<StandardMaterial> {
    self
      .made
      .iter()
      .find(|(each, _)| *each == coat)
      .map(|(_, handle)| handle.clone())
      .unwrap_or_else(|| {
        let Coat { surface, glow } = coat;
        let handle = materials.add(StandardMaterial {
          base_color: Color::WHITE,
          emissive: glow,
          base_color_texture: surface.grain().map(|grain| match grain {
            Grain::Wood => self.wood.clone(),
            Grain::Plank => self.planks.clone()
          }),
          uv_transform: Affine2::from_scale(Vec2::ONE / surface.tiling()),
          perceptual_roughness: surface.roughness(),
          reflectance: surface.reflectance(),
          metallic: surface.metallic(),
          alpha_mode: surface.clear().then_some(AlphaMode::Blend).unwrap_or_default(),
          double_sided: surface.limp(),
          cull_mode: (!surface.limp()).then_some(Face::Back),
          ..default()
        });
        self.made.push((coat, handle.clone()));
        handle
      })
  }
}

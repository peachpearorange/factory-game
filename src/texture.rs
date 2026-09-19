use {bevy::{asset::RenderAssetUsages,
            image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
            prelude::*,
            render::render_resource::{Extent3d, TextureDimension, TextureFormat}},
     std::f32::consts::TAU};

const SIZE: u32 = 256;

pub const GRAIN: Vec2 = Vec2::new(0.13, 0.62);

fn hash(x: i32, y: i32, across: i32, along: i32) -> f32 {
  let seed = (x.rem_euclid(across) as u32).wrapping_mul(2_654_435_761)
    ^ (y.rem_euclid(along) as u32).wrapping_mul(2_246_822_519);
  let mixed = seed ^ (seed >> 13);
  (mixed.wrapping_mul(1_274_126_177) >> 8) as f32 / (1 << 24) as f32
}

fn noise(u: f32, v: f32, across: i32, along: i32) -> f32 {
  let (x, y) = (u * across as f32, v * along as f32);
  let (column, row) = (x.floor() as i32, y.floor() as i32);
  let ease = |t: f32| t * t * (3.0 - 2.0 * t);
  let (fx, fy) = (ease(x - column as f32), ease(y - row as f32));
  let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
  mix(
    mix(hash(column, row, across, along), hash(column + 1, row, across, along), fx),
    mix(
      hash(column, row + 1, across, along),
      hash(column + 1, row + 1, across, along),
      fx
    ),
    fy
  )
}

fn tiling_as(format: TextureFormat, pixels: impl Fn(f32, f32) -> [u8; 4]) -> Image {
  let data = (0..SIZE * SIZE)
    .flat_map(|index| {
      pixels((index % SIZE) as f32 / SIZE as f32, (index / SIZE) as f32 / SIZE as f32)
    })
    .collect();
  let mut image = Image::new(
    Extent3d { width: SIZE, height: SIZE, depth_or_array_layers: 1 },
    TextureDimension::D2,
    data,
    format,
    RenderAssetUsages::RENDER_WORLD
  );
  image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
    address_mode_u: ImageAddressMode::Repeat,
    address_mode_v: ImageAddressMode::Repeat,
    ..ImageSamplerDescriptor::linear()
  });
  image
}

fn tiling(pixels: impl Fn(f32, f32) -> [u8; 4]) -> Image {
  tiling_as(TextureFormat::Rgba8UnormSrgb, pixels)
}

pub fn concrete() -> Image {
  tiling(|u, v| {
    let grain = 0.55 * noise(u, v, 112, 112)
      + 0.30 * noise(u, v, 40, 40)
      + 0.15 * noise(u, v, 11, 11);
    let shade = (0.96 + 0.075 * (grain - 0.5)).clamp(0.0, 1.0);
    [(shade * 255.0) as u8, (shade * 254.0) as u8, (shade * 250.0) as u8, 255]
  })
}

pub const PLANK: Vec2 = Vec2::new(0.96, 0.96);
const BOARDS: f32 = 4.0;

pub fn planks() -> Image {
  tiling(|u, v| {
    let seam = |t: f32| {
      let edge = (t * BOARDS).fract();
      (edge.min(1.0 - edge) * BOARDS * 7.0).min(1.0)
    };
    let fibre = 0.62 * noise(u, v, 70, 14) + 0.38 * noise(u, v, 18, 6);
    let board = hash((v * BOARDS) as i32, 0, BOARDS as i32, 1);
    let shade = (0.86 + 0.09 * fibre + 0.05 * board) * (0.76 + 0.24 * seam(v));
    let level = (shade.clamp(0.0, 1.0) * 255.0) as u8;
    [level, level, (level as f32 * 0.98) as u8, 255]
  })
}

pub const SWELL: Vec2 = Vec2::new(16.0, 16.0);
const CRESTS: [(f32, f32, f32); 3] =
  [(1.0, 0.0, 1.0), (0.0, 1.0, 0.62), (2.0, 3.0, 0.20)];
const CHOP: [(i32, f32); 2] = [(22, 0.42), (61, 0.22)];
const SWELL_TILT: f32 = 0.028;

pub fn swell() -> Image {
  let step = 1.0 / SIZE as f32;
  tiling_as(TextureFormat::Rgba8Unorm, move |u, v| {
    let rolling = CRESTS.into_iter().fold(Vec2::ZERO, |slope, (fu, fv, amplitude)| {
      let wave = (TAU * (fu * u + fv * v)).cos() * amplitude * TAU;
      slope + Vec2::new(fu * wave, fv * wave)
    });
    let chop = CHOP.into_iter().fold(Vec2::ZERO, |slope, (grid, amplitude)| {
      let lean = |du: f32, dv: f32| {
        noise(u + du, v + dv, grid, grid) - noise(u - du, v - dv, grid, grid)
      };
      slope + Vec2::new(lean(step, 0.0), lean(0.0, step)) * amplitude / (2.0 * step)
    });
    let normal = (-(rolling + chop) * SWELL_TILT).extend(1.0).normalize();
    let level = |axis: f32| ((0.5 + 0.5 * axis) * 255.0) as u8;
    [level(normal.x), level(normal.y), level(normal.z), 255]
  })
}

pub fn wood() -> Image {
  const PALE: Vec3 = Vec3::new(0.56, 0.40, 0.23);
  const DARK: Vec3 = Vec3::new(0.19, 0.11, 0.05);
  const KNOTS: [(f32, f32, f32); 3] =
    [(0.24, 0.17, 1.0), (0.71, 0.54, 0.8), (0.42, 0.83, 0.6)];
  const KNOT: Vec2 = Vec2::new(0.16, 0.16 * GRAIN.x / GRAIN.y);
  tiling(|u, v| {
    let sway =
      2.1 * noise(u, v, 3, 3) + 1.1 * noise(u, v, 6, 7) + 0.4 * noise(u, v, 11, 17);
    let rings = 0.5 + 0.5 * ((u * 7.0 + sway) * TAU).sin();
    let fibre = 0.62 * noise(u, v, 210, 34) + 0.38 * noise(u, v, 420, 90);
    let sap = noise(u, v, 4, 9);
    let grain = 0.24 + 0.44 * rings.powi(3) + 0.17 * fibre + 0.19 * sap;
    let shade = KNOTS.into_iter().fold(grain, |shade, (cu, cv, weight)| {
      let spread = ((u - cu) / KNOT.x).hypot((v - cv) / KNOT.y);
      let swirl = 0.20 + 0.30 * (spread * 9.0).cos();
      shade + weight * (1.0 - spread.min(1.0)).powi(2) * (swirl - shade)
    });

    let color = DARK.lerp(PALE, shade.clamp(0.0, 1.0));
    [(color.x * 255.0) as u8, (color.y * 255.0) as u8, (color.z * 255.0) as u8, 255]
  })
}

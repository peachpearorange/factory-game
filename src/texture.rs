use bevy::{asset::RenderAssetUsages,
           image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
           prelude::*,
           render::render_resource::{Extent3d, TextureDimension, TextureFormat}};

const SIZE: u32 = 256;

fn hash(x: i32, y: i32, cells: i32) -> f32 {
  let seed = (x.rem_euclid(cells) as u32).wrapping_mul(2_654_435_761)
    ^ (y.rem_euclid(cells) as u32).wrapping_mul(2_246_822_519);
  let mixed = seed ^ (seed >> 13);
  (mixed.wrapping_mul(1_274_126_177) >> 8) as f32 / (1 << 24) as f32
}

fn noise(u: f32, v: f32, cells: i32) -> f32 {
  let (x, y) = (u * cells as f32, v * cells as f32);
  let (column, row) = (x.floor() as i32, y.floor() as i32);
  let ease = |t: f32| t * t * (3.0 - 2.0 * t);
  let (fx, fy) = (ease(x - column as f32), ease(y - row as f32));
  let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
  mix(
    mix(hash(column, row, cells), hash(column + 1, row, cells), fx),
    mix(hash(column, row + 1, cells), hash(column + 1, row + 1, cells), fx),
    fy
  )
}

fn tiling(pixels: impl Fn(f32, f32) -> [u8; 4]) -> Image {
  let data = (0..SIZE * SIZE)
    .flat_map(|index| {
      pixels((index % SIZE) as f32 / SIZE as f32, (index / SIZE) as f32 / SIZE as f32)
    })
    .collect();
  let mut image = Image::new(
    Extent3d { width: SIZE, height: SIZE, depth_or_array_layers: 1 },
    TextureDimension::D2,
    data,
    TextureFormat::Rgba8UnormSrgb,
    RenderAssetUsages::RENDER_WORLD
  );
  image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
    address_mode_u: ImageAddressMode::Repeat,
    address_mode_v: ImageAddressMode::Repeat,
    ..ImageSamplerDescriptor::linear()
  });
  image
}

pub fn concrete() -> Image {
  tiling(|u, v| {
    let grain = 0.55 * noise(u, v, 112) + 0.30 * noise(u, v, 40) + 0.15 * noise(u, v, 11);
    let shade = (0.96 + 0.075 * (grain - 0.5)).clamp(0.0, 1.0);
    [(shade * 255.0) as u8, (shade * 254.0) as u8, (shade * 250.0) as u8, 255]
  })
}

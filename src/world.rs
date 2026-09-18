use {crate::{sdf, texture},
     avian3d::prelude::*,
     bevy::{light::{CascadeShadowConfigBuilder, NotShadowCaster, light_consts::lux},
            math::Affine2,
            prelude::*},
     std::f32::consts::TAU};

pub const GROUND: f32 = 0.0;
pub const PLATFORM_HALF: Vec3 = Vec3::new(21.0, 0.6, 21.0);
const ISLAND_RADIUS: f32 = 44.0;
const ISLAND_DEPTH: f32 = 8.0;
const SKY_RADIUS: f32 = 420.0;
const STAR_COUNT: usize = 520;
const CONCRETE_TILE: f32 = 8.0;

#[derive(Resource, Default)]
pub struct Daylight(pub f32);

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct SunDisc;

#[derive(Resource)]
struct StarField(Handle<StandardMaterial>);

#[derive(Resource)]
struct DayLength(f32);

impl Default for DayLength {
  fn default() -> Self { Self(crate::env_secs("FACTORY_DAY").unwrap_or(240.0)) }
}

fn platform_shape() -> fidget::context::Tree { sdf::rounded_box(PLATFORM_HALF, 0.3) }

fn star_at(index: usize) -> (Vec3, f32) {
  let scatter = |salt: u32| {
    let seed = (index as u32)
      .wrapping_mul(1_664_525)
      .wrapping_add(salt.wrapping_mul(1_013_904_223));
    let mixed = seed ^ (seed >> 15);
    (mixed.wrapping_mul(2_246_822_519) >> 8) as f32 / (1 << 24) as f32
  };
  let height = 0.05 + 0.95 * scatter(7);
  let ring = (1.0 - height * height).sqrt();
  let angle = TAU * scatter(31);
  (
    Vec3::new(ring * angle.cos(), height, ring * angle.sin()) * SKY_RADIUS,
    0.9 + 1.4 * scatter(53)
  )
}

fn spawn_sky(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>
) {
  let star_mesh = meshes.add(Sphere::new(1.0).mesh().ico(1).expect("star mesh"));
  let star_material = materials.add(StandardMaterial {
    base_color: Color::srgba(0.92, 0.95, 1.0, 0.0),
    unlit: true,
    alpha_mode: AlphaMode::Blend,
    ..default()
  });

  for index in 0..STAR_COUNT {
    let (position, size) = star_at(index);
    commands.spawn((
      Mesh3d(star_mesh.clone()),
      MeshMaterial3d(star_material.clone()),
      NotShadowCaster,
      Transform::from_translation(position).with_scale(Vec3::splat(size))
    ));
  }

  let mut halo = |radius: f32, glow: LinearRgba, alpha: f32| {
    (
      Mesh3d(meshes.add(Sphere::new(radius).mesh().ico(3).expect("sun mesh"))),
      MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, alpha),
        emissive: glow,
        alpha_mode: AlphaMode::Add,
        ..default()
      })),
      NotShadowCaster
    )
  };

  commands.spawn((
    Name::new("Sun Disc"),
    SunDisc,
    halo(12.0, LinearRgba::rgb(260.0, 210.0, 130.0), 1.0),
    children![
      halo(19.0, LinearRgba::rgb(40.0, 26.0, 11.0), 1.0),
      halo(31.0, LinearRgba::rgb(9.0, 5.5, 2.2), 1.0),
      halo(52.0, LinearRgba::rgb(2.2, 1.3, 0.5), 1.0),
    ]
  ));

  commands.insert_resource(StarField(star_material));
}

fn spawn_world(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut images: ResMut<Assets<Image>>
) {
  commands.spawn((
    Name::new("Island"),
    RigidBody::Static,
    Collider::cylinder(ISLAND_RADIUS, ISLAND_DEPTH),
    Friction::new(0.9),
    Mesh3d(meshes.add(Cylinder::new(ISLAND_RADIUS, ISLAND_DEPTH))),
    MeshMaterial3d(materials.add(StandardMaterial {
      base_color: Color::srgb(0.24, 0.42, 0.18),
      perceptual_roughness: 1.0,
      ..default()
    })),
    Transform::from_xyz(0.0, GROUND - ISLAND_DEPTH / 2.0 - PLATFORM_HALF.y * 2.0, 0.0)
  ));

  commands.spawn((
    Name::new("Platform"),
    RigidBody::Static,
    Collider::cuboid(PLATFORM_HALF.x * 2.0, PLATFORM_HALF.y * 2.0, PLATFORM_HALF.z * 2.0),
    Friction::new(0.9),
    Mesh3d(meshes.add(sdf::bake(platform_shape(), sdf::Bounds::new(22.0, 6)))),
    MeshMaterial3d(materials.add(StandardMaterial {
      base_color: Color::srgb(0.62, 0.61, 0.59),
      base_color_texture: Some(images.add(texture::concrete())),
      uv_transform: Affine2::from_scale(Vec2::splat(1.0 / CONCRETE_TILE)),
      perceptual_roughness: 0.95,
      ..default()
    })),
    Transform::from_xyz(0.0, GROUND - PLATFORM_HALF.y, 0.0)
  ));

  commands.spawn((
    Name::new("Sun"),
    Sun,
    DirectionalLight {
      color: Color::srgb(1.0, 0.96, 0.88),
      illuminance: lux::AMBIENT_DAYLIGHT,
      shadow_maps_enabled: true,
      ..default()
    },
    CascadeShadowConfigBuilder {
      num_cascades: 4,
      maximum_distance: 90.0,
      first_cascade_far_bound: 12.0,
      ..default()
    }
    .build(),
    Transform::default().looking_to(Vec3::new(-0.4, -0.8, -0.45), Vec3::Y)
  ));
}

fn cycle_day(
  time: Res<Time>,
  length: Res<DayLength>,
  field: Res<StarField>,
  sun: Single<(&mut Transform, &mut DirectionalLight), With<Sun>>,
  disc: Single<(&mut Transform, &mut Visibility), (With<SunDisc>, Without<Sun>)>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut daylight: ResMut<Daylight>,
  mut ambient: ResMut<GlobalAmbientLight>,
  mut clear: ResMut<ClearColor>
) {
  let (mut transform, mut light) = sun.into_inner();
  let angle = time.elapsed_secs() / length.0 * TAU;
  let elevation = angle.sin();
  let toward_sun = Vec3::new(0.35, elevation, angle.cos()).normalize();
  *transform = Transform::default().looking_to(-toward_sun, Vec3::Y);

  daylight.0 = elevation.max(0.0);
  let day = daylight.0;
  let night = (1.0 - elevation * 4.0).clamp(0.0, 1.0);

  let (mut disc_transform, mut disc_visibility) = disc.into_inner();
  disc_transform.translation = toward_sun * SKY_RADIUS;
  *disc_visibility =
    (elevation > -0.12).then_some(Visibility::Inherited).unwrap_or(Visibility::Hidden);

  if let Some(mut stars) = materials.get_mut(&field.0) {
    stars.base_color = stars.base_color.with_alpha(night);
  }

  light.illuminance = lux::AMBIENT_DAYLIGHT * day + 2600.0;
  light.color = Color::srgb(0.62 + 0.38 * day, 0.70 + 0.26 * day, 0.95 - 0.07 * day);
  ambient.brightness = 120.0 * day + 55.0;
  clear.0 = Color::srgb(0.05 + 0.41 * day, 0.07 + 0.54 * day, 0.16 + 0.72 * day);
}

pub fn plugin(app: &mut App) {
  app
    .init_resource::<DayLength>()
    .init_resource::<Daylight>()
    .add_systems(Startup, (spawn_world, spawn_sky))
    .add_systems(Update, cycle_day);
}

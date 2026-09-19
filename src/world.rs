use {crate::{sdf, texture},
     avian3d::prelude::*,
     bevy::{color::Mix,
            light::{CascadeShadowConfigBuilder, NotShadowCaster, light_consts::lux},
            math::Affine2,
            prelude::*},
     std::f32::consts::TAU};

pub const GROUND: f32 = 0.0;
pub const PLATFORM_HALF: Vec3 = Vec3::new(21.0, 0.6, 21.0);
const ISLAND_TOP: f32 = GROUND - PLATFORM_HALF.y * 2.0 + 0.35;
const ISLAND_FLOOR: f32 = -13.0;
const SEA_LEVEL: f32 = ISLAND_TOP - 4.2;
const SHORE_RADIUS: f32 = 61.0;
const SKY_RADIUS: f32 = 420.0;
const SUN_RADIUS: f32 = 9.0;
const STAR_COUNT: usize = 520;
const CONCRETE_TILE: f32 = 8.0;

const GRASS: LinearRgba = LinearRgba::rgb(0.13, 0.36, 0.09);
const SAND: LinearRgba = LinearRgba::rgb(0.46, 0.39, 0.23);
const ROCK: LinearRgba = LinearRgba::rgb(0.26, 0.27, 0.25);
const SEABED: LinearRgba = LinearRgba::rgb(0.14, 0.17, 0.15);

#[derive(Resource, Default)]
pub struct Daylight(pub f32);

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct SunDisc;

#[derive(Component)]
struct SkyDome;

#[derive(Component)]
struct Ocean;

const DRIFT: Vec2 = Vec2::new(0.031, 0.017);
const DEEP: Color = Color::srgb(0.008, 0.032, 0.075);
const SHALLOW: Color = Color::srgb(0.06, 0.30, 0.42);

fn stir_ocean(
  time: Res<Time>,
  daylight: Res<Daylight>,
  ocean: Single<&MeshMaterial3d<StandardMaterial>, With<Ocean>>,
  mut materials: ResMut<Assets<StandardMaterial>>
) {
  if let Some(mut water) = materials.get_mut(&ocean.0) {
    let day = daylight.0.sqrt();
    water.uv_transform.translation = DRIFT * time.elapsed_secs();
    water.base_color = DEEP.mix(&SHALLOW, day);
    water.reflectance = 0.30 + 0.55 * day;
    water.perceptual_roughness = 0.34 - 0.20 * day;
  }
}

#[derive(Resource)]
struct StarField(Handle<StandardMaterial>);

#[derive(Resource)]
pub struct DayClock {
  length: f32,
  start: f32
}

impl DayClock {
  fn angle(&self, elapsed: f32) -> f32 { (self.start + elapsed) / self.length * TAU }

  pub fn pin(&mut self, phase: f32, elapsed: f32) {
    self.start = phase * self.length - elapsed;
  }
}

impl Default for DayClock {
  fn default() -> Self {
    Self {
      length: crate::env_secs("FACTORY_DAY").unwrap_or(240.0),
      start: crate::env_secs("FACTORY_TIME").unwrap_or(0.0)
    }
  }
}

fn platform_shape() -> fidget::context::Tree { sdf::rounded_box(PLATFORM_HALF, 0.3) }

fn island_shape() -> fidget::context::Tree {
  let plateau = |reach: f32, angle: f32, radius: f32| {
    sdf::at(
      sdf::rounded_cylinder(radius, -ISLAND_FLOOR / 2.0, 3.2),
      Vec3::new(reach * angle.cos(), ISLAND_FLOOR / 2.0, reach * angle.sin())
    )
  };
  let terrace = |radius: f32, top: f32| {
    sdf::at(
      sdf::rounded_cylinder(radius, (top - ISLAND_FLOOR) / 2.0, 2.6),
      Vec3::new(0.0, (top + ISLAND_FLOOR) / 2.0, 0.0)
    )
  };
  let crag = |x: f32, z: f32, peak: f32, half: Vec2, spin: f32| {
    let block = |lift: f32, shrink: f32, twist: f32| {
      sdf::at(
        sdf::rotate_y(
          sdf::rounded_box(
            Vec3::new(
              half.x * shrink,
              (peak * lift - ISLAND_FLOOR) / 2.0,
              half.y * shrink
            ),
            1.2
          ),
          spin + twist
        ),
        Vec3::new(x, (peak * lift + ISLAND_FLOOR) / 2.0, z)
      )
    };
    sdf::union([block(0.5, 1.0, 0.0), block(1.0, 0.6, 0.75)])
  };

  let land = sdf::union([
    plateau(0.0, 0.0, 43.0),
    plateau(23.0, 0.6, 25.0),
    plateau(26.0, 2.3, 23.0),
    plateau(22.0, 3.8, 26.0),
    plateau(27.0, 5.2, 22.0)
  ]);
  let skirt = sdf::smooth_union(
    sdf::smooth_union(terrace(48.0, -3.0), terrace(SHORE_RADIUS - 6.0, -7.5), 3.4),
    terrace(SHORE_RADIUS, -11.0),
    3.4
  );
  let crags = sdf::union([
    crag(34.0, -13.0, 7.0, Vec2::new(9.0, 7.0), 0.4),
    crag(-34.0, 27.0, 10.0, Vec2::new(7.0, 8.5), 1.1),
    crag(5.0, -38.0, 5.5, Vec2::new(12.0, 8.0), -0.25),
    crag(-15.0, 35.0, 8.0, Vec2::new(8.0, 7.0), 0.7),
    crag(41.0, 17.0, 12.0, Vec2::new(6.5, 6.0), 2.2)
  ]);

  sdf::smooth_union(sdf::smooth_union(land, skirt, 3.0), crags, 2.2)
}

fn island_paint(position: Vec3, normal: Vec3) -> LinearRgba {
  let shore = ((-position.y - 0.5) / 2.0).clamp(0.0, 1.0);
  let depth = ((-position.y - 4.0) / 3.5).clamp(0.0, 1.0);
  let cliff = ((0.74 - normal.y) / 0.22).clamp(0.0, 1.0);
  let crest = ((position.y - 3.5) / 3.5).clamp(0.0, 1.0);
  GRASS.mix(&SAND, shore).mix(&SEABED, depth).mix(&ROCK, cliff.max(crest))
}

fn star_at(index: usize) -> (Vec3, f32) {
  let scatter = |salt: u32| {
    let seed = (index as u32)
      .wrapping_mul(1_664_525)
      .wrapping_add(salt.wrapping_mul(1_013_904_223));
    let mixed = seed ^ (seed >> 15);
    (mixed.wrapping_mul(2_246_822_519) >> 8) as f32 / (1 << 24) as f32
  };
  let height = 2.0 * scatter(7) - 1.0;
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

  let dome = commands
    .spawn((Name::new("Sky Dome"), SkyDome, Transform::default(), Visibility::default()))
    .id();
  for index in 0..STAR_COUNT {
    let (position, size) = star_at(index);
    commands.spawn((
      Mesh3d(star_mesh.clone()),
      MeshMaterial3d(star_material.clone()),
      NotShadowCaster,
      Transform::from_translation(position).with_scale(Vec3::splat(size)),
      ChildOf(dome)
    ));
  }

  commands.spawn((
    Name::new("Sun Disc"),
    SunDisc,
    Mesh3d(meshes.add(Sphere::new(SUN_RADIUS).mesh().ico(4).expect("sun mesh"))),
    MeshMaterial3d(materials.add(StandardMaterial {
      base_color: Color::BLACK,
      emissive: LinearRgba::rgb(3400.0, 2500.0, 1350.0),
      ..default()
    })),
    NotShadowCaster
  ));

  commands.insert_resource(StarField(star_material));
}

fn spawn_world(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut images: ResMut<Assets<Image>>
) {
  let island = sdf::bake_painted(
    island_shape(),
    sdf::Bounds::around(Vec3::new(0.0, -2.0, 0.0), 66.0, 7),
    island_paint
  );

  commands.spawn((
    Name::new("Island"),
    RigidBody::Static,
    Collider::trimesh_from_mesh(&island).expect("island collider"),
    Friction::new(0.9),
    Mesh3d(meshes.add(island)),
    MeshMaterial3d(
      materials.add(StandardMaterial { perceptual_roughness: 0.92, ..default() })
    ),
    Transform::from_xyz(0.0, ISLAND_TOP, 0.0)
  ));

  let ocean = materials.add(StandardMaterial {
    base_color: DEEP,
    normal_map_texture: Some(images.add(texture::swell())),
    uv_transform: Affine2::from_scale(Vec2::splat(SKY_RADIUS * 2.0) / texture::SWELL),
    perceptual_roughness: 0.14,
    reflectance: 0.80,
    ..default()
  });
  commands.spawn((
    Name::new("Ocean"),
    Ocean,
    Mesh3d(
      meshes.add(
        Plane3d::default()
          .mesh()
          .size(SKY_RADIUS * 2.0, SKY_RADIUS * 2.0)
          .build()
          .with_generated_tangents()
          .expect("ocean tangents")
      )
    ),
    MeshMaterial3d(ocean),
    NotShadowCaster,
    Transform::from_xyz(0.0, SEA_LEVEL, 0.0)
  ));

  commands.spawn((
    Name::new("Platform"),
    RigidBody::Static,
    Collider::cuboid(PLATFORM_HALF.x * 2.0, PLATFORM_HALF.y * 2.0, PLATFORM_HALF.z * 2.0),
    Friction::new(0.9),
    Mesh3d(meshes.add(sdf::bake(platform_shape(), sdf::Bounds::new(22.0, 6)))),
    MeshMaterial3d(materials.add(StandardMaterial {
      base_color: Color::srgb(0.66, 0.67, 0.67),
      base_color_texture: Some(images.add(texture::concrete())),
      uv_transform: Affine2::from_scale(Vec2::splat(1.0 / CONCRETE_TILE)),
      perceptual_roughness: 0.58,
      reflectance: 0.28,
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
  clock: Res<DayClock>,
  field: Res<StarField>,
  sun: Single<(&mut Transform, &mut DirectionalLight), With<Sun>>,
  disc: Single<(&mut Transform, &mut Visibility), (With<SunDisc>, Without<Sun>)>,
  dome: Single<
    (&mut Transform, &mut Visibility),
    (With<SkyDome>, Without<Sun>, Without<SunDisc>)
  >,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut daylight: ResMut<Daylight>,
  mut ambient: ResMut<GlobalAmbientLight>,
  mut clear: ResMut<ClearColor>
) {
  let (mut transform, mut light) = sun.into_inner();
  let angle = clock.angle(time.elapsed_secs());
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
  let (mut dome_transform, mut dome_visibility) = dome.into_inner();
  dome_transform.rotation = Quat::from_rotation_x(-angle);
  *dome_visibility =
    (night > 0.0).then_some(Visibility::Inherited).unwrap_or(Visibility::Hidden);

  if let Some(mut stars) = materials.get_mut(&field.0) {
    stars.base_color = stars.base_color.with_alpha(night);
  }

  light.illuminance = lux::AMBIENT_DAYLIGHT * 0.28 * day + 2600.0;
  light.color = Color::srgb(0.62 + 0.38 * day, 0.70 + 0.26 * day, 0.95 - 0.07 * day);
  ambient.brightness = 80.0 * day + 55.0;
  clear.0 = Color::srgb(0.05 + 0.41 * day, 0.07 + 0.54 * day, 0.16 + 0.72 * day);
}

pub fn plugin(app: &mut App) {
  app
    .init_resource::<DayClock>()
    .init_resource::<Daylight>()
    .add_systems(Startup, (spawn_world, spawn_sky))
    .add_systems(Update, (cycle_day, stir_ocean).chain());
}

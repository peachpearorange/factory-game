use {crate::{sdf, world::GROUND},
     avian3d::prelude::*,
     bevy::prelude::*};

const SETTLED: f32 = 0.42;
const FADE: f32 = 2.0;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Effects(u8);

impl Effects {
  pub const NONE: Self = Self(0);
  pub const FIERY: Self = Self(1);
  pub const WET: Self = Self(1 << 1);
  pub const RADIOACTIVE: Self = Self(1 << 2);

  const COUNT: usize = 1 << 3;
  const NAMED: [(Self, &'static str); 3] =
    [(Self::FIERY, "Fiery"), (Self::WET, "Wet"), (Self::RADIOACTIVE, "Radioactive")];

  pub fn label(self) -> String {
    let named: Vec<&str> = Self::NAMED
      .into_iter()
      .filter(|&(effect, _)| self.contains(effect))
      .map(|(_, name)| name)
      .collect();
    named.is_empty().then(|| "Plain".to_string()).unwrap_or_else(|| named.join(" "))
  }

  pub fn with(self, other: Self) -> Self { Self(self.0 | other.0) }

  pub fn contains(self, other: Self) -> bool { self.0 & other.0 == other.0 }

  fn tint(self) -> Color {
    let base = Vec3::new(0.42, 0.40, 0.38);
    let blend = |color: Vec3, tint: Vec3, active: bool| {
      active.then(|| color.lerp(tint, 0.65)).unwrap_or(color)
    };
    let color = blend(base, Vec3::new(0.95, 0.35, 0.10), self.contains(Self::FIERY));
    let color = blend(color, Vec3::new(0.15, 0.45, 0.85), self.contains(Self::WET));
    let color =
      blend(color, Vec3::new(0.35, 0.95, 0.25), self.contains(Self::RADIOACTIVE));
    Color::srgb(color.x, color.y, color.z)
  }

  fn emissive(self) -> LinearRgba {
    self
      .contains(Self::FIERY)
      .then(|| LinearRgba::rgb(1.6, 0.35, 0.02))
      .or_else(|| {
        self.contains(Self::RADIOACTIVE).then(|| LinearRgba::rgb(0.15, 1.1, 0.10))
      })
      .unwrap_or(LinearRgba::BLACK)
  }
}

#[derive(Component)]
pub struct Ore {
  pub value: f32,
  pub effects: Effects
}

#[derive(Resource)]
pub struct OreLimit(pub usize);

impl Default for OreLimit {
  fn default() -> Self { Self(120) }
}

#[derive(Resource)]
pub struct OreAssets {
  mesh: Handle<Mesh>,
  collider: Collider,
  materials: Vec<Handle<StandardMaterial>>
}

impl OreAssets {
  const HALF: Vec3 = Vec3::new(0.34, 0.28, 0.30);

  fn shape() -> fidget::context::Tree {
    sdf::smooth_union(
      sdf::rounded_box(Self::HALF, 0.06),
      sdf::at(sdf::sphere(0.22), Vec3::new(0.14, 0.12, -0.08)),
      0.10
    )
  }

  pub fn spawn(&self, position: Vec3, value: f32) -> impl Bundle {
    (
      Ore { value, effects: Effects::NONE },
      RigidBody::Dynamic,
      self.collider.clone(),
      CollisionMargin(0.01),
      Friction::new(0.7),
      SleepingDisabled,
      CollisionEventsEnabled,
      Mesh3d(self.mesh.clone()),
      MeshMaterial3d(self.materials[Effects::NONE.0 as usize].clone()),
      Transform::from_translation(position)
    )
  }

  pub fn material(&self, effects: Effects) -> Handle<StandardMaterial> {
    self.materials[effects.0 as usize].clone()
  }
}

fn load_ore_assets(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>
) {
  let mesh = sdf::bake(OreAssets::shape(), sdf::Bounds::new(0.55, 6));
  let collider =
    Collider::convex_hull(sdf::points(&mesh)).expect("ore mesh has no convex hull");
  let materials = (0..Effects::COUNT as u8)
    .map(|bits| {
      let effects = Effects(bits);
      materials.add(StandardMaterial {
        base_color: effects.tint(),
        emissive: effects.emissive(),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
      })
    })
    .collect();

  commands.insert_resource(OreAssets { mesh: meshes.add(mesh), collider, materials });
}

#[derive(Component)]
struct Fading(Timer);

fn fade_dropped_ores(
  time: Res<Time>,
  assets: Res<OreAssets>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut ores: Query<(
    Entity,
    &Ore,
    &Transform,
    &mut MeshMaterial3d<StandardMaterial>,
    Option<&mut Fading>
  )>,
  mut commands: Commands
) {
  for (entity, ore, transform, mut painted, fading) in &mut ores {
    let floored = transform.translation.y < GROUND + SETTLED;
    let recovered = !floored && fading.is_some();
    if floored && let Some(mut fading) = fading {
      let left = 1.0 - fading.0.tick(time.delta()).fraction();
      if let Some(mut material) = materials.get_mut(&painted.0) {
        material.base_color = material.base_color.with_alpha(left);
      }
      if fading.0.is_finished() {
        commands.entity(entity).despawn();
      }
    } else if floored {
      let solid = assets.material(ore.effects);
      let mut dissolving = materials.get(&solid).cloned().unwrap_or_default();
      dissolving.alpha_mode = AlphaMode::Blend;
      painted.0 = materials.add(dissolving);
      commands.entity(entity).insert(Fading(Timer::from_seconds(FADE, TimerMode::Once)));
    } else if recovered {
      painted.0 = assets.material(ore.effects);
      commands.entity(entity).remove::<Fading>();
    }
  }
}

pub fn plugin(app: &mut App) {
  app
    .init_resource::<OreLimit>()
    .add_systems(PreStartup, load_ore_assets)
    .add_systems(Update, fade_dropped_ores);
}

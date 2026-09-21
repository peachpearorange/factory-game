use {crate::{catalog::{CELL, MachineAssets, MachineKind, place, spawn_ghost_arrows},
             menu::playing,
             player::{MainCamera, Player, UiHover}},
     avian3d::prelude::*,
     bevy::{platform::collections::HashMap, prelude::*},
     std::f32::consts::FRAC_PI_2};

const GRID_HALF: i32 = 10;
const REACH: f32 = 40.0;
const DOOM_SWELL: f32 = 1.04;

#[derive(Component)]
pub struct PlacedMachine {
  pub kind: MachineKind,
  pub cell: IVec2,
  pub turns: u8
}

#[derive(Component)]
struct Ghost;

#[derive(Component)]
struct Doomed;

fn covered(cell: IVec2, kind: MachineKind, turns: u8) -> impl Iterator<Item = IVec2> {
  let span = kind.span(turns);
  (0..span.x).flat_map(move |along| {
    (0..span.y).map(move |across| cell + IVec2::new(along, across))
  })
}

#[derive(Resource, Default)]
pub struct BuildGrid(HashMap<IVec2, Entity>);

impl BuildGrid {
  fn free(&self, cell: IVec2, kind: MachineKind, turns: u8) -> bool {
    covered(cell, kind, turns).all(|cell| {
      cell.x.abs() <= GRID_HALF
        && cell.y.abs() <= GRID_HALF
        && !self.0.contains_key(&cell)
    })
  }
}

#[derive(Resource, Default, PartialEq, Eq)]
pub enum BuildMode {
  #[default]
  Idle,
  Placing {
    kind: MachineKind,
    turns: u8
  },
  Deleting
}

#[derive(Resource)]
pub struct Inventory([u32; MachineKind::COUNT]);

impl Inventory {
  pub fn count(&self, kind: MachineKind) -> u32 { self.0[kind.index()] }

  pub fn add(&mut self, kind: MachineKind) { self.0[kind.index()] += 1; }

  fn take(&mut self, kind: MachineKind) -> bool {
    let slot = &mut self.0[kind.index()];
    (*slot > 0).then(|| *slot -= 1).is_some()
  }
}

impl Default for Inventory {
  fn default() -> Self {
    let mut stock = Self([0; MachineKind::COUNT]);
    for kind in [MachineKind::Furnace, MachineKind::Dropper, MachineKind::Conveyor] {
      stock.add(kind);
    }
    stock
  }
}

pub fn aim_ray(camera: &Camera, eye: &GlobalTransform, window: &Window) -> Option<Ray3d> {
  window
    .cursor_position()
    .and_then(|position| camera.viewport_to_world(eye, position).ok())
}

pub fn aimed_entity(
  spatial: &SpatialQuery,
  ray: Ray3d,
  ignore: Entity
) -> Option<Entity> {
  spatial
    .cast_ray(
      ray.origin,
      ray.direction,
      REACH,
      true,
      &SpatialQueryFilter::from_excluded_entities([ignore])
    )
    .map(|hit| hit.entity)
}

pub fn machine_root(
  entity: Entity,
  parents: &Query<&ChildOf>,
  machines: &Query<&PlacedMachine>
) -> Option<Entity> {
  machines
    .contains(entity)
    .then_some(entity)
    .or_else(|| parents.get(entity).ok().map(|parent| parent.parent()))
    .filter(|&root| machines.contains(root))
}

fn cell_transform(cell: IVec2, kind: MachineKind, turns: u8) -> Transform {
  let center = (cell.as_vec2() + (kind.span(turns).as_vec2() - Vec2::ONE) / 2.0) * CELL;
  Transform::from_xyz(center.x, 0.0, center.y)
    .with_rotation(Quat::from_rotation_y(turns as f32 * -FRAC_PI_2))
}

fn aimed_cell(ray: Ray3d) -> Option<IVec2> {
  (ray.direction.y < -0.02)
    .then(|| ray.origin.y / -ray.direction.y)
    .filter(|&distance| distance < REACH)
    .map(|distance| {
      let hit = ray.origin + ray.direction * distance;
      IVec2::new((hit.x / CELL).round() as i32, (hit.z / CELL).round() as i32)
    })
    .filter(|cell| cell.x.abs() <= GRID_HALF && cell.y.abs() <= GRID_HALF)
}

fn steer_build(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<BuildMode>) {
  if let BuildMode::Placing { kind, turns } = *mode
    && keys.just_pressed(KeyCode::KeyR)
  {
    *mode = BuildMode::Placing { kind, turns: (turns + 1) % 4 };
  }
}

fn update_ghost(
  mode: Res<BuildMode>,
  grid: Res<BuildGrid>,
  hovering: Res<UiHover>,
  assets: Res<MachineAssets>,
  eye: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
  window: Single<&Window>,
  ghosts: Query<Entity, With<Ghost>>,
  mut commands: Commands
) {
  for entity in &ghosts {
    commands.entity(entity).despawn();
  }
  let (camera, transform) = *eye;
  if let BuildMode::Placing { kind, turns } = *mode
    && !hovering.0
    && let Some(ray) = aim_ray(camera, transform, *window)
    && let Some(cell) = aimed_cell(ray)
  {
    let blocked = !grid.free(cell, kind, turns);
    let ghost = commands
      .spawn((
        Ghost,
        Mesh3d(assets.mesh(kind)),
        MeshMaterial3d(
          blocked
            .then(|| assets.ghost_blocked.clone())
            .unwrap_or_else(|| assets.ghost_valid.clone())
        ),
        cell_transform(cell, kind, turns)
      ))
      .id();
    if kind.carries_belt() {
      spawn_ghost_arrows(&mut commands, &assets, kind, ghost);
    }
  }
}

fn mark_doomed(
  mode: Res<BuildMode>,
  hovering: Res<UiHover>,
  spatial: SpatialQuery,
  assets: Res<MachineAssets>,
  eye: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
  window: Single<&Window>,
  player: Single<Entity, With<Player>>,
  parents: Query<&ChildOf>,
  machines: Query<&PlacedMachine>,
  doomed: Query<Entity, With<Doomed>>,
  mut commands: Commands
) {
  for entity in &doomed {
    commands.entity(entity).despawn();
  }
  let (camera, transform) = *eye;
  if *mode == BuildMode::Deleting
    && !hovering.0
    && let Some(ray) = aim_ray(camera, transform, *window)
    && let Some(hit) = aimed_entity(&spatial, ray, *player)
    && let Some(root) = machine_root(hit, &parents, &machines)
    && let Ok(&PlacedMachine { kind, cell, turns }) = machines.get(root)
  {
    commands.spawn((
      Doomed,
      Mesh3d(assets.mesh(kind)),
      MeshMaterial3d(assets.ghost_blocked.clone()),
      cell_transform(cell, kind, turns).with_scale(Vec3::splat(DOOM_SWELL))
    ));
  }
}

fn place_machine(
  mouse: Res<ButtonInput<MouseButton>>,
  hovering: Res<UiHover>,
  assets: Res<MachineAssets>,
  eye: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
  window: Single<&Window>,
  mut mode: ResMut<BuildMode>,
  mut grid: ResMut<BuildGrid>,
  mut inventory: ResMut<Inventory>,
  mut armed: Local<bool>,
  mut commands: Commands
) {
  let (camera, transform) = *eye;
  *armed = (*armed && !mode.is_changed()) || !mouse.pressed(MouseButton::Left);
  if let BuildMode::Placing { kind, turns } = *mode
    && !hovering.0
    && *armed
    && mouse.pressed(MouseButton::Left)
    && let Some(ray) = aim_ray(camera, transform, *window)
    && let Some(cell) = aimed_cell(ray)
    && grid.free(cell, kind, turns)
    && inventory.take(kind)
  {
    let machine = place(&mut commands, &assets, kind, cell_transform(cell, kind, turns));
    commands.entity(machine).insert(PlacedMachine { kind, cell, turns });
    for cell in covered(cell, kind, turns) {
      grid.0.insert(cell, machine);
    }
    if inventory.count(kind) == 0 {
      *mode = BuildMode::Idle;
    }
  }
}

fn take_machine(
  mouse: Res<ButtonInput<MouseButton>>,
  hovering: Res<UiHover>,
  spatial: SpatialQuery,
  eye: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
  window: Single<&Window>,
  player: Single<Entity, With<Player>>,
  parents: Query<&ChildOf>,
  machines: Query<&PlacedMachine>,
  mut mode: ResMut<BuildMode>,
  mut grid: ResMut<BuildGrid>,
  mut inventory: ResMut<Inventory>,
  mut armed: Local<bool>,
  mut commands: Commands
) {
  let (camera, transform) = *eye;
  let grabbing = *mode == BuildMode::Idle;
  let reaching = grabbing
    .then(|| mouse.just_pressed(MouseButton::Left))
    .unwrap_or(mouse.pressed(MouseButton::Left));
  *armed = (*armed && !mode.is_changed()) || !mouse.pressed(MouseButton::Left);
  if !hovering.0
    && (grabbing || *mode == BuildMode::Deleting)
    && *armed
    && reaching
    && let Some(ray) = aim_ray(camera, transform, *window)
    && let Some(hit) = aimed_entity(&spatial, ray, *player)
    && let Some(root) = machine_root(hit, &parents, &machines)
    && let Ok(&PlacedMachine { kind, cell, turns }) = machines.get(root)
  {
    inventory.add(kind);
    for cell in covered(cell, kind, turns) {
      grid.0.remove(&cell);
    }
    commands.entity(root).despawn();
    if grabbing {
      *mode = BuildMode::Placing { kind, turns };
    }
  }
}

pub fn plugin(app: &mut App) {
  app
    .init_resource::<BuildGrid>()
    .init_resource::<BuildMode>()
    .init_resource::<Inventory>()
    .add_systems(
      Update,
      (steer_build, update_ghost, mark_doomed, take_machine, place_machine)
        .chain()
        .run_if(playing)
    );
}

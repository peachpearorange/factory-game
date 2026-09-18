use crate::catalog::{CELL, MachineAssets, MachineKind, place};
use crate::player::CursorMode;
use avian3d::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

const GRID_HALF: i32 = 9;
const REACH: f32 = 26.0;

#[derive(Component)]
pub struct PlacedMachine {
    pub kind: MachineKind,
    pub cell: IVec2,
}

#[derive(Component)]
struct Ghost;

#[derive(Resource, Default)]
pub struct BuildGrid(HashMap<IVec2, Entity>);

#[derive(Resource, Default, PartialEq, Eq)]
pub enum BuildMode {
    #[default]
    Idle,
    Placing {
        kind: MachineKind,
        turns: u8,
    },
}

#[derive(Resource, Default)]
pub struct Inventory([u32; MachineKind::COUNT]);

impl Inventory {
    pub fn count(&self, kind: MachineKind) -> u32 {
        self.0[kind.index()]
    }

    pub fn add(&mut self, kind: MachineKind) {
        self.0[kind.index()] += 1;
    }

    fn take(&mut self, kind: MachineKind) -> bool {
        let slot = &mut self.0[kind.index()];
        (*slot > 0).then(|| *slot -= 1).is_some()
    }
}

fn cell_transform(cell: IVec2, turns: u8) -> Transform {
    Transform::from_xyz(cell.x as f32 * CELL, 0.0, cell.y as f32 * CELL)
        .with_rotation(Quat::from_rotation_y(turns as f32 * -FRAC_PI_2))
}

fn aimed_cell(camera: &GlobalTransform) -> Option<IVec2> {
    let origin = camera.translation();
    let direction = camera.forward().as_vec3();
    (direction.y < -0.02)
        .then(|| origin.y / -direction.y)
        .filter(|&distance| distance < REACH)
        .map(|distance| {
            let hit = origin + direction * distance;
            IVec2::new((hit.x / CELL).round() as i32, (hit.z / CELL).round() as i32)
        })
        .filter(|cell| cell.x.abs() <= GRID_HALF && cell.y.abs() <= GRID_HALF)
}

pub fn aimed_entity(spatial: &SpatialQuery, camera: &GlobalTransform, ignore: Entity) -> Option<Entity> {
    spatial
        .cast_ray(
            camera.translation(),
            camera.forward(),
            REACH,
            true,
            &SpatialQueryFilter::from_excluded_entities([ignore]),
        )
        .map(|hit| hit.entity)
}

pub fn machine_root(entity: Entity, parents: &Query<&ChildOf>, machines: &Query<&PlacedMachine>) -> Option<Entity> {
    machines
        .contains(entity)
        .then_some(entity)
        .or_else(|| parents.get(entity).ok().map(|parent| parent.parent()))
        .filter(|&root| machines.contains(root))
}

fn steer_build(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mode: ResMut<BuildMode>,
) {
    if let BuildMode::Placing { kind, turns } = *mode {
        if keys.just_pressed(KeyCode::KeyR) {
            *mode = BuildMode::Placing {
                kind,
                turns: (turns + 1) % 4,
            };
        } else if keys.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right) {
            *mode = BuildMode::Idle;
        }
    }
}

fn update_ghost(
    mode: Res<BuildMode>,
    grid: Res<BuildGrid>,
    assets: Res<MachineAssets>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    ghosts: Query<Entity, With<Ghost>>,
    mut commands: Commands,
) {
    for entity in &ghosts {
        commands.entity(entity).despawn();
    }
    if let BuildMode::Placing { kind, turns } = *mode
        && let Some(cell) = aimed_cell(*camera)
    {
        let blocked = grid.0.contains_key(&cell);
        commands.spawn((
            Ghost,
            Mesh3d(assets.mesh(kind)),
            MeshMaterial3d(
                blocked
                    .then(|| assets.ghost_blocked.clone())
                    .unwrap_or_else(|| assets.ghost_valid.clone()),
            ),
            cell_transform(cell, turns),
        ));
    }
}

fn place_machine(
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<CursorMode>,
    assets: Res<MachineAssets>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    mut mode: ResMut<BuildMode>,
    mut grid: ResMut<BuildGrid>,
    mut inventory: ResMut<Inventory>,
    mut commands: Commands,
) {
    if let BuildMode::Placing { kind, turns } = *mode
        && *pointer == CursorMode::Look
        && !mode.is_changed()
        && mouse.just_pressed(MouseButton::Left)
        && let Some(cell) = aimed_cell(*camera)
        && !grid.0.contains_key(&cell)
        && inventory.take(kind)
    {
        let machine = place(&mut commands, &assets, kind, cell_transform(cell, turns));
        commands.entity(machine).insert(PlacedMachine { kind, cell });
        grid.0.insert(cell, machine);
        if inventory.count(kind) == 0 {
            *mode = BuildMode::Idle;
        }
    }
}

fn remove_machine(
    keys: Res<ButtonInput<KeyCode>>,
    pointer: Res<CursorMode>,
    spatial: SpatialQuery,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    player: Single<Entity, With<crate::player::Player>>,
    parents: Query<&ChildOf>,
    machines: Query<&PlacedMachine>,
    mut grid: ResMut<BuildGrid>,
    mut inventory: ResMut<Inventory>,
    mut commands: Commands,
) {
    if *pointer == CursorMode::Look
        && keys.just_pressed(KeyCode::KeyX)
        && let Some(hit) = aimed_entity(&spatial, *camera, *player)
        && let Some(root) = machine_root(hit, &parents, &machines)
        && let Ok(machine) = machines.get(root)
    {
        inventory.add(machine.kind);
        grid.0.remove(&machine.cell);
        commands.entity(root).despawn();
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<BuildGrid>()
        .init_resource::<BuildMode>()
        .init_resource::<Inventory>()
        .add_systems(
            Update,
            (steer_build, update_ghost, place_machine, remove_machine).chain(),
        );
}

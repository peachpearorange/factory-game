use crate::ore::{Effects, Ore, OreAssets, OreLimit};
use crate::sdf;
use avian3d::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::system::{SystemParam, lifetimeless::Read};
use bevy::prelude::*;

pub const BELT_LEVEL: f32 = 0.7;
const BELT_SPAN: f32 = 14.0;
const BELT_WIDTH: f32 = 2.4;
const DROPPER_X: f32 = -5.6;
const UPGRADER_X: f32 = 0.5;
const FURNACE_X: f32 = 6.5;

#[derive(Component)]
#[require(ActiveCollisionHooks::MODIFY_CONTACTS)]
pub struct ConveyorBelt {
    pub local_direction: Vec3,
    pub speed: f32,
}

#[derive(SystemParam)]
pub struct ConveyorHooks<'w, 's> {
    belts: Query<'w, 's, (Read<ConveyorBelt>, Read<GlobalTransform>)>,
}

impl CollisionHooks for ConveyorHooks<'_, '_> {
    fn modify_contacts(&self, contacts: &mut ContactPair, _commands: &mut Commands) -> bool {
        if let Some((belt, transform, sign)) =
            [(contacts.collider1, -1.0), (contacts.collider2, 1.0)]
                .into_iter()
                .find_map(|(entity, sign)| {
                    self.belts
                        .get(entity)
                        .ok()
                        .map(|(belt, transform)| (belt, transform, sign))
                })
        {
            let direction = transform.rotation() * belt.local_direction;
            for manifold in contacts.manifolds.iter_mut() {
                manifold.tangent_velocity = sign * belt.speed * direction;
            }
        }
        true
    }
}

#[derive(Component)]
pub struct Dropper {
    pub timer: Timer,
    pub value: f32,
}

#[derive(Component)]
pub struct Upgrader {
    pub multiplier: f32,
    pub effects: Effects,
}

#[derive(Component)]
pub struct Furnace;

#[derive(Resource, Default)]
pub struct Money(pub f32);

fn machine_and_ore<'w, T: Component>(
    event: &CollisionStart,
    machines: &'w Query<&T>,
) -> Option<(&'w T, Entity)> {
    [
        (event.collider1, event.collider2),
        (event.collider2, event.collider1),
    ]
    .into_iter()
    .find_map(|(machine, ore)| machines.get(machine).ok().map(|found| (found, ore)))
}

fn drop_ores(
    time: Res<Time>,
    limit: Res<OreLimit>,
    assets: Res<OreAssets>,
    ores: Query<(), With<Ore>>,
    mut droppers: Query<(&mut Dropper, &GlobalTransform)>,
    mut commands: Commands,
) {
    let mut live = ores.iter().count();
    for (mut dropper, transform) in &mut droppers {
        dropper.timer.tick(time.delta());
        if dropper.timer.just_finished() && live < limit.0 {
            let spawn = transform.translation() + Vec3::new(0.0, -0.9, 0.0);
            commands.spawn(assets.spawn(spawn, dropper.value));
            live += 1;
        }
    }
}

fn upgrade_ores(
    mut events: MessageReader<CollisionStart>,
    upgraders: Query<&Upgrader>,
    assets: Res<OreAssets>,
    mut ores: Query<(&mut Ore, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    for event in events.read() {
        if let Some((upgrader, entity)) = machine_and_ore(event, &upgraders)
            && let Ok((mut ore, mut material)) = ores.get_mut(entity)
            && !ore.effects.contains(upgrader.effects)
        {
            ore.value *= upgrader.multiplier;
            ore.effects = ore.effects.with(upgrader.effects);
            material.0 = assets.material(ore.effects);
        }
    }
}

fn burn_ores(
    mut events: MessageReader<CollisionStart>,
    furnaces: Query<&Furnace>,
    ores: Query<&Ore>,
    mut money: ResMut<Money>,
    mut commands: Commands,
) {
    let mut burned = EntityHashSet::default();
    for event in events.read() {
        if let Some((_, entity)) = machine_and_ore(event, &furnaces)
            && burned.insert(entity)
            && let Ok(ore) = ores.get(entity)
        {
            money.0 += ore.value;
            commands.entity(entity).despawn();
        }
    }
}

fn dropper_shape() -> fidget::context::Tree {
    sdf::union([
        sdf::at(
            sdf::rounded_box(Vec3::new(1.1, 0.9, 1.1), 0.12),
            Vec3::new(0.0, 1.0, 0.0),
        ),
        sdf::at(sdf::cylinder(0.55, 0.5), Vec3::new(0.0, 0.0, 0.0)),
        sdf::at(
            sdf::cuboid(Vec3::new(0.18, 0.75, 0.18)),
            Vec3::new(0.8, -0.85, 0.8),
        ),
        sdf::at(
            sdf::cuboid(Vec3::new(0.18, 0.75, 0.18)),
            Vec3::new(-0.8, -0.85, -0.8),
        ),
    ])
}

fn upgrader_shape() -> fidget::context::Tree {
    sdf::union([
        sdf::difference(
            sdf::at(
                sdf::rounded_box(Vec3::new(0.7, 1.5, 1.7), 0.15),
                Vec3::new(0.0, 0.4, 0.0),
            ),
            sdf::at(
                sdf::cuboid(Vec3::new(1.2, 0.95, 1.25)),
                Vec3::new(0.0, -0.55, 0.0),
            ),
        ),
        sdf::at(
            sdf::cylinder(0.28, 0.5),
            Vec3::new(0.0, 1.9, 0.0),
        ),
    ])
}

fn furnace_shape() -> fidget::context::Tree {
    sdf::union([
        sdf::difference(
            sdf::smooth_union(
                sdf::at(
                    sdf::rounded_box(Vec3::new(1.3, 1.0, 1.3), 0.14),
                    Vec3::new(0.0, 0.6, 0.0),
                ),
                sdf::at(sdf::sphere(0.95), Vec3::new(0.0, 1.5, 0.0)),
                0.35,
            ),
            sdf::at(
                sdf::rounded_box(Vec3::new(0.9, 0.5, 0.75), 0.1),
                Vec3::new(-1.0, 0.45, 0.0),
            ),
        ),
        sdf::at(sdf::cylinder(0.3, 0.7), Vec3::new(0.0, 2.6, 0.0)),
    ])
}

fn spawn_machines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let steel = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.57, 0.60),
        perceptual_roughness: 0.45,
        metallic: 0.8,
        ..default()
    });
    let belt_surface = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.12, 0.14),
        perceptual_roughness: 0.95,
        ..default()
    });
    let hot = materials.add(StandardMaterial {
        base_color: Color::srgb(0.30, 0.13, 0.10),
        emissive: LinearRgba::rgb(1.4, 0.32, 0.05),
        perceptual_roughness: 0.7,
        ..default()
    });

    commands.spawn((
        Name::new("Conveyor"),
        RigidBody::Static,
        Collider::cuboid(BELT_SPAN, 0.2, BELT_WIDTH),
        Friction::new(1.0),
        ConveyorBelt {
            local_direction: Vec3::X,
            speed: 3.5,
        },
        Mesh3d(meshes.add(Cuboid::new(BELT_SPAN, 0.2, BELT_WIDTH))),
        MeshMaterial3d(belt_surface),
        Transform::from_xyz(0.0, BELT_LEVEL, 0.0),
    ));

    let rail = meshes.add(Cuboid::new(BELT_SPAN, 0.3, 0.16));
    for side in [-1.0, 1.0] {
        commands.spawn((
            RigidBody::Static,
            Collider::cuboid(BELT_SPAN, 0.3, 0.16),
            Mesh3d(rail.clone()),
            MeshMaterial3d(steel.clone()),
            Transform::from_xyz(0.0, BELT_LEVEL + 0.2, side * (BELT_WIDTH / 2.0 + 0.08)),
        ));
    }

    commands.spawn((
        Name::new("Dropper"),
        Dropper {
            timer: Timer::from_seconds(0.65, TimerMode::Repeating),
            value: 12.0,
        },
        Mesh3d(meshes.add(sdf::bake(dropper_shape(), sdf::Bounds::new(2.4, 5)))),
        MeshMaterial3d(steel.clone()),
        Transform::from_xyz(DROPPER_X, BELT_LEVEL + 2.2, 0.0),
    ));

    commands.spawn((
        Name::new("Upgrader"),
        Mesh3d(meshes.add(sdf::bake(upgrader_shape(), sdf::Bounds::new(2.6, 5)))),
        MeshMaterial3d(steel.clone()),
        Transform::from_xyz(UPGRADER_X, BELT_LEVEL + 0.1, 0.0),
    ));

    commands.spawn((
        Name::new("Upgrader field"),
        Upgrader {
            multiplier: 2.5,
            effects: Effects::FIERY,
        },
        RigidBody::Static,
        Collider::cuboid(0.6, 1.0, BELT_WIDTH),
        Sensor,
        CollisionEventsEnabled,
        Transform::from_xyz(UPGRADER_X, BELT_LEVEL + 0.6, 0.0),
    ));

    commands.spawn((
        Name::new("Furnace"),
        RigidBody::Static,
        Collider::cuboid(2.6, 2.0, 2.6),
        Mesh3d(meshes.add(sdf::bake(furnace_shape(), sdf::Bounds::new(3.4, 5)))),
        MeshMaterial3d(hot),
        Transform::from_xyz(FURNACE_X + 2.7, BELT_LEVEL - 0.2, 0.0),
    ));

    commands.spawn((
        Name::new("Furnace mouth"),
        Furnace,
        RigidBody::Static,
        Collider::cuboid(0.5, 1.2, BELT_WIDTH),
        Sensor,
        CollisionEventsEnabled,
        Transform::from_xyz(FURNACE_X, BELT_LEVEL + 0.7, 0.0),
    ));
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Money>()
        .add_systems(Startup, spawn_machines)
        .add_systems(Update, (drop_ores, upgrade_ores, burn_ores));
}

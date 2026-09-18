use crate::sdf;
use avian3d::prelude::*;
use bevy::light::{CascadeShadowConfigBuilder, light_consts::lux};
use bevy::prelude::*;
use std::f32::consts::TAU;

pub const PLATFORM_HALF: Vec3 = Vec3::new(21.0, 0.6, 21.0);
const ISLAND_RADIUS: f32 = 44.0;
const ISLAND_DEPTH: f32 = 8.0;

#[derive(Component)]
struct Sun;

#[derive(Resource)]
struct DayLength(f32);

impl Default for DayLength {
    fn default() -> Self {
        Self(240.0)
    }
}

fn platform_shape() -> fidget::context::Tree {
    sdf::difference(
        sdf::rounded_box(PLATFORM_HALF, 0.35),
        sdf::at(
            sdf::rounded_box(
                Vec3::new(PLATFORM_HALF.x - 1.2, 0.3, PLATFORM_HALF.z - 1.2),
                0.2,
            ),
            Vec3::new(0.0, PLATFORM_HALF.y, 0.0),
        ),
    )
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        Transform::from_xyz(0.0, -ISLAND_DEPTH / 2.0 - PLATFORM_HALF.y, 0.0),
    ));

    commands.spawn((
        Name::new("Platform"),
        RigidBody::Static,
        Collider::cuboid(
            PLATFORM_HALF.x * 2.0,
            PLATFORM_HALF.y * 2.0,
            PLATFORM_HALF.z * 2.0,
        ),
        Friction::new(0.9),
        Mesh3d(meshes.add(sdf::bake(
            platform_shape(),
            sdf::Bounds::new(PLATFORM_HALF.x * 1.2, 7),
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.66, 0.65, 0.63),
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_xyz(0.0, -PLATFORM_HALF.y, 0.0),
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
        Transform::default().looking_to(Vec3::new(-0.4, -0.8, -0.45), Vec3::Y),
    ));
}

fn cycle_day(
    time: Res<Time>,
    day: Res<DayLength>,
    sun: Single<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
) {
    let (mut transform, mut light) = sun.into_inner();
    let angle = time.elapsed_secs() / day.0 * TAU;
    let elevation = angle.sin();
    let toward_sun = Vec3::new(0.35, elevation, angle.cos()).normalize();
    *transform = Transform::default().looking_to(-toward_sun, Vec3::Y);

    let daylight = elevation.max(0.0);
    light.illuminance = lux::AMBIENT_DAYLIGHT * daylight + lux::FULL_MOON_NIGHT;
    light.color = Color::srgb(1.0, 0.80 + 0.16 * daylight, 0.58 + 0.30 * daylight);
    ambient.brightness = 90.0 * daylight + 6.0;
    clear.0 = Color::srgb(
        0.02 + 0.44 * daylight,
        0.03 + 0.58 * daylight,
        0.08 + 0.80 * daylight,
    );
}

pub fn plugin(app: &mut App) {
    app.init_resource::<DayLength>()
        .add_systems(Startup, spawn_world)
        .add_systems(Update, cycle_day);
}

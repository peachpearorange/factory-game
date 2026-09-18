use crate::sdf;
use avian3d::math::AdjustPrecision;
use avian3d::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use std::ops::Range;

const GRAVITY: f32 = -26.0;
const JUMP_SPEED: f32 = 9.5;
const GROUND_PROBE: f32 = 0.18;

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
pub enum CursorMode {
    #[default]
    Look,
    Ui,
}

#[derive(Component)]
#[require(
    RigidBody::Kinematic,
    CustomPositionIntegration,
    SpeculativeMargin(0.0),
    LinearVelocity
)]
pub struct Player {
    speed: f32,
}

#[derive(Resource)]
struct CameraRig {
    distance: f32,
    height: f32,
    yaw_speed: f32,
    pitch_speed: f32,
    pitch_range: Range<f32>,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            distance: 11.0,
            height: 1.9,
            yaw_speed: 0.0038,
            pitch_speed: 0.0030,
            pitch_range: -1.15..0.62,
        }
    }
}

fn player_shape() -> fidget::context::Tree {
    let limb = |offset: Vec3, half: Vec3| sdf::at(sdf::rounded_box(half, 0.07), offset);
    sdf::union([
        limb(Vec3::new(0.0, 0.35, 0.0), Vec3::new(0.42, 0.55, 0.24)),
        limb(Vec3::new(0.0, 1.22, 0.0), Vec3::new(0.32, 0.32, 0.32)),
        limb(Vec3::new(0.63, 0.32, 0.0), Vec3::new(0.18, 0.55, 0.18)),
        limb(Vec3::new(-0.63, 0.32, 0.0), Vec3::new(0.18, 0.55, 0.18)),
        limb(Vec3::new(0.22, -0.74, 0.0), Vec3::new(0.19, 0.58, 0.20)),
        limb(Vec3::new(-0.22, -0.74, 0.0), Vec3::new(0.19, 0.58, 0.20)),
    ])
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Player"),
        Player { speed: 7.5 },
        Collider::capsule(0.42, 1.5),
        Mesh3d(meshes.add(sdf::bake(player_shape(), sdf::Bounds::new(2.0, 6)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.90, 0.72, 0.20),
            perceptual_roughness: 0.75,
            ..default()
        })),
        Transform::from_xyz(-13.0, 2.5, 7.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 68f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(-13.0, 6.0, 18.0).looking_at(Vec3::new(-13.0, 2.5, 7.0), Vec3::Y),
    ));
}

fn sync_cursor(mode: Res<CursorMode>, mut cursor: Single<&mut CursorOptions>) {
    let looking = *mode == CursorMode::Look;
    cursor.visible = !looking;
    cursor.grab_mode = looking
        .then_some(CursorGrabMode::Locked)
        .unwrap_or(CursorGrabMode::None);
}

fn move_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mode: Res<CursorMode>,
    camera: Single<&Transform, (With<Camera3d>, Without<Player>)>,
    player: Single<(Entity, &Player, &Collider, &mut Transform, &mut LinearVelocity)>,
    move_and_slide: MoveAndSlide,
) {
    let (entity, player, collider, mut transform, mut velocity) = player.into_inner();
    let flat = camera.forward().as_vec3().with_y(0.0).normalize_or_zero();
    let right = Vec3::new(-flat.z, 0.0, flat.x);
    let wish = (*mode == CursorMode::Look)
        .then(|| {
            [
                (KeyCode::KeyW, flat),
                (KeyCode::KeyS, -flat),
                (KeyCode::KeyD, right),
                (KeyCode::KeyA, -right),
            ]
            .into_iter()
            .filter(|&(key, _)| keys.pressed(key))
            .fold(Vec3::ZERO, |sum, (_, direction)| sum + direction)
            .normalize_or_zero()
        })
        .unwrap_or(Vec3::ZERO)
        * player.speed;

    let filter = SpatialQueryFilter::from_excluded_entities([entity]);
    let grounded = move_and_slide
        .spatial_query
        .cast_shape(
            collider,
            transform.translation.adjust_precision(),
            transform.rotation.adjust_precision(),
            Dir3::NEG_Y,
            &ShapeCastConfig::from_max_distance(GROUND_PROBE),
            &filter,
        )
        .is_some();

    let fall = velocity.y + GRAVITY * time.delta_secs();
    velocity.0 = Vec3::new(
        wish.x,
        if grounded && *mode == CursorMode::Look && keys.just_pressed(KeyCode::Space) {
            JUMP_SPEED
        } else if grounded {
            fall.max(GRAVITY * time.delta_secs())
        } else {
            fall
        },
        wish.z,
    );

    if wish.length_squared() > 0.0 {
        transform.rotation = Quat::from_rotation_y(wish.x.atan2(wish.z));
    }

    let MoveAndSlideOutput {
        position,
        projected_velocity,
    } = move_and_slide.move_and_slide(
        collider,
        transform.translation.adjust_precision(),
        transform.rotation.adjust_precision(),
        velocity.0,
        time.delta(),
        &MoveAndSlideConfig::default(),
        &filter,
        |_| MoveAndSlideHitResponse::Accept,
    );
    transform.translation = position;
    velocity.0 = projected_velocity;
}

fn follow_player(
    rig: Res<CameraRig>,
    mode: Res<CursorMode>,
    motion: Res<AccumulatedMouseMotion>,
    player: Single<&Transform, With<Player>>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<Player>)>,
) {
    let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    let delta = (*mode == CursorMode::Look)
        .then_some(motion.delta)
        .unwrap_or(Vec2::ZERO);
    camera.rotation = Quat::from_euler(
        EulerRot::YXZ,
        yaw - delta.x * rig.yaw_speed,
        (pitch - delta.y * rig.pitch_speed).clamp(rig.pitch_range.start, rig.pitch_range.end),
        0.0,
    );
    let focus = player.translation + Vec3::Y * rig.height;
    camera.translation = focus - camera.forward() * rig.distance;
}

pub fn plugin(app: &mut App) {
    app.init_resource::<CameraRig>()
        .init_resource::<CursorMode>()
        .add_systems(Startup, spawn_player)
        .add_systems(Update, (sync_cursor, move_player, follow_player).chain());
}

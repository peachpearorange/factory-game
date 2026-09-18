use {crate::{machine::ConveyorBelt,
             menu::playing,
             sdf,
             world::{Daylight, GROUND}},
     avian3d::{math::AdjustPrecision, prelude::*},
     bevy::{asset::AssetId,
            input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
            light::NotShadowCaster,
            platform::collections::{HashMap, HashSet},
            post_process::bloom::Bloom,
            prelude::*,
            window::{CursorGrabMode, CursorOptions}},
     std::{f32::consts::PI, ops::Range}};

const GRAVITY: f32 = -26.0;
const JUMP_SPEED: f32 = 9.5;
const GROUND_PROBE: f32 = 0.18;
const WALK_SPEED: f32 = 7.5;
const TURN_RATE: f32 = 12.0;
const FOOT: f32 = -1.17;
const HIP: f32 = -0.27;
const SHOULDER: f32 = 0.63;
const LEG_HALF: Vec3 = Vec3::new(0.19, 0.45, 0.20);
const ARM_HALF: Vec3 = Vec3::new(0.17, 0.45, 0.19);
const TORSO_HALF: Vec3 = Vec3::new(0.42, 0.45, 0.24);
const HEAD_HALF: f32 = 0.28;
const HOLD_ANGLE: f32 = 1.35;
const LIFT_EASE: f32 = 7.0;
const LIMB_BLEND: f32 = 9.0;
const HAND: f32 = -2.0 * ARM_HALF.y;
const FADE_ALPHA: f32 = 0.22;

#[derive(Resource, Default)]
pub struct UiHover(pub bool);

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
#[require(
  RigidBody::Kinematic,
  CustomPositionIntegration,
  SpeculativeMargin(0.0),
  LinearVelocity,
  Visibility
)]
pub struct Player;

#[derive(Component, Default)]
struct Gait {
  cycle: f32,
  lift: f32
}

#[derive(Component)]
struct Limb {
  swing: f32,
  lifted: f32,
  holds_light: bool
}

#[derive(Component)]
struct Flashlight;

#[derive(Resource)]
struct CameraRig {
  distance: f32,
  height: f32,
  yaw_speed: f32,
  pitch_speed: f32,
  zoom_speed: f32,
  zoom_range: Range<f32>,
  pitch_range: Range<f32>
}

impl Default for CameraRig {
  fn default() -> Self {
    Self {
      distance: 11.0,
      height: 1.9,
      yaw_speed: 0.0047,
      pitch_speed: 0.0037,
      zoom_speed: 2.8,
      zoom_range: 3.5..34.0,
      pitch_range: -1.15..0.62
    }
  }
}

fn body_shape() -> fidget::context::Tree {
  sdf::union([
    sdf::at(sdf::rounded_box(TORSO_HALF, 0.07), Vec3::new(0.0, HIP + TORSO_HALF.y, 0.0)),
    sdf::at(
      sdf::rounded_box(Vec3::splat(HEAD_HALF), 0.08),
      Vec3::new(0.0, SHOULDER + HEAD_HALF, 0.0)
    )
  ])
}

fn hanging_shape(half: Vec3) -> fidget::context::Tree {
  sdf::at(sdf::rounded_box(half, 0.06), Vec3::new(0.0, -half.y, 0.0))
}

fn torch_shape() -> fidget::context::Tree {
  sdf::union([
    sdf::at(sdf::along_z(sdf::cylinder(0.098, 0.25)), Vec3::new(0.0, 0.0, 0.17)),
    sdf::at(sdf::along_z(sdf::cylinder(0.165, 0.085)), Vec3::new(0.0, 0.0, -0.09))
  ])
}

fn spawn_player(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>
) {
  let skin = materials.add(StandardMaterial {
    base_color: Color::srgb(0.90, 0.72, 0.20),
    perceptual_roughness: 0.75,
    ..default()
  });
  let body = meshes.add(sdf::bake(
    body_shape(),
    sdf::Bounds::around(Vec3::new(0.0, 0.45, 0.0), 0.85, 6)
  ));
  let arm = meshes.add(sdf::bake(
    hanging_shape(ARM_HALF),
    sdf::Bounds::around(Vec3::new(0.0, -0.45, 0.0), 0.55, 6)
  ));
  let leg = meshes.add(sdf::bake(
    hanging_shape(LEG_HALF),
    sdf::Bounds::around(Vec3::new(0.0, -0.45, 0.0), 0.55, 6)
  ));
  let torch = meshes.add(sdf::bake(
    torch_shape(),
    sdf::Bounds::around(Vec3::new(0.0, 0.0, 0.09), 0.48, 6)
  ));
  let lens = meshes.add(sdf::bake(
    sdf::at(sdf::along_z(sdf::cylinder(0.138, 0.022)), Vec3::new(0.0, 0.0, -0.172)),
    sdf::Bounds::around(Vec3::new(0.0, 0.0, -0.17), 0.26, 6)
  ));

  let player = commands
    .spawn((
      Name::new("Player"),
      Player,
      Gait::default(),
      Collider::capsule(0.42, 2.0 * (-FOOT - 0.42)),
      Transform::from_xyz(-13.0, GROUND - FOOT + 1.2, 7.0)
    ))
    .id();

  commands.spawn((Mesh3d(body), MeshMaterial3d(skin.clone()), ChildOf(player)));
  for side in [-1.0, 1.0] {
    let arm = commands
      .spawn((
        Limb { swing: side, lifted: -2.3, holds_light: side > 0.0 },
        Mesh3d(arm.clone()),
        MeshMaterial3d(skin.clone()),
        Transform::from_xyz(side * 0.61, SHOULDER, 0.0),
        ChildOf(player)
      ))
      .id();
    commands.spawn((
      Limb { swing: -side, lifted: -0.45, holds_light: false },
      Mesh3d(leg.clone()),
      MeshMaterial3d(skin.clone()),
      Transform::from_xyz(side * 0.21, HIP, 0.0),
      ChildOf(player)
    ));

    if side > 0.0 {
      commands.spawn((
        Flashlight,
        SpotLight {
          color: Color::srgb(1.0, 0.95, 0.80),
          intensity: 0.0,
          range: 65.0,
          inner_angle: 0.26,
          outer_angle: 0.62,
          shadow_maps_enabled: true,
          ..default()
        },
        Mesh3d(torch.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
          base_color: Color::srgb(0.16, 0.17, 0.19),
          perceptual_roughness: 0.45,
          metallic: 0.6,
          ..default()
        })),
        NotShadowCaster,
        Transform::from_xyz(0.0, HAND + 0.10, 0.24)
          .with_rotation(Quat::from_rotation_x(HOLD_ANGLE) * Quat::from_rotation_y(PI)),
        children![(
          Mesh3d(lens.clone()),
          MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.97, 0.86),
            emissive: LinearRgba::rgb(22.0, 19.0, 11.0),
            unlit: true,
            ..default()
          })),
          NotShadowCaster,
        )],
        ChildOf(arm)
      ));
    }
  }

  commands.spawn((
    MainCamera,
    Camera3d::default(),
    Bloom { intensity: 0.34, ..Bloom::NATURAL },
    Projection::Perspective(PerspectiveProjection {
      fov: 68f32.to_radians(),
      far: 900.0,
      ..default()
    }),
    Transform::from_xyz(-13.0, 6.0, 18.0).looking_at(Vec3::new(-13.0, 2.5, 7.0), Vec3::Y)
  ));
}

fn sync_cursor(
  mouse: Res<ButtonInput<MouseButton>>,
  mut cursor: Single<&mut CursorOptions>
) {
  let turning = mouse.pressed(MouseButton::Right);
  cursor.visible = !turning;
  cursor.grab_mode =
    turning.then_some(CursorGrabMode::Locked).unwrap_or(CursorGrabMode::None);
}

fn move_player(
  time: Res<Time>,
  keys: Res<ButtonInput<KeyCode>>,
  belts: Query<(&ConveyorBelt, &GlobalTransform)>,
  camera: Single<&Transform, (With<MainCamera>, Without<Player>)>,
  player: Single<
    (Entity, &Collider, &mut Transform, &mut LinearVelocity, &mut Gait),
    With<Player>
  >,
  move_and_slide: MoveAndSlide
) {
  let (entity, collider, mut transform, mut velocity, mut gait) = player.into_inner();
  let flat = camera.forward().as_vec3().with_y(0.0).normalize_or_zero();
  let right = Vec3::new(-flat.z, 0.0, flat.x);
  let wish = [
    (KeyCode::KeyW, flat),
    (KeyCode::KeyS, -flat),
    (KeyCode::KeyD, right),
    (KeyCode::KeyA, -right)
  ]
  .into_iter()
  .filter(|&(key, _)| keys.pressed(key))
  .fold(Vec3::ZERO, |sum, (_, direction)| sum + direction)
  .normalize_or_zero()
    * WALK_SPEED;

  let filter = SpatialQueryFilter::from_excluded_entities([entity]);
  let footing = move_and_slide.spatial_query.cast_shape(
    collider,
    transform.translation.adjust_precision(),
    transform.rotation.adjust_precision(),
    Dir3::NEG_Y,
    &ShapeCastConfig::from_max_distance(GROUND_PROBE),
    &filter
  );
  let grounded = footing.is_some();
  let carry = footing
    .and_then(|hit| belts.get(hit.entity).ok())
    .map(|(belt, belt_transform)| {
      belt_transform.rotation() * belt.local_direction * belt.speed
    })
    .unwrap_or(Vec3::ZERO);

  let fall = velocity.y + GRAVITY * time.delta_secs();
  velocity.0 = Vec3::new(
    wish.x + carry.x,
    if grounded && keys.just_pressed(KeyCode::Space) {
      JUMP_SPEED
    } else if grounded {
      fall.max(GRAVITY * time.delta_secs())
    } else {
      fall
    },
    wish.z + carry.z
  );
  let flying = (!grounded && velocity.y.abs() > 0.5).then_some(1.0).unwrap_or(0.0);
  gait.lift = gait.lift.lerp(flying, 1.0 - (-LIFT_EASE * time.delta_secs()).exp());

  if wish.length_squared() > 0.0 {
    let facing = Quat::from_rotation_y(wish.x.atan2(wish.z));
    transform.rotation =
      transform.rotation.slerp(facing, 1.0 - (-TURN_RATE * time.delta_secs()).exp());
  }

  let MoveAndSlideOutput { position, projected_velocity } = move_and_slide
    .move_and_slide(
      collider,
      transform.translation.adjust_precision(),
      transform.rotation.adjust_precision(),
      velocity.0,
      time.delta(),
      &MoveAndSlideConfig::default(),
      &filter,
      |_| MoveAndSlideHitResponse::Accept
    );
  transform.translation = position;
  velocity.0 = projected_velocity;
}

fn nightfall(daylight: &Daylight) -> f32 { (1.0 - daylight.0 * 3.0).clamp(0.0, 1.0) }

fn animate_body(
  time: Res<Time>,
  daylight: Res<Daylight>,
  walker: Single<(&LinearVelocity, &mut Gait)>,
  mut limbs: Query<(&Limb, &mut Transform)>
) {
  let (velocity, mut gait) = walker.into_inner();
  let pace = velocity.0.with_y(0.0).length();
  gait.cycle += pace * time.delta_secs() * 2.1;
  let stride = gait.cycle.sin() * (pace / WALK_SPEED).min(1.0) * 0.85;
  let hold = (nightfall(&daylight) * 6.0).min(1.0);
  let blend = 1.0 - (-LIMB_BLEND * time.delta_secs()).exp();

  for (limb, mut transform) in &mut limbs {
    let posed = (stride * limb.swing).lerp(limb.lifted, gait.lift);
    let angle = limb.holds_light.then(|| posed.lerp(-HOLD_ANGLE, hold)).unwrap_or(posed);
    transform.rotation = transform.rotation.slerp(Quat::from_rotation_x(angle), blend);
  }
}

fn sweep_flashlight(
  daylight: Res<Daylight>,
  lamp: Single<(&mut SpotLight, &mut Visibility), With<Flashlight>>
) {
  let night = nightfall(&daylight);
  let (mut light, mut visibility) = lamp.into_inner();
  light.intensity = 6_500_000.0 * night;
  *visibility =
    (night > 0.05).then_some(Visibility::Inherited).unwrap_or(Visibility::Hidden);
}

fn follow_player(
  rig: Res<CameraRig>,
  mouse: Res<ButtonInput<MouseButton>>,
  motion: Res<AccumulatedMouseMotion>,
  hovering: Res<UiHover>,
  scroll: Res<AccumulatedMouseScroll>,
  player: Single<&Transform, With<Player>>,
  mut distance: Local<Option<f32>>,
  mut camera: Single<&mut Transform, (With<MainCamera>, Without<Player>)>
) {
  let reach = distance.get_or_insert(rig.distance);
  if !hovering.0 {
    *reach = (*reach - scroll.delta.y * rig.zoom_speed)
      .clamp(rig.zoom_range.start, rig.zoom_range.end);
  }

  let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
  let delta =
    mouse.pressed(MouseButton::Right).then_some(motion.delta).unwrap_or(Vec2::ZERO);
  camera.rotation = Quat::from_euler(
    EulerRot::YXZ,
    yaw - delta.x * rig.yaw_speed,
    (pitch - delta.y * rig.pitch_speed).clamp(rig.pitch_range.start, rig.pitch_range.end),
    0.0
  );
  let focus = player.translation + Vec3::Y * rig.height;
  camera.translation = focus - camera.forward() * *reach;
}

#[derive(Resource, Default)]
struct FadeCache(HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>);

#[derive(Component)]
struct Faded(Handle<StandardMaterial>);

fn fade_occluders(
  spatial: SpatialQuery,
  camera: Single<&GlobalTransform, With<MainCamera>>,
  player: Single<(Entity, &GlobalTransform), With<Player>>,
  parents: Query<&ChildOf>,
  mut painted: Query<(Entity, &mut MeshMaterial3d<StandardMaterial>, Option<&Faded>)>,
  mut cache: ResMut<FadeCache>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut commands: Commands
) {
  let (player, target) = *player;
  let eye = camera.translation();
  let gap = target.translation() - eye;
  let blocking: HashSet<Entity> = Dir3::new(gap)
    .map(|direction| {
      spatial
        .ray_hits(
          eye,
          direction,
          gap.length(),
          24,
          true,
          &SpatialQueryFilter::from_excluded_entities([player])
        )
        .into_iter()
        .filter_map(|hit| {
          std::iter::once(hit.entity)
            .chain(parents.iter_ancestors(hit.entity))
            .find(|&entity| painted.contains(entity))
        })
        .collect()
    })
    .unwrap_or_default();

  for (entity, mut material, faded) in &mut painted {
    match (blocking.contains(&entity), faded) {
      (true, None) => {
        let solid = material.0.clone();
        let ghost = cache
          .0
          .entry(solid.id())
          .or_insert_with(|| {
            let mut washed = materials.get(&solid).cloned().unwrap_or_default();
            washed.base_color = washed.base_color.with_alpha(FADE_ALPHA);
            washed.alpha_mode = AlphaMode::Blend;
            materials.add(washed)
          })
          .clone();
        commands.entity(entity).insert((Faded(solid), NotShadowCaster));
        material.0 = ghost;
      }
      (false, Some(Faded(solid))) => {
        material.0 = solid.clone();
        commands.entity(entity).remove::<(Faded, NotShadowCaster)>();
      }
      _ => {}
    }
  }
}

pub fn plugin(app: &mut App) {
  app
    .init_resource::<CameraRig>()
    .init_resource::<UiHover>()
    .init_resource::<FadeCache>()
    .add_systems(Startup, spawn_player)
    .add_systems(
      Update,
      (sync_cursor, move_player, follow_player, fade_occluders).chain().run_if(playing)
    )
    .add_systems(Update, (animate_body, sweep_flashlight));
}

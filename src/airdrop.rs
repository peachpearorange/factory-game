use {crate::{catalog::{MachineAssets, MachineKind},
             construction::Inventory,
             icon,
             material::{self, Coats},
             menu::playing,
             part::{self, Assembly, Axis, group},
             player::Player,
             store::{ToastStack, announce},
             style::{self, label, tinted},
             world::{GROUND, Roll}},
     bevy::{ecs::spawn::SpawnIter, light::NotShadowCaster, prelude::*},
     std::f32::consts::{FRAC_PI_2, PI, TAU}};

pub const INTERVAL: f32 = 200.0;

const CRUISE_HEIGHT: f32 = 56.0;
const CRUISE_SPEED: f32 = 28.0;
const RUN_HALF: f32 = 210.0;
const SCATTER: f32 = 6.0;
const PROP_SPEED: f32 = 34.0;

const CRATE_SIZE: f32 = 1.8;
const CRATE_BATTEN: f32 = 0.16;
const CRATE_REST: f32 = GROUND + CRATE_SIZE / 2.0;
const FALL_SPEED: f32 = 5.2;
const SWAY_RATE: f32 = 0.85;
const SWAY_REACH: f32 = 1.4;

const CANOPY_SPAN: f32 = 7.2;
const CANOPY_DOME: f32 = 2.9;
const CANOPY_WAIST: f32 = 0.66;
const CANOPY_GORES: u32 = 12;
const CANOPY_RISE: f32 = 5.4;
const DEFLATE: f32 = 1.1;

const OPEN_RANGE: f32 = 5.0;
const SHARDS: u32 = 18;
const SHARD_LIFE: f32 = 1.6;
const SHARD_FALL: f32 = 17.0;
const SHARD_BURST: f32 = 5.5;

const PRIZE_LIFE: f32 = 2.4;
const PRIZE_RISE: f32 = 5.0;
const PRIZE_SPIN: f32 = 2.6;
const PRIZE_GLOW: Color = Color::srgb(1.0, 0.88, 0.56);

const HULL_LENGTH: f32 = 13.0;
const HULL_WIDE: f32 = 2.4;
const HULL_TALL: f32 = 2.7;
const NOSE: f32 = HULL_LENGTH / 2.0;
const TAIL: f32 = -HULL_LENGTH / 2.0;
const WING_SPAN: f32 = 22.0;
const WING_CHORD: f32 = 3.4;
const WING_THICK: f32 = 0.44;
const WING_TOP: f32 = HULL_TALL / 2.0 + 0.2;
const ENGINE_OUT: f32 = 4.8;
const ENGINE_NOSE: f32 = 1.9;
const FIN_TALL: f32 = 3.8;
const GEAR_DROP: f32 = 1.35;

fn canopy() -> Assembly {
  let (rope, apex) = (material::ROPE, Vec3::Y * (CANOPY_RISE + CANOPY_DOME));
  let hem = |radius: f32, lift: f32, gore: u32| {
    let angle = gore as f32 * TAU / CANOPY_GORES as f32;
    Vec3::new(radius * angle.cos(), CANOPY_RISE + lift, radius * angle.sin())
  };
  let silk = |gore: u32| {
    let waist = CANOPY_SPAN * 0.5 * CANOPY_WAIST;
    let (inner, outer) = (
      [gore, gore + 1].map(|corner| hem(waist, CANOPY_DOME * 0.55, corner)),
      [gore, gore + 1].map(|corner| hem(CANOPY_SPAN / 2.0, 0.0, corner))
    );
    (gore % 2 == 0)
      .then_some(material::SILK)
      .unwrap_or(material::SIGNAL)
      .faces([vec![apex, inner[1], inner[0]], vec![
        inner[0], inner[1], outer[1], outer[0],
      ]])
  };
  let cord = |gore: u32| {
    let shoulder = Vec3::new(
      CRATE_SIZE / 2.0 * (gore as f32 * TAU / CANOPY_GORES as f32).cos(),
      CRATE_SIZE / 2.0,
      CRATE_SIZE / 2.0 * (gore as f32 * TAU / CANOPY_GORES as f32).sin()
    );
    rope.beam(1.0, 0.05).span(hem(CANOPY_SPAN / 2.0, 0.0, gore), shoulder)
  };
  (0..CANOPY_GORES).flat_map(|gore| [silk(gore), cord(gore)]).collect()
}

fn supply_crate() -> Assembly {
  let (boards, batten, iron, stencil, strap) =
    (material::BOARD, material::TIMBER, material::IRON, material::BRASS, material::BARN);
  let half = CRATE_SIZE / 2.0;
  let post = group([
    batten
      .beam(CRATE_SIZE, CRATE_BATTEN)
      .at(Vec3::new(half, 0.0, half))
      .tinted(material::TIMBER.shaded(0.8).color),
    iron.cube(0.24).at(Vec3::new(half, half, half)),
    iron.cube(0.24).at(Vec3::new(half, -half, half))
  ])
  .mirrored(Axis::X)
  .mirrored(Axis::Z);
  let rail = group([
    batten.slab(Vec3::new(CRATE_SIZE, CRATE_BATTEN, CRATE_BATTEN)).at(Vec3::new(
      0.0,
      half - 0.02,
      half
    )),
    batten.slab(Vec3::new(CRATE_BATTEN, CRATE_BATTEN, CRATE_SIZE)).at(Vec3::new(
      half,
      half - 0.02,
      0.0
    ))
  ])
  .mirrored(Axis::Y)
  .mirrored(Axis::X)
  .mirrored(Axis::Z);
  let band = |across: f32| {
    strap
      .slab(Vec3::new(CRATE_SIZE + 0.04, 0.16, CRATE_SIZE + 0.04))
      .at(Vec3::new(0.0, across, 0.0))
  };

  [
    group([
      boards.cube(CRATE_SIZE).solid(),
      band(0.42),
      band(-0.42),
      stencil.slab(Vec3::new(0.62, 0.05, 0.44)).on(Vec3::new(0.22, half, 0.0)),
      iron.rod(0.34, 0.07).on(Vec3::new(-0.42, half, 0.0)).rolled(FRAC_PI_2)
    ]),
    post,
    rail
  ]
  .into_iter()
  .collect()
}

fn splinter() -> Assembly { group([material::BOARD.slab(Vec3::new(0.78, 0.09, 0.30))]) }

fn engine() -> Assembly {
  let (cowl, iron, soot, brass) =
    (material::CASING, material::IRON, material::SOOT, material::BRASS);
  [
    group([
      cowl.rod(1.55, 3.3).tilted(FRAC_PI_2),
      iron.rod(1.74, 0.42).at(Vec3::X * 1.5).tilted(FRAC_PI_2),
      iron.rod(1.22, 0.34).at(Vec3::X * -1.7).tilted(FRAC_PI_2),
      brass.rod(0.52, 0.30).at(Vec3::X * 1.78).tilted(FRAC_PI_2),
      soot.slab(Vec3::new(1.5, 0.22, 0.22)).at(Vec3::new(-1.1, -0.70, 0.38)),
      soot.slab(Vec3::new(1.5, 0.22, 0.22)).at(Vec3::new(-1.1, -0.70, -0.38)),
      cowl.slab(Vec3::new(1.2, 0.72, 0.18)).at(Vec3::new(-0.2, -0.86, 0.0))
    ]),
    group([iron.cube(0.15)]).ringed(10, 0.70).tilted(FRAC_PI_2).at(Vec3::X * 1.5)
  ]
  .into_iter()
  .collect()
}

fn propeller() -> Assembly {
  let (blade, boss) = (material::SOOT, material::BRASS);
  [
    group([
      blade.slab(Vec3::new(1.9, 0.11, 0.52)).at(Vec3::X * 1.15).rolled(0.42),
      blade.slab(Vec3::new(0.5, 0.13, 0.34)).at(Vec3::X * 0.3)
    ])
    .ringed(4, 0.0)
    .tilted(FRAC_PI_2),
    group([
      boss.rod(0.5, 0.5).tilted(FRAC_PI_2),
      boss.ball(0.46).at(Vec3::X * 0.28).tinted(material::BRASS.shaded(0.8).color)
    ])
  ]
  .into_iter()
  .collect()
}

fn cargo_plane() -> Assembly {
  let (skin, trim, iron, glass, soot, brass) = (
    material::CASING,
    material::BARN,
    material::IRON,
    material::GLASS,
    material::SOOT,
    material::BRASS
  );
  let hull = group([
    skin.slab(Vec3::new(HULL_LENGTH, HULL_TALL, HULL_WIDE)),
    skin.slab(Vec3::new(1.4, HULL_TALL * 0.86, HULL_WIDE * 0.88)).at(Vec3::new(
      NOSE + 0.7,
      -0.06,
      0.0
    )),
    skin.slab(Vec3::new(1.1, HULL_TALL * 0.62, HULL_WIDE * 0.68)).at(Vec3::new(
      NOSE + 1.7,
      -0.22,
      0.0
    )),
    skin.ball(1.02).at(Vec3::new(NOSE + 1.95, -0.26, 0.0)),
    skin.slab(Vec3::new(2.6, HULL_TALL * 0.82, HULL_WIDE * 0.82)).at(Vec3::new(
      TAIL - 1.2,
      0.34,
      0.0
    )),
    skin.slab(Vec3::new(2.0, HULL_TALL * 0.46, HULL_WIDE * 0.48)).at(Vec3::new(
      TAIL - 2.7,
      0.92,
      0.0
    )),
    soot.slab(Vec3::new(0.5, HULL_TALL * 0.62, HULL_WIDE * 0.8)).at(Vec3::new(
      TAIL - 0.2,
      -0.2,
      0.0
    )),
    iron
      .slab(Vec3::new(3.2, 0.18, HULL_WIDE * 0.76))
      .at(Vec3::new(TAIL - 1.5, 0.15 - HULL_TALL / 2.0, 0.0))
      .tilted(0.5),
    trim
      .slab(Vec3::new(HULL_LENGTH + 2.0, 0.34, HULL_WIDE + 0.04))
      .at(Vec3::new(0.4, -0.55, 0.0)),
    trim.slab(Vec3::new(0.5, HULL_TALL * 0.9, HULL_WIDE + 0.06)).at(Vec3::new(
      TAIL + 1.0,
      0.1,
      0.0
    ))
  ]);
  let cockpit = group([
    glass.slab(Vec3::new(1.5, 0.78, HULL_WIDE * 0.86)).at(Vec3::new(
      NOSE + 0.5,
      0.52,
      0.0
    )),
    iron.slab(Vec3::new(0.09, 0.82, HULL_WIDE * 0.88)).at(Vec3::new(
      NOSE + 0.5,
      0.52,
      0.0
    )),
    skin
      .wedge(Vec3::new(1.1, 0.62, HULL_WIDE * 0.86))
      .at(Vec3::new(NOSE + 1.5, 0.42, 0.0))
      .turned(PI)
  ]);
  let wing = group([
    skin
      .slab(Vec3::new(WING_CHORD, WING_THICK, WING_SPAN))
      .at(Vec3::new(-0.5, WING_TOP, 0.0)),
    skin.slab(Vec3::new(1.1, 0.30, WING_SPAN * 0.98)).at(Vec3::new(
      -2.2,
      WING_TOP - 0.06,
      0.0
    ))
  ]);
  let tip = group([
    skin
      .slab(Vec3::new(WING_CHORD * 0.78, WING_THICK, 2.6))
      .at(Vec3::new(-0.6, WING_TOP + 0.36, WING_SPAN / 2.0 - 0.6))
      .rolled(-0.26),
    trim.slab(Vec3::new(WING_CHORD * 0.5, 0.22, 0.9)).at(Vec3::new(
      -0.6,
      WING_TOP + 0.7,
      WING_SPAN / 2.0 + 0.3
    ))
  ])
  .mirrored(Axis::Z);
  let tail = group([
    skin.slab(Vec3::new(2.6, FIN_TALL, 0.36)).on(Vec3::new(TAIL - 1.7, 1.0, 0.0)),
    trim.slab(Vec3::new(1.4, 0.9, 0.40)).at(Vec3::new(TAIL - 2.4, 3.9, 0.0)),
    skin.slab(Vec3::new(1.7, 0.26, 7.4)).at(Vec3::new(TAIL - 2.1, 1.15, 0.0)),
    brass.slab(Vec3::new(0.52, 0.30, 0.06)).at(Vec3::new(
      TAIL + 2.2,
      0.2,
      HULL_WIDE / 2.0
    ))
  ]);
  let gear = group([
    skin.slab(Vec3::new(2.6, 1.1, 1.0)).at(Vec3::new(
      -0.4,
      -HULL_TALL / 2.0 - 0.15,
      HULL_WIDE / 2.0 + 0.3
    )),
    iron.beam(GEAR_DROP, 0.22).under(Vec3::new(
      -0.4,
      -HULL_TALL / 2.0 - 0.6,
      HULL_WIDE / 2.0 + 0.3
    )),
    soot
      .rod(1.1, 0.42)
      .at(Vec3::new(-0.4, -HULL_TALL / 2.0 - 0.6 - GEAR_DROP, HULL_WIDE / 2.0 + 0.3))
      .rolled(FRAC_PI_2),
    iron.beam(1.4, 0.14).span(
      Vec3::new(-1.3, -HULL_TALL / 2.0 - 0.3, HULL_WIDE / 2.0 + 0.3),
      Vec3::new(-0.4, -HULL_TALL / 2.0 - 0.6 - GEAR_DROP, HULL_WIDE / 2.0 + 0.3)
    )
  ])
  .mirrored(Axis::Z);
  let mount = group([
    iron.slab(Vec3::new(1.0, 0.7, 0.9)).at(Vec3::new(-1.0, WING_TOP - 0.2, ENGINE_OUT)),
    iron.beam(1.1, 0.14).span(
      Vec3::new(-1.6, WING_TOP - 0.5, ENGINE_OUT),
      Vec3::new(-0.2, WING_TOP + 0.1, ENGINE_OUT - 0.9)
    )
  ])
  .mirrored(Axis::Z);

  [
    hull,
    cockpit,
    wing,
    tip,
    tail,
    gear,
    mount,
    engine().at(Vec3::new(0.2, WING_TOP, ENGINE_OUT)).mirrored(Axis::Z)
  ]
  .into_iter()
  .collect()
}

type Model = Vec<(Handle<Mesh>, Handle<StandardMaterial>)>;

fn built(
  assembly: Assembly,
  meshes: &mut Assets<Mesh>,
  coats: &mut Coats,
  materials: &mut Assets<StandardMaterial>
) -> Model {
  part::assembled(assembly)
    .into_iter()
    .map(|(coat, mesh)| (meshes.add(mesh), coats.of(coat, materials)))
    .collect()
}

fn modelled(model: &Model) -> impl Bundle {
  Children::spawn(SpawnIter(
    model
      .clone()
      .into_iter()
      .map(|(mesh, material)| (Mesh3d(mesh), MeshMaterial3d(material)))
  ))
}

#[derive(Resource)]
struct Kit {
  supply: Model,
  canopy: Model,
  splinter: Model
}

#[derive(Component)]
struct Plane;

#[derive(Component)]
struct Propeller;

#[derive(Component)]
struct Payload {
  kind: MachineKind,
  sway: f32,
  landed: bool
}

#[derive(Component)]
struct Canopy;

#[derive(Component)]
struct Deflating(Timer);

#[derive(Component)]
struct Splinter {
  drift: Vec3,
  tumble: Vec3,
  timer: Timer
}

#[derive(Component)]
struct Prize {
  kind: MachineKind,
  turn: f32,
  from: Vec3,
  timer: Timer
}

struct Flight {
  along: f32,
  bearing: f32,
  target: Vec3,
  cargo: Option<MachineKind>
}

fn waiting() -> Timer { Timer::from_seconds(crate::opts::opts().drop, TimerMode::Once) }

#[derive(Resource)]
struct Airdrop {
  clock: Timer,
  sortie: u32,
  flight: Option<Flight>
}

impl Default for Airdrop {
  fn default() -> Self { Self { clock: waiting(), sortie: 0, flight: None } }
}

fn spawn_airdrop(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut coats: ResMut<Coats>
) {
  let plane = commands
    .spawn((
      Name::new("Cargo Plane"),
      Plane,
      Visibility::Hidden,
      Transform::default(),
      part::spawned(
        part::assembled(cargo_plane()),
        &mut meshes,
        &mut coats,
        &mut materials
      )
    ))
    .id();
  let blades = built(propeller(), &mut meshes, &mut coats, &mut materials);
  for side in [-1.0, 1.0] {
    commands.spawn((
      Propeller,
      modelled(&blades),
      NotShadowCaster,
      Transform::from_xyz(ENGINE_NOSE + 0.2, WING_TOP, side * ENGINE_OUT),
      Visibility::default(),
      ChildOf(plane)
    ));
  }

  commands.insert_resource(Kit {
    supply: built(supply_crate(), &mut meshes, &mut coats, &mut materials),
    canopy: built(canopy(), &mut meshes, &mut coats, &mut materials),
    splinter: built(splinter(), &mut meshes, &mut coats, &mut materials)
  });
}

fn spin_propellers(time: Res<Time>, mut blades: Query<&mut Transform, With<Propeller>>) {
  for mut transform in &mut blades {
    transform.rotate_local_x(PROP_SPEED * time.delta_secs());
  }
}

fn fly_plane(
  time: Res<Time>,
  kit: Res<Kit>,
  stack: Single<Entity, With<ToastStack>>,
  plane: Single<(&mut Transform, &mut Visibility), With<Plane>>,
  mut airdrop: ResMut<Airdrop>,
  mut commands: Commands
) {
  let Airdrop { clock, sortie, flight } = &mut *airdrop;
  let (mut transform, mut visibility) = plane.into_inner();
  let leaving = match flight {
    None => {
      if clock.tick(time.delta()).is_finished() {
        *sortie += 1;
        let mut roll = Roll::seeded(time.elapsed_secs(), *sortie);
        let scatter = roll.next() * TAU;
        *flight = Some(Flight {
          along: -RUN_HALF,
          bearing: roll.next() * TAU,
          target: Vec3::new(
            scatter.cos() * roll.between(0.0, SCATTER),
            GROUND,
            scatter.sin() * roll.between(0.0, SCATTER)
          ),
          cargo: Some(MachineKind::ALL[roll.below(MachineKind::COUNT)])
        })
      }
      false
    }
    Some(Flight { along, bearing, target, cargo }) => {
      *along += CRUISE_SPEED * time.delta_secs();
      let heading = Quat::from_rotation_y(*bearing);
      let lane = *target + Vec3::Y * CRUISE_HEIGHT + heading * Vec3::X * *along;
      let swell = time.elapsed_secs();
      *transform =
        Transform::from_translation(lane + Vec3::Y * (swell * 0.7).sin() * 0.5)
          .with_rotation(
            heading
              * Quat::from_rotation_z((swell * 0.5).sin() * 0.02)
              * Quat::from_rotation_x((swell * 0.9).sin() * 0.05)
          );
      if *along >= 0.0
        && let Some(kind) = cargo.take()
      {
        let mut roll = Roll::seeded(time.elapsed_secs(), kind.index() as u32);
        commands
          .spawn((
            Payload { kind, sway: roll.next() * TAU, landed: false },
            modelled(&kit.supply),
            Transform::from_translation(lane),
            Visibility::default()
          ))
          .with_child((
            Canopy,
            modelled(&kit.canopy),
            Transform::default(),
            Visibility::default()
          ));
        announce(&mut commands, *stack, "A cargo plane is dropping a crate", style::TRADE)
      }
      *along > RUN_HALF
    }
  };
  if leaving {
    *flight = None;
    *clock = waiting()
  }
  *visibility =
    flight.is_some().then_some(Visibility::Inherited).unwrap_or(Visibility::Hidden);
}

fn sink_payload(
  time: Res<Time>,
  stack: Single<Entity, With<ToastStack>>,
  canopies: Query<(Entity, &ChildOf), With<Canopy>>,
  mut payloads: Query<(Entity, &mut Payload, &mut Transform)>,
  mut commands: Commands
) {
  for (entity, mut payload, mut transform) in &mut payloads {
    if !payload.landed {
      let swell = time.elapsed_secs() * SWAY_RATE + payload.sway;
      let glide = SWAY_REACH * time.delta_secs();
      transform.translation += Vec3::new(
        swell.cos() * glide,
        -FALL_SPEED * time.delta_secs(),
        (swell * 1.3).sin() * glide
      );
      transform.rotation = Quat::from_rotation_y(swell * 0.35)
        * Quat::from_rotation_z(swell.sin() * 0.05)
        * Quat::from_rotation_x((swell * 1.3).cos() * 0.04);
      payload.landed = transform.translation.y <= CRATE_REST;
      if payload.landed {
        transform.translation.y = CRATE_REST;
        transform.rotation = Quat::from_rotation_y(swell * 0.35);
        if let Some((canopy, _)) =
          canopies.iter().find(|(_, hung)| hung.parent() == entity)
        {
          commands
            .entity(canopy)
            .insert(Deflating(Timer::from_seconds(DEFLATE, TimerMode::Once)));
        }
        announce(&mut commands, *stack, "A supply crate has landed", style::TRADE)
      }
    }
  }
}

fn deflate_canopy(
  time: Res<Time>,
  mut canopies: Query<(Entity, &mut Deflating, &mut Transform)>,
  mut commands: Commands
) {
  for (entity, mut deflating, mut transform) in &mut canopies {
    let age = deflating.0.tick(time.delta()).fraction();
    transform.scale =
      Vec3::new(1.0 + age * 0.35, (1.0 - age).max(0.02), 1.0 + age * 0.35);
    transform.translation = Vec3::Y * -CANOPY_RISE * age * 0.8;
    if deflating.0.is_finished() {
      commands.entity(entity).despawn()
    }
  }
}

#[derive(Component)]
struct CratePrompt;

fn spawn_crate_prompt(mut commands: Commands) {
  commands.spawn((
    CratePrompt,
    GlobalZIndex(25),
    Node {
      position_type: PositionType::Absolute,
      bottom: px(230),
      left: percent(50),
      margin: UiRect::left(px(-150)),
      width: px(300),
      flex_direction: FlexDirection::Column,
      align_items: AlignItems::Center,
      row_gap: px(4),
      padding: UiRect::all(px(10)),
      border_radius: BorderRadius::all(px(9)),
      display: Display::None,
      ..default()
    },
    BackgroundColor(style::POPUP),
    children![
      (
        Node { align_items: AlignItems::Center, column_gap: px(8), ..default() },
        children![icon::parachute(style::TRADE), label("Supply Crate", style::BODY)],
      ),
      tinted("Press G to break it open", style::SMALL, style::TEXT_DIM),
    ]
  ));
}

fn reachable(at: Vec3, payload: &Payload, transform: &Transform) -> bool {
  payload.landed && transform.translation.distance(at) < OPEN_RANGE
}

fn show_crate_prompt(
  player: Single<&Transform, With<Player>>,
  payloads: Query<(&Payload, &Transform), Without<Player>>,
  mut prompt: Single<&mut Node, With<CratePrompt>>
) {
  prompt.display = payloads
    .iter()
    .any(|(payload, transform)| reachable(player.translation, payload, transform))
    .then_some(Display::Flex)
    .unwrap_or(Display::None);
}

fn open_crate(
  time: Res<Time>,
  keys: Res<ButtonInput<KeyCode>>,
  kit: Res<Kit>,
  assets: Res<MachineAssets>,
  player: Single<&Transform, With<Player>>,
  payloads: Query<(Entity, &Payload, &Transform), Without<Player>>,
  mut commands: Commands
) {
  if keys.just_pressed(KeyCode::KeyG)
    && let Some((entity, payload, transform)) =
      payloads.iter().find(|(_, payload, at)| reachable(player.translation, payload, at))
  {
    let burst = transform.translation;
    let mut roll = Roll::seeded(time.elapsed_secs(), payload.kind.index() as u32);
    commands.entity(entity).despawn();
    for splinter in 0..SHARDS {
      let angle = splinter as f32 * TAU / SHARDS as f32 + roll.between(-0.3, 0.3);
      let reach = roll.between(0.4, 1.0);
      commands.spawn((
        Splinter {
          drift: Vec3::new(
            angle.cos() * SHARD_BURST * reach,
            roll.between(2.5, 7.0),
            angle.sin() * SHARD_BURST * reach
          ),
          tumble: Vec3::new(
            roll.between(-9.0, 9.0),
            roll.between(-9.0, 9.0),
            roll.between(-9.0, 9.0)
          ),
          timer: Timer::from_seconds(SHARD_LIFE, TimerMode::Once)
        },
        modelled(&kit.splinter),
        NotShadowCaster,
        Transform::from_translation(
          burst
            + Vec3::new(
              roll.between(-0.6, 0.6),
              roll.between(-0.6, 0.6),
              roll.between(-0.6, 0.6)
            )
        ),
        Visibility::default()
      ));
    }
    commands.spawn((
      Prize {
        kind: payload.kind,
        turn: 0.0,
        from: burst,
        timer: Timer::from_seconds(PRIZE_LIFE, TimerMode::Once)
      },
      assets.model(payload.kind),
      PointLight { color: PRIZE_GLOW, intensity: 900_000.0, range: 22.0, ..default() },
      Transform::from_translation(burst),
      Visibility::default()
    ));
  }
}

fn scatter_splinters(
  time: Res<Time>,
  mut splinters: Query<(Entity, &mut Splinter, &mut Transform)>,
  mut commands: Commands
) {
  let step = time.delta_secs();
  for (entity, mut splinter, mut transform) in &mut splinters {
    splinter.drift.y -= SHARD_FALL * step;
    let (drift, tumble) = (splinter.drift, splinter.tumble);
    transform.translation += drift * step;
    transform.rotate(Quat::from_scaled_axis(tumble * step));
    if splinter.timer.tick(time.delta()).is_finished() {
      commands.entity(entity).despawn()
    }
  }
}

fn hoist_prize(
  time: Res<Time>,
  stack: Single<Entity, With<ToastStack>>,
  mut prizes: Query<(Entity, &mut Prize, &mut Transform)>,
  mut inventory: ResMut<Inventory>,
  mut commands: Commands
) {
  for (entity, mut prize, mut transform) in &mut prizes {
    let age = prize.timer.tick(time.delta()).fraction();
    prize.turn += PRIZE_SPIN * (0.6 + 6.0 * age * age) * time.delta_secs();
    let rise = (age / 0.55).min(1.0);
    let shrink = 1.0 - ((age - 0.8) / 0.2).clamp(0.0, 1.0);
    transform.translation = prize.from + Vec3::Y * PRIZE_RISE * rise * (2.0 - rise);
    transform.rotation =
      Quat::from_rotation_y(prize.turn) * Quat::from_rotation_z((age * TAU).sin() * 0.16);
    transform.scale = Vec3::splat((0.24 + 0.6 * rise).min(0.78) * shrink * shrink);
    if prize.timer.is_finished() {
      inventory.add(prize.kind);
      announce(
        &mut commands,
        *stack,
        &format!("{} lands in your inventory", prize.kind.name()),
        style::GRANTED
      );
      commands.entity(entity).despawn()
    }
  }
}

pub fn plugin(app: &mut App) {
  app
    .init_resource::<Airdrop>()
    .add_systems(Startup, (spawn_airdrop, spawn_crate_prompt))
    .add_systems(
      Update,
      (
        fly_plane,
        sink_payload,
        deflate_canopy,
        show_crate_prompt,
        open_crate,
        scatter_splinters,
        hoist_prize
      )
        .chain()
        .run_if(playing)
        .run_if(crate::showcase::idle)
    )
    .add_systems(Update, spin_propellers.run_if(crate::showcase::idle));
}

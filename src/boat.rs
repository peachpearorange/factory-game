use {crate::{block::{self, Block},
             catalog::{MachineKind, MachinePreviews},
             construction::Inventory,
             icon,
             machine::Money,
             menu::playing,
             player::Player,
             sdf,
             store::{self, HoverInfo, ToastStack, announce},
             style::{self, Bold, heavy, label, tinted},
             texture,
             world::{GROUND, SEA_LEVEL}},
     avian3d::prelude::*,
     bevy::{light::NotShadowCaster, math::Affine2, prelude::*}};

const DECK: f32 = GROUND - 0.8;
const WALK_FROM: f32 = 38.0;
const WALK_TO: f32 = 49.0;
const HEAD_FROM: f32 = 49.0;
const HEAD_TO: f32 = 55.0;
const WALK_HALF: f32 = 2.0;
const HEAD_HALF: f32 = 5.0;
const PLANK_HALF: f32 = 0.09;
const PILE_FOOT: f32 = -6.6;
const LANTERN_HEIGHT: f32 = 2.6;

const MOORING: Vec3 = Vec3::new(58.2, SEA_LEVEL, 0.0);
const OFFING: f32 = 110.0;
const CRUISE: f32 = 11.0;
const EASING: f32 = 26.0;
const VISIT: f32 = 75.0;
const INTERVAL: f32 = 170.0;
const TRADE_RANGE: f32 = 16.0;
const SLOTS: usize = 4;

const TIMBER: LinearRgba = LinearRgba::rgb(0.52, 0.36, 0.21);
const PILING: LinearRgba = LinearRgba::rgb(0.35, 0.26, 0.17);
const HULL: LinearRgba = LinearRgba::rgb(0.52, 0.15, 0.12);
const STRIPE: LinearRgba = LinearRgba::rgb(0.92, 0.84, 0.60);
const CABIN: LinearRgba = LinearRgba::rgb(0.11, 0.42, 0.50);
const PANE: LinearRgba = LinearRgba::rgb(0.05, 0.09, 0.12);
const CANVAS: LinearRgba = LinearRgba::rgb(1.0, 0.97, 0.90);
const PENNANT: LinearRgba = LinearRgba::rgb(0.84, 0.16, 0.14);
const CRATE: LinearRgba = LinearRgba::rgb(0.49, 0.34, 0.19);
const LANTERN_GLOW: Color = Color::srgb(1.0, 0.82, 0.48);

const BEAM: f32 = 2.4;
const LENGTH: f32 = 7.0;
const KEEL: f32 = -1.6;
const BOAT_DECK: f32 = 2.8;
const BULWARK: f32 = 3.5;
const PLANKING: f32 = 0.34;
const MAST_AT: f32 = -1.2;
const MAST_TOP: f32 = 10.0;
const YARD: f32 = 8.6;
const SAIL_MID: f32 = 6.3;
const CASTLE: f32 = 4.4;
const CASTLE_TOP: f32 = BOAT_DECK + 1.9;

const BOOT: LinearRgba = LinearRgba::rgb(0.13, 0.15, 0.16);

fn edge(from: Vec2, to: Vec2) -> (Vec2, f32) {
  let along = (to - from).normalize();
  let normal = Vec2::new(along.y, -along.x);
  let outward = normal * normal.x.signum();
  (outward, outward.dot(from))
}

fn hulled(bottom: f32, top: f32, half_length: f32, inset: f32) -> fidget::context::Tree {
  let (bow, bow_at) = edge(Vec2::new(BEAM, 1.5), Vec2::new(0.0, -LENGTH));
  let (chine, chine_at) = edge(Vec2::new(BEAM, 1.2), Vec2::new(0.9, KEEL));
  let planes = [-1.0, 1.0].into_iter().flat_map(move |side| {
    [
      sdf::half_space(Vec3::new(side * bow.x, 0.0, bow.y), bow_at - inset),
      sdf::half_space(Vec3::new(side * chine.x, chine.y, 0.0), chine_at - inset)
    ]
  });
  sdf::intersection(planes.chain([sdf::at(
    sdf::rounded_box(Vec3::new(BEAM - inset, (top - bottom) / 2.0, half_length), 0.1),
    Vec3::new(0.0, (top + bottom) / 2.0, 0.0)
  )]))
}

fn hull_shape() -> fidget::context::Tree {
  sdf::difference(
    hulled(KEEL, BULWARK, LENGTH, 0.0),
    hulled(BOAT_DECK, BULWARK + 2.0, LENGTH - PLANKING, PLANKING)
  )
}

fn hull_paint(at: Vec3, _: Vec3) -> LinearRgba {
  if at.y > BOAT_DECK + 0.52 {
    STRIPE
  } else if at.y > BOAT_DECK - 0.12 {
    TIMBER
  } else if at.y > 0.42 {
    HULL
  } else if at.y > -0.1 {
    STRIPE
  } else {
    BOOT
  }
}

fn timber(half: Vec3, at: Vec3) -> Block { Block::new(half, at, TIMBER) }

fn dock_blocks() -> impl Iterator<Item = Block> {
  let walk = |from: f32, to: f32, half: f32| {
    timber(
      Vec3::new((to - from) / 2.0, PLANK_HALF, half),
      Vec3::new((from + to) / 2.0, DECK - PLANK_HALF, 0.0)
    )
  };
  let piling = |at: Vec3| {
    Block::new(
      Vec3::new(0.18, (DECK - PILE_FOOT) / 2.0, 0.18),
      Vec3::new(at.x, (DECK + PILE_FOOT) / 2.0, at.z),
      PILING
    )
  };
  let bollard = |along: f32| {
    Block::new(
      Vec3::new(0.16, 0.34, 0.16),
      Vec3::new(HEAD_TO - 0.5, DECK + 0.34, along),
      PILING
    )
  };
  let lamp_post = |across: f32| {
    timber(
      Vec3::new(0.12, LANTERN_HEIGHT / 2.0, 0.12),
      Vec3::new(HEAD_FROM + 0.8, DECK + LANTERN_HEIGHT / 2.0, across)
    )
  };
  let rail = |across: f32| {
    timber(
      Vec3::new((WALK_TO - WALK_FROM) / 2.0, 0.16, 0.1),
      Vec3::new((WALK_FROM + WALK_TO) / 2.0, DECK + 0.5, across * WALK_HALF)
    )
  };
  let walk_piles = [WALK_FROM + 1.0, 42.0, 46.0]
    .into_iter()
    .flat_map(|along| [-1.0, 1.0].map(|side| Vec3::new(along, 0.0, side * 1.7)));
  let head_piles = [HEAD_FROM + 0.8, 52.0, HEAD_TO - 0.8].into_iter().flat_map(|along| {
    [-1.0, 1.0].map(|side| Vec3::new(along, 0.0, side * (HEAD_HALF - 0.5)))
  });

  [
    walk(WALK_FROM, WALK_TO, WALK_HALF),
    walk(HEAD_FROM, HEAD_TO, HEAD_HALF),
    rail(1.0),
    rail(-1.0),
    bollard(3.4),
    bollard(-3.4),
    lamp_post(HEAD_HALF - 0.9),
    lamp_post(0.9 - HEAD_HALF)
  ]
  .into_iter()
  .chain(walk_piles.chain(head_piles).map(piling))
}

fn fittings() -> impl Iterator<Item = Block> {
  let crate_at = |at: Vec3, half: f32| Block::new(Vec3::splat(half), at, CRATE);
  [
    Block::new(Vec3::new(0.26, 1.1, 0.42), Vec3::new(0.0, BULWARK + 0.7, -6.4), TIMBER),
    Block::new(Vec3::new(1.85, 1.1, 1.7), Vec3::new(0.0, BOAT_DECK + 1.1, CASTLE), CABIN),
    Block::new(
      Vec3::new(1.76, 0.22, 1.56),
      Vec3::new(0.0, BOAT_DECK + 1.2, CASTLE),
      PANE
    ),
    Block::new(Vec3::new(1.88, 0.12, 1.68), Vec3::new(0.0, CASTLE_TOP, CASTLE), TIMBER),
    Block::new(
      Vec3::new(0.17, (MAST_TOP - BOAT_DECK) / 2.0, 0.17),
      Vec3::new(0.0, (MAST_TOP + BOAT_DECK) / 2.0, MAST_AT),
      TIMBER
    ),
    Block::new(Vec3::new(3.2, 0.13, 0.13), Vec3::new(0.0, YARD, MAST_AT), TIMBER),
    crate_at(Vec3::new(0.9, BOAT_DECK + 0.5, -3.2), 0.5),
    crate_at(Vec3::new(-0.9, BOAT_DECK + 0.45, -2.2), 0.45),
    crate_at(Vec3::new(0.9, BOAT_DECK + 1.38, -3.2), 0.38)
  ]
  .into_iter()
}

fn canvas_blocks() -> impl Iterator<Item = Block> {
  [
    Block::new(Vec3::new(2.9, 2.2, 0.05), Vec3::new(0.0, SAIL_MID, MAST_AT), CANVAS),
    Block::new(
      Vec3::new(2.9, 0.42, 0.07),
      Vec3::new(0.0, SAIL_MID - 0.1, MAST_AT),
      PENNANT
    ),
    Block::new(
      Vec3::new(0.85, 0.34, 0.04),
      Vec3::new(0.85, MAST_TOP - 0.4, MAST_AT),
      PENNANT
    )
  ]
  .into_iter()
}

#[derive(Component)]
struct Boat;

fn spawn_harbour(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut images: ResMut<Assets<Image>>
) {
  let planked = |texture: Handle<Image>, tiling: Vec2| StandardMaterial {
    base_color_texture: Some(texture),
    uv_transform: Affine2::from_scale(Vec2::ONE / tiling),
    perceptual_roughness: 0.85,
    reflectance: 0.18,
    ..default()
  };
  let lantern = materials.add(StandardMaterial {
    base_color: LANTERN_GLOW,
    emissive: LinearRgba::rgb(30.0, 18.0, 7.0),
    ..default()
  });
  let flame = meshes.add(Cuboid::from_length(0.3));
  let glow = |at: Vec3| {
    (
      PointLight { color: LANTERN_GLOW, intensity: 900_000.0, range: 30.0, ..default() },
      Mesh3d(flame.clone()),
      MeshMaterial3d(lantern.clone()),
      NotShadowCaster,
      Transform::from_translation(at)
    )
  };

  let dock = commands
    .spawn((
      Name::new("Dock"),
      RigidBody::Static,
      Mesh3d(meshes.add(block::assembled(dock_blocks()))),
      MeshMaterial3d(
        materials.add(planked(images.add(texture::planks()), texture::PLANK))
      )
    ))
    .id();
  for (from, to, half) in
    [(WALK_FROM, WALK_TO, WALK_HALF), (HEAD_FROM, HEAD_TO, HEAD_HALF)]
  {
    commands.spawn((
      Collider::cuboid(to - from, PLANK_HALF * 2.0, half * 2.0),
      Transform::from_xyz((from + to) / 2.0, DECK - PLANK_HALF, 0.0),
      ChildOf(dock)
    ));
  }
  for across in [HEAD_HALF - 0.9, 0.9 - HEAD_HALF] {
    commands.spawn((
      glow(Vec3::new(HEAD_FROM + 0.8, DECK + LANTERN_HEIGHT + 0.2, across)),
      ChildOf(dock)
    ));
  }

  let boat = commands
    .spawn((
      Name::new("Trade Boat"),
      Boat,
      RigidBody::Kinematic,
      Visibility::Hidden,
      Mesh3d(meshes.add(block::merged(
        sdf::bake_painted(
          hull_shape(),
          sdf::Bounds::around(Vec3::new(0.0, 1.0, 0.0), LENGTH + 0.4, 8),
          hull_paint
        ),
        block::assembled(fittings())
      ))),
      MeshMaterial3d(materials.add(planked(images.add(texture::wood()), texture::GRAIN))),
      Transform::from_translation(MOORING + Vec3::Z * OFFING)
    ))
    .id();
  commands.spawn((
    Mesh3d(meshes.add(block::assembled(canvas_blocks()))),
    MeshMaterial3d(materials.add(StandardMaterial {
      perceptual_roughness: 0.95,
      reflectance: 0.05,
      double_sided: true,
      cull_mode: None,
      ..default()
    })),
    ChildOf(boat)
  ));
  commands.spawn((
    Collider::cuboid(BEAM * 2.0, BULWARK - KEEL, LENGTH * 2.0),
    Transform::from_xyz(0.0, (BULWARK + KEEL) / 2.0, 0.0),
    ChildOf(boat)
  ));
  commands.spawn((glow(Vec3::new(0.0, CASTLE_TOP + 0.4, CASTLE)), ChildOf(boat)));
}

struct Roll(u32);

impl Roll {
  fn next(&mut self) -> f32 {
    self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    let mixed = self.0 ^ (self.0 >> 15);
    (mixed.wrapping_mul(2_246_822_519) >> 8) as f32 / (1 << 24) as f32
  }

  fn below(&mut self, bound: usize) -> usize {
    ((self.next() * bound as f32) as usize).min(bound - 1)
  }

  fn drawn<T>(&mut self, from: &mut Vec<T>) -> T {
    let slot = self.below(from.len());
    from.remove(slot)
  }
}

struct Offer {
  kind: MachineKind,
  price: f32,
  sale: bool,
  sold: bool
}

impl Offer {
  fn caption(&self) -> String {
    if self.sold {
      "Sold".to_string()
    } else if self.sale {
      format!("SALE ${:.0}", self.price)
    } else {
      format!("${:.0}", self.price)
    }
  }
}

fn roll_stock(seed: u32) -> Vec<Offer> {
  let mut roll = Roll(seed);
  let mut shelf: Vec<MachineKind> =
    MachineKind::ALL.into_iter().filter(|kind| kind.spec().unlock.is_none()).collect();
  let discounted = roll.below(SLOTS);
  (0..SLOTS)
    .map(|slot| {
      let kind = roll.drawn(&mut shelf);
      let sale = slot == discounted;
      let haggle = (0.75 + 0.55 * roll.next()) * sale.then_some(0.5).unwrap_or(1.0);
      Offer { kind, price: (kind.spec().price * haggle).round(), sale, sold: false }
    })
    .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Passage {
  Away,
  Arriving,
  Docked,
  Leaving
}

#[derive(Resource)]
struct Voyage {
  passage: Passage,
  clock: Timer,
  lane: f32,
  visits: u32,
  stock: Vec<Offer>,
  fresh: bool
}

fn waiting() -> Timer {
  Timer::from_seconds(
    crate::env_secs("FACTORY_TRADE").unwrap_or(INTERVAL),
    TimerMode::Once
  )
}

impl Default for Voyage {
  fn default() -> Self {
    Self {
      passage: Passage::Away,
      clock: waiting(),
      lane: OFFING,
      visits: 0,
      stock: Vec::new(),
      fresh: false
    }
  }
}

fn sail_boat(
  time: Res<Time>,
  stack: Single<Entity, With<ToastStack>>,
  boat: Single<(&mut Transform, &mut Visibility), With<Boat>>,
  mut voyage: ResMut<Voyage>,
  mut commands: Commands
) {
  let glide = |lane: f32| CRUISE * (0.16 + 0.84 * (lane.abs() / EASING).min(1.0));
  match voyage.passage {
    Passage::Away => {
      if voyage.clock.tick(time.delta()).is_finished() {
        voyage.passage = Passage::Arriving;
      }
    }
    Passage::Arriving => {
      voyage.lane -= glide(voyage.lane) * time.delta_secs();
      if voyage.lane <= 0.0 {
        voyage.lane = 0.0;
        voyage.passage = Passage::Docked;
        voyage.visits += 1;
        voyage.stock = roll_stock(
          (time.elapsed_secs() * 997.0) as u32
            ^ voyage.visits.wrapping_mul(2_654_435_761)
        );
        voyage.fresh = true;
        voyage.clock = Timer::from_seconds(VISIT, TimerMode::Once);
        announce(&mut commands, *stack, "The trade boat is at the dock", style::TRADE);
      }
    }
    Passage::Docked => {
      if voyage.clock.tick(time.delta()).is_finished() {
        voyage.passage = Passage::Leaving;
        announce(&mut commands, *stack, "The trade boat casts off", style::SEALED);
      }
    }
    Passage::Leaving => {
      voyage.lane -= glide(voyage.lane) * time.delta_secs();
      if voyage.lane <= -OFFING {
        voyage.passage = Passage::Away;
        voyage.lane = OFFING;
        voyage.stock.clear();
        voyage.clock = waiting();
      }
    }
  }

  let (mut transform, mut visibility) = boat.into_inner();
  *visibility = (voyage.passage != Passage::Away)
    .then_some(Visibility::Inherited)
    .unwrap_or(Visibility::Hidden);
  let swell = time.elapsed_secs();
  transform.translation =
    MOORING + Vec3::new(0.0, (swell * 0.9).sin() * 0.08, voyage.lane);
  transform.rotation = Quat::from_rotation_z((swell * 0.7).sin() * 0.035)
    * Quat::from_rotation_x((swell * 1.3).sin() * 0.02);
}

#[derive(Component)]
struct TradePanel;

#[derive(Component)]
struct TradeShelf;

#[derive(Component)]
struct TradeClock;

#[derive(Component)]
struct TradeTile(usize);

#[derive(Component)]
struct TradePrice(usize);

fn spawn_trade_panel(mut commands: Commands) {
  let width =
    SLOTS as f32 * store::TILE + (SLOTS - 1) as f32 * store::GAP + 2.0 * store::PADDING;
  let panel = commands
    .spawn((
      TradePanel,
      Node {
        position_type: PositionType::Absolute,
        bottom: px(186),
        left: percent(50),
        margin: UiRect::left(px(-width / 2.0)),
        width: px(width),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(store::GAP),
        padding: UiRect::all(px(store::PADDING)),
        border_radius: BorderRadius::all(px(12)),
        display: Display::None,
        ..default()
      },
      BackgroundColor(style::PANEL),
      children![
        (
          Node { align_items: AlignItems::Center, column_gap: px(9), ..default() },
          children![icon::anchor(style::TRADE), label("Trade Boat", style::TITLE)],
        ),
        (tinted("", style::SMALL, style::TEXT_DIM), TradeClock),
      ]
    ))
    .id();
  commands.spawn((TradeShelf, store::grid(), ChildOf(panel)));
}

fn restock_shelf(
  previews: Res<MachinePreviews>,
  bold: Res<Bold>,
  shelf: Single<Entity, With<TradeShelf>>,
  tiles: Query<Entity, With<TradeTile>>,
  mut voyage: ResMut<Voyage>,
  mut commands: Commands
) {
  if voyage.fresh {
    voyage.fresh = false;
    for stale in &tiles {
      commands.entity(stale).despawn();
    }
    for (slot, offer) in voyage.stock.iter().enumerate() {
      let spec = offer.kind.spec();
      commands.spawn((
        TradeTile(slot),
        HoverInfo {
          title: spec.name.to_string(),
          detail: format!("{}\nOne only, while the boat is in.", spec.blurb)
        },
        store::tile(
          previews.image(offer.kind),
          spec.tier,
          spec.name,
          false,
          (heavy("", style::SMALL, style::INK, &bold), TradePrice(slot)),
          &bold
        ),
        ChildOf(*shelf)
      ));
    }
  }
}

fn refresh_trade(
  voyage: Res<Voyage>,
  money: Res<Money>,
  player: Single<&Transform, With<Player>>,
  mut panel: Single<&mut Node, With<TradePanel>>,
  mut clock: Single<&mut Text, With<TradeClock>>,
  mut prices: Query<(&TradePrice, &mut Text), Without<TradeClock>>,
  mut tiles: Query<(&TradeTile, &mut BackgroundColor)>
) {
  let trading = voyage.passage == Passage::Docked
    && player.translation.distance(MOORING) < TRADE_RANGE;
  panel.display = trading.then_some(Display::Flex).unwrap_or(Display::None);
  ***clock = format!("Casting off in {:.0}s", voyage.clock.remaining_secs());

  for (TradePrice(slot), mut text) in &mut prices {
    **text = voyage.stock.get(*slot).map(Offer::caption).unwrap_or_default();
  }
  for (TradeTile(slot), mut background) in &mut tiles {
    background.0 = voyage
      .stock
      .get(*slot)
      .map(|offer| {
        let swatch = offer.kind.spec().tier.swatch();
        (!offer.sold && money.0 >= offer.price)
          .then_some(swatch)
          .unwrap_or_else(|| swatch.mix(&style::MUTED, 0.6))
      })
      .unwrap_or(style::MUTED);
  }
}

fn trade(
  tiles: Query<(&TradeTile, &Interaction), Changed<Interaction>>,
  stack: Single<Entity, With<ToastStack>>,
  mut voyage: ResMut<Voyage>,
  mut money: ResMut<Money>,
  mut inventory: ResMut<Inventory>,
  mut commands: Commands
) {
  for (TradeTile(slot), interaction) in &tiles {
    if *interaction == Interaction::Pressed
      && let Some(offer) = voyage.stock.get_mut(*slot)
    {
      let spec = offer.kind.spec();
      let (announcement, tint) = if offer.sold {
        (format!("The {} is already gone", spec.name), style::SEALED)
      } else if money.0 < offer.price {
        (
          format!("${:.0} short of the {}", offer.price - money.0, spec.name),
          style::DENIED
        )
      } else {
        money.0 -= offer.price;
        inventory.add(offer.kind);
        offer.sold = true;
        (format!("Traded for a {}", spec.name), style::GRANTED)
      };
      announce(&mut commands, *stack, &announcement, tint);
    }
  }
}

pub fn plugin(app: &mut App) {
  app
    .init_resource::<Voyage>()
    .add_systems(Startup, (spawn_harbour, spawn_trade_panel))
    .add_systems(
      Update,
      (sail_boat, restock_shelf, trade, refresh_trade).chain().run_if(playing)
    );
}

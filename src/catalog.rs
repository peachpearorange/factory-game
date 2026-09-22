use {crate::{machine::{ConveyorBelt, Dropper, Furnace, Upgrader},
             material::{self, Coat, Coats, Finish},
             ore::{Effects, OreForm},
             part::{self, Assembly, Axis, IntoParts, Part, group},
             sdf},
     avian3d::prelude::*,
     bevy::{camera::{RenderTarget,
                     visibility::{NoFrustumCulling, RenderLayers}},
            ecs::spawn::SpawnIter,
            light::NotShadowCaster,
            prelude::*,
            render::render_resource::TextureFormat},
     bevy_hanabi::{AccelModifier, Attribute, ColorOverLifetimeModifier, EffectAsset,
                   ExprWriter, HanabiPlugin, LinearDragModifier, OrientMode,
                   OrientModifier, ParticleEffect, SetAttributeModifier,
                   SetPositionSphereModifier, ShapeDimension, SimulationSpace,
                   SizeOverLifetimeModifier, SpawnerSettings, VectorType},
     enum_assoc::Assoc,
     fidget::context::Tree,
     std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU}};

pub const CELL: f32 = 2.0;
pub const BELT_TOP: f32 = 0.22;
const ARROWS_PER_CELL: usize = 3;
const ARROW_INSET: f32 = 0.8;
const BELT_MID: f32 = 0.12;
const BELT_CURVE: f32 = 0.078;
const SLAT_THICK: f32 = 0.044;
const SLAT_HALF: f32 = 0.86;
const SLAT_PITCH: f32 = 0.21;
const SLAT_GAP: f32 = 0.05;
const BELT_SPEED: f32 = 1.8;
const RAIL_TOP: f32 = BELT_MID;
const HEARTH_HALF: f32 = 0.94;
const HEARTH_WALL: f32 = 0.11;
const HEARTH_PAN: f32 = 0.14;
const HEARTH_LIP: f32 = 0.16;
const HEARTH_RIM: f32 = 0.70;
const HEARTH_BACK: f32 = 1.30;
const HEARTH_COALS: f32 = HEARTH_PAN + 0.06;
const FURNACE_STACK: f32 = 1.86;
const TUB_CELLS: i32 = 2;
const TUB_STAVES: u32 = 8;
const TUB_WEIR_STAVE: u32 = 4;
const TUB_RADIUS: f32 = 1.40;
const TUB_STAVE: f32 = 0.16;
const TUB_WIDE: f32 = 1.20;
const TUB_HIGH: f32 = 0.94;
const TUB_RIM: f32 = TUB_HIGH + 0.10;
const TUB_BORE: f32 = 2.58;
const TUB_FLOOR: f32 = 0.14;
const TUB_WATER: f32 = 0.48;
const TUB_LEDGE: f32 = 0.16;
const TUB_WEIR: f32 = TUB_LEDGE + 0.06;
const TUB_BENCH: f32 = 0.28;
const TUB_HOOP_LOW: f32 = 0.26;
const TUB_HOOP_HIGH: f32 = 0.74;
const TUB_JET: f32 = 0.30;
const TUB_PUMP_WAY: f32 = -FRAC_PI_4;
const TUB_PUMP_OUT: f32 = 2.06;
const TUB_STEPS_WAY: f32 = 3.0 * FRAC_PI_4;
const TUB_STEPS_OUT: f32 = 2.09;
const TUB_COVER_WAY: f32 = FRAC_PI_4;
const TUB_COVER_OUT: f32 = 1.70;
const TUB_DUCK: Vec3 = Vec3::new(0.46, TUB_WATER, -0.62);
const TUB_GLOW: Color = Color::srgb(0.46, 0.90, 0.96);
const PAIL_LEG: f32 = -0.62;
const PAIL_LEG_THICK: f32 = 0.16;
const PAIL_SPAN: f32 = 0.92;
const PAIL_HEAD: f32 = 2.30;
const PAIL_TIE: f32 = 1.16;
const PAIL_SPOUT: Vec3 = Vec3::new(0.16, 1.46, 0.0);
const PAIL_TIP: f32 = 0.92;
const PAIL_DEEP: f32 = 1.12;
const PAIL_BORE: f32 = 1.00;
const PAIL_FOOT: f32 = 0.62;
const PAIL_SIDES: u32 = 10;
const PAIL_FACET: f32 = 0.34;
const PAIL_CHEEK: f32 = 0.52;
const PAIL_TRUNNION: f32 = 0.52;
const PAIL_WHEEL: f32 = 0.24;
const PAIL_TEETH: u32 = 20;
const PAIL_QUADRANT: u32 = 7;
const PAIL_TANK_BORE: f32 = 0.54;
const PAIL_TANK_LONG: f32 = 1.16;
const PAIL_TANK_RISE: f32 = 0.42;
const PAIL_PAN: Vec3 = Vec3::new(-0.92, 0.70, 0.0);
const PAIL_SPLASH: f32 = BELT_TOP + 0.03;
const JET_HEIGHT: f32 = BELT_TOP + 0.42;
const DROPPER_BACK: f32 = -0.42;
const CHUTE_FLOOR: f32 = 1.16;
const CHUTE_REACH: f32 = 1.52;
const DROP_HEIGHT: f32 = 0.92;
const CHUTE_SPOUT: Vec3 = Vec3::new(CHUTE_REACH, DROP_HEIGHT, 0.0);
const COOP_EAVES: f32 = CHUTE_FLOOR + 0.86;
const MINE_CELLS: i32 = 2;
const MINE_FACE: f32 = 0.30;
const MINE_BACK: f32 = -2.0;
const MINE_CREST: f32 = 2.60;
const MINE_DECK: f32 = 1.05;
const MINE_MOUTH: f32 = 1.20;
const MINE_BORE: f32 = 0.66;
const MINE_LIP: f32 = 2.45;
const MINE_TAIL: f32 = -0.20;
const MINE_HEAD: f32 = 2.40;
const MINE_SPOUT: Vec3 = Vec3::new(MINE_LIP - 0.12, MINE_DECK - 0.38, 0.0);
const WASH_BAR: f32 = 1.62;
const BRUSH_TOP: f32 = 1.30;
const BRUSH_HALF: f32 = 0.50;
const FLAPS_PER_CURTAIN: usize = 5;
const FLAP_REACH: f32 = 0.87;
const CHILL_CELLS: i32 = 2;
const GATES: [(f32, f32); 3] = [(0.95, 0.52), (1.20, 0.68), (1.45, 0.86)];
const GATE_WALL: f32 = 0.10;
const GATE_HALF: f32 = 0.08;
const GATE_STEP: f32 = 0.80;
const SPINE_TILT: f32 = 0.30;
const SPINE_MID: f32 = BELT_TOP + 1.30;
const DRUM_AT: Vec3 = Vec3::new(-0.34, 0.62, -0.78);
const GAUGE_AT: Vec3 = Vec3::new(DRUM_AT.x + 0.20, DRUM_AT.y + 0.18, DRUM_AT.z - 0.18);
const BONFIRE_TOP: f32 = 0.92;
const TORCH_HEAD: f32 = 1.25;
const LAMP_HEIGHT: f32 = 2.0;
const FLOOD_HEIGHT: f32 = 2.5;
const LAMP_SHADE: f32 = 0.34;
const FLOOD_SPAN: f32 = 0.44;
const LAMP_TILT: f32 = 0.55;
const FLOOD_TILT: f32 = 0.65;
const TORCH_GLOW: Color = Color::srgb(1.0, 0.66, 0.30);
const EMBER_GLOW: Color = Color::srgb(1.0, 0.42, 0.10);
const LAMP_GLOW: Color = Color::srgb(1.0, 0.95, 0.86);
const GAUGE_GLOW: Color = Color::srgb(1.0, 0.84, 0.36);

const fn belt_half(cells: i32) -> f32 { cells as f32 * CELL / 2.0 - 0.01 }

#[derive(Clone, Copy, PartialEq, Eq, Assoc)]
#[func(pub const fn swatch(self) -> Color)]
pub enum Tier {
  #[assoc(swatch = Color::srgb(0.91, 0.91, 0.92))]
  Plain,
  #[assoc(swatch = Color::srgb(0.75, 0.86, 0.96))]
  Sturdy,
  #[assoc(swatch = Color::srgb(0.71, 0.93, 0.74))]
  Refined,
  #[assoc(swatch = Color::srgb(0.99, 0.87, 0.55))]
  Exotic,
  #[assoc(swatch = Color::srgb(0.93, 0.72, 0.97))]
  Mythic
}

const fn stamps(multiplier: f32, effects: Effects) -> Upgrader {
  Upgrader { multiplier, effects, growth: 1.0 }
}

fn drops(every: f32, value: f32, form: OreForm, spout: Vec3) -> Dropper {
  Dropper { timer: Timer::from_seconds(every, TimerMode::Repeating), value, form, spout }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Assoc)]
#[func(pub const fn name(self) -> &'static str)]
#[func(pub const fn blurb(self) -> &'static str)]
#[func(pub const fn price(self) -> f32)]
#[func(pub const fn tier(self) -> Tier)]
#[func(pub const fn unlock(self) -> Option<&'static str>)]
#[func(pub const fn footprint(self) -> IVec2 { IVec2::ONE })]
#[func(pub const fn carries_belt(self) -> bool { false })]
#[func(pub const fn upgrade(self) -> Option<Upgrader>)]
#[func(const fn preview_spin(self) -> f32 { -0.6 })]
#[func(const fn finish(self) -> Finish { material::PLAIN })]
#[func(const fn paint(self) -> Option<fn(Vec3, Vec3) -> LinearRgba>)]
#[func(fn shape(self) -> Tree { arch() })]
#[func(fn parts(self) -> Option<Vec<Part>>)]
#[func(fn dropper(self) -> Option<Dropper>)]
#[func(fn model(self) -> Vec<(Coat, Mesh)> {
  self.parts().map(part::assembled).unwrap_or_else(|| {
    let finish = self.finish();
    vec![(
      finish.coat(),
      self
        .paint()
        .map(|paint| sdf::bake_painted(self.shape(), machine_bounds(self), paint))
        .unwrap_or_else(|| {
          part::painted(sdf::bake(self.shape(), machine_bounds(self)), finish.color)
        })
    )]
  })
})]
pub enum MachineKind {
  #[assoc(
    name = "Conveyor",
    blurb = "Carries ore one cell onward. Everything is downstream of something.",
    price = 25.0,
    tier = Tier::Plain,
    carries_belt = true,
    finish = material::RUBBER.tinted(LinearRgba::rgb(0.0732, 0.0783, 0.0946)),
    shape = belt_deck(belt_half(1))
  )]
  Conveyor,
  #[assoc(
    name = "Ore Dropper",
    blurb = "Coughs up a lump of rock every so often. Aim it at a belt.",
    price = 150.0,
    tier = Tier::Plain,
    paint = dropper_paint,
    shape = dropper_body(),
    dropper = drops(3.0, 12.0, OreForm::Rock, CHUTE_SPOUT)
  )]
  Dropper,
  #[assoc(
    name = "Chicken Coop",
    blurb = "A hen broods in the nest box and rolls a fresh egg down the ramp.",
    price = 110.0,
    tier = Tier::Plain,
    model = coop_model(),
    parts = coop_parts().solid().parts(),
    dropper = drops(4.2, 6.0, OreForm::Egg, CHUTE_SPOUT)
  )]
  Coop,
  #[assoc(
    name = "Gold Mine",
    blurb = "A shaft cut into a hill of gold-veined rock. Carts trundle out along \
             the trestle and tip a nugget off the end.",
    price = 2400.0,
    tier = Tier::Exotic,
    footprint = IVec2::splat(MINE_CELLS),
    parts = gold_mine().parts(),
    dropper = drops(5.0, 95.0, OreForm::Nugget, MINE_SPOUT)
  )]
  GoldMine,
  #[assoc(
    name = "Furnace",
    blurb = "Swallows whatever reaches it and pays out its value.",
    price = 250.0,
    tier = Tier::Sturdy,
    preview_spin = 2.3,
    parts = hearth().parts()
  )]
  Furnace,
  #[assoc(
    name = "Hot Tub",
    blurb = "A cedar tub kept at a rolling simmer. Ore slides over the weir, the \
             water swallows it, and the jets froth up its worth.",
    price = 900.0,
    tier = Tier::Refined,
    footprint = IVec2::splat(TUB_CELLS),
    preview_spin = 2.3,
    parts = hot_tub().parts()
  )]
  HotTub,
  #[assoc(
    name = "Flame Forge",
    blurb = "Sets passing ore alight and multiplies what it is worth.",
    price = 400.0,
    tier = Tier::Refined,
    carries_belt = true,
    upgrade = stamps(2.5, Effects::FIERY),
    finish = material::PLAIN.tinted(LinearRgba::rgb(0.1626, 0.0470, 0.0272)).lit(LinearRgba::rgb(0.85, 0.22, 0.03))
  )]
  Forge,
  #[assoc(
    name = "Flame Jet",
    blurb = "Blasts a lance of fire across the belt. Whatever passes comes out burning.",
    price = 620.0,
    tier = Tier::Refined,
    carries_belt = true,
    upgrade = stamps(3.2, Effects::FIERY),
    finish = material::PLATE.tinted(LinearRgba::rgb(0.1789, 0.1873, 0.2140)),
    shape = jet_nozzle()
  )]
  FlameJet,
  #[assoc(
    name = "Mist Coil",
    blurb = "Soaks ore through. Wet things carry charge differently.",
    price = 900.0,
    tier = Tier::Exotic,
    unlock = "Burn an ore worth over $500",
    carries_belt = true,
    upgrade = stamps(4.0, Effects::WET),
    finish = material::PLAIN.tinted(LinearRgba::rgb(0.0397, 0.0946, 0.1960)).lit(LinearRgba::rgb(0.06, 0.40, 0.85))
  )]
  MistCoil,
  #[assoc(
    name = "The Orewash",
    blurb = "Your ores need to be at the orewash to wash them.",
    price = 700.0,
    tier = Tier::Refined,
    carries_belt = true,
    upgrade = stamps(3.0, Effects::WET),
    finish = material::SHELL,
    paint = orewash_paint,
    shape = orewash_tunnel()
  )]
  Orewash,
  #[assoc(
    name = "Yellow Paint Bucket",
    blurb = "A pail of yellow gloss tipped over the belt on a jib. Anything that \
             crawls through the pour comes out yellow, and yellow sells better.",
    price = 800.0,
    tier = Tier::Refined,
    carries_belt = true,
    upgrade = stamps(3.4, Effects::YELLOW),
    parts = paint_bucket().parts()
  )]
  PaintBucket,
  #[assoc(
    name = "Chill Beam",
    blurb = "A long frost gantry. The beam rimes whatever crawls beneath it.",
    price = 1500.0,
    tier = Tier::Exotic,
    footprint = IVec2::new(CHILL_CELLS, 1),
    carries_belt = true,
    upgrade = stamps(5.0, Effects::FROSTY),
    finish = material::SHELL,
    paint = chill_paint,
    shape = chill_gantry()
  )]
  ChillBeam,
  #[assoc(
    name = "Decay Chamber",
    blurb = "Leaves ore humming and faintly green for a very long time.",
    price = 2200.0,
    tier = Tier::Mythic,
    unlock = "Burn 250 ore",
    carries_belt = true,
    upgrade = stamps(9.0, Effects::RADIOACTIVE),
    finish = material::PLAIN.tinted(LinearRgba::rgb(0.0470, 0.1329, 0.0397)).lit(LinearRgba::rgb(0.10, 0.85, 0.12))
  )]
  DecayChamber,
  #[assoc(
    name = "The Embiggener",
    blurb = "Ore goes in, a bigger ore comes out. A perfectly cromulent way to \
             raise its worth. The gates are solid, so an ore that has already \
             grown twice will not fit through the inlet.",
    price = 1100.0,
    tier = Tier::Exotic,
    carries_belt = true,
    upgrade = Upgrader { multiplier: 2.0, effects: Effects::NONE, growth: 1.35 },
    finish = material::PLATE,
    paint = embiggener_paint,
    shape = embiggener_frame()
  )]
  Embiggener,
  #[assoc(
    name = "Torch",
    blurb = "A burning brand on a stake. Keeps the dark off a corner of the floor.",
    price = 40.0,
    tier = Tier::Plain,
    finish = material::SANDED,
    shape = torch_post()
  )]
  Torch,
  #[assoc(
    name = "Bonfire",
    blurb = "A stacked heap of branches, well alight. Warms a wide stretch of floor.",
    price = 120.0,
    tier = Tier::Plain,
    finish = material::SANDED,
    shape = bonfire_pile()
  )]
  Bonfire,
  #[assoc(
    name = "Lamp Post",
    blurb = "Angles a tight beam across the floor. Rotate it to aim where you want.",
    price = 180.0,
    tier = Tier::Sturdy,
    finish = material::PLATE.tinted(LinearRgba::rgb(0.3424, 0.3672, 0.4200)),
    shape = lamp_post(LAMP_HEIGHT, LAMP_SHADE, LAMP_TILT)
  )]
  Lamp,
  #[assoc(
    name = "Floodlight",
    blurb = "A taller mast with a wide, hard beam. Lights a whole bank of machines.",
    price = 520.0,
    tier = Tier::Refined,
    finish = material::PLATE,
    paint = floodlight_paint,
    shape = flood_mast()
  )]
  Floodlight
}

impl MachineKind {
  pub const ALL: [Self; 18] = [
    Self::Conveyor,
    Self::Dropper,
    Self::Coop,
    Self::GoldMine,
    Self::Furnace,
    Self::HotTub,
    Self::Forge,
    Self::FlameJet,
    Self::MistCoil,
    Self::Orewash,
    Self::PaintBucket,
    Self::ChillBeam,
    Self::DecayChamber,
    Self::Embiggener,
    Self::Torch,
    Self::Bonfire,
    Self::Lamp,
    Self::Floodlight
  ];
  pub const COUNT: usize = Self::ALL.len();

  pub const fn index(self) -> usize { self as usize }

  pub const fn span(self, turns: u8) -> IVec2 {
    let footprint = self.footprint();
    match turns % 2 {
      0 => footprint,
      _ => IVec2::new(footprint.y, footprint.x)
    }
  }
}

fn belt_deck(half: f32) -> Tree {
  let drum = |side: f32| {
    sdf::at(
      sdf::along_z(sdf::cylinder(BELT_CURVE - 0.008, SLAT_HALF - 0.04)),
      Vec3::new(side * (half - BELT_CURVE), BELT_MID, 0.0)
    )
  };
  let rail = |side: f32| {
    sdf::at(
      sdf::cuboid(Vec3::new(half, RAIL_TOP / 2.0, 0.06)),
      Vec3::new(0.0, RAIL_TOP / 2.0, side * 0.95)
    )
  };
  sdf::union([
    sdf::at(
      sdf::cuboid(Vec3::new(half - BELT_CURVE, BELT_CURVE - 0.008, SLAT_HALF - 0.04)),
      Vec3::new(0.0, BELT_MID, 0.0)
    ),
    drum(1.0),
    drum(-1.0),
    rail(1.0),
    rail(-1.0)
  ])
}

fn belt_bed(half: f32) -> Assembly {
  let band = Vec3::new(
    (half - BELT_CURVE) * 2.0,
    (BELT_CURVE - 0.008) * 2.0,
    (SLAT_HALF - 0.04) * 2.0
  );
  let drums = group([material::STEEL
    .rod(band.y, band.z)
    .at(Vec3::new(half - BELT_CURVE, BELT_MID, 0.0))
    .rolled(FRAC_PI_2)])
  .mirrored(Axis::X);
  let rails = group([material::IRON
    .slab(Vec3::new(half * 2.0, RAIL_TOP, 0.12))
    .on(Vec3::new(0.0, 0.0, 0.95))])
  .mirrored(Axis::Z);

  group([material::RUBBER.slab(band).at(Vec3::Y * BELT_MID)]).with(drums).with(rails)
}

fn paint_bucket() -> Assembly {
  let (steel, iron, brass) = (material::STEEL, material::IRON, material::BRASS);
  let (paint, enamel, soot) = (material::PAINT, material::ENAMEL, material::SOOT);

  let tip = Quat::from_rotation_z(-PAIL_TIP);
  let heel = PAIL_SPOUT - tip * Vec3::new(PAIL_BORE / 2.0, PAIL_DEEP, 0.0);
  let hinge = heel + tip * (Vec3::Y * PAIL_TRUNNION);
  let mouth = heel + tip * (Vec3::Y * PAIL_DEEP);
  let yoke = Vec3::new(PAIL_LEG, PAIL_HEAD - 0.14, 0.0);
  let tank = Vec3::new(PAIL_LEG, PAIL_HEAD + PAIL_TANK_RISE, 0.0);

  let girth = |rise: f32| (PAIL_FOOT + (PAIL_BORE - PAIL_FOOT) * rise / PAIL_DEEP) / 2.0;
  let hoop = |rise: f32, thick: f32| {
    group([brass
      .slab(Vec3::new(0.05, thick, PAIL_FACET))
      .at(Vec3::X * (girth(rise) + 0.01))])
    .ringed(PAIL_SIDES, 0.0)
    .at(Vec3::Y * rise)
  };
  let lugs = group([
    steel.slab(Vec3::new(0.24, 0.32, 0.10)).at(Vec3::new(
      0.0,
      PAIL_TRUNNION,
      girth(PAIL_TRUNNION) + 0.03
    )),
    brass
      .rod(0.11, 0.40)
      .at(Vec3::new(0.0, PAIL_TRUNNION, girth(PAIL_TRUNNION) + 0.21))
      .rolled(FRAC_PI_2)
  ])
  .mirrored(Axis::Z);
  let bail = group([brass.beam(0.4, 0.05).span(
    Vec3::new(-0.06, PAIL_DEEP - 0.20, girth(PAIL_DEEP - 0.20) - 0.02),
    Vec3::new(-PAIL_BORE / 2.0 - 0.16, PAIL_DEEP + 0.16, 0.0)
  )])
  .mirrored(Axis::Z);
  let dribble = |across: f32, drop: f32| {
    enamel.slab(Vec3::new(0.06, drop, 0.11)).under(Vec3::new(
      girth(PAIL_DEEP) - 0.02,
      PAIL_DEEP - 0.07,
      across
    ))
  };
  let pail = group([
    paint.tapered(PAIL_FOOT, PAIL_BORE, PAIL_DEEP, PAIL_SIDES).on(Vec3::ZERO),
    enamel
      .tapered(PAIL_BORE - 0.05, PAIL_BORE - 0.05, 0.06, PAIL_SIDES)
      .under(Vec3::Y * (PAIL_DEEP - 0.01)),
    brass.tapered(PAIL_FOOT + 0.09, PAIL_FOOT + 0.09, 0.08, PAIL_SIDES).on(Vec3::ZERO),
    dribble(0.13, 0.42),
    dribble(-0.19, 0.25),
    dribble(0.30, 0.14),
    material::GRIT.slab(Vec3::new(0.04, 0.46, 0.24)).under(Vec3::new(
      -girth(PAIL_DEEP * 0.72),
      PAIL_DEEP * 0.72,
      0.0
    )),
    soot.slab(Vec3::new(0.28, 0.19, 0.03)).at(Vec3::new(
      0.0,
      PAIL_DEEP / 2.0,
      girth(PAIL_DEEP / 2.0) - 0.01
    ))
  ])
  .with(hoop(PAIL_DEEP - 0.09, 0.10))
  .with(hoop(0.34, 0.08))
  .with(lugs)
  .with(bail)
  .tilted(-PAIL_TIP)
  .at(heel);

  let stripes = group([soot
    .slab(Vec3::new(PAIL_LEG_THICK + 0.02, 0.14, PAIL_LEG_THICK + 0.02))
    .on(Vec3::new(PAIL_LEG, 0.24, PAIL_SPAN))])
  .repeated(3, Vec3::Y * 0.28);
  let footbolts =
    group([brass.cube(0.07)]).ringed(6, 0.18).at(Vec3::new(PAIL_LEG, 0.13, PAIL_SPAN));
  let leg = group([
    iron
      .slab(Vec3::new(0.46, 0.10, 0.26))
      .on(Vec3::new(PAIL_LEG, 0.0, PAIL_SPAN))
      .solid(),
    steel
      .beam(PAIL_HEAD - 0.10, PAIL_LEG_THICK)
      .on(Vec3::new(PAIL_LEG, 0.10, PAIL_SPAN))
      .solid(),
    steel.beam(0.6, 0.09).span(
      Vec3::new(PAIL_LEG + 0.46, 0.14, PAIL_SPAN),
      Vec3::new(PAIL_LEG + 0.02, PAIL_HEAD - 0.70, PAIL_SPAN)
    ),
    steel.slab(Vec3::new(0.34, 0.09, 0.22)).at(Vec3::new(
      PAIL_LEG + 0.10,
      PAIL_TIE,
      PAIL_SPAN
    ))
  ])
  .with(stripes)
  .with(footbolts)
  .mirrored(Axis::Z);
  let rungs = group([steel
    .rod(0.05, 0.36)
    .at(Vec3::new(PAIL_LEG - 0.13, 0.62, PAIL_SPAN))
    .rolled(FRAC_PI_2)])
  .repeated(5, Vec3::Y * 0.34);
  let gantry = group([
    steel
      .slab(Vec3::new(0.24, 0.16, PAIL_SPAN * 2.0 + 0.22))
      .under(Vec3::new(PAIL_LEG, PAIL_HEAD, 0.0)),
    steel.slab(Vec3::new(0.14, 0.12, PAIL_SPAN * 2.0)).at(Vec3::new(
      PAIL_LEG + 0.14,
      PAIL_TIE,
      0.0
    )),
    brass.slab(Vec3::new(0.26, 0.18, 0.03)).at(Vec3::new(
      PAIL_LEG,
      1.62,
      PAIL_SPAN - 0.10
    )),
    steel.slab(Vec3::new(0.30, 0.10, PAIL_CHEEK * 2.0 + 0.26)).at(yoke)
  ])
  .with(leg)
  .with(rungs);

  let cheeks = group([steel
    .slab(Vec3::new(0.30, 0.0, 0.08))
    .span(yoke + Vec3::Z * PAIL_CHEEK, hinge + Vec3::Z * PAIL_CHEEK)])
  .mirrored(Axis::Z);
  let handwheel = group([brass.rod(0.15, 0.12)])
    .with(
      group([iron
        .slab(Vec3::new(PAIL_WHEEL, 0.05, 0.05))
        .at(Vec3::X * PAIL_WHEEL / 2.0)])
      .ringed(5, 0.0)
    )
    .with(group([iron.slab(Vec3::new(0.08, 0.07, 0.15))]).ringed(12, PAIL_WHEEL))
    .rolled(FRAC_PI_2)
    .at(hinge + Vec3::Z * (PAIL_CHEEK + 0.20));
  let ratchet = part::ring(PAIL_TEETH, |tooth| {
    group([(tooth < PAIL_QUADRANT)
      .then(|| iron.slab(Vec3::new(0.10, 0.05, 0.08)).at(Vec3::X * 0.36))])
  })
  .with(iron.slab(Vec3::new(0.60, 0.06, 0.07)).at(Vec3::ZERO))
  .rolled(FRAC_PI_2)
  .at(hinge - Vec3::Z * (PAIL_CHEEK + 0.09));
  let pawl = iron.beam(0.3, 0.06).span(
    hinge + Vec3::new(0.0, 0.36, -PAIL_CHEEK - 0.09),
    hinge + Vec3::new(0.34, 0.04, -PAIL_CHEEK - 0.09)
  );

  let saddle = group([steel
    .slab(Vec3::new(0.10, PAIL_TANK_RISE, PAIL_TANK_BORE))
    .on(Vec3::new(PAIL_LEG, PAIL_HEAD, PAIL_TANK_LONG / 2.0 - 0.14))])
  .mirrored(Axis::Z);
  let hoops = group([brass
    .rod(PAIL_TANK_BORE + 0.05, 0.06)
    .at(tank + Vec3::Z * (PAIL_TANK_LONG / 2.0 - 0.22))
    .rolled(FRAC_PI_2)])
  .mirrored(Axis::Z);
  let reservoir = group([
    material::BARN.rod(PAIL_TANK_BORE, PAIL_TANK_LONG).at(tank).rolled(FRAC_PI_2),
    material::BARN
      .shaded(0.7)
      .rod(PAIL_TANK_BORE - 0.02, 0.05)
      .at(tank + Vec3::Z * PAIL_TANK_LONG / 2.0)
      .rolled(FRAC_PI_2),
    brass.rod(0.20, 0.10).on(tank + Vec3::new(0.0, PAIL_TANK_BORE / 2.0 - 0.02, 0.10)),
    material::GLASS
      .slab(Vec3::new(0.05, 0.36, 0.12))
      .at(tank + Vec3::new(0.26, 0.0, 0.0)),
    paint.slab(Vec3::new(0.04, 0.20, 0.10)).at(tank + Vec3::new(0.27, -0.08, 0.0)),
    soot.slab(Vec3::new(0.22, 0.18, 0.03)).at(tank + Vec3::new(0.0, 0.10, -0.30))
  ])
  .with(saddle)
  .with(hoops);

  let spigot = tank - Vec3::Y * (PAIL_TANK_BORE / 2.0);
  let elbow = Vec3::new(mouth.x, spigot.y - 0.14, 0.0);
  let nozzle = Vec3::new(mouth.x, mouth.y + 0.34, 0.0);
  let pipe = |from: Vec3, to: Vec3| brass.beam(0.5, 0.10).span(from, to);
  let valve = group([brass.slab(Vec3::new(0.18, 0.05, 0.05)).at(Vec3::X * 0.10)])
    .ringed(4, 0.0)
    .at(spigot - Vec3::Y * 0.20);
  let plumbing = group([
    pipe(spigot, elbow),
    pipe(elbow, nozzle),
    brass.ball(0.15).at(elbow),
    brass.rod(0.19, 0.24).under(nozzle),
    enamel.rod(0.09, 0.18).under(nozzle - Vec3::Y * 0.22),
    brass.rod(0.09, 0.22).at(spigot - Vec3::Y * 0.20)
  ])
  .with(valve);

  let hanger = group([steel.beam(0.4, 0.06).span(
    Vec3::new(PAIL_LEG, PAIL_TIE, PAIL_SPAN - 0.30),
    PAIL_PAN + Vec3::new(0.10, 0.06, 0.34)
  )])
  .mirrored(Axis::Z);
  let pan = group([
    iron.slab(Vec3::new(0.52, 0.06, 0.78)).under(PAIL_PAN),
    enamel.slab(Vec3::new(0.44, 0.09, 0.70)).under(PAIL_PAN - Vec3::Y * 0.02),
    paint.ball(0.11).at(PAIL_PAN + Vec3::new(0.10, 0.03, -0.22))
  ])
  .with(
    group([iron
      .slab(Vec3::new(0.52, 0.16, 0.05))
      .on(PAIL_PAN + Vec3::new(0.0, -0.06, 0.365))])
    .mirrored(Axis::Z)
  )
  .with(
    iron.slab(Vec3::new(0.05, 0.16, 0.78)).on(PAIL_PAN + Vec3::new(-0.235, -0.06, 0.0))
  )
  .with(hanger);

  let pour = group([
    enamel.tapered(0.18, 0.34, PAIL_SPOUT.y - PAIL_SPLASH, 8).on(Vec3::new(
      PAIL_SPOUT.x,
      PAIL_SPLASH,
      0.0
    )),
    enamel.ball(0.14).at(PAIL_SPOUT + Vec3::new(0.21, -0.42, 0.17)),
    enamel.ball(0.10).at(PAIL_SPOUT + Vec3::new(-0.13, -0.86, -0.08)),
    enamel.tapered(0.74, 0.62, 0.04, 8).on(Vec3::new(PAIL_SPOUT.x, PAIL_SPLASH, 0.0)),
    enamel.tapered(0.38, 0.30, 0.03, 7).on(Vec3::new(
      PAIL_SPOUT.x + 0.42,
      PAIL_SPLASH,
      -0.20
    )),
    enamel.tapered(0.26, 0.20, 0.03, 6).on(Vec3::new(
      PAIL_SPOUT.x - 0.38,
      PAIL_SPLASH,
      0.28
    )),
    paint.slab(Vec3::new(0.36, 0.04, 0.13)).on(Vec3::new(
      0.30,
      RAIL_TOP,
      PAIL_SPAN + 0.03
    )),
    paint.slab(Vec3::new(0.24, 0.04, 0.13)).on(Vec3::new(
      -0.40,
      RAIL_TOP,
      -PAIL_SPAN - 0.03
    ))
  ]);

  group([belt_bed(belt_half(1)), pail, gantry, cheeks, handwheel, ratchet, reservoir])
    .with(pawl)
    .with(plumbing)
    .with(pan)
    .with(pour)
}

fn arch() -> Tree {
  sdf::union([
    belt_deck(belt_half(1)),
    sdf::difference(
      sdf::at(
        sdf::rounded_box(Vec3::new(0.52, 1.25, 0.99), 0.12),
        Vec3::new(0.0, 1.05, 0.0)
      ),
      sdf::at(sdf::cuboid(Vec3::new(0.9, 0.86, 0.74)), Vec3::new(0.0, 0.76, 0.0))
    ),
    sdf::at(sdf::cylinder(0.2, 0.26), Vec3::new(0.0, 2.4, 0.0))
  ])
}

fn embiggener_frame() -> Tree {
  let gate = |slot: usize| {
    let (high, wide) = GATES[slot];
    let frame =
      |grow: f32| sdf::rounded_box(Vec3::new(GATE_HALF, high + grow, wide + grow), 0.06);
    sdf::at(
      sdf::difference(frame(GATE_WALL), frame(0.0)),
      Vec3::new(slot as f32 * GATE_STEP - GATE_STEP, BELT_TOP, 0.0)
    )
  };
  let spine = |side: f32| {
    sdf::at(
      sdf::rotate_z(sdf::cuboid(Vec3::new(0.84, 0.05, 0.05)), SPINE_TILT),
      Vec3::new(0.0, SPINE_MID, side * 0.55)
    )
  };
  let riser = sdf::union([
    sdf::at(sdf::cylinder(0.07, 0.30), Vec3::new(DRUM_AT.x, DRUM_AT.y + 0.76, DRUM_AT.z)),
    sdf::at(
      sdf::along_z(sdf::cylinder(0.07, 0.18)),
      Vec3::new(DRUM_AT.x, DRUM_AT.y + 1.02, DRUM_AT.z + 0.18)
    )
  ]);
  sdf::union([
    belt_deck(belt_half(1)),
    sdf::intersection([
      sdf::union([gate(0), gate(1), gate(2), spine(1.0), spine(-1.0)]),
      sdf::half_space(Vec3::NEG_Y, -BELT_TOP)
    ]),
    sdf::at(sdf::rounded_box(Vec3::new(0.26, 0.40, 0.22), 0.12), DRUM_AT),
    sdf::at(sdf::cylinder(0.13, 0.06), Vec3::new(DRUM_AT.x, DRUM_AT.y + 0.46, DRUM_AT.z)),
    riser
  ])
}

fn embiggener_paint(at: Vec3, _: Vec3) -> LinearRgba {
  const DECK: LinearRgba = LinearRgba::rgb(0.24, 0.25, 0.28);
  const STEEL: LinearRgba = LinearRgba::rgb(0.55, 0.58, 0.63);
  const BRASS: LinearRgba = LinearRgba::rgb(0.78, 0.58, 0.20);
  const GOLD: LinearRgba = LinearRgba::rgb(0.96, 0.78, 0.26);
  const BOILER: LinearRgba = LinearRgba::rgb(0.70, 0.17, 0.13);
  const COPPER: LinearRgba = LinearRgba::rgb(0.72, 0.40, 0.19);

  let boiler = (at.x - DRUM_AT.x).abs() < 0.34 && at.z < DRUM_AT.z + 0.40;

  if at.y < BELT_TOP + 0.02 {
    DECK
  } else if boiler {
    (at.y > DRUM_AT.y + 0.40).then(|| COPPER).unwrap_or(BOILER)
  } else if at.x < -GATE_STEP / 2.0 {
    STEEL
  } else if at.x < GATE_STEP / 2.0 {
    BRASS
  } else {
    GOLD
  }
}

fn dropper_body() -> Tree {
  let strut = |side: f32| {
    sdf::at(
      sdf::rounded_box(Vec3::new(0.05, 0.26, 0.05), 0.03),
      Vec3::new(0.30, CHUTE_FLOOR + 0.30, side * 0.26)
    )
  };
  sdf::union([
    sdf::at(
      sdf::rounded_box(Vec3::new(0.66, 0.08, 0.66), 0.05),
      Vec3::new(DROPPER_BACK, 0.08, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.34, 0.64, 0.34), 0.08),
      Vec3::new(DROPPER_BACK, 0.72, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.60, 0.46, 0.60), 0.10),
      Vec3::new(DROPPER_BACK, 1.78, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.70, 0.08, 0.70), 0.05),
      Vec3::new(DROPPER_BACK, 2.26, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.24, 0.34, 0.34), 0.08),
      Vec3::new(DROPPER_BACK + 0.10, 1.26, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(CHUTE_REACH / 2.0, 0.05, 0.30), 0.04),
      Vec3::new(CHUTE_REACH / 2.0 - 0.10, CHUTE_FLOOR, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.08, 0.26, 0.36), 0.06),
      Vec3::new(CHUTE_REACH - 0.14, CHUTE_FLOOR + 0.20, 0.0)
    ),
    strut(1.0),
    strut(-1.0)
  ])
}

fn dropper_paint(at: Vec3, _: Vec3) -> LinearRgba {
  const STEEL: LinearRgba = LinearRgba::rgb(0.20, 0.22, 0.26);
  const SHELL: LinearRgba = LinearRgba::rgb(0.62, 0.66, 0.72);
  const HULL: LinearRgba = LinearRgba::rgb(0.05, 0.19, 0.44);
  const HAZARD: LinearRgba = LinearRgba::rgb(0.92, 0.42, 0.02);

  if at.y > 2.16 || at.y < 0.20 {
    HAZARD
  } else if at.y > 1.30 {
    HULL
  } else if at.x > CHUTE_REACH - 0.24 {
    HAZARD
  } else if at.x > 0.0 {
    SHELL
  } else {
    STEEL
  }
}

fn machine_bounds(kind: MachineKind) -> sdf::Bounds {
  let reach = (kind.footprint().max_element() as f32 * CELL / 2.0 + 0.13).max(1.7);
  sdf::Bounds::around(Vec3::new(0.0, 1.45, 0.0), reach, 7)
}

const COOP_FRONT: f32 = 0.08;
const COOP_WALL: f32 = 0.06;
const COOP_MID: f32 = CHUTE_FLOOR + 0.44;
const RAMP_TOP: f32 = CHUTE_FLOOR + 0.05;

fn coop_parts() -> Assembly {
  let (timber, barn, shingle, straw) =
    (material::TIMBER, material::BARN, material::SHINGLE, material::STRAW);
  let legs = group([timber.beam(CHUTE_FLOOR, 0.14).on(Vec3::new(0.46, 0.0, 0.50))])
    .mirrored(Axis::X)
    .mirrored(Axis::Z)
    .at(Vec3::X * DROPPER_BACK);
  let walls = group([
    barn.slab(Vec3::new(1.12, 0.84, COOP_WALL * 2.0)).at(Vec3::new(
      DROPPER_BACK,
      COOP_MID,
      0.54
    )),
    barn
      .slab(Vec3::new(COOP_WALL * 2.0, 0.84, 0.40))
      .at(Vec3::new(COOP_FRONT, COOP_MID, 0.40))
  ])
  .mirrored(Axis::Z);
  let roofs = group([shingle
    .slab(Vec3::new(1.10, 0.10, 1.56))
    .at(Vec3::new(0.46, COOP_EAVES + 0.11, 0.0))
    .tilted(-0.6)])
  .mirrored(Axis::X)
  .at(Vec3::X * DROPPER_BACK);
  let rails = group([straw
    .slab(Vec3::new(CHUTE_REACH - 0.20, 0.12, 0.08))
    .at(Vec3::new(CHUTE_REACH / 2.0, CHUTE_FLOOR + 0.06, 0.30))])
  .mirrored(Axis::Z);

  let body = group([
    timber.slab(Vec3::new(1.24, 0.12, 1.32)).under(Vec3::new(
      DROPPER_BACK,
      CHUTE_FLOOR + 0.02,
      0.0
    )),
    barn.slab(Vec3::new(COOP_WALL * 2.0, 0.84, 1.20)).at(Vec3::new(
      DROPPER_BACK - 0.50,
      COOP_MID,
      0.0
    )),
    barn
      .slab(Vec3::new(COOP_WALL * 2.0, 0.26, 0.40))
      .under(Vec3::new(COOP_FRONT, COOP_EAVES, 0.0)),
    shingle.slab(Vec3::new(0.14, 0.10, 1.60)).at(Vec3::new(
      DROPPER_BACK,
      CHUTE_FLOOR + 1.26,
      0.0
    )),
    straw.slab(Vec3::new(CHUTE_REACH, 0.10, 0.64)).at(Vec3::new(
      CHUTE_REACH / 2.0 - 0.10,
      CHUTE_FLOOR,
      0.0
    )),
    straw.slab(Vec3::new(0.12, 0.24, 0.68)).at(Vec3::new(
      CHUTE_REACH - 0.14,
      CHUTE_FLOOR + 0.10,
      0.0
    ))
  ]);

  group([body, legs, walls, roofs, rails])
}

fn hen() -> Tree {
  let shank = |side: f32| {
    sdf::at(
      sdf::rounded_box(Vec3::new(0.035, 0.08, 0.035), 0.025),
      Vec3::new(0.68, RAMP_TOP + 0.08, side * 0.09)
    )
  };
  sdf::union([
    sdf::smooth_union(
      sdf::at(sdf::sphere(0.23), Vec3::new(0.66, CHUTE_FLOOR + 0.32, 0.0)),
      sdf::at(sdf::sphere(0.14), Vec3::new(0.82, CHUTE_FLOOR + 0.56, 0.0)),
      0.11
    ),
    sdf::at(
      sdf::rotate_z(sdf::rounded_box(Vec3::new(0.18, 0.05, 0.11), 0.04), 0.7),
      Vec3::new(0.42, CHUTE_FLOOR + 0.50, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.08, 0.04, 0.045), 0.03),
      Vec3::new(0.98, CHUTE_FLOOR + 0.54, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.06, 0.07, 0.03), 0.025),
      Vec3::new(0.82, CHUTE_FLOOR + 0.74, 0.0)
    ),
    shank(1.0),
    shank(-1.0)
  ])
}

fn hen_paint(at: Vec3, _: Vec3) -> LinearRgba {
  const PLUMAGE: LinearRgba = LinearRgba::rgb(0.99, 0.98, 0.95);
  const COMB: LinearRgba = LinearRgba::rgb(0.95, 0.16, 0.11);
  const BEAK: LinearRgba = LinearRgba::rgb(0.99, 0.74, 0.14);

  if at.x > 0.90 || at.y < CHUTE_FLOOR + 0.22 {
    BEAK
  } else if at.x > 0.60 && at.y > CHUTE_FLOOR + 0.70 {
    COMB
  } else {
    PLUMAGE
  }
}

fn coop_model() -> Vec<(Coat, Mesh)> {
  part::coated(
    part::assembled(coop_parts()),
    material::PLAIN.coat(),
    sdf::bake_painted(
      hen(),
      sdf::Bounds::around(Vec3::new(0.70, CHUTE_FLOOR + 0.42, 0.0), 0.62, 7),
      hen_paint
    )
  )
}

const MINE_FLANK: f32 = (1.95 - MINE_BORE) / 2.0;
const MINE_ROOF: f32 = MINE_DECK + MINE_MOUTH;

fn gold_mine() -> Assembly {
  let (stone, gold, timber) = (material::STONE, material::GOLD, material::TIMBER);
  let deck_long = MINE_LIP - MINE_TAIL;
  let deck_at = (MINE_LIP + MINE_TAIL) / 2.0;
  let bore_long = MINE_FACE - MINE_BACK - 0.85;
  let bore_at = MINE_FACE - bore_long / 2.0;
  let face_seam = |rise: f32, across: f32, reach: f32| {
    gold.slab(Vec3::new(0.04, 0.14, reach * 2.0)).at(Vec3::new(
      MINE_FACE + 0.02,
      rise,
      across
    ))
  };
  let side_seam = |side: f32, rise: f32, along: f32, reach: f32| {
    gold.slab(Vec3::new(reach * 2.0, 0.12, 0.04)).at(Vec3::new(along, rise, side * 1.97))
  };

  let hill = group([stone
    .slab(Vec3::new(MINE_FACE - MINE_BACK, MINE_CREST, MINE_FLANK * 2.0))
    .on(Vec3::new((MINE_FACE + MINE_BACK) / 2.0, 0.0, MINE_BORE + MINE_FLANK))
    .solid()])
  .mirrored(Axis::Z);
  let trestle = group([
    timber.beam(MINE_DECK, 0.16).on(Vec3::new(0.75, 0.0, 0.56)).solid(),
    timber.beam(MINE_HEAD - MINE_DECK, 0.14).on(Vec3::new(0.65, MINE_DECK, 0.62)).solid()
  ])
  .repeated(2, Vec3::X * 1.40)
  .mirrored(Axis::Z);
  let flanks = group([
    timber.slab(Vec3::new(deck_long, 0.14, 0.12)).on(Vec3::new(deck_at, MINE_DECK, 0.56)),
    material::STEEL
      .slab(Vec3::new(deck_long, 0.08, 0.10))
      .on(Vec3::new(deck_at, MINE_DECK, 0.24)),
    timber.slab(Vec3::new(1.40, 0.14, 0.14)).at(Vec3::new(1.35, MINE_HEAD, 0.62)),
    timber
      .slab(Vec3::new(0.20, MINE_MOUTH, 0.18))
      .on(Vec3::new(MINE_FACE, MINE_DECK, MINE_BORE + 0.09))
      .solid()
  ])
  .mirrored(Axis::Z);
  let sleepers = group([timber.slab(Vec3::new(0.14, 0.06, 0.76)).on(Vec3::new(
    MINE_TAIL + 0.35,
    MINE_DECK,
    0.0
  ))])
  .repeated(5, Vec3::X * 0.52);
  let yokes =
    group([timber.slab(Vec3::new(0.14, 0.14, 1.24)).at(Vec3::new(0.65, MINE_HEAD, 0.0))])
      .repeated(2, Vec3::X * 1.40);

  let seams = group([
    stone
      .slab(Vec3::new(0.85, MINE_CREST, 3.90))
      .on(Vec3::new(MINE_BACK + 0.425, 0.0, 0.0))
      .solid(),
    stone
      .slab(Vec3::new(bore_long, MINE_CREST - MINE_ROOF, MINE_BORE * 2.0))
      .under(Vec3::new(bore_at, MINE_CREST, 0.0))
      .solid(),
    stone
      .slab(Vec3::new(bore_long, MINE_DECK, MINE_BORE * 2.0))
      .on(Vec3::new(bore_at, 0.0, 0.0))
      .solid(),
    material::SHADOW.slab(Vec3::new(0.06, MINE_MOUTH, MINE_BORE * 2.0)).on(Vec3::new(
      MINE_FACE - bore_long + 0.03,
      MINE_DECK,
      0.0
    )),
    stone.slab(Vec3::new(1.70, 0.70, 2.90)).on(Vec3::new(-1.15, MINE_CREST, 0.0)).solid(),
    stone
      .slab(Vec3::new(0.90, 0.56, 1.60))
      .on(Vec3::new(-1.40, MINE_CREST + 0.70, 0.0))
      .solid(),
    stone.ball(1.04).at(Vec3::new(0.56, 0.52 * 0.78, 1.42)).solid(),
    stone.ball(0.88).at(Vec3::new(0.56, 0.44 * 0.78, -1.42)).solid(),
    face_seam(1.92, 1.24, 0.46),
    face_seam(0.74, -1.30, 0.38),
    face_seam(2.34, -0.95, 0.30),
    side_seam(1.0, 1.55, -0.60, 0.52),
    side_seam(1.0, 0.62, -1.45, 0.34),
    side_seam(-1.0, 2.05, -1.10, 0.44),
    side_seam(-1.0, 1.02, -0.35, 0.30),
    gold.slab(Vec3::new(0.60, 0.04, 0.72)).at(Vec3::new(-1.30, MINE_CREST + 0.72, 0.30)),
    gold.slab(Vec3::new(0.48, 0.04, 0.60)).at(Vec3::new(-0.55, MINE_CREST + 0.02, -0.80)),
    timber
      .slab(Vec3::new(deck_long, 0.12, 1.24))
      .under(Vec3::new(deck_at, MINE_DECK, 0.0))
      .solid(),
    material::STEEL
      .rod(0.48, 0.12)
      .at(Vec3::new(1.35, MINE_HEAD - 0.24, 0.0))
      .rolled(FRAC_PI_2),
    timber
      .slab(Vec3::new(0.20, 0.22, (MINE_BORE + 0.20) * 2.0))
      .on(Vec3::new(MINE_FACE, MINE_ROOF, 0.0))
  ]);

  group([seams, hill, trestle, flanks, sleepers, yokes])
}

fn hearth() -> Assembly {
  let (iron, brass) = (material::IRON, material::BRASS);
  let walls = group([
    iron
      .slab(Vec3::new(HEARTH_HALF * 2.0, HEARTH_RIM - HEARTH_PAN, HEARTH_WALL * 2.0))
      .on(Vec3::new(0.0, HEARTH_PAN, HEARTH_HALF - HEARTH_WALL)),
    brass
      .slab(Vec3::new(HEARTH_HALF * 2.0, 0.08, (HEARTH_WALL + 0.02) * 2.0))
      .on(Vec3::new(0.0, HEARTH_RIM, HEARTH_HALF - HEARTH_WALL))
  ])
  .mirrored(Axis::Z);

  let pan = group([
    material::SOOT
      .slab(Vec3::new(HEARTH_HALF * 2.0, HEARTH_PAN, HEARTH_HALF * 2.0))
      .on(Vec3::ZERO)
      .solid(),
    material::CINDER
      .slab(Vec3::new(
        (HEARTH_HALF - HEARTH_WALL) * 2.0,
        0.06,
        (HEARTH_HALF - HEARTH_WALL) * 2.0
      ))
      .on(Vec3::Y * HEARTH_PAN),
    iron
      .slab(Vec3::new(HEARTH_WALL * 2.0, HEARTH_BACK - HEARTH_PAN, HEARTH_HALF * 2.0))
      .on(Vec3::new(HEARTH_HALF - HEARTH_WALL, HEARTH_PAN, 0.0))
      .solid(),
    brass.slab(Vec3::new(HEARTH_WALL * 2.0, 0.08, HEARTH_HALF * 2.0)).on(Vec3::new(
      HEARTH_HALF - HEARTH_WALL,
      HEARTH_BACK,
      0.0
    )),
    brass
      .slab(Vec3::new(HEARTH_WALL * 2.0, HEARTH_LIP, HEARTH_HALF * 2.0))
      .on(Vec3::new(HEARTH_WALL - HEARTH_HALF, 0.0, 0.0))
      .solid(),
    iron.slab(Vec3::new(0.36, FURNACE_STACK - HEARTH_BACK, 0.36)).on(Vec3::new(
      HEARTH_HALF - 0.30,
      HEARTH_BACK,
      0.0
    )),
    brass.slab(Vec3::new(0.48, 0.12, 0.48)).on(Vec3::new(
      HEARTH_HALF - 0.30,
      FURNACE_STACK,
      0.0
    ))
  ]);

  group([pan, walls])
}

fn hot_tub() -> Assembly {
  let (cedar, coping, brass, iron) =
    (material::BOARD, material::TIMBER, material::BRASS, material::IRON);
  let (water, foam) = (material::WATER, material::FOAM);

  let ring = part::ring(TUB_STAVES, |spoke| {
    let weir = spoke == TUB_WEIR_STAVE;
    let hoop = |rise: f32| {
      brass.slab(Vec3::new(0.05, 0.09, TUB_WIDE + 0.05)).at(Vec3::new(
        TUB_RADIUS + TUB_STAVE / 2.0,
        rise,
        0.0
      ))
    };
    group([cedar
      .slab(Vec3::new(TUB_STAVE, weir.then_some(TUB_LEDGE).unwrap_or(TUB_HIGH), TUB_WIDE))
      .on(Vec3::X * TUB_RADIUS)
      .solid()])
    .with(weir.then(|| {
      brass
        .slab(Vec3::new(0.30, TUB_WEIR - TUB_LEDGE, TUB_WIDE))
        .on(Vec3::new(TUB_RADIUS, TUB_LEDGE, 0.0))
    }))
    .with((!weir).then(|| {
      group([
        coping.slab(Vec3::new(0.36, TUB_RIM - TUB_HIGH, TUB_WIDE + 0.08)).on(Vec3::new(
          TUB_RADIUS + 0.03,
          TUB_HIGH,
          0.0
        )),
        hoop(TUB_HOOP_LOW),
        hoop(TUB_HOOP_HIGH),
        cedar.slab(Vec3::new(0.42, 0.10, TUB_WIDE * 0.84)).on(Vec3::new(
          TUB_RADIUS - 0.48,
          TUB_BENCH,
          0.0
        )),
        brass
          .rod(0.15, 0.14)
          .at(Vec3::new(TUB_RADIUS - 0.24, TUB_JET, 0.0))
          .tilted(FRAC_PI_2)
      ])
    }))
  });
  let bolts = group([brass.cube(0.08).under(Vec3::X * (TUB_RADIUS + 0.14))])
    .at(Vec3::Y * (TUB_RIM - 0.02))
    .ringed(16, 0.0);
  let pump = group([
    iron.slab(Vec3::new(0.62, 0.52, 0.80)).on(Vec3::ZERO).solid(),
    material::STEEL.slab(Vec3::new(0.66, 0.06, 0.84)).on(Vec3::Y * 0.52),
    material::SOOT.rod(0.34, 0.30).at(Vec3::Y * 0.30).rolled(FRAC_PI_2),
    brass.rod(0.10, 0.26).at(Vec3::new(0.0, 0.66, 0.24)),
    material::GLASS.ball(0.20).at(Vec3::new(0.0, 0.62, -0.28))
  ])
  .radial(TUB_PUMP_WAY, TUB_PUMP_OUT);
  let steps = group([
    cedar.slab(Vec3::new(0.40, 0.14, 0.86)).on(Vec3::ZERO),
    cedar.slab(Vec3::new(0.40, 0.14, 0.86)).on(Vec3::new(-0.34, 0.30, 0.0)),
    cedar.beam(0.30, 0.10).on(Vec3::new(-0.10, 0.14, 0.34)),
    cedar.beam(0.30, 0.10).on(Vec3::new(-0.10, 0.14, -0.34))
  ])
  .radial(TUB_STEPS_WAY, TUB_STEPS_OUT);
  let cover = group([
    cedar.slab(Vec3::new(1.36, 0.12, 1.28)).at(Vec3::new(-0.10, 0.66, 0.0)).tilted(1.26),
    brass.slab(Vec3::new(1.32, 0.04, 0.10)).at(Vec3::new(-0.04, 0.66, 0.0)).tilted(1.26),
    brass.rod(0.12, 0.10).at(Vec3::new(-0.32, 1.28, 0.0)).rolled(FRAC_PI_2)
  ])
  .radial(TUB_COVER_WAY, TUB_COVER_OUT);
  let towel = group([
    material::SIGNAL.slab(Vec3::new(0.50, 0.05, 0.44)).on(Vec3::Y * TUB_RIM),
    material::SIGNAL
      .slab(Vec3::new(0.06, 0.62, 0.44))
      .under(Vec3::new(0.24, TUB_RIM, 0.0))
      .tilted(0.16)
  ])
  .radial(-FRAC_PI_2, TUB_RADIUS + 0.06);
  let panel = group([
    iron.slab(Vec3::new(0.16, 0.26, 0.42)).on(Vec3::Y * TUB_RIM).solid(),
    material::GLASS.slab(Vec3::new(0.04, 0.16, 0.30)).at(Vec3::new(
      0.09,
      TUB_RIM + 0.15,
      0.0
    )),
    brass
      .cube(0.06)
      .at(Vec3::new(0.06, TUB_RIM + 0.05, 0.14))
      .lit(LinearRgba::rgb(1.8, 0.9, 0.2)),
    brass
      .cube(0.06)
      .at(Vec3::new(0.06, TUB_RIM + 0.05, -0.02))
      .lit(LinearRgba::rgb(0.2, 1.6, 0.6))
  ])
  .radial(FRAC_PI_2, TUB_RADIUS);
  let duck = group([
    material::BUOY.ball(0.30).at(TUB_DUCK),
    material::BUOY.ball(0.19).at(TUB_DUCK + Vec3::new(0.13, 0.19, 0.0)),
    material::TANGERINE
      .wedge(Vec3::new(0.14, 0.07, 0.10))
      .at(TUB_DUCK + Vec3::new(0.27, 0.18, 0.0))
  ]);
  let pipe = |thick: f32, rise: f32| {
    brass.beam(0.86, thick).span(
      part::around(TUB_PUMP_WAY, TUB_PUMP_OUT - 0.30) + Vec3::Y * rise,
      part::around(TUB_PUMP_WAY, TUB_RADIUS) + Vec3::Y * rise
    )
  };
  let basin = group([
    cedar.rod(TUB_BORE, TUB_FLOOR).on(Vec3::ZERO).solid(),
    water.rod(TUB_BORE - 0.06, TUB_WATER - TUB_FLOOR).on(Vec3::Y * TUB_FLOOR),
    foam.ball(0.46).at(Vec3::new(-0.34, TUB_WATER, 0.52)),
    foam.ball(0.30).at(Vec3::new(0.62, TUB_WATER - 0.02, 0.18)),
    foam.ball(0.36).at(Vec3::new(0.08, TUB_WATER + 0.02, -0.54)),
    water
      .slab(Vec3::new(0.20, TUB_WATER - TUB_WEIR + 0.10, TUB_WIDE - 0.34))
      .under(Vec3::new(-(TUB_RADIUS + 0.02), TUB_WATER - 0.02, 0.0))
      .tilted(-0.18),
    brass.slab(Vec3::new(0.34, 0.03, 0.18)).at(Vec3::new(
      TUB_RADIUS + TUB_STAVE / 2.0 + 0.02,
      0.50,
      0.34
    )),
    pipe(0.11, 0.34),
    pipe(0.09, TUB_HOOP_HIGH)
  ]);

  group([basin, ring, bolts, pump, steps, cover, towel, panel, duck])
}

fn bonfire_pile() -> Tree {
  let stick = sdf::rounded_box(Vec3::new(0.07, 0.62, 0.07), 0.055);
  let leaning = (0..7).map(|spoke| {
    let spin = spoke as f32 * TAU / 7.0;
    sdf::rotate_y(
      sdf::at(sdf::rotate_z(stick.clone(), 0.42), Vec3::new(0.26, 0.50, 0.0)),
      spin
    )
  });
  let logs = (0..3).map(|log| {
    let spin = log as f32 * TAU / 3.0 + 0.4;
    sdf::rotate_y(
      sdf::at(
        sdf::along_x(sdf::rounded_cylinder(0.11, 0.66, 0.06)),
        Vec3::new(0.0, 0.11, 0.30)
      ),
      spin
    )
  });
  sdf::union(leaning.chain(logs))
}

fn jet_nozzle() -> Tree {
  sdf::union([
    belt_deck(belt_half(1)),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.34, 0.46, 0.30), 0.09),
      Vec3::new(0.0, 0.46, -1.12)
    ),
    sdf::at(
      sdf::along_z(sdf::rounded_cylinder(0.17, 0.34, 0.06)),
      Vec3::new(0.0, JET_HEIGHT, -0.74)
    ),
    sdf::at(sdf::along_z(sdf::cylinder(0.09, 0.22)), Vec3::new(0.0, JET_HEIGHT, -0.36)),
    sdf::at(sdf::cylinder(0.13, 0.55), Vec3::new(0.0, 1.30, -1.12)),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.30, 0.12, 0.12), 0.06),
      Vec3::new(0.0, 1.85, -1.12)
    )
  ])
}

fn orewash_tunnel() -> Tree {
  let rail = |side: f32| {
    sdf::at(
      sdf::rounded_box(Vec3::new(0.94, 0.20, 0.08), 0.06),
      Vec3::new(0.0, 0.46, side * 0.93)
    )
  };
  let nozzle = |along: f32| {
    sdf::at(sdf::cylinder(0.07, 0.16), Vec3::new(along * 0.46, WASH_BAR - 0.16, 0.0))
  };
  let posts = (0..4).map(|post| {
    let (side, along) = ((post % 2) as f32 * 2.0 - 1.0, (post / 2) as f32 * 2.0 - 1.0);
    sdf::at(
      sdf::rounded_box(Vec3::new(0.07, WASH_BAR / 2.0, 0.07), 0.04),
      Vec3::new(along * 0.86, WASH_BAR / 2.0, side * 0.90)
    )
  });
  let brush = |side: f32| {
    let bristles = (0..9).map(move |fin| {
      sdf::rotate_y(
        sdf::at(
          sdf::rounded_box(Vec3::new(0.21, BRUSH_HALF, 0.05), 0.04),
          Vec3::new(0.0, BRUSH_TOP - BRUSH_HALF, side * 0.62)
        ),
        fin as f32 * TAU / 9.0
      )
    });
    sdf::union(bristles.chain([sdf::at(
      sdf::rounded_cylinder(0.10, BRUSH_HALF, 0.06),
      Vec3::new(0.0, BRUSH_TOP - BRUSH_HALF, side * 0.62)
    )]))
  };
  sdf::union(
    [
      belt_deck(belt_half(1)),
      rail(1.0),
      rail(-1.0),
      nozzle(1.0),
      nozzle(-1.0),
      brush(1.0),
      brush(-1.0),
      sdf::at(
        sdf::rounded_box(Vec3::new(0.99, 0.08, 0.38), 0.06),
        Vec3::new(0.0, WASH_BAR + 0.18, 0.0)
      ),
      sdf::at(
        sdf::rounded_box(Vec3::new(0.64, 0.24, 0.05), 0.05),
        Vec3::new(0.0, WASH_BAR + 0.48, 0.0)
      ),
      sdf::at(sdf::along_x(sdf::cylinder(0.08, 0.92)), Vec3::new(0.0, WASH_BAR, 0.0))
    ]
    .into_iter()
    .chain(posts)
  )
}

fn orewash_paint(at: Vec3, _: Vec3) -> LinearRgba {
  const DECK: LinearRgba = LinearRgba::rgb(0.05, 0.05, 0.06);
  const SHELL: LinearRgba = LinearRgba::rgb(0.86, 0.90, 0.94);
  const TRIM: LinearRgba = LinearRgba::rgb(0.76, 0.04, 0.06);
  const SPRAY: LinearRgba = LinearRgba::rgb(0.95, 0.60, 0.03);
  const BRISTLE_WARM: LinearRgba = LinearRgba::rgb(0.70, 0.04, 0.40);
  const BRISTLE_COOL: LinearRgba = LinearRgba::rgb(0.02, 0.42, 0.62);

  if at.y < 0.34 {
    DECK
  } else if at.y > WASH_BAR + 0.30 {
    SPRAY
  } else if at.y > WASH_BAR + 0.10 {
    TRIM
  } else if at.y > WASH_BAR - 0.40 && at.z.abs() < 0.86 {
    SPRAY
  } else if at.y > BRUSH_TOP {
    SHELL
  } else if at.z.abs() < 0.86 {
    (at.z > 0.0).then_some(BRISTLE_WARM).unwrap_or(BRISTLE_COOL)
  } else if at.x.abs() < 0.74 {
    TRIM
  } else {
    SHELL
  }
}

const CHILL_HALF: f32 = belt_half(CHILL_CELLS);
const CHILL_GANTRY: f32 = 2.05;
const CHILL_DRUM: f32 = 0.36;
const CHILL_MOUTH: f32 = 1.34;
const CHILL_LEG: f32 = CHILL_HALF - 0.36;
const CHILL_RAIL: f32 = 0.92;
const CHILL_TANK: f32 = 1.02;
const CHILL_GLOW: Color = Color::srgb(0.58, 0.90, 1.0);

fn chill_gantry() -> Tree {
  let leg = |along: f32, across: f32| {
    sdf::at(
      sdf::rounded_box(Vec3::new(0.10, CHILL_GANTRY / 2.0, 0.10), 0.04),
      Vec3::new(along * CHILL_LEG, CHILL_GANTRY / 2.0, across * CHILL_RAIL)
    )
  };
  let rail = |across: f32| {
    sdf::at(
      sdf::along_x(sdf::rounded_cylinder(0.11, CHILL_LEG, 0.05)),
      Vec3::new(0.0, CHILL_GANTRY, across * CHILL_RAIL)
    )
  };
  let tank = |across: f32| {
    sdf::at(
      sdf::along_x(sdf::rounded_cylinder(0.24, 1.26, 0.08)),
      Vec3::new(0.0, CHILL_TANK, across * CHILL_RAIL)
    )
  };
  let yoke = |along: f32| {
    sdf::at(
      sdf::cuboid(Vec3::new(0.09, 0.09, CHILL_RAIL)),
      Vec3::new(along * 0.86, CHILL_GANTRY, 0.0)
    )
  };
  let fin = |along: f32| {
    sdf::at(
      sdf::along_x(sdf::rounded_cylinder(CHILL_DRUM + 0.12, 0.04, 0.03)),
      Vec3::new(along * 0.46, CHILL_GANTRY, 0.0)
    )
  };
  let spike = |along: f32, across: f32| {
    sdf::at(
      sdf::rotate_z(sdf::rounded_box(Vec3::new(0.06, 0.20, 0.06), 0.02), across * 0.26),
      Vec3::new(along * 0.92, BELT_TOP + 0.20, across * 0.95)
    )
  };
  sdf::union([
    belt_deck(CHILL_HALF),
    tank(1.0),
    tank(-1.0),
    rail(1.0),
    rail(-1.0),
    leg(1.0, 1.0),
    leg(1.0, -1.0),
    leg(-1.0, 1.0),
    leg(-1.0, -1.0),
    yoke(1.0),
    yoke(-1.0),
    sdf::at(
      sdf::along_x(sdf::rounded_cylinder(CHILL_DRUM, 0.95, 0.10)),
      Vec3::new(0.0, CHILL_GANTRY, 0.0)
    ),
    fin(1.0),
    fin(-1.0),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.26, 0.32, 0.26), 0.07),
      Vec3::new(0.0, CHILL_GANTRY - 0.44, 0.0)
    ),
    sdf::at(
      sdf::rounded_cylinder(0.19, 0.18, 0.05),
      Vec3::new(0.0, CHILL_MOUTH + 0.16, 0.0)
    ),
    spike(1.0, 1.0),
    spike(1.0, -1.0),
    spike(-1.0, 1.0),
    spike(-1.0, -1.0)
  ])
}

fn chill_paint(at: Vec3, _: Vec3) -> LinearRgba {
  const DECK: LinearRgba = LinearRgba::rgb(0.05, 0.07, 0.09);
  const ICE: LinearRgba = LinearRgba::rgb(0.40, 0.66, 0.84);
  const RIME: LinearRgba = LinearRgba::rgb(0.88, 0.96, 1.0);
  const STEEL: LinearRgba = LinearRgba::rgb(0.13, 0.27, 0.40);
  const BEAM: LinearRgba = LinearRgba::rgb(0.26, 0.86, 1.0);

  let core = at.z.abs() < 0.46;
  if at.y < 0.34 {
    DECK
  } else if at.y < BELT_TOP + 0.46 && at.x.abs() < 1.2 {
    RIME
  } else if at.y > CHILL_GANTRY + CHILL_DRUM - 0.12 {
    RIME
  } else if core && at.y < CHILL_MOUTH + 0.22 {
    BEAM
  } else if core {
    STEEL
  } else {
    ICE
  }
}

fn torch_post() -> Tree {
  sdf::union([
    sdf::at(
      sdf::rounded_box(Vec3::new(0.06, TORCH_HEAD / 2.0, 0.06), 0.045),
      Vec3::new(0.0, TORCH_HEAD / 2.0, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.13, 0.15, 0.13), 0.09),
      Vec3::new(0.0, TORCH_HEAD, 0.0)
    )
  ])
}

fn fire_gradient() -> bevy_hanabi::Gradient<Vec4> {
  bevy_hanabi::Gradient::from_keys([
    (0.0, Vec4::new(5.0, 2.6, 0.7, 1.0)),
    (0.3, Vec4::new(3.4, 1.1, 0.16, 1.0)),
    (0.7, Vec4::new(1.3, 0.28, 0.04, 0.7)),
    (1.0, Vec4::new(0.35, 0.05, 0.02, 0.0))
  ])
}

fn water_gradient() -> bevy_hanabi::Gradient<Vec4> {
  bevy_hanabi::Gradient::from_keys([
    (0.0, Vec4::new(0.88, 0.96, 1.0, 0.0)),
    (0.15, Vec4::new(0.74, 0.92, 1.0, 0.85)),
    (0.7, Vec4::new(0.42, 0.70, 0.98, 0.6)),
    (1.0, Vec4::new(0.26, 0.52, 0.90, 0.0))
  ])
}

fn steam_gradient() -> bevy_hanabi::Gradient<Vec4> {
  bevy_hanabi::Gradient::from_keys([
    (0.0, Vec4::new(0.94, 0.97, 0.99, 0.0)),
    (0.2, Vec4::new(0.90, 0.95, 0.98, 0.30)),
    (0.6, Vec4::new(0.84, 0.91, 0.96, 0.18)),
    (1.0, Vec4::new(0.78, 0.87, 0.93, 0.0))
  ])
}

fn frost_gradient() -> bevy_hanabi::Gradient<Vec4> {
  bevy_hanabi::Gradient::from_keys([
    (0.0, Vec4::new(0.60, 2.40, 4.60, 1.0)),
    (0.25, Vec4::new(0.30, 1.40, 3.20, 0.9)),
    (0.7, Vec4::new(0.12, 0.55, 1.60, 0.6)),
    (1.0, Vec4::new(0.04, 0.14, 0.50, 0.0))
  ])
}

fn paint_gradient() -> bevy_hanabi::Gradient<Vec4> {
  bevy_hanabi::Gradient::from_keys([
    (0.0, Vec4::new(1.0, 0.84, 0.10, 0.0)),
    (0.15, Vec4::new(1.0, 0.82, 0.08, 1.0)),
    (0.7, Vec4::new(0.92, 0.70, 0.04, 0.95)),
    (1.0, Vec4::new(0.80, 0.58, 0.02, 0.0))
  ])
}

struct Plume {
  name: &'static str,
  thrust: Vec3,
  spread: f32,
  source: f32,
  girth: f32,
  life: f32,
  rate: f32,
  lift: f32,
  colors: fn() -> bevy_hanabi::Gradient<Vec4>,
  blend: bevy_hanabi::AlphaMode
}

impl Plume {
  const TORCH: Self = Self {
    name: "torch flame",
    thrust: Vec3::new(0.0, 1.5, 0.0),
    spread: 0.22,
    source: 0.085,
    girth: 0.17,
    life: 0.85,
    rate: 160.0,
    lift: 1.1,
    colors: fire_gradient,
    blend: bevy_hanabi::AlphaMode::Add
  };
  const BONFIRE: Self = Self {
    name: "bonfire",
    thrust: Vec3::new(0.0, 2.4, 0.0),
    spread: 0.55,
    source: 0.22,
    girth: 0.44,
    life: 1.15,
    rate: 420.0,
    lift: 1.1,
    colors: fire_gradient,
    blend: bevy_hanabi::AlphaMode::Add
  };
  const JET: Self = Self {
    name: "flame jet",
    thrust: Vec3::new(0.0, 0.45, 6.2),
    spread: 0.42,
    source: 0.13,
    girth: 0.26,
    life: 0.42,
    rate: 900.0,
    lift: 1.1,
    colors: fire_gradient,
    blend: bevy_hanabi::AlphaMode::Add
  };
  const WASH: Self = Self {
    name: "orewash spray",
    thrust: Vec3::new(0.0, -0.9, 0.0),
    spread: 1.3,
    source: 0.62,
    girth: 0.11,
    life: 0.6,
    rate: 520.0,
    lift: -4.5,
    colors: water_gradient,
    blend: bevy_hanabi::AlphaMode::Blend
  };

  const CHILL: Self = Self {
    name: "chill beam",
    thrust: Vec3::new(0.0, -5.2, 0.0),
    spread: 0.30,
    source: 0.11,
    girth: 0.17,
    life: 0.55,
    rate: 820.0,
    lift: -3.0,
    colors: frost_gradient,
    blend: bevy_hanabi::AlphaMode::Add
  };

  const DRIP: Self = Self {
    name: "paint drip",
    thrust: Vec3::new(0.0, -1.7, 0.0),
    spread: 0.11,
    source: 0.10,
    girth: 0.14,
    life: 0.6,
    rate: 110.0,
    lift: -6.0,
    colors: paint_gradient,
    blend: bevy_hanabi::AlphaMode::Blend
  };

  const STEAM: Self = Self {
    name: "hot tub steam",
    thrust: Vec3::new(0.0, 0.62, 0.0),
    spread: 0.34,
    source: 0.95,
    girth: 0.66,
    life: 2.6,
    rate: 60.0,
    lift: 0.30,
    colors: steam_gradient,
    blend: bevy_hanabi::AlphaMode::Blend
  };

  fn asset(self) -> EffectAsset {
    let writer = ExprWriter::new();
    let drift = (writer.rand(VectorType::VEC3F) * writer.lit(2.0) - writer.lit(1.0))
      * writer.lit(self.spread);
    let init_pos = SetPositionSphereModifier {
      center: writer.lit(Vec3::ZERO).expr(),
      radius: writer.lit(self.source).expr(),
      dimension: ShapeDimension::Volume
    };
    let init_vel = SetAttributeModifier::new(
      Attribute::VELOCITY,
      (drift + writer.lit(self.thrust)).expr()
    );
    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
    let init_lifetime = SetAttributeModifier::new(
      Attribute::LIFETIME,
      writer.lit(self.life * 0.6).uniform(writer.lit(self.life)).expr()
    );
    let drag = LinearDragModifier::new(writer.lit(1.2).expr());
    let lift = AccelModifier::new(writer.lit(Vec3::Y * self.lift).expr());
    let size = SizeOverLifetimeModifier {
      gradient: bevy_hanabi::Gradient::from_keys([
        (0.0, Vec3::splat(self.girth)),
        (0.35, Vec3::splat(self.girth * 0.82)),
        (1.0, Vec3::ZERO)
      ]),
      screen_space_size: false
    };

    EffectAsset::new(4096, SpawnerSettings::rate(self.rate.into()), writer.finish())
      .with_name(self.name)
      .with_simulation_space(SimulationSpace::Local)
      .with_alpha_mode(self.blend)
      .init(init_pos)
      .init(init_vel)
      .init(init_age)
      .init(init_lifetime)
      .update(drag)
      .update(lift)
      .render(ColorOverLifetimeModifier::new((self.colors)()))
      .render(size)
      .render(OrientModifier::new(OrientMode::FaceCameraPosition))
  }
}

fn flood_mast() -> Tree {
  let head = |side: f32| {
    sdf::union([
      sdf::at(
        sdf::rotate_z(sdf::rounded_box(Vec3::new(0.30, 0.26, 0.22), 0.05), FLOOD_TILT),
        Vec3::new(0.0, FLOOD_HEIGHT, side * FLOOD_SPAN)
      ),
      sdf::at(
        sdf::rounded_box(Vec3::new(0.07, 0.12, 0.07), 0.04),
        Vec3::new(0.0, FLOOD_HEIGHT + 0.22, side * FLOOD_SPAN)
      )
    ])
  };
  sdf::union([
    sdf::at(
      sdf::rounded_box(Vec3::new(0.46, 0.08, 0.46), 0.05),
      Vec3::new(0.0, 0.08, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.11, FLOOD_HEIGHT / 2.0, 0.11), 0.04),
      Vec3::new(0.0, FLOOD_HEIGHT / 2.0, 0.0)
    ),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.08, 0.07, FLOOD_SPAN + 0.14), 0.05),
      Vec3::new(0.0, FLOOD_HEIGHT + 0.30, 0.0)
    ),
    head(1.0),
    head(-1.0)
  ])
}

fn floodlight_paint(at: Vec3, _: Vec3) -> LinearRgba {
  const MAST: LinearRgba = LinearRgba::rgb(0.52, 0.56, 0.62);
  const HAZARD: LinearRgba = LinearRgba::rgb(0.92, 0.62, 0.02);
  const HOUSING: LinearRgba = LinearRgba::rgb(0.08, 0.09, 0.11);

  if at.y > FLOOD_HEIGHT - 0.38 {
    HOUSING
  } else if at.y < 0.20 || (at.y > 0.42 && at.y < 0.72) {
    HAZARD
  } else {
    MAST
  }
}

fn lamp_post(height: f32, shade: f32, tilt: f32) -> Tree {
  let hood = sdf::difference(
    sdf::rounded_cylinder(shade, 0.26, 0.1),
    sdf::at(sdf::cylinder(shade * 0.84, 0.2), Vec3::new(0.0, -0.24, 0.0))
  );
  sdf::union([
    sdf::at(sdf::rounded_cylinder(shade * 0.62, 0.08, 0.06), Vec3::new(0.0, 0.08, 0.0)),
    sdf::at(
      sdf::rounded_box(Vec3::new(0.1, height / 2.0, 0.1), 0.035),
      Vec3::new(0.0, height / 2.0, 0.0)
    ),
    sdf::at(sdf::rotate_z(hood, tilt), Vec3::new(0.0, height, 0.0))
  ])
}

#[derive(Resource)]
pub struct MachineAssets {
  meshes: [Handle<Mesh>; MachineKind::COUNT],
  parts: [Vec<(Handle<Mesh>, Handle<StandardMaterial>)>; MachineKind::COUNT],
  arrow_mesh: Handle<Mesh>,
  arrow_material: Handle<StandardMaterial>,
  slat_mesh: Handle<Mesh>,
  slat_materials: [Handle<StandardMaterial>; 2],
  glow_mesh: Handle<Mesh>,
  torch_glow: Handle<StandardMaterial>,
  torch_flame: Handle<EffectAsset>,
  bonfire_flame: Handle<EffectAsset>,
  jet_flame: Handle<EffectAsset>,
  wash_spray: Handle<EffectAsset>,
  chill_frost: Handle<EffectAsset>,
  paint_drip: Handle<EffectAsset>,
  tub_steam: Handle<EffectAsset>,
  chute_mesh: Handle<Mesh>,
  chute_material: Handle<StandardMaterial>,
  lens_mesh: Handle<Mesh>,
  flap_mesh: Handle<Mesh>,
  flap_material: Handle<StandardMaterial>,
  sheet_mesh: Handle<Mesh>,
  sheet_material: Handle<StandardMaterial>,
  coals_mesh: Handle<Mesh>,
  coals_glow: Handle<StandardMaterial>,
  lamp_glow: Handle<StandardMaterial>,
  chill_glow: Handle<StandardMaterial>,
  gauge_glow: Handle<StandardMaterial>,
  pub hover_glow: Handle<StandardMaterial>,
  pub ghost_valid: Handle<StandardMaterial>,
  pub ghost_blocked: Handle<StandardMaterial>
}

impl MachineAssets {
  pub fn mesh(&self, kind: MachineKind) -> Handle<Mesh> {
    self.meshes[kind.index()].clone()
  }

  pub fn model(&self, kind: MachineKind) -> impl Bundle {
    Children::spawn(SpawnIter(
      self.parts[kind.index()]
        .clone()
        .into_iter()
        .map(|(mesh, material)| (Mesh3d(mesh), MeshMaterial3d(material)))
    ))
  }
}

#[derive(Component)]
struct PreviewCamera;

fn freeze_previews(
  mut cameras: Query<&mut Camera, With<PreviewCamera>>,
  mut frames: Local<u32>
) {
  *frames += 1;
  if *frames == 30 {
    for mut camera in &mut cameras {
      camera.is_active = false;
    }
  }
}

#[derive(Resource)]
pub struct MachinePreviews([Handle<Image>; MachineKind::COUNT]);

impl MachinePreviews {
  const RESOLUTION: u32 = 256;

  pub fn image(&self, kind: MachineKind) -> Handle<Image> { self.0[kind.index()].clone() }
}

fn load_machine_assets(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut images: ResMut<Assets<Image>>,
  mut effects: ResMut<Assets<EffectAsset>>
) {
  let ghost = |color: Color| StandardMaterial {
    base_color: color,
    alpha_mode: AlphaMode::Blend,
    unlit: true,
    ..default()
  };

  let mut coats = Coats::new(&mut images);
  let machine_models = MachineKind::ALL.map(|kind| kind.model());
  let machine_meshes =
    machine_models.each_ref().map(|model| meshes.add(part::whole(model)));
  let machine_parts = machine_models.map(|model| {
    model
      .into_iter()
      .map(|(coat, mesh)| (meshes.add(mesh), coats.of(coat, &mut materials)))
      .collect::<Vec<_>>()
  });

  let previews = MachineKind::ALL.map(|kind| {
    let image = images.add(Image::new_target_texture(
      MachinePreviews::RESOLUTION,
      MachinePreviews::RESOLUTION,
      TextureFormat::Rgba8UnormSrgb,
      None
    ));
    let layer = RenderLayers::layer(kind.index() + 1);
    let stage = Vec3::new(0.0, -600.0 - 40.0 * kind.index() as f32, 0.0);
    let focus = stage + Vec3::Y * 1.5;

    let posed = commands
      .spawn((
        Transform::from_translation(stage)
          .with_rotation(Quat::from_rotation_y(kind.preview_spin())),
        Visibility::default(),
        layer.clone()
      ))
      .id();
    for (mesh, material) in &machine_parts[kind.index()] {
      commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        layer.clone(),
        ChildOf(posed)
      ));
    }
    commands.spawn((
      PreviewCamera,
      Camera3d::default(),
      Camera {
        order: -1 - kind.index() as isize,
        clear_color: ClearColorConfig::Custom(Color::NONE),
        ..default()
      },
      RenderTarget::Image(image.clone().into()),
      AmbientLight {
        color: Color::srgb(0.76, 0.81, 0.95),
        brightness: 900.0,
        ..default()
      },
      Transform::from_translation(
        stage
          + Vec3::new(3.3, 3.0, 3.9)
            * (0.72 + 0.28 * kind.footprint().max_element() as f32)
      )
      .looking_at(focus, Vec3::Y),
      layer
    ));
    image
  });

  commands.spawn((
    DirectionalLight { illuminance: 6000.0, ..default() },
    Transform::from_translation(Vec3::new(4.0, 6.0, 5.0))
      .looking_at(Vec3::Y * 1.5, Vec3::Y),
    RenderLayers::from_layers(&MachineKind::ALL.map(|kind| kind.index() + 1))
  ));
  commands.insert_resource(MachinePreviews(previews));
  commands.insert_resource(coats);
  commands.insert_resource(MachineAssets {
    meshes: machine_meshes,
    parts: machine_parts,
    arrow_mesh: meshes.add(Triangle2d::new(
      Vec2::new(0.36, 0.0),
      Vec2::new(-0.22, 0.40),
      Vec2::new(-0.22, -0.40)
    )),
    arrow_material: materials.add(StandardMaterial {
      base_color: Color::srgb(0.99, 0.78, 0.22),
      emissive: LinearRgba::rgb(0.75, 0.45, 0.04),
      unlit: true,
      double_sided: true,
      cull_mode: None,
      ..default()
    }),
    slat_mesh: meshes.add(Cuboid::new(
      SLAT_PITCH - SLAT_GAP,
      SLAT_THICK,
      SLAT_HALF * 2.0
    )),
    slat_materials: [Color::srgb(0.12, 0.13, 0.15), Color::srgb(0.19, 0.20, 0.23)].map(
      |tone| {
        materials.add(StandardMaterial {
          base_color: tone,
          perceptual_roughness: 0.82,
          reflectance: 0.24,
          ..default()
        })
      }
    ),
    glow_mesh: meshes.add(Sphere::new(1.0).mesh().ico(3).expect("glow mesh")),
    torch_glow: materials.add(StandardMaterial {
      base_color: TORCH_GLOW,
      emissive: LinearRgba::rgb(46.0, 15.0, 2.2),
      ..default()
    }),
    torch_flame: effects.add(Plume::TORCH.asset()),
    bonfire_flame: effects.add(Plume::BONFIRE.asset()),
    jet_flame: effects.add(Plume::JET.asset()),
    wash_spray: effects.add(Plume::WASH.asset()),
    chill_frost: effects.add(Plume::CHILL.asset()),
    paint_drip: effects.add(Plume::DRIP.asset()),
    tub_steam: effects.add(Plume::STEAM.asset()),
    chute_mesh: meshes.add(Cuboid::new(CHUTE_REACH - 0.18, 0.44, 0.44)),
    chute_material: materials.add(StandardMaterial {
      base_color: Color::srgba(0.42, 0.86, 0.98, 0.30),
      perceptual_roughness: 0.2,
      alpha_mode: AlphaMode::Blend,
      double_sided: true,
      cull_mode: None,
      ..default()
    }),
    lens_mesh: meshes.add(Cuboid::new(0.44, 0.40, 0.08)),
    flap_mesh: meshes.add(Cuboid::new(0.05, 0.92, 0.19)),
    flap_material: materials.add(StandardMaterial {
      base_color: Color::srgba(0.32, 0.82, 0.96, 0.34),
      perceptual_roughness: 0.3,
      alpha_mode: AlphaMode::Blend,
      double_sided: true,
      cull_mode: None,
      ..default()
    }),
    sheet_mesh: meshes.add(Cuboid::new(0.34, 1.06, 0.58)),
    sheet_material: materials.add(StandardMaterial {
      base_color: Color::srgba(0.52, 0.86, 1.0, 0.16),
      emissive: LinearRgba::rgb(0.03, 0.14, 0.22),
      perceptual_roughness: 0.15,
      alpha_mode: AlphaMode::Blend,
      double_sided: true,
      cull_mode: None,
      ..default()
    }),
    coals_mesh: meshes.add(Cuboid::new(
      (HEARTH_HALF - HEARTH_WALL) * 2.0,
      0.12,
      (HEARTH_HALF - HEARTH_WALL) * 2.0
    )),
    coals_glow: materials.add(StandardMaterial {
      base_color: EMBER_GLOW,
      emissive: LinearRgba::rgb(3.4, 0.62, 0.06),
      ..default()
    }),
    lamp_glow: materials.add(StandardMaterial {
      base_color: LAMP_GLOW,
      emissive: LinearRgba::rgb(38.0, 34.0, 25.0),
      ..default()
    }),
    chill_glow: materials.add(StandardMaterial {
      base_color: CHILL_GLOW,
      emissive: LinearRgba::rgb(0.6, 5.0, 12.0),
      ..default()
    }),
    gauge_glow: materials.add(StandardMaterial {
      base_color: GAUGE_GLOW,
      emissive: LinearRgba::rgb(2.6, 1.5, 0.22),
      perceptual_roughness: 0.2,
      ..default()
    }),
    hover_glow: materials.add(ghost(Color::srgba(0.45, 1.0, 0.55, 0.16))),
    ghost_valid: materials.add(ghost(Color::srgba(0.25, 0.95, 0.45, 0.35))),
    ghost_blocked: materials.add(ghost(Color::srgba(0.95, 0.25, 0.25, 0.30)))
  });
}

fn belt_loop(half: f32) -> f32 { 4.0 * (half - BELT_CURVE) + TAU * BELT_CURVE }

fn slat_pose(travel: f32, half: f32) -> Transform {
  let straight = 2.0 * (half - BELT_CURVE);
  let arc = PI * BELT_CURVE;
  let along = travel.rem_euclid(1.0) * belt_loop(half);
  let curl = |turn: f32, end: f32| {
    (
      Vec2::new(end * straight / 2.0, 0.0)
        + BELT_CURVE * Vec2::new(turn.sin(), turn.cos()),
      turn
    )
  };
  let (spot, turn) = if along < straight {
    (Vec2::new(along - straight / 2.0, BELT_CURVE), 0.0)
  } else if along < straight + arc {
    curl((along - straight) / BELT_CURVE, 1.0)
  } else if along < 2.0 * straight + arc {
    (Vec2::new(1.5 * straight + arc - along, -BELT_CURVE), PI)
  } else {
    curl(PI + (along - 2.0 * straight - arc) / BELT_CURVE, -1.0)
  };
  Transform::from_xyz(spot.x, BELT_MID + spot.y, 0.0)
    .with_rotation(Quat::from_rotation_z(-turn))
}

fn belt_half_of(kind: MachineKind) -> f32 { belt_half(kind.footprint().x) }

fn slat_travel(phase: f32, half: f32, secs: f32) -> f32 {
  phase + secs * BELT_SPEED / belt_loop(half)
}

fn slats_along(half: f32) -> usize { (belt_loop(half) / SLAT_PITCH).round() as usize }

fn slat_at(
  assets: &MachineAssets,
  material: &Handle<StandardMaterial>,
  travel: f32,
  half: f32
) -> impl Bundle {
  (
    Mesh3d(assets.slat_mesh.clone()),
    MeshMaterial3d(material.clone()),
    slat_pose(travel, half)
  )
}

#[derive(Component)]
struct BeltSlat {
  phase: f32,
  half: f32
}

fn slide_belt_slats(time: Res<Time>, mut slats: Query<(&BeltSlat, &mut Transform)>) {
  for (slat, mut transform) in &mut slats {
    *transform =
      slat_pose(slat_travel(slat.phase, slat.half, time.elapsed_secs()), slat.half);
  }
}

fn arrow_span(kind: MachineKind) -> f32 { kind.footprint().x as f32 * CELL - ARROW_INSET }

fn arrows_along(kind: MachineKind) -> usize {
  ARROWS_PER_CELL * kind.footprint().x as usize
}

fn arrow_at(assets: &MachineAssets, offset: f32) -> impl Bundle {
  (
    Mesh3d(assets.arrow_mesh.clone()),
    MeshMaterial3d(assets.arrow_material.clone()),
    Transform::from_xyz(offset, BELT_TOP + 0.02, 0.0)
      .with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
  )
}

pub fn spawn_ghost_belt(
  commands: &mut Commands,
  assets: &MachineAssets,
  material: &Handle<StandardMaterial>,
  kind: MachineKind,
  secs: f32,
  root: Entity
) {
  let arrows = arrows_along(kind);
  for step in 0..arrows {
    let slide = step as f32 / (arrows - 1) as f32 - 0.5;
    commands.spawn((arrow_at(assets, slide * arrow_span(kind)), ChildOf(root)));
  }
  let half = belt_half_of(kind);
  let slats = slats_along(half);
  for step in 0..slats {
    commands.spawn((
      slat_at(
        assets,
        material,
        slat_travel(step as f32 / slats as f32, half, secs),
        half
      ),
      ChildOf(root)
    ));
  }
}

pub fn place(
  commands: &mut Commands,
  assets: &MachineAssets,
  kind: MachineKind,
  transform: Transform
) -> Entity {
  let root = commands
    .spawn((Name::new(kind.name()), RigidBody::Static, transform, assets.model(kind)))
    .id();

  let bolt = |commands: &mut Commands, collider: Collider, at: Transform| {
    commands.spawn((collider, at, ChildOf(root)));
  };
  for (collider, at) in part::colliders(kind.parts().unwrap_or_default()) {
    bolt(commands, collider, at);
  }
  if let Some(dropper) = kind.dropper() {
    commands.entity(root).insert(dropper);
  }

  match kind {
    MachineKind::Dropper => {
      bolt(
        commands,
        Collider::cuboid(0.9, 1.5, 0.9),
        Transform::from_xyz(DROPPER_BACK, 0.75, 0.0)
      );
      bolt(
        commands,
        Collider::cuboid(1.3, 1.1, 1.3),
        Transform::from_xyz(DROPPER_BACK, 1.78, 0.0)
      );
      commands.spawn((
        Mesh3d(assets.chute_mesh.clone()),
        MeshMaterial3d(assets.chute_material.clone()),
        NotShadowCaster,
        Transform::from_xyz(CHUTE_REACH / 2.0 - 0.14, CHUTE_FLOOR + 0.22, 0.0),
        ChildOf(root)
      ));
    }
    MachineKind::Furnace => {
      commands.spawn((
        Furnace,
        Collider::cuboid(
          (HEARTH_HALF - HEARTH_WALL) * 2.0,
          HEARTH_RIM * 2.0,
          (HEARTH_HALF - HEARTH_WALL) * 2.0
        ),
        Sensor,
        CollisionEventsEnabled,
        Mesh3d(assets.coals_mesh.clone()),
        MeshMaterial3d(assets.coals_glow.clone()),
        NotShadowCaster,
        NoFrustumCulling,
        PointLight { color: EMBER_GLOW, intensity: 420_000.0, range: 14.0, ..default() },
        Transform::from_xyz(0.0, HEARTH_COALS, 0.0),
        ChildOf(root)
      ));
    }
    MachineKind::HotTub => {
      commands.spawn((
        Furnace,
        Collider::cuboid(TUB_BORE, TUB_WATER + 0.5, TUB_BORE),
        Sensor,
        CollisionEventsEnabled,
        Transform::from_xyz(0.0, TUB_FLOOR + (TUB_WATER + 0.5) / 2.0, 0.0),
        ChildOf(root)
      ));
      commands.spawn((
        ParticleEffect::new(assets.tub_steam.clone()),
        Transform::from_xyz(0.0, TUB_WATER + 0.10, 0.0),
        ChildOf(root)
      ));
      commands.spawn((
        PointLight { color: TUB_GLOW, intensity: 220_000.0, range: 12.0, ..default() },
        Transform::from_xyz(0.0, TUB_WATER - 0.12, 0.0),
        ChildOf(root)
      ));
    }
    MachineKind::Bonfire => {
      bolt(
        commands,
        Collider::cylinder(0.62, BONFIRE_TOP),
        Transform::from_xyz(0.0, BONFIRE_TOP / 2.0, 0.0)
      );
      commands.spawn((
        PointLight {
          color: TORCH_GLOW,
          intensity: 4_200_000.0,
          range: 40.0,
          ..default()
        },
        Mesh3d(assets.glow_mesh.clone()),
        MeshMaterial3d(assets.torch_glow.clone()),
        NotShadowCaster,
        NoFrustumCulling,
        Transform::from_xyz(0.0, BONFIRE_TOP * 0.7, 0.0).with_scale(Vec3::splat(0.26)),
        ChildOf(root)
      ));
      commands.spawn((
        ParticleEffect::new(assets.bonfire_flame.clone()),
        Transform::from_xyz(0.0, BONFIRE_TOP * 0.55, 0.0),
        ChildOf(root)
      ));
    }
    MachineKind::Torch => {
      bolt(
        commands,
        Collider::cylinder(0.2, TORCH_HEAD + 0.3),
        Transform::from_xyz(0.0, (TORCH_HEAD + 0.3) / 2.0, 0.0)
      );
      commands.spawn((
        PointLight {
          color: TORCH_GLOW,
          intensity: 1_400_000.0,
          range: 26.0,
          ..default()
        },
        Mesh3d(assets.glow_mesh.clone()),
        MeshMaterial3d(assets.torch_glow.clone()),
        NotShadowCaster,
        NoFrustumCulling,
        Transform::from_xyz(0.0, TORCH_HEAD + 0.2, 0.0)
          .with_scale(Vec3::new(0.11, 0.15, 0.11)),
        ChildOf(root)
      ));
      commands.spawn((
        ParticleEffect::new(assets.torch_flame.clone()),
        Transform::from_xyz(0.0, TORCH_HEAD + 0.12, 0.0),
        ChildOf(root)
      ));
    }
    MachineKind::Floodlight => {
      let beam = Vec3::new(FLOOD_TILT.sin(), -FLOOD_TILT.cos(), 0.0);
      bolt(
        commands,
        Collider::cuboid(0.7, FLOOD_HEIGHT + 0.6, FLOOD_SPAN * 2.0 + 0.6),
        Transform::from_xyz(0.0, (FLOOD_HEIGHT + 0.6) / 2.0, 0.0)
      );
      commands.spawn((
        SpotLight {
          color: LAMP_GLOW,
          intensity: 9_000_000.0,
          range: 70.0,
          inner_angle: 0.30,
          outer_angle: 0.62,
          shadow_maps_enabled: true,
          shadow_depth_bias: 0.1,
          shadow_normal_bias: 3.4,
          ..default()
        },
        Transform::from_translation(Vec3::new(0.0, FLOOD_HEIGHT, 0.0) + beam * 0.3)
          .looking_to(beam, Vec3::Y),
        ChildOf(root)
      ));
      for side in [-1.0, 1.0] {
        commands.spawn((
          Mesh3d(assets.lens_mesh.clone()),
          MeshMaterial3d(assets.lamp_glow.clone()),
          NotShadowCaster,
          Transform::from_translation(
            Vec3::new(0.0, FLOOD_HEIGHT, side * FLOOD_SPAN) + beam * 0.24
          )
          .looking_to(beam, Vec3::Y),
          ChildOf(root)
        ));
      }
    }
    MachineKind::Lamp => {
      let beam = Vec3::new(LAMP_TILT.sin(), -LAMP_TILT.cos(), 0.0);
      bolt(
        commands,
        Collider::cylinder(LAMP_SHADE, LAMP_HEIGHT + 0.5),
        Transform::from_xyz(0.0, (LAMP_HEIGHT + 0.5) / 2.0, 0.0)
      );
      commands.spawn((
        SpotLight {
          color: LAMP_GLOW,
          intensity: 3_200_000.0,
          range: 34.0,
          inner_angle: 0.18,
          outer_angle: 0.42,
          shadow_maps_enabled: true,
          shadow_depth_bias: 0.1,
          shadow_normal_bias: 3.4,
          ..default()
        },
        Mesh3d(assets.glow_mesh.clone()),
        MeshMaterial3d(assets.lamp_glow.clone()),
        NotShadowCaster,
        NoFrustumCulling,
        Transform::from_translation(Vec3::new(0.0, LAMP_HEIGHT, 0.0) + beam * 0.24)
          .looking_to(beam, Vec3::Y)
          .with_scale(Vec3::new(LAMP_SHADE * 0.78, LAMP_SHADE * 0.78, LAMP_SHADE * 0.4)),
        ChildOf(root)
      ));
    }
    _ => {}
  }

  if kind.carries_belt() {
    let length = kind.footprint().x as f32 * CELL - 0.02;
    commands.spawn((
      ConveyorBelt { local_direction: Vec3::X, speed: BELT_SPEED },
      Collider::cuboid(length, BELT_TOP, CELL - 0.02),
      Friction::new(1.0),
      Transform::from_xyz(0.0, BELT_TOP / 2.0, 0.0),
      ChildOf(root)
    ));
    let half = belt_half_of(kind);
    let slats = slats_along(half);
    for step in 0..slats {
      let phase = step as f32 / slats as f32;
      commands.spawn((
        BeltSlat { phase, half },
        slat_at(assets, &assets.slat_materials[step % 2], phase, half),
        ChildOf(root)
      ));
    }
    if let Some(upgrader) = kind.upgrade() {
      if kind == MachineKind::FlameJet {
        bolt(
          commands,
          Collider::cuboid(0.8, 2.0, 0.7),
          Transform::from_xyz(0.0, 1.0, -1.12)
        );
        commands.spawn((
          ParticleEffect::new(assets.jet_flame.clone()),
          Transform::from_xyz(0.0, JET_HEIGHT, -0.26),
          ChildOf(root)
        ));
      } else if kind == MachineKind::ChillBeam {
        for along in [-1.0, 1.0] {
          for across in [-1.0, 1.0] {
            bolt(
              commands,
              Collider::cuboid(0.26, CHILL_GANTRY, 0.26),
              Transform::from_xyz(
                along * CHILL_LEG,
                CHILL_GANTRY / 2.0,
                across * CHILL_RAIL
              )
            );
          }
          bolt(
            commands,
            Collider::cuboid(2.6, 0.58, 0.58),
            Transform::from_xyz(0.0, CHILL_TANK, along * CHILL_RAIL)
          );
        }
        bolt(
          commands,
          Collider::cuboid(CHILL_LEG * 2.0, 1.1, 2.1),
          Transform::from_xyz(0.0, CHILL_GANTRY, 0.0)
        );
        commands.spawn((
          ParticleEffect::new(assets.chill_frost.clone()),
          Transform::from_xyz(0.0, CHILL_MOUTH, 0.0),
          ChildOf(root)
        ));
        commands.spawn((
          PointLight {
            color: CHILL_GLOW,
            intensity: 240_000.0,
            range: 12.0,
            ..default()
          },
          Mesh3d(assets.glow_mesh.clone()),
          MeshMaterial3d(assets.chill_glow.clone()),
          NotShadowCaster,
          NoFrustumCulling,
          Transform::from_xyz(0.0, CHILL_MOUTH, 0.0).with_scale(Vec3::splat(0.12)),
          ChildOf(root)
        ));
      } else if kind == MachineKind::PaintBucket {
        commands.spawn((
          ParticleEffect::new(assets.paint_drip.clone()),
          Transform::from_translation(PAIL_SPOUT - Vec3::Y * 0.12),
          ChildOf(root)
        ));
      } else if kind == MachineKind::Embiggener {
        for (slot, (high, wide)) in GATES.into_iter().enumerate() {
          let along = slot as f32 * GATE_STEP - GATE_STEP;
          for side in [-1.0, 1.0] {
            bolt(
              commands,
              Collider::cuboid(GATE_HALF * 2.0, high + GATE_WALL, GATE_WALL),
              Transform::from_xyz(
                along,
                BELT_TOP + (high + GATE_WALL) / 2.0,
                side * (wide + GATE_WALL / 2.0)
              )
            );
          }
          bolt(
            commands,
            Collider::cuboid(GATE_HALF * 2.0, GATE_WALL, (wide + GATE_WALL) * 2.0),
            Transform::from_xyz(along, BELT_TOP + high + GATE_WALL / 2.0, 0.0)
          );
        }
        commands.spawn((
          Mesh3d(assets.glow_mesh.clone()),
          MeshMaterial3d(assets.gauge_glow.clone()),
          NotShadowCaster,
          Transform::from_translation(GAUGE_AT).with_scale(Vec3::splat(0.11)),
          ChildOf(root)
        ));
      } else {
        bolt(
          commands,
          Collider::cuboid(1.04, 2.5, 0.5),
          Transform::from_xyz(0.0, 1.05, 0.87)
        );
        bolt(
          commands,
          Collider::cuboid(1.04, 2.5, 0.5),
          Transform::from_xyz(0.0, 1.05, -0.87)
        );
      }
      if kind == MachineKind::Orewash {
        commands.spawn((
          ParticleEffect::new(assets.wash_spray.clone()),
          Transform::from_xyz(0.0, WASH_BAR - 0.34, 0.0),
          ChildOf(root)
        ));
        commands.spawn((
          Mesh3d(assets.sheet_mesh.clone()),
          MeshMaterial3d(assets.sheet_material.clone()),
          NotShadowCaster,
          Transform::from_xyz(0.0, 0.88, 0.0),
          ChildOf(root)
        ));
        for slot in 0..FLAPS_PER_CURTAIN * 2 {
          let across =
            (slot % FLAPS_PER_CURTAIN) as f32 / (FLAPS_PER_CURTAIN - 1) as f32 - 0.5;
          let along = (slot / FLAPS_PER_CURTAIN) as f32 * 2.0 - 1.0;
          commands.spawn((
            Mesh3d(assets.flap_mesh.clone()),
            MeshMaterial3d(assets.flap_material.clone()),
            NotShadowCaster,
            Transform::from_xyz(along * FLAP_REACH, 0.90, across * 1.42),
            ChildOf(root)
          ));
        }
      }
      let sensed =
        (kind == MachineKind::Embiggener).then(|| -GATE_STEP / 2.0).unwrap_or_default();
      commands.spawn((
        upgrader,
        Collider::cuboid(0.5, 0.8, 1.5),
        Sensor,
        CollisionEventsEnabled,
        Transform::from_xyz(sensed, BELT_TOP + 0.4, 0.0),
        ChildOf(root)
      ));
    }
  }

  root
}

pub fn plugin(app: &mut App) {
  app
    .add_plugins(HanabiPlugin)
    .add_systems(PreStartup, load_machine_assets)
    .add_systems(Update, (freeze_previews, slide_belt_slats));
}

use {crate::{machine::{ARROW_SPAN, BeltArrow, ConveyorBelt, Dropper, Furnace, Upgrader},
             ore::{Effects, OreForm},
             sdf, texture},
     avian3d::prelude::*,
     bevy::{camera::{RenderTarget, visibility::RenderLayers},
            light::NotShadowCaster,
            math::Affine2,
            prelude::*,
            render::render_resource::TextureFormat},
     bevy_hanabi::{AccelModifier, Attribute, ColorOverLifetimeModifier, EffectAsset,
                   ExprWriter, HanabiPlugin, LinearDragModifier, ParticleEffect,
                   SetAttributeModifier, SetPositionSphereModifier, ShapeDimension,
                   SimulationSpace, SizeOverLifetimeModifier, SpawnerSettings,
                   VectorType},
     fidget::context::Tree,
     std::f32::consts::{FRAC_PI_2, TAU}};

pub const CELL: f32 = 2.0;
pub const BELT_TOP: f32 = 0.22;
const ARROWS_PER_BELT: usize = 3;
const FURNACE_FOOT: f32 = 0.34;
const FURNACE_MOUTH: f32 = FURNACE_FOOT + 0.52;
const JET_HEIGHT: f32 = BELT_TOP + 0.42;
const DROPPER_BACK: f32 = -0.42;
const CHUTE_FLOOR: f32 = 1.16;
pub const CHUTE_REACH: f32 = 1.52;
const COOP_EAVES: f32 = CHUTE_FLOOR + 0.86;
const WASH_BAR: f32 = 1.62;
const BRUSH_TOP: f32 = 1.30;
const BRUSH_HALF: f32 = 0.50;
const FLAPS_PER_CURTAIN: usize = 5;
const FLAP_REACH: f32 = 0.87;
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MachineKind {
  Conveyor,
  Dropper,
  Coop,
  Furnace,
  Forge,
  FlameJet,
  MistCoil,
  Orewash,
  DecayChamber,
  Torch,
  Bonfire,
  Lamp,
  Floodlight
}

struct Finish {
  roughness: f32,
  metallic: f32,
  reflectance: f32
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Surface {
  Painted,
  Plastic,
  Wood,
  Planked,
  Metal
}

impl Surface {
  const fn finish(self) -> Finish {
    match self {
      Self::Painted => Finish { roughness: 0.6, metallic: 0.35, reflectance: 0.5 },
      Self::Plastic => Finish { roughness: 0.42, metallic: 0.0, reflectance: 0.38 },
      Self::Wood => Finish { roughness: 0.88, metallic: 0.0, reflectance: 0.14 },
      Self::Planked => Finish { roughness: 0.80, metallic: 0.0, reflectance: 0.20 },
      Self::Metal => Finish { roughness: 0.24, metallic: 0.95, reflectance: 0.72 }
    }
  }

  const fn tiling(self) -> Option<Vec2> {
    match self {
      Self::Wood => Some(texture::GRAIN),
      Self::Planked => Some(texture::PLANK),
      _ => None
    }
  }
}

pub struct MachineSpec {
  pub name: &'static str,
  pub blurb: &'static str,
  pub price: f32,
  pub tier: Tier,
  pub unlock: Option<&'static str>
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tier {
  Plain,
  Sturdy,
  Refined,
  Exotic,
  Mythic
}

impl Tier {
  pub const fn swatch(self) -> Color {
    match self {
      Self::Plain => Color::srgb(0.91, 0.91, 0.92),
      Self::Sturdy => Color::srgb(0.75, 0.86, 0.96),
      Self::Refined => Color::srgb(0.71, 0.93, 0.74),
      Self::Exotic => Color::srgb(0.99, 0.87, 0.55),
      Self::Mythic => Color::srgb(0.93, 0.72, 0.97)
    }
  }
}

impl MachineKind {
  pub const ALL: [Self; 13] = [
    Self::Conveyor,
    Self::Dropper,
    Self::Coop,
    Self::Furnace,
    Self::Forge,
    Self::FlameJet,
    Self::MistCoil,
    Self::Orewash,
    Self::DecayChamber,
    Self::Torch,
    Self::Bonfire,
    Self::Lamp,
    Self::Floodlight
  ];
  pub const COUNT: usize = Self::ALL.len();

  pub const fn index(self) -> usize { self as usize }

  pub const fn carries_belt(self) -> bool {
    matches!(
      self,
      Self::Conveyor
        | Self::Forge
        | Self::FlameJet
        | Self::MistCoil
        | Self::Orewash
        | Self::DecayChamber
    )
  }

  pub const fn spec(self) -> MachineSpec {
    match self {
      Self::Conveyor => MachineSpec {
        name: "Conveyor",
        blurb: "Carries ore one cell onward. Everything is downstream of something.",
        price: 25.0,
        tier: Tier::Plain,
        unlock: None
      },
      Self::Dropper => MachineSpec {
        name: "Ore Dropper",
        blurb: "Coughs up a lump of rock every so often. Aim it at a belt.",
        price: 150.0,
        tier: Tier::Plain,
        unlock: None
      },
      Self::Coop => MachineSpec {
        name: "Chicken Coop",
        blurb: "A hen broods in the nest box and rolls a fresh egg down the ramp.",
        price: 340.0,
        tier: Tier::Sturdy,
        unlock: None
      },
      Self::Forge => MachineSpec {
        name: "Flame Forge",
        blurb: "Sets passing ore alight and multiplies what it is worth.",
        price: 400.0,
        tier: Tier::Refined,
        unlock: None
      },
      Self::Furnace => MachineSpec {
        name: "Furnace",
        blurb: "Swallows whatever reaches it and pays out its value.",
        price: 250.0,
        tier: Tier::Sturdy,
        unlock: None
      },
      Self::FlameJet => MachineSpec {
        name: "Flame Jet",
        blurb: "Blasts a lance of fire across the belt. Whatever passes comes out burning.",
        price: 620.0,
        tier: Tier::Refined,
        unlock: None
      },
      Self::MistCoil => MachineSpec {
        name: "Mist Coil",
        blurb: "Soaks ore through. Wet things carry charge differently.",
        price: 900.0,
        tier: Tier::Exotic,
        unlock: Some("Burn an ore worth over $500")
      },
      Self::Orewash => MachineSpec {
        name: "The Orewash",
        blurb: "Your ores need to be at the orewash to wash them.",
        price: 700.0,
        tier: Tier::Refined,
        unlock: None
      },
      Self::DecayChamber => MachineSpec {
        name: "Decay Chamber",
        blurb: "Leaves ore humming and faintly green for a very long time.",
        price: 2200.0,
        tier: Tier::Mythic,
        unlock: Some("Burn 250 ore")
      },
      Self::Torch => MachineSpec {
        name: "Torch",
        blurb: "A burning brand on a stake. Keeps the dark off a corner of the floor.",
        price: 40.0,
        tier: Tier::Plain,
        unlock: None
      },
      Self::Bonfire => MachineSpec {
        name: "Bonfire",
        blurb: "A stacked heap of branches, well alight. Warms a wide stretch of floor.",
        price: 120.0,
        tier: Tier::Plain,
        unlock: None
      },
      Self::Lamp => MachineSpec {
        name: "Lamp Post",
        blurb: "Angles a tight beam across the floor. Rotate it to aim where you want.",
        price: 180.0,
        tier: Tier::Sturdy,
        unlock: None
      },
      Self::Floodlight => MachineSpec {
        name: "Floodlight",
        blurb: "A taller mast with a wide, hard beam. Lights a whole bank of machines.",
        price: 520.0,
        tier: Tier::Refined,
        unlock: None
      }
    }
  }

  pub const fn upgrade(self) -> Option<Upgrader> {
    match self {
      Self::Forge => Some(Upgrader { multiplier: 2.5, effects: Effects::FIERY }),
      Self::FlameJet => Some(Upgrader { multiplier: 3.2, effects: Effects::FIERY }),
      Self::MistCoil => Some(Upgrader { multiplier: 4.0, effects: Effects::WET }),
      Self::Orewash => Some(Upgrader { multiplier: 3.0, effects: Effects::WET }),
      Self::DecayChamber => {
        Some(Upgrader { multiplier: 9.0, effects: Effects::RADIOACTIVE })
      }
      _ => None
    }
  }

  const fn surface(self) -> Surface {
    match self {
      Self::Orewash => Surface::Plastic,
      Self::Torch | Self::Bonfire => Surface::Wood,
      Self::Coop => Surface::Planked,
      Self::Lamp | Self::Floodlight | Self::FlameJet => Surface::Metal,
      _ => Surface::Painted
    }
  }

  const fn paint(self) -> Option<fn(Vec3, Vec3) -> LinearRgba> {
    match self {
      Self::Dropper => Some(dropper_paint),
      Self::Coop => Some(coop_paint),
      Self::Orewash => Some(orewash_paint),
      Self::Floodlight => Some(floodlight_paint),
      _ => None
    }
  }

  fn accent(self) -> (Color, LinearRgba) {
    match self {
      Self::Conveyor => (Color::srgb(0.30, 0.31, 0.34), LinearRgba::BLACK),
      Self::Dropper | Self::Coop => (Color::WHITE, LinearRgba::BLACK),
      Self::Forge => (Color::srgb(0.44, 0.24, 0.18), LinearRgba::rgb(0.85, 0.22, 0.03)),
      Self::FlameJet => (Color::srgb(0.46, 0.47, 0.50), LinearRgba::BLACK),
      Self::Furnace => (Color::srgb(0.30, 0.17, 0.13), LinearRgba::BLACK),
      Self::MistCoil => {
        (Color::srgb(0.22, 0.34, 0.48), LinearRgba::rgb(0.06, 0.40, 0.85))
      }
      Self::Orewash => (Color::WHITE, LinearRgba::BLACK),
      Self::DecayChamber => {
        (Color::srgb(0.24, 0.40, 0.22), LinearRgba::rgb(0.10, 0.85, 0.12))
      }
      Self::Torch | Self::Bonfire => (Color::WHITE, LinearRgba::BLACK),
      Self::Lamp => (Color::srgb(0.62, 0.64, 0.68), LinearRgba::BLACK),
      Self::Floodlight => (Color::WHITE, LinearRgba::BLACK)
    }
  }
}

fn belt_deck() -> Tree {
  sdf::union([
    sdf::at(
      sdf::rounded_box(Vec3::new(0.99, BELT_TOP / 2.0, 0.92), 0.05),
      Vec3::new(0.0, BELT_TOP / 2.0, 0.0)
    ),
    sdf::at(sdf::cuboid(Vec3::new(0.99, 0.10, 0.06)), Vec3::new(0.0, 0.16, 0.95)),
    sdf::at(sdf::cuboid(Vec3::new(0.99, 0.10, 0.06)), Vec3::new(0.0, 0.16, -0.95))
  ])
}

fn arch() -> Tree {
  sdf::union([
    belt_deck(),
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

fn machine_bounds() -> sdf::Bounds {
  sdf::Bounds::around(Vec3::new(0.0, 1.45, 0.0), 1.7, 7)
}

fn board(half: Vec3, at: Vec3) -> Tree {
  let bounds = machine_bounds();
  let (low, high) = (bounds.snap(at - half), bounds.snap(at + half));
  sdf::at(sdf::cuboid((high - low) / 2.0), (low + high) / 2.0)
}

fn coop_body() -> Tree {
  let leg = |side: f32, along: f32| {
    board(
      Vec3::new(0.07, CHUTE_FLOOR / 2.0, 0.07),
      Vec3::new(DROPPER_BACK + along * 0.46, CHUTE_FLOOR / 2.0, side * 0.50)
    )
  };
  let roof = |slope: f32| {
    sdf::at(
      sdf::rotate_z(sdf::cuboid(Vec3::new(0.55, 0.05, 0.78)), -slope * 0.6),
      Vec3::new(DROPPER_BACK + slope * 0.46, COOP_EAVES + 0.11, 0.0)
    )
  };
  let rail = |side: f32| {
    board(
      Vec3::new(CHUTE_REACH / 2.0 - 0.10, 0.06, 0.04),
      Vec3::new(CHUTE_REACH / 2.0, CHUTE_FLOOR + 0.06, side * 0.30)
    )
  };
  let shank = |side: f32| {
    sdf::at(
      sdf::rounded_box(Vec3::new(0.035, 0.08, 0.035), 0.025),
      Vec3::new(0.68, CHUTE_FLOOR + 0.13, side * 0.09)
    )
  };
  let hen = sdf::union([
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
  ]);
  sdf::union([
    sdf::difference(
      board(
        Vec3::new(0.56, 0.42, 0.60),
        Vec3::new(DROPPER_BACK, CHUTE_FLOOR + 0.44, 0.0)
      ),
      sdf::at(
        sdf::along_x(sdf::cylinder(0.19, 0.40)),
        Vec3::new(DROPPER_BACK + 0.40, CHUTE_FLOOR + 0.32, 0.0)
      )
    ),
    board(Vec3::new(0.62, 0.06, 0.66), Vec3::new(DROPPER_BACK, CHUTE_FLOOR - 0.04, 0.0)),
    board(
      Vec3::new(CHUTE_REACH / 2.0, 0.05, 0.32),
      Vec3::new(CHUTE_REACH / 2.0 - 0.10, CHUTE_FLOOR, 0.0)
    ),
    board(
      Vec3::new(0.06, 0.12, 0.34),
      Vec3::new(CHUTE_REACH - 0.14, CHUTE_FLOOR + 0.10, 0.0)
    ),
    roof(1.0),
    roof(-1.0),
    rail(1.0),
    rail(-1.0),
    leg(1.0, 1.0),
    leg(1.0, -1.0),
    leg(-1.0, 1.0),
    leg(-1.0, -1.0),
    hen
  ])
}

fn coop_paint(at: Vec3, _: Vec3) -> LinearRgba {
  const TIMBER: LinearRgba = LinearRgba::rgb(0.58, 0.40, 0.24);
  const BARN: LinearRgba = LinearRgba::rgb(0.92, 0.24, 0.17);
  const SHINGLE: LinearRgba = LinearRgba::rgb(0.96, 0.96, 0.94);
  const STRAW: LinearRgba = LinearRgba::rgb(0.92, 0.76, 0.36);
  const PLUMAGE: LinearRgba = LinearRgba::rgb(0.99, 0.98, 0.95);
  const COMB: LinearRgba = LinearRgba::rgb(0.95, 0.16, 0.11);
  const BEAK: LinearRgba = LinearRgba::rgb(0.99, 0.74, 0.14);

  if (0.40..1.10).contains(&at.x) && at.z.abs() < 0.26 && at.y > CHUTE_FLOOR + 0.06 {
    if at.x > 0.90 || at.y < CHUTE_FLOOR + 0.22 {
      BEAK
    } else if at.x > 0.60 && at.y > CHUTE_FLOOR + 0.70 {
      COMB
    } else {
      PLUMAGE
    }
  } else if at.y < CHUTE_FLOOR - 0.12 {
    TIMBER
  } else if at.y > COOP_EAVES
    || (at.y > CHUTE_FLOOR + 0.56 && !(DROPPER_BACK - 0.58..0.16).contains(&at.x))
  {
    SHINGLE
  } else if at.x > 0.16 {
    STRAW
  } else {
    BARN
  }
}

fn oven_shell() -> Tree {
  let mouth = sdf::union([
    sdf::at(sdf::along_x(sdf::cylinder(0.40, 1.4)), Vec3::new(0.0, FURNACE_MOUTH, 0.0)),
    sdf::at(
      sdf::cuboid(Vec3::new(1.4, FURNACE_MOUTH / 2.0, 0.40)),
      Vec3::new(0.0, FURNACE_MOUTH / 2.0, 0.0)
    )
  ]);
  let legs = (0..4).map(|corner| {
    let (side, back) = ((corner % 2) as f32 * 2.0 - 1.0, (corner / 2) as f32 * 2.0 - 1.0);
    sdf::at(
      sdf::rounded_box(Vec3::new(0.09, FURNACE_FOOT / 2.0, 0.09), 0.04),
      Vec3::new(0.30 + side * 0.34, FURNACE_FOOT / 2.0, back * 0.62)
    )
  });
  sdf::union(
    [
      sdf::difference(
        sdf::at(
          sdf::rounded_box(Vec3::new(0.72, 0.62, 0.80), 0.10),
          Vec3::new(0.32, FURNACE_FOOT + 0.62, 0.0)
        ),
        mouth
      ),
      sdf::at(
        sdf::rounded_box(Vec3::new(0.86, 0.07, 0.92), 0.05),
        Vec3::new(0.32, FURNACE_FOOT + 1.28, 0.0)
      ),
      sdf::at(
        sdf::rounded_box(Vec3::new(0.46, 0.06, 0.52), 0.04),
        Vec3::new(0.32, FURNACE_FOOT + 1.44, 0.0)
      ),
      sdf::at(sdf::cylinder(0.17, 0.50), Vec3::new(0.32, FURNACE_FOOT + 1.92, 0.0)),
      sdf::at(
        sdf::rounded_cylinder(0.25, 0.08, 0.05),
        Vec3::new(0.32, FURNACE_FOOT + 2.40, 0.0)
      ),
      sdf::at(
        sdf::rounded_box(Vec3::new(0.05, 0.05, 0.62), 0.04),
        Vec3::new(-0.44, FURNACE_FOOT + 1.10, 0.0)
      )
    ]
    .into_iter()
    .chain(legs)
  )
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
    belt_deck(),
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
      belt_deck(),
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
  materials: [Handle<StandardMaterial>; MachineKind::COUNT],
  arrow_mesh: Handle<Mesh>,
  arrow_material: Handle<StandardMaterial>,
  glow_mesh: Handle<Mesh>,
  torch_glow: Handle<StandardMaterial>,
  torch_flame: Handle<EffectAsset>,
  bonfire_flame: Handle<EffectAsset>,
  jet_flame: Handle<EffectAsset>,
  wash_spray: Handle<EffectAsset>,
  chute_mesh: Handle<Mesh>,
  chute_material: Handle<StandardMaterial>,
  lens_mesh: Handle<Mesh>,
  flap_mesh: Handle<Mesh>,
  flap_material: Handle<StandardMaterial>,
  sheet_mesh: Handle<Mesh>,
  sheet_material: Handle<StandardMaterial>,
  ember_mesh: Handle<Mesh>,
  ember_glow: Handle<StandardMaterial>,
  lamp_glow: Handle<StandardMaterial>,
  pub ghost_valid: Handle<StandardMaterial>,
  pub ghost_blocked: Handle<StandardMaterial>
}

impl MachineAssets {
  pub fn mesh(&self, kind: MachineKind) -> Handle<Mesh> {
    self.meshes[kind.index()].clone()
  }

  fn baked(kind: MachineKind) -> Mesh {
    kind
      .paint()
      .map(|paint| sdf::bake_painted(Self::shape(kind), machine_bounds(), paint))
      .unwrap_or_else(|| sdf::bake(Self::shape(kind), machine_bounds()))
  }

  fn shape(kind: MachineKind) -> Tree {
    match kind {
      MachineKind::Conveyor => belt_deck(),
      MachineKind::Dropper => dropper_body(),
      MachineKind::Coop => coop_body(),
      MachineKind::Furnace => oven_shell(),
      MachineKind::FlameJet => jet_nozzle(),
      MachineKind::Orewash => orewash_tunnel(),
      MachineKind::Torch => torch_post(),
      MachineKind::Bonfire => bonfire_pile(),
      MachineKind::Lamp => lamp_post(LAMP_HEIGHT, LAMP_SHADE, LAMP_TILT),
      MachineKind::Floodlight => flood_mast(),
      _ => arch()
    }
  }
}

#[derive(Resource)]
pub struct MachinePreviews([Handle<Image>; MachineKind::COUNT]);

impl MachinePreviews {
  const RESOLUTION: u32 = 192;

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

  let machine_meshes =
    MachineKind::ALL.map(|kind| meshes.add(MachineAssets::baked(kind)));
  let grain = images.add(texture::wood());
  let boards = images.add(texture::planks());
  let machine_materials = MachineKind::ALL.map(|kind| {
    let (base_color, emissive) = kind.accent();
    let surface = kind.surface();
    let Finish { roughness, metallic, reflectance } = surface.finish();
    let tiling = surface.tiling().unwrap_or(Vec2::ONE);
    materials.add(StandardMaterial {
      base_color,
      emissive,
      base_color_texture: (surface == Surface::Wood)
        .then(|| grain.clone())
        .or_else(|| (surface == Surface::Planked).then(|| boards.clone())),
      uv_transform: Affine2::from_scale(Vec2::ONE / tiling),
      perceptual_roughness: roughness,
      reflectance,
      metallic,
      ..default()
    })
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

    commands.spawn((
      Mesh3d(machine_meshes[kind.index()].clone()),
      MeshMaterial3d(machine_materials[kind.index()].clone()),
      Transform::from_translation(stage).with_rotation(Quat::from_rotation_y(-0.6)),
      layer.clone()
    ));
    commands.spawn((
      DirectionalLight { illuminance: 6000.0, ..default() },
      Transform::from_translation(stage + Vec3::new(4.0, 6.0, 5.0))
        .looking_at(focus, Vec3::Y),
      layer.clone()
    ));
    commands.spawn((
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
      Transform::from_translation(stage + Vec3::new(4.0, 3.7, 4.8))
        .looking_at(focus, Vec3::Y),
      layer
    ));
    image
  });

  commands.insert_resource(MachinePreviews(previews));
  commands.insert_resource(MachineAssets {
    meshes: machine_meshes,
    materials: machine_materials,
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
    ember_mesh: meshes.add(Sphere::new(1.0).mesh().ico(2).expect("ember mesh")),
    ember_glow: materials.add(StandardMaterial {
      base_color: EMBER_GLOW,
      emissive: LinearRgba::rgb(14.0, 3.4, 0.35),
      ..default()
    }),
    lamp_glow: materials.add(StandardMaterial {
      base_color: LAMP_GLOW,
      emissive: LinearRgba::rgb(38.0, 34.0, 25.0),
      ..default()
    }),
    ghost_valid: materials.add(ghost(Color::srgba(0.25, 0.95, 0.45, 0.35))),
    ghost_blocked: materials.add(ghost(Color::srgba(0.95, 0.25, 0.25, 0.30)))
  });
}

fn arrow_at(assets: &MachineAssets, slide: f32) -> impl Bundle {
  (
    Mesh3d(assets.arrow_mesh.clone()),
    MeshMaterial3d(assets.arrow_material.clone()),
    Transform::from_xyz(slide * ARROW_SPAN, BELT_TOP + 0.02, 0.0)
      .with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
  )
}

pub fn spawn_ghost_arrows(commands: &mut Commands, assets: &MachineAssets, root: Entity) {
  for step in 0..ARROWS_PER_BELT {
    let slide = step as f32 / (ARROWS_PER_BELT - 1) as f32 - 0.5;
    commands.spawn((arrow_at(assets, slide), ChildOf(root)));
  }
}

pub fn place(
  commands: &mut Commands,
  assets: &MachineAssets,
  kind: MachineKind,
  transform: Transform
) -> Entity {
  let root = commands
    .spawn((
      Name::new(kind.spec().name),
      RigidBody::Static,
      Mesh3d(assets.mesh(kind)),
      MeshMaterial3d(assets.materials[kind.index()].clone()),
      transform
    ))
    .id();

  let mut parts: Vec<(Collider, Transform)> = Vec::new();

  match kind {
    MachineKind::Dropper => {
      parts.push((
        Collider::cuboid(0.9, 1.5, 0.9),
        Transform::from_xyz(DROPPER_BACK, 0.75, 0.0)
      ));
      parts.push((
        Collider::cuboid(1.3, 1.1, 1.3),
        Transform::from_xyz(DROPPER_BACK, 1.78, 0.0)
      ));
      commands.spawn((
        Mesh3d(assets.chute_mesh.clone()),
        MeshMaterial3d(assets.chute_material.clone()),
        NotShadowCaster,
        Transform::from_xyz(CHUTE_REACH / 2.0 - 0.14, CHUTE_FLOOR + 0.22, 0.0),
        ChildOf(root)
      ));
      commands.entity(root).insert(Dropper {
        timer: Timer::from_seconds(0.65, TimerMode::Repeating),
        value: 12.0,
        form: OreForm::Rock
      });
    }
    MachineKind::Coop => {
      parts.push((
        Collider::cuboid(1.3, CHUTE_FLOOR + 1.0, 1.3),
        Transform::from_xyz(DROPPER_BACK, (CHUTE_FLOOR + 1.0) / 2.0, 0.0)
      ));
      parts.push((
        Collider::cuboid(CHUTE_REACH, 0.14, 0.68),
        Transform::from_xyz(CHUTE_REACH / 2.0, CHUTE_FLOOR, 0.0)
      ));
      commands.entity(root).insert(Dropper {
        timer: Timer::from_seconds(1.1, TimerMode::Repeating),
        value: 34.0,
        form: OreForm::Egg
      });
    }
    MachineKind::Furnace => {
      parts.push((
        Collider::cuboid(1.5, 2.6, 1.7),
        Transform::from_xyz(0.57, FURNACE_FOOT + 1.3, 0.0)
      ));
      commands.spawn((
        Furnace,
        Collider::cuboid(1.2, 0.76, 0.76),
        Sensor,
        CollisionEventsEnabled,
        Mesh3d(assets.ember_mesh.clone()),
        MeshMaterial3d(assets.ember_glow.clone()),
        NotShadowCaster,
        PointLight { color: EMBER_GLOW, intensity: 700_000.0, range: 14.0, ..default() },
        Transform::from_xyz(-0.15, FURNACE_MOUTH, 0.0)
          .with_scale(Vec3::new(0.52, 0.34, 0.36)),
        ChildOf(root)
      ));
    }
    MachineKind::Bonfire => {
      parts.push((
        Collider::cylinder(0.62, BONFIRE_TOP),
        Transform::from_xyz(0.0, BONFIRE_TOP / 2.0, 0.0)
      ));
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
      parts.push((
        Collider::cylinder(0.2, TORCH_HEAD + 0.3),
        Transform::from_xyz(0.0, (TORCH_HEAD + 0.3) / 2.0, 0.0)
      ));
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
      parts.push((
        Collider::cuboid(0.7, FLOOD_HEIGHT + 0.6, FLOOD_SPAN * 2.0 + 0.6),
        Transform::from_xyz(0.0, (FLOOD_HEIGHT + 0.6) / 2.0, 0.0)
      ));
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
      parts.push((
        Collider::cylinder(LAMP_SHADE, LAMP_HEIGHT + 0.5),
        Transform::from_xyz(0.0, (LAMP_HEIGHT + 0.5) / 2.0, 0.0)
      ));
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
        Transform::from_translation(Vec3::new(0.0, LAMP_HEIGHT, 0.0) + beam * 0.24)
          .looking_to(beam, Vec3::Y)
          .with_scale(Vec3::new(LAMP_SHADE * 0.78, LAMP_SHADE * 0.78, LAMP_SHADE * 0.4)),
        ChildOf(root)
      ));
    }
    _ => {
      commands.spawn((
        ConveyorBelt { local_direction: Vec3::X, speed: 3.5 },
        Collider::cuboid(1.98, BELT_TOP, 1.84),
        Friction::new(1.0),
        Transform::from_xyz(0.0, BELT_TOP / 2.0, 0.0),
        ChildOf(root)
      ));
      for side in [-1.0, 1.0] {
        parts.push((
          Collider::cuboid(1.98, 0.20, 0.12),
          Transform::from_xyz(0.0, 0.16, side * 0.95)
        ));
      }
      for step in 0..ARROWS_PER_BELT {
        commands.spawn((
          BeltArrow(step as f32 / ARROWS_PER_BELT as f32),
          arrow_at(assets, 0.0),
          ChildOf(root)
        ));
      }
      if let Some(upgrader) = kind.upgrade() {
        if kind == MachineKind::FlameJet {
          parts.push((
            Collider::cuboid(0.8, 2.0, 0.7),
            Transform::from_xyz(0.0, 1.0, -1.12)
          ));
          commands.spawn((
            ParticleEffect::new(assets.jet_flame.clone()),
            Transform::from_xyz(0.0, JET_HEIGHT, -0.26),
            ChildOf(root)
          ));
        } else {
          parts.push((
            Collider::cuboid(1.04, 2.5, 0.5),
            Transform::from_xyz(0.0, 1.05, 0.87)
          ));
          parts.push((
            Collider::cuboid(1.04, 2.5, 0.5),
            Transform::from_xyz(0.0, 1.05, -0.87)
          ));
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
        commands.spawn((
          upgrader,
          Collider::cuboid(0.5, 0.8, 1.5),
          Sensor,
          CollisionEventsEnabled,
          Transform::from_xyz(0.0, BELT_TOP + 0.4, 0.0),
          ChildOf(root)
        ));
      }
    }
  }

  for (collider, offset) in parts {
    commands.spawn((collider, offset, ChildOf(root)));
  }
  root
}

pub fn plugin(app: &mut App) {
  app.add_plugins(HanabiPlugin).add_systems(PreStartup, load_machine_assets);
}

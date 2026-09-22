use {crate::{machine::{Money, OreSold},
             material::{self, Coats},
             menu::playing,
             part::{self, Assembly, group},
             player::Player,
             store::{ToastStack, announce},
             style,
             world::ISLAND_TOP},
     bevy::prelude::*};

const BAG_AT: Vec3 = Vec3::new(-27.0, ISLAND_TOP, 9.0);
const BAG_TURN: f32 = 0.42;
const BAG_LEAN: f32 = 0.09;
const BAG_REACH: f32 = 2.2;
const BAG_PURSE: f32 = 100_000.0;
const BAG_BELLY: f32 = 1.10;
const BAG_MOUTH: f32 = 0.34;
const BAG_NECK: f32 = 1.02;
const BAG_TIE: f32 = 1.08;
const BAG_CROWN: f32 = 1.38;
const BAG_FACE: f32 = 0.56;
const BAG_GLYPH: f32 = 0.46;
const BAG_SIDES: u32 = 9;
const BAG_FRILLS: u32 = 6;

fn dollar() -> Assembly {
  let ink = material::GOLD;
  let bar = |rise: f32, wide: f32| {
    ink.slab(Vec3::new(0.05, 0.08, wide)).at(Vec3::new(BAG_FACE, rise, 0.0))
  };
  let riser = |rise: f32, across: f32| {
    ink.slab(Vec3::new(0.05, 0.21, 0.08)).at(Vec3::new(BAG_FACE, rise, across))
  };
  group([
    bar(0.24, 0.38),
    bar(0.0, 0.38),
    bar(-0.24, 0.38),
    riser(0.12, 0.15),
    riser(-0.12, -0.15),
    ink.slab(Vec3::new(0.05, 0.62, 0.07)).at(Vec3::new(BAG_FACE, 0.0, 0.0))
  ])
  .at(Vec3::Y * BAG_GLYPH)
}

fn money_bag() -> Assembly {
  let (canvas, rope, gold) = (material::CANVAS, material::ROPE, material::GOLD);
  let sack = |foot: f32, bore: f32, low: f32, high: f32| {
    canvas.tapered(foot, bore, high - low, BAG_SIDES).on(Vec3::Y * low)
  };
  let frills = part::ring(BAG_FRILLS, |frill| {
    let lean = 0.5 + 0.22 * (frill % 3) as f32;
    group([canvas
      .slab(Vec3::new(0.26, 0.05, 0.13))
      .on(Vec3::new(0.10, BAG_TIE + 0.04, 0.0))
      .tilted(-lean)])
  });
  let tie = group([rope.slab(Vec3::new(0.06, 0.09, 0.16)).at(Vec3::X * 0.21)])
    .ringed(BAG_SIDES, 0.0)
    .at(Vec3::Y * BAG_TIE);
  let tail = group([
    rope
      .beam(0.3, 0.05)
      .span(Vec3::new(0.16, BAG_TIE, 0.14), Vec3::new(0.40, BAG_TIE - 0.22, 0.30)),
    rope
      .beam(0.3, 0.05)
      .span(Vec3::new(0.40, BAG_TIE - 0.22, 0.30), Vec3::new(0.46, BAG_TIE - 0.52, 0.22))
  ]);
  let coin = |at: Vec3, tilt: f32, turn: f32| {
    gold.tapered(0.21, 0.21, 0.035, 8).on(at).tilted(tilt).turned(turn)
  };
  let spill = group([
    coin(Vec3::new(0.74, 0.0, -0.30), 0.0, 0.0),
    coin(Vec3::new(0.92, 0.01, -0.12), 0.0, 0.6),
    coin(Vec3::new(0.66, 0.06, 0.44), 0.42, 1.1),
    coin(Vec3::new(-0.58, 0.0, 0.62), 0.0, 0.3),
    gold.slab(Vec3::new(0.30, 0.11, 0.17)).on(Vec3::new(-0.72, 0.0, -0.36)).turned(0.5),
    gold.slab(Vec3::new(0.30, 0.11, 0.17)).on(Vec3::new(-0.62, 0.11, -0.30)).turned(0.34)
  ]);

  group([
    sack(0.64, 0.94, 0.0, 0.16),
    sack(0.94, BAG_BELLY, 0.16, 0.54),
    sack(BAG_BELLY, 0.74, 0.54, 0.82),
    sack(0.74, 0.42, 0.82, BAG_NECK),
    sack(0.42, BAG_MOUTH, BAG_NECK, BAG_CROWN),
    sack(BAG_MOUTH, 0.08, BAG_CROWN, BAG_CROWN + 0.09),
    canvas.ball(0.22).at(Vec3::new(-0.34, 0.30, 0.26)),
    canvas.ball(0.17).at(Vec3::new(0.24, 0.22, -0.40))
  ])
  .with(tie)
  .with(tail)
  .with(frills)
  .with(dollar())
  .tilted(BAG_LEAN)
  .with(spill)
}

#[derive(Component)]
struct MoneyBag;

fn drop_money_bag(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut coats: ResMut<Coats>
) {
  commands.spawn((
    Name::new("Bag of Money"),
    MoneyBag,
    Transform::from_translation(BAG_AT).with_rotation(Quat::from_rotation_y(BAG_TURN)),
    Visibility::default(),
    part::spawned(part::assembled(money_bag()), &mut meshes, &mut coats, &mut materials)
  ));
}

fn pocket_money_bag(
  stack: Single<Entity, With<ToastStack>>,
  player: Single<&Transform, With<Player>>,
  bags: Query<(Entity, &Transform), (With<MoneyBag>, Without<Player>)>,
  mut money: ResMut<Money>,
  mut sold: MessageWriter<OreSold>,
  mut commands: Commands
) {
  for (bag, at) in &bags {
    if at.translation.distance(player.translation) < BAG_REACH {
      money.0 += BAG_PURSE;
      sold.write(OreSold { at: at.translation + Vec3::Y * BAG_CROWN, value: BAG_PURSE });
      announce(&mut commands, *stack, "You pocket the bag of money", style::CASH);
      commands.entity(bag).despawn();
    }
  }
}

pub fn plugin(app: &mut App) {
  app
    .add_systems(Startup, drop_money_bag)
    .add_systems(Update, pocket_money_bag.run_if(playing).run_if(crate::showcase::idle));
}

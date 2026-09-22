use {crate::ore::{Effects, GIRTH_CAP, Ore, OreAssets, OreForm, OreLimit, SEAT},
     avian3d::prelude::*,
     bevy::{ecs::{entity::EntityHashSet,
                  system::{SystemParam, lifetimeless::Read}},
            prelude::*}};

const GIRTH_DETAIL: u32 = 8;
pub const PURSE: f32 = 600.0;

#[derive(Component)]
#[require(ActiveCollisionHooks::MODIFY_CONTACTS)]
pub struct ConveyorBelt {
  pub local_direction: Vec3,
  pub speed: f32
}

#[derive(SystemParam)]
pub struct ConveyorHooks<'w, 's> {
  belts: Query<'w, 's, (Read<ConveyorBelt>, Read<GlobalTransform>)>
}

impl CollisionHooks for ConveyorHooks<'_, '_> {
  fn modify_contacts(
    &self,
    contacts: &mut ContactPair,
    _commands: &mut Commands
  ) -> bool {
    if let Some((belt, transform, sign)) =
      [(contacts.collider1, -1.0), (contacts.collider2, 1.0)].into_iter().find_map(
        |(entity, sign)| {
          self.belts.get(entity).ok().map(|(belt, transform)| (belt, transform, sign))
        }
      )
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
  pub form: OreForm,
  pub spout: Vec3
}

#[derive(Component)]
pub struct Upgrader {
  pub multiplier: f32,
  pub effects: Effects,
  pub growth: f32
}

impl Upgrader {
  fn takes(&self, ore: &Ore) -> bool {
    !ore.effects.contains(self.effects) || (self.growth > 1.0 && ore.girth < GIRTH_CAP)
  }
}

#[derive(Component)]
pub struct Furnace;

#[derive(Message)]
pub struct OreSold {
  pub at: Vec3,
  pub value: f32
}

#[derive(Resource)]
pub struct Money(pub f32);

impl Default for Money {
  fn default() -> Self { Self(crate::opts::opts().money) }
}

fn machine_and_ore<'w, T: Component>(
  event: &CollisionStart,
  machines: &'w Query<&T>
) -> Option<(&'w T, Entity)> {
  [(event.collider1, event.collider2), (event.collider2, event.collider1)]
    .into_iter()
    .find_map(|(machine, ore)| machines.get(machine).ok().map(|found| (found, ore)))
}

fn drop_ores(
  time: Res<Time>,
  limit: Res<OreLimit>,
  assets: Res<OreAssets>,
  ores: Query<(), With<Ore>>,
  mut droppers: Query<(&mut Dropper, &GlobalTransform)>,
  mut commands: Commands
) {
  let mut live = ores.iter().count();
  for (mut dropper, transform) in &mut droppers {
    dropper.timer.tick(time.delta());
    if dropper.timer.just_finished() && live < limit.0 {
      let spawn = transform.transform_point(dropper.spout);
      commands.spawn(assets.spawn(dropper.form, spawn, dropper.value));
      live += 1;
    }
  }
}

fn upgrade_ores(
  mut events: MessageReader<CollisionStart>,
  upgraders: Query<&Upgrader>,
  assets: Res<OreAssets>,
  mut ores: Query<(
    &mut Ore,
    &mut MeshMaterial3d<StandardMaterial>,
    &mut Transform,
    &mut Collider
  )>
) {
  for event in events.read() {
    if let Some((upgrader, entity)) = machine_and_ore(event, &upgraders)
      && let Ok((mut ore, mut material, mut transform, mut collider)) =
        ores.get_mut(entity)
      && upgrader.takes(&ore)
    {
      ore.value *= upgrader.multiplier;
      ore.effects = ore.effects.with(upgrader.effects);
      let grown = (ore.girth * upgrader.growth).min(GIRTH_CAP);
      material.0 = assets.material(ore.form, ore.effects);
      transform.translation.y += SEAT * (grown - ore.girth);
      ore.girth = grown;
      transform.scale = Vec3::splat(ore.girth);
      collider.set_scale(Vec3::splat(ore.girth), GIRTH_DETAIL);
    }
  }
}

fn burn_ores(
  mut events: MessageReader<CollisionStart>,
  furnaces: Query<&Furnace>,
  ores: Query<(&Ore, &GlobalTransform)>,
  mut sold: MessageWriter<OreSold>,
  mut money: ResMut<Money>,
  mut commands: Commands
) {
  let mut burned = EntityHashSet::default();
  for event in events.read() {
    if let Some((_, entity)) = machine_and_ore(event, &furnaces)
      && burned.insert(entity)
      && let Ok((ore, transform)) = ores.get(entity)
    {
      money.0 += ore.value;
      sold.write(OreSold { at: transform.translation(), value: ore.value });
      commands.entity(entity).try_despawn();
    }
  }
}

fn cull_fallen_ores(
  ores: Query<(Entity, &Transform), With<Ore>>,
  mut commands: Commands
) {
  for (entity, transform) in &ores {
    if transform.translation.y < -30.0 {
      commands.entity(entity).try_despawn();
    }
  }
}

pub fn plugin(app: &mut App) {
  app
    .init_resource::<Money>()
    .add_message::<OreSold>()
    .add_systems(Update, (drop_ores, upgrade_ores, burn_ores, cull_fallen_ores));
}

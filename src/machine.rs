use {crate::{catalog::CHUTE_REACH,
             ore::{Effects, Ore, OreAssets, OreForm, OreLimit}},
     avian3d::prelude::*,
     bevy::{ecs::{entity::EntityHashSet,
                  system::{SystemParam, lifetimeless::Read}},
            prelude::*}};

const DROP_HEIGHT: f32 = 0.92;
const ARROW_SPEED: f32 = 0.45;
pub const ARROW_SPAN: f32 = 1.2;

#[derive(Component)]
#[require(ActiveCollisionHooks::MODIFY_CONTACTS)]
pub struct ConveyorBelt {
  pub local_direction: Vec3,
  pub speed: f32
}

#[derive(Component)]
pub struct BeltArrow(pub f32);

fn slide_belt_arrows(time: Res<Time>, mut arrows: Query<(&BeltArrow, &mut Transform)>) {
  for (BeltArrow(phase), mut transform) in &mut arrows {
    let travel = (phase + time.elapsed_secs() * ARROW_SPEED).rem_euclid(1.0);
    transform.translation.x = (travel - 0.5) * ARROW_SPAN;
    transform.scale = Vec3::splat((4.0 * travel * (1.0 - travel)).sqrt());
  }
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
  pub form: OreForm
}

#[derive(Component)]
pub struct Upgrader {
  pub multiplier: f32,
  pub effects: Effects
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
  fn default() -> Self { Self(600.0) }
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
      let spawn = transform.transform_point(Vec3::new(CHUTE_REACH, DROP_HEIGHT, 0.0));
      commands.spawn(assets.spawn(dropper.form, spawn, dropper.value));
      live += 1;
    }
  }
}

fn upgrade_ores(
  mut events: MessageReader<CollisionStart>,
  upgraders: Query<&Upgrader>,
  assets: Res<OreAssets>,
  mut ores: Query<(&mut Ore, &mut MeshMaterial3d<StandardMaterial>)>
) {
  for event in events.read() {
    if let Some((upgrader, entity)) = machine_and_ore(event, &upgraders)
      && let Ok((mut ore, mut material)) = ores.get_mut(entity)
      && !ore.effects.contains(upgrader.effects)
    {
      ore.value *= upgrader.multiplier;
      ore.effects = ore.effects.with(upgrader.effects);
      material.0 = assets.material(ore.form, ore.effects);
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
      commands.entity(entity).despawn();
    }
  }
}

fn cull_fallen_ores(
  ores: Query<(Entity, &Transform), With<Ore>>,
  mut commands: Commands
) {
  for (entity, transform) in &ores {
    if transform.translation.y < -30.0 {
      commands.entity(entity).despawn();
    }
  }
}

pub fn plugin(app: &mut App) {
  app.init_resource::<Money>().add_message::<OreSold>().add_systems(
    Update,
    (drop_ores, upgrade_ores, burn_ores, cull_fallen_ores, slide_belt_arrows)
  );
}

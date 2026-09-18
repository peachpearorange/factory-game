use crate::catalog::CELL;
use crate::ore::{Effects, Ore, OreAssets, OreLimit};
use avian3d::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::system::{SystemParam, lifetimeless::Read};
use bevy::prelude::*;

const DROP_HEIGHT: f32 = 1.35;

#[derive(Component)]
#[require(ActiveCollisionHooks::MODIFY_CONTACTS)]
pub struct ConveyorBelt {
    pub local_direction: Vec3,
    pub speed: f32,
}

#[derive(SystemParam)]
pub struct ConveyorHooks<'w, 's> {
    belts: Query<'w, 's, (Read<ConveyorBelt>, Read<GlobalTransform>)>,
}

impl CollisionHooks for ConveyorHooks<'_, '_> {
    fn modify_contacts(&self, contacts: &mut ContactPair, _commands: &mut Commands) -> bool {
        if let Some((belt, transform, sign)) =
            [(contacts.collider1, -1.0), (contacts.collider2, 1.0)]
                .into_iter()
                .find_map(|(entity, sign)| {
                    self.belts
                        .get(entity)
                        .ok()
                        .map(|(belt, transform)| (belt, transform, sign))
                })
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
}

#[derive(Component)]
pub struct Upgrader {
    pub multiplier: f32,
    pub effects: Effects,
}

#[derive(Component)]
pub struct Furnace;

#[derive(Resource)]
pub struct Money(pub f32);

impl Default for Money {
    fn default() -> Self {
        Self(600.0)
    }
}

fn machine_and_ore<'w, T: Component>(
    event: &CollisionStart,
    machines: &'w Query<&T>,
) -> Option<(&'w T, Entity)> {
    [
        (event.collider1, event.collider2),
        (event.collider2, event.collider1),
    ]
    .into_iter()
    .find_map(|(machine, ore)| machines.get(machine).ok().map(|found| (found, ore)))
}

fn drop_ores(
    time: Res<Time>,
    limit: Res<OreLimit>,
    assets: Res<OreAssets>,
    ores: Query<(), With<Ore>>,
    mut droppers: Query<(&mut Dropper, &GlobalTransform)>,
    mut commands: Commands,
) {
    let mut live = ores.iter().count();
    for (mut dropper, transform) in &mut droppers {
        dropper.timer.tick(time.delta());
        if dropper.timer.just_finished() && live < limit.0 {
            let spawn = transform.transform_point(Vec3::new(CELL, DROP_HEIGHT, 0.0));
            commands.spawn(assets.spawn(spawn, dropper.value));
            live += 1;
        }
    }
}

fn upgrade_ores(
    mut events: MessageReader<CollisionStart>,
    upgraders: Query<&Upgrader>,
    assets: Res<OreAssets>,
    mut ores: Query<(&mut Ore, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    for event in events.read() {
        if let Some((upgrader, entity)) = machine_and_ore(event, &upgraders)
            && let Ok((mut ore, mut material)) = ores.get_mut(entity)
            && !ore.effects.contains(upgrader.effects)
        {
            ore.value *= upgrader.multiplier;
            ore.effects = ore.effects.with(upgrader.effects);
            material.0 = assets.material(ore.effects);
        }
    }
}

fn burn_ores(
    mut events: MessageReader<CollisionStart>,
    furnaces: Query<&Furnace>,
    ores: Query<&Ore>,
    mut money: ResMut<Money>,
    mut commands: Commands,
) {
    let mut burned = EntityHashSet::default();
    for event in events.read() {
        if let Some((_, entity)) = machine_and_ore(event, &furnaces)
            && burned.insert(entity)
            && let Ok(ore) = ores.get(entity)
        {
            money.0 += ore.value;
            commands.entity(entity).despawn();
        }
    }
}

fn cull_fallen_ores(ores: Query<(Entity, &Transform), With<Ore>>, mut commands: Commands) {
    for (entity, transform) in &ores {
        if transform.translation.y < -30.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Money>().add_systems(
        Update,
        (drop_ores, upgrade_ores, burn_ores, cull_fallen_ores),
    );
}

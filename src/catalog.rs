use crate::machine::{ConveyorBelt, Dropper, Furnace, Upgrader};
use crate::ore::Effects;
use crate::sdf;
use avian3d::prelude::*;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use fidget::context::Tree;

pub const CELL: f32 = 2.0;
pub const BELT_TOP: f32 = 0.71;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MachineKind {
    Conveyor,
    Dropper,
    Forge,
    Furnace,
    MistCoil,
    DecayChamber,
}

pub struct MachineSpec {
    pub name: &'static str,
    pub blurb: &'static str,
    pub price: f32,
    pub unlock: Option<&'static str>,
}

impl MachineKind {
    pub const ALL: [Self; 6] = [
        Self::Conveyor,
        Self::Dropper,
        Self::Forge,
        Self::Furnace,
        Self::MistCoil,
        Self::DecayChamber,
    ];
    pub const COUNT: usize = Self::ALL.len();

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn spec(self) -> MachineSpec {
        match self {
            Self::Conveyor => MachineSpec {
                name: "Conveyor",
                blurb: "Carries ore one cell onward. Everything is downstream of something.",
                price: 25.0,
                unlock: None,
            },
            Self::Dropper => MachineSpec {
                name: "Ore Dropper",
                blurb: "Coughs up a lump of rock every so often. Aim it at a belt.",
                price: 150.0,
                unlock: None,
            },
            Self::Forge => MachineSpec {
                name: "Flame Forge",
                blurb: "Sets passing ore alight and multiplies what it is worth.",
                price: 400.0,
                unlock: None,
            },
            Self::Furnace => MachineSpec {
                name: "Furnace",
                blurb: "Swallows whatever reaches it and pays out its value.",
                price: 250.0,
                unlock: None,
            },
            Self::MistCoil => MachineSpec {
                name: "Mist Coil",
                blurb: "Soaks ore through. Wet things carry charge differently.",
                price: 900.0,
                unlock: Some("Burn an ore worth over $500"),
            },
            Self::DecayChamber => MachineSpec {
                name: "Decay Chamber",
                blurb: "Leaves ore humming and faintly green for a very long time.",
                price: 2200.0,
                unlock: Some("Burn 250 ore"),
            },
        }
    }

    pub const fn upgrade(self) -> Option<Upgrader> {
        match self {
            Self::Forge => Some(Upgrader {
                multiplier: 2.5,
                effects: Effects::FIERY,
            }),
            Self::MistCoil => Some(Upgrader {
                multiplier: 4.0,
                effects: Effects::WET,
            }),
            Self::DecayChamber => Some(Upgrader {
                multiplier: 9.0,
                effects: Effects::RADIOACTIVE,
            }),
            _ => None,
        }
    }

    fn accent(self) -> (Color, LinearRgba) {
        match self {
            Self::Conveyor => (Color::srgb(0.30, 0.31, 0.34), LinearRgba::BLACK),
            Self::Dropper => (Color::srgb(0.52, 0.54, 0.58), LinearRgba::BLACK),
            Self::Forge => (Color::srgb(0.44, 0.24, 0.18), LinearRgba::rgb(0.85, 0.22, 0.03)),
            Self::Furnace => (Color::srgb(0.32, 0.15, 0.12), LinearRgba::rgb(1.30, 0.30, 0.04)),
            Self::MistCoil => (Color::srgb(0.22, 0.34, 0.48), LinearRgba::rgb(0.06, 0.40, 0.85)),
            Self::DecayChamber => {
                (Color::srgb(0.24, 0.40, 0.22), LinearRgba::rgb(0.10, 0.85, 0.12))
            }
        }
    }
}

fn belt_deck() -> Tree {
    sdf::union([
        sdf::at(
            sdf::rounded_box(Vec3::new(0.98, 0.09, 0.86), 0.04),
            Vec3::new(0.0, BELT_TOP - 0.09, 0.0),
        ),
        sdf::at(
            sdf::cuboid(Vec3::new(0.98, 0.11, 0.07)),
            Vec3::new(0.0, BELT_TOP + 0.04, 0.93),
        ),
        sdf::at(
            sdf::cuboid(Vec3::new(0.98, 0.11, 0.07)),
            Vec3::new(0.0, BELT_TOP + 0.04, -0.93),
        ),
        sdf::at(
            sdf::cuboid(Vec3::new(0.9, 0.31, 0.08)),
            Vec3::new(0.0, 0.31, 0.72),
        ),
        sdf::at(
            sdf::cuboid(Vec3::new(0.9, 0.31, 0.08)),
            Vec3::new(0.0, 0.31, -0.72),
        ),
    ])
}

fn arch() -> Tree {
    sdf::union([
        belt_deck(),
        sdf::difference(
            sdf::at(
                sdf::rounded_box(Vec3::new(0.52, 1.25, 0.99), 0.12),
                Vec3::new(0.0, 1.05, 0.0),
            ),
            sdf::at(
                sdf::cuboid(Vec3::new(0.9, 0.86, 0.74)),
                Vec3::new(0.0, 0.76, 0.0),
            ),
        ),
        sdf::at(sdf::cylinder(0.2, 0.26), Vec3::new(0.0, 2.4, 0.0)),
    ])
}

fn dropper_body() -> Tree {
    sdf::union([
        sdf::at(
            sdf::rounded_box(Vec3::new(0.78, 0.5, 0.78), 0.12),
            Vec3::new(0.0, 1.85, 0.0),
        ),
        sdf::at(
            sdf::cuboid(Vec3::new(0.42, 0.78, 0.42)),
            Vec3::new(0.0, 0.7, 0.0),
        ),
        sdf::at(
            sdf::rounded_box(Vec3::new(0.78, 0.17, 0.28), 0.08),
            Vec3::new(0.86, 1.4, 0.0),
        ),
    ])
}

fn furnace_body() -> Tree {
    sdf::union([
        sdf::difference(
            sdf::smooth_union(
                sdf::at(
                    sdf::rounded_box(Vec3::new(0.88, 0.72, 0.88), 0.14),
                    Vec3::new(0.0, 0.72, 0.0),
                ),
                sdf::at(sdf::sphere(0.68), Vec3::new(0.0, 1.5, 0.0)),
                0.3,
            ),
            sdf::at(
                sdf::rounded_box(Vec3::new(0.55, 0.36, 0.52), 0.08),
                Vec3::new(-0.72, 0.8, 0.0),
            ),
        ),
        sdf::at(sdf::cylinder(0.2, 0.55), Vec3::new(0.0, 2.35, 0.0)),
    ])
}

#[derive(Resource)]
pub struct MachineAssets {
    meshes: [Handle<Mesh>; MachineKind::COUNT],
    materials: [Handle<StandardMaterial>; MachineKind::COUNT],
    pub ghost_valid: Handle<StandardMaterial>,
    pub ghost_blocked: Handle<StandardMaterial>,
}

impl MachineAssets {
    pub fn mesh(&self, kind: MachineKind) -> Handle<Mesh> {
        self.meshes[kind.index()].clone()
    }

    fn shape(kind: MachineKind) -> Tree {
        match kind {
            MachineKind::Conveyor => belt_deck(),
            MachineKind::Dropper => dropper_body(),
            MachineKind::Furnace => furnace_body(),
            _ => arch(),
        }
    }
}

#[derive(Resource)]
pub struct MachinePreviews([Handle<Image>; MachineKind::COUNT]);

impl MachinePreviews {
    const RESOLUTION: u32 = 192;

    pub fn image(&self, kind: MachineKind) -> Handle<Image> {
        self.0[kind.index()].clone()
    }
}

fn load_machine_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let ghost = |color: Color| StandardMaterial {
        base_color: color,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };

    let machine_meshes = MachineKind::ALL
        .map(|kind| meshes.add(sdf::bake(MachineAssets::shape(kind), sdf::Bounds::new(2.8, 6))));
    let machine_materials = MachineKind::ALL.map(|kind| {
        let (base_color, emissive) = kind.accent();
        materials.add(StandardMaterial {
            base_color,
            emissive,
            perceptual_roughness: 0.6,
            metallic: 0.35,
            ..default()
        })
    });

    let previews = MachineKind::ALL.map(|kind| {
        let image = images.add(Image::new_target_texture(
            MachinePreviews::RESOLUTION,
            MachinePreviews::RESOLUTION,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        let layer = RenderLayers::layer(kind.index() + 1);
        let stage = Vec3::new(0.0, -600.0 - 40.0 * kind.index() as f32, 0.0);
        let focus = stage + Vec3::Y * 1.3;

        commands.spawn((
            Mesh3d(machine_meshes[kind.index()].clone()),
            MeshMaterial3d(machine_materials[kind.index()].clone()),
            Transform::from_translation(stage).with_rotation(Quat::from_rotation_y(-0.6)),
            layer.clone(),
        ));
        commands.spawn((
            DirectionalLight {
                illuminance: 6000.0,
                ..default()
            },
            Transform::from_translation(stage + Vec3::new(4.0, 6.0, 5.0)).looking_at(focus, Vec3::Y),
            layer.clone(),
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
            Transform::from_translation(stage + Vec3::new(4.0, 3.7, 4.8)).looking_at(focus, Vec3::Y),
            layer,
        ));
        image
    });

    commands.insert_resource(MachinePreviews(previews));
    commands.insert_resource(MachineAssets {
        meshes: machine_meshes,
        materials: machine_materials,
        ghost_valid: materials.add(ghost(Color::srgba(0.25, 0.95, 0.45, 0.35))),
        ghost_blocked: materials.add(ghost(Color::srgba(0.95, 0.25, 0.25, 0.30))),
    });
}

pub fn place(
    commands: &mut Commands,
    assets: &MachineAssets,
    kind: MachineKind,
    transform: Transform,
) -> Entity {
    let root = commands
        .spawn((
            Name::new(kind.spec().name),
            RigidBody::Static,
            Mesh3d(assets.mesh(kind)),
            MeshMaterial3d(assets.materials[kind.index()].clone()),
            transform,
        ))
        .id();

    let mut parts: Vec<(Collider, Transform)> = Vec::new();

    match kind {
        MachineKind::Dropper => {
            parts.push((
                Collider::cuboid(1.0, 1.6, 1.0),
                Transform::from_xyz(0.0, 0.8, 0.0),
            ));
            parts.push((
                Collider::cuboid(1.7, 1.0, 1.7),
                Transform::from_xyz(0.0, 1.85, 0.0),
            ));
            commands.entity(root).insert(Dropper {
                timer: Timer::from_seconds(0.65, TimerMode::Repeating),
                value: 12.0,
            });
        }
        MachineKind::Furnace => {
            parts.push((
                Collider::cuboid(1.0, 2.0, 1.8),
                Transform::from_xyz(0.5, 1.0, 0.0),
            ));
            commands.spawn((
                Furnace,
                Collider::cuboid(1.9, 1.1, 1.7),
                Sensor,
                CollisionEventsEnabled,
                Transform::from_xyz(0.0, BELT_TOP + 0.45, 0.0),
                ChildOf(root),
            ));
        }
        _ => {
            commands.spawn((
                ConveyorBelt {
                    local_direction: Vec3::X,
                    speed: 3.5,
                },
                Collider::cuboid(1.96, 0.18, 1.72),
                Friction::new(1.0),
                Transform::from_xyz(0.0, BELT_TOP - 0.09, 0.0),
                ChildOf(root),
            ));
            for side in [-1.0, 1.0] {
                parts.push((
                    Collider::cuboid(1.96, 0.22, 0.14),
                    Transform::from_xyz(0.0, BELT_TOP + 0.04, side * 0.93),
                ));
            }
            if let Some(upgrader) = kind.upgrade() {
                parts.push((
                    Collider::cuboid(1.04, 2.5, 0.5),
                    Transform::from_xyz(0.0, 1.05, 0.87),
                ));
                parts.push((
                    Collider::cuboid(1.04, 2.5, 0.5),
                    Transform::from_xyz(0.0, 1.05, -0.87),
                ));
                commands.spawn((
                    upgrader,
                    Collider::cuboid(0.5, 0.8, 1.5),
                    Sensor,
                    CollisionEventsEnabled,
                    Transform::from_xyz(0.0, BELT_TOP + 0.4, 0.0),
                    ChildOf(root),
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
    app.add_systems(PreStartup, load_machine_assets);
}

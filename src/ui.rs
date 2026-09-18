use crate::construction::{BuildMode, PlacedMachine, aimed_entity, machine_root};
use crate::machine::Money;
use crate::ore::{Ore, OreLimit};
use crate::player::{CursorMode, Player};
use avian3d::prelude::*;
use bevy::prelude::*;

const METER_WIDTH: f32 = 200.0;
const DIM: Color = Color::srgb(0.55, 0.58, 0.63);
const BRIGHT: Color = Color::srgb(0.93, 0.94, 0.96);

#[derive(Component)]
struct OreMeterFill;

#[derive(Component)]
struct OreMeterLabel;

#[derive(Component)]
struct MoneyLabel;

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct Tooltip;

#[derive(Component)]
struct TooltipTitle;

#[derive(Component)]
struct TooltipBody;

pub fn label(text: &str, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            padding: UiRect::all(px(12)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.62)),
        children![
            (label("$0", 26.0, BRIGHT), MoneyLabel),
            (label("Ore  0 / 0", 16.0, BRIGHT), OreMeterLabel),
            (
                Node {
                    width: px(METER_WIDTH),
                    height: px(12),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.10, 0.11, 0.13)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.35)),
                children![(
                    Node {
                        width: percent(0),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.98, 0.70, 0.16)),
                    OreMeterFill,
                )],
            ),
            label("B store    Tab inventory    X remove", 12.0, DIM),
        ],
    ));

    commands.spawn((
        Crosshair,
        Node {
            position_type: PositionType::Absolute,
            top: percent(50),
            left: percent(50),
            width: px(5),
            height: px(5),
            margin: UiRect::all(px(-2)),
            border_radius: BorderRadius::all(px(3)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
    ));

    commands.spawn((
        Tooltip,
        Node {
            position_type: PositionType::Absolute,
            top: percent(56),
            left: percent(50),
            margin: UiRect::left(px(-150)),
            width: px(300),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.86)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.18)),
        children![
            (label("", 17.0, BRIGHT), TooltipTitle),
            (label("", 13.0, DIM), TooltipBody),
        ],
    ));
}

fn update_hud(
    money: Res<Money>,
    limit: Res<OreLimit>,
    ores: Query<(), With<Ore>>,
    mut fill: Single<&mut Node, With<OreMeterFill>>,
    mut meter: Single<&mut Text, (With<OreMeterLabel>, Without<MoneyLabel>)>,
    mut cash: Single<&mut Text, (With<MoneyLabel>, Without<OreMeterLabel>)>,
) {
    let live = ores.iter().count();
    fill.width = percent(100.0 * live as f32 / limit.0.max(1) as f32);
    ***meter = (live >= limit.0)
        .then(|| format!("Ore  {live} / {}  — at capacity", limit.0))
        .unwrap_or_else(|| format!("Ore  {live} / {}", limit.0));
    ***cash = format!("${:.0}", money.0);
}

fn update_tooltip(
    mode: Res<BuildMode>,
    pointer: Res<CursorMode>,
    spatial: SpatialQuery,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    player: Single<Entity, With<Player>>,
    parents: Query<&ChildOf>,
    machines: Query<&PlacedMachine>,
    ores: Query<&Ore>,
    mut panel: Single<&mut Node, With<Tooltip>>,
    mut title: Single<&mut Text, (With<TooltipTitle>, Without<TooltipBody>)>,
    mut body: Single<&mut Text, (With<TooltipBody>, Without<TooltipTitle>)>,
) {
    let looked_at = (*pointer == CursorMode::Look && *mode == BuildMode::Idle)
        .then(|| aimed_entity(&spatial, *camera, *player))
        .flatten();

    let described = looked_at.and_then(|entity| {
        ores.get(entity)
            .ok()
            .map(|ore| {
                (
                    format!("{} Ore", ore.effects.label()),
                    format!("Worth ${:.0}", ore.value),
                )
            })
            .or_else(|| {
                machine_root(entity, &parents, &machines)
                    .and_then(|root| machines.get(root).ok())
                    .map(|machine| {
                        let spec = machine.kind.spec();
                        (
                            spec.name.to_string(),
                            format!("{}\nX takes it back to your inventory.", spec.blurb),
                        )
                    })
            })
    });

    panel.display = described
        .as_ref()
        .map(|_| Display::Flex)
        .unwrap_or(Display::None);
    if let Some((heading, detail)) = described {
        ***title = heading;
        ***body = detail;
    }
}

fn update_crosshair(pointer: Res<CursorMode>, mut crosshair: Single<&mut Node, With<Crosshair>>) {
    crosshair.display = (*pointer == CursorMode::Look)
        .then_some(Display::Flex)
        .unwrap_or(Display::None);
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_hud)
        .add_systems(Update, (update_hud, update_tooltip, update_crosshair));
}

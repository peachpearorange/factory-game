use crate::construction::{BuildMode, PlacedMachine, aim_ray, aimed_entity, machine_root};
use crate::icon;
use crate::machine::Money;
use crate::ore::{Effects, Ore, OreLimit};
use crate::player::{Player, UiHover};
use crate::store::HoverInfo;
use avian3d::prelude::*;
use bevy::prelude::*;

const METER_WIDTH: f32 = 200.0;
const DIM: Color = Color::srgb(0.55, 0.58, 0.63);
const BRIGHT: Color = Color::srgb(0.93, 0.94, 0.96);
const FIRE: Color = Color::srgb(1.0, 0.45, 0.15);
const WATER: Color = Color::srgb(0.35, 0.65, 1.0);
const DECAY: Color = Color::srgb(0.45, 0.95, 0.35);

#[derive(Component)]
struct OreMeterFill;

#[derive(Component)]
struct OreMeterLabel;

#[derive(Component)]
struct MoneyLabel;

#[derive(Component)]
struct Tooltip;

#[derive(Component)]
struct TooltipTitle;

#[derive(Component)]
struct TooltipBody;

#[derive(Component)]
struct EffectIcon(Effects);

#[derive(Component)]
struct Hint;

#[derive(Component)]
struct HintTitle;

#[derive(Component)]
struct HintDetail;

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

fn floating(bottom: f32, width: f32) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            bottom: px(bottom),
            left: percent(50),
            margin: UiRect::left(px(-width / 2.0)),
            width: px(width),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(3),
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.9)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.18)),
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
            label("Right-drag to look    R rotate    X remove", 12.0, DIM),
        ],
    ));

    commands.spawn((
        Tooltip,
        floating(150.0, 320.0),
        children![
            (
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(6),
                    ..default()
                },
                children![
                    (icon::flame(FIRE), EffectIcon(Effects::FIERY)),
                    (icon::droplet(WATER), EffectIcon(Effects::WET)),
                    (icon::radiation(DECAY), EffectIcon(Effects::RADIOACTIVE)),
                    (label("", 17.0, BRIGHT), TooltipTitle),
                ],
            ),
            (label("", 13.0, DIM), TooltipBody),
        ],
    ));

    commands.spawn((
        Hint,
        floating(72.0, 260.0),
        children![
            (label("", 15.0, BRIGHT), HintTitle),
            (label("", 12.0, DIM), HintDetail),
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

fn update_hint(
    hovered: Query<(&Interaction, &HoverInfo)>,
    mut panel: Single<&mut Node, With<Hint>>,
    mut title: Single<&mut Text, (With<HintTitle>, Without<HintDetail>)>,
    mut detail: Single<&mut Text, (With<HintDetail>, Without<HintTitle>)>,
) {
    let shown = hovered
        .iter()
        .find(|(state, _)| **state != Interaction::None)
        .map(|(_, info)| info);
    panel.display = shown.map(|_| Display::Flex).unwrap_or(Display::None);
    if let Some(info) = shown {
        ***title = info.title.clone();
        ***detail = info.detail.clone();
    }
}

fn update_tooltip(
    mode: Res<BuildMode>,
    hovering: Res<UiHover>,
    spatial: SpatialQuery,
    eye: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window>,
    player: Single<Entity, With<Player>>,
    parents: Query<&ChildOf>,
    machines: Query<&PlacedMachine>,
    ores: Query<&Ore>,
    mut panel: Single<&mut Node, With<Tooltip>>,
    mut icons: Query<(&EffectIcon, &mut Node), Without<Tooltip>>,
    mut title: Single<&mut Text, (With<TooltipTitle>, Without<TooltipBody>)>,
    mut body: Single<&mut Text, (With<TooltipBody>, Without<TooltipTitle>)>,
) {
    let (camera, transform) = *eye;
    let looked_at = (!hovering.0 && *mode == BuildMode::Idle)
        .then(|| aim_ray(camera, transform, *window))
        .flatten()
        .and_then(|ray| aimed_entity(&spatial, ray, *player));

    let described = looked_at.and_then(|entity| {
        ores.get(entity)
            .ok()
            .map(|ore| {
                (
                    format!("{} Ore", ore.effects.label()),
                    format!("Worth ${:.0}", ore.value),
                    ore.effects,
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
                            Effects::NONE,
                        )
                    })
            })
    });

    panel.display = described
        .as_ref()
        .map(|_| Display::Flex)
        .unwrap_or(Display::None);
    let effects = described
        .as_ref()
        .map(|(_, _, effects)| *effects)
        .unwrap_or(Effects::NONE);
    for (EffectIcon(effect), mut node) in &mut icons {
        node.display = effects
            .contains(*effect)
            .then_some(Display::Flex)
            .unwrap_or(Display::None);
    }
    if let Some((heading, detail, _)) = described {
        ***title = heading;
        ***body = detail;
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_hud)
        .add_systems(Update, (update_hud, update_hint, update_tooltip));
}

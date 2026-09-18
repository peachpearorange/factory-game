use crate::machine::Money;
use crate::ore::{Ore, OreLimit};
use bevy::prelude::*;

const METER_WIDTH: f32 = 200.0;

#[derive(Component)]
struct OreMeterFill;

#[derive(Component)]
struct OreMeterLabel;

#[derive(Component)]
struct MoneyLabel;

fn panel_text(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.93, 0.95)),
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
            (panel_text("$0", 26.0), MoneyLabel),
            (panel_text("Ore  0 / 0", 16.0), OreMeterLabel),
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
    let full = live >= limit.0;
    fill.width = percent(100.0 * live as f32 / limit.0.max(1) as f32);
    ***meter = full
        .then(|| format!("Ore  {live} / {}  — at capacity", limit.0))
        .unwrap_or_else(|| format!("Ore  {live} / {}", limit.0));
    ***cash = format!("${:.0}", money.0);
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_hud)
        .add_systems(Update, update_hud);
}

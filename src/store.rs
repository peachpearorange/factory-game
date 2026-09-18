use crate::catalog::{MachineKind, MachinePreviews};
use crate::construction::{BuildMode, Inventory};
use crate::icon;
use crate::machine::Money;
use crate::player::UiHover;
use crate::ui::label;
use bevy::prelude::*;

const DIM: Color = Color::srgb(0.52, 0.55, 0.60);
const BRIGHT: Color = Color::srgb(0.94, 0.95, 0.97);
const TILE: f32 = 112.0;

#[derive(Resource, Default)]
struct Panels {
    store: bool,
    inventory: bool,
}

#[derive(Component)]
struct StorePanel;

#[derive(Component)]
struct InventoryPanel;

#[derive(Component)]
pub struct HoverInfo {
    pub title: String,
    pub detail: String,
}

#[derive(Component)]
struct BuyTile(MachineKind);

#[derive(Component)]
struct StockTile(MachineKind);

#[derive(Component)]
struct StockEntry(MachineKind);

#[derive(Component)]
struct PriceLabel(MachineKind);

#[derive(Component)]
struct CountLabel(MachineKind);

#[derive(Component)]
struct StoreToggle;

#[derive(Component)]
struct InventoryToggle;

fn panel_root() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            top: percent(14),
            left: percent(50),
            margin: UiRect::left(px(-292)),
            width: px(584),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            padding: UiRect::all(px(20)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(12)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.95)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.16)),
    )
}

fn header(glyph: impl Bundle, title: &str, hint: &str) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        children![
            (
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(9),
                    ..default()
                },
                children![glyph, label(title, 23.0, BRIGHT)],
            ),
            label(hint, 12.0, DIM),
        ],
    )
}

fn grid() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(12),
        row_gap: px(12),
        ..default()
    }
}

fn tile(preview: Handle<Image>, locked: bool) -> impl Bundle {
    (
        Button,
        Node {
            width: px(TILE),
            height: px(TILE),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(14)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.09, 0.11)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.12)),
        children![(
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(13)),
                ..default()
            },
            ImageNode::new(preview).with_color(
                locked
                    .then_some(Color::srgb(0.44, 0.46, 0.54))
                    .unwrap_or(Color::WHITE)
            ),
            children![icon::lock(
                locked
                    .then_some(Color::srgba(0.95, 0.96, 1.0, 0.9))
                    .unwrap_or(Color::NONE)
            )],
        )],
    )
}

fn entry(name: &str, face: impl Bundle, caption: impl Bundle) -> impl Bundle {
    (
        Node {
            width: px(TILE),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(3),
            ..default()
        },
        children![face, label(name, 13.0, BRIGHT), caption],
    )
}

fn spawn_panels(mut commands: Commands, previews: Res<MachinePreviews>) {
    let store = commands
        .spawn((
            StorePanel,
            panel_root(),
            children![header(
                icon::store(BRIGHT),
                "Store",
                "What you buy goes to your inventory.",
            )],
        ))
        .id();
    let store_grid = commands.spawn((grid(), ChildOf(store))).id();

    let inventory = commands
        .spawn((
            InventoryPanel,
            panel_root(),
            children![header(
                icon::inventory(BRIGHT),
                "Inventory",
                "Pick one, then click the platform to place it. R rotates, X takes it back.",
            )],
        ))
        .id();
    let inventory_grid = commands.spawn((grid(), ChildOf(inventory))).id();

    for kind in MachineKind::ALL {
        let spec = kind.spec();

        commands.spawn((
            entry(
                spec.name,
                (
                    BuyTile(kind),
                    HoverInfo {
                        title: spec.name.to_string(),
                        detail: spec
                            .unlock
                            .map(|task| format!("{}\nLocked — {task}", spec.blurb))
                            .unwrap_or_else(|| spec.blurb.to_string()),
                    },
                    tile(previews.image(kind), spec.unlock.is_some()),
                ),
                (label("", 12.0, DIM), PriceLabel(kind)),
            ),
            ChildOf(store_grid),
        ));

        commands.spawn((
            StockEntry(kind),
            entry(
                spec.name,
                (
                    StockTile(kind),
                    HoverInfo {
                        title: spec.name.to_string(),
                        detail: spec.blurb.to_string(),
                    },
                    tile(previews.image(kind), false),
                ),
                (label("", 12.0, DIM), CountLabel(kind)),
            ),
            ChildOf(inventory_grid),
        ));
    }
}

fn toggle_button(glyph: impl Bundle, title: &str, key: &str) -> impl Bundle {
    (
        Button,
        HoverInfo {
            title: title.to_string(),
            detail: format!("Press {key}"),
        },
        Node {
            width: px(40),
            height: px(40),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.85)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.18)),
        children![glyph],
    )
}

fn spawn_toolbar(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(20),
            left: percent(50),
            margin: UiRect::left(px(-44)),
            column_gap: px(8),
            ..default()
        },
        children![
            (StoreToggle, toggle_button(icon::store(BRIGHT), "Store", "B")),
            (
                InventoryToggle,
                toggle_button(icon::inventory(BRIGHT), "Inventory", "Tab"),
            ),
        ],
    ));
}

fn toggle_panels(
    keys: Res<ButtonInput<KeyCode>>,
    store: Query<&Interaction, (With<StoreToggle>, Changed<Interaction>)>,
    inventory: Query<&Interaction, (With<InventoryToggle>, Changed<Interaction>)>,
    mut panels: ResMut<Panels>,
) {
    let pressed = |state: &Interaction| *state == Interaction::Pressed;
    if keys.just_pressed(KeyCode::KeyB) || store.iter().any(pressed) {
        panels.store = !panels.store;
        panels.inventory = false;
    }
    if keys.just_pressed(KeyCode::Tab) || inventory.iter().any(pressed) {
        panels.inventory = !panels.inventory;
        panels.store = false;
    }
    if keys.just_pressed(KeyCode::Escape) {
        panels.store = false;
        panels.inventory = false;
    }
}

fn track_ui_hover(panels: Res<Panels>, buttons: Query<&Interaction>, mut hovering: ResMut<UiHover>) {
    hovering.0 = panels.store
        || panels.inventory
        || buttons.iter().any(|state| *state != Interaction::None);
}

fn refresh_panels(
    panels: Res<Panels>,
    inventory: Res<Inventory>,
    money: Res<Money>,
    mut store_panel: Single<&mut Node, (With<StorePanel>, Without<InventoryPanel>)>,
    mut stock_panel: Single<&mut Node, (With<InventoryPanel>, Without<StorePanel>)>,
    mut entries: Query<
        (&StockEntry, &mut Node),
        (Without<StorePanel>, Without<InventoryPanel>),
    >,
    mut prices: Query<(&PriceLabel, &mut Text), Without<CountLabel>>,
    mut counts: Query<(&CountLabel, &mut Text), Without<PriceLabel>>,
    mut tiles: Query<(&BuyTile, &mut BorderColor)>,
) {
    let shown = |open: bool| open.then_some(Display::Flex).unwrap_or(Display::None);
    store_panel.display = shown(panels.store);
    stock_panel.display = shown(panels.inventory);

    for (PriceLabel(kind), mut text) in &mut prices {
        let spec = kind.spec();
        **text = spec
            .unlock
            .map(|_| "Locked".to_string())
            .unwrap_or_else(|| format!("${:.0}", spec.price));
    }
    for (CountLabel(kind), mut text) in &mut counts {
        **text = format!("x{}", inventory.count(*kind));
    }
    for (StockEntry(kind), mut node) in &mut entries {
        node.display = shown(inventory.count(*kind) > 0);
    }
    for (BuyTile(kind), mut border) in &mut tiles {
        let spec = kind.spec();
        *border = BorderColor::all(
            (spec.unlock.is_none() && money.0 >= spec.price)
                .then_some(Color::srgba(0.45, 0.95, 0.55, 0.55))
                .unwrap_or(Color::srgba(1.0, 1.0, 1.0, 0.10)),
        );
    }
}

fn buy_machines(
    tiles: Query<(&BuyTile, &Interaction), Changed<Interaction>>,
    mut money: ResMut<Money>,
    mut inventory: ResMut<Inventory>,
) {
    for (BuyTile(kind), interaction) in &tiles {
        let spec = kind.spec();
        if *interaction == Interaction::Pressed && spec.unlock.is_none() && money.0 >= spec.price {
            money.0 -= spec.price;
            inventory.add(*kind);
        }
    }
}

fn select_machines(
    tiles: Query<(&StockTile, &Interaction), Changed<Interaction>>,
    inventory: Res<Inventory>,
    mut mode: ResMut<BuildMode>,
    mut panels: ResMut<Panels>,
) {
    for (StockTile(kind), interaction) in &tiles {
        if *interaction == Interaction::Pressed && inventory.count(*kind) > 0 {
            *mode = BuildMode::Placing {
                kind: *kind,
                turns: 0,
            };
            panels.inventory = false;
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Panels>()
        .add_systems(Startup, (spawn_panels, spawn_toolbar))
        .add_systems(
            Update,
            (
                toggle_panels,
                buy_machines,
                select_machines,
                track_ui_hover,
                refresh_panels,
            )
                .chain(),
        );
}

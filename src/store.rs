use crate::catalog::MachineKind;
use crate::construction::{BuildMode, Inventory};
use crate::machine::Money;
use crate::player::CursorMode;
use crate::ui::label;
use bevy::prelude::*;

const DIM: Color = Color::srgb(0.52, 0.55, 0.60);
const BRIGHT: Color = Color::srgb(0.94, 0.95, 0.97);

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
struct BuyButton(MachineKind);

#[derive(Component)]
struct BuyLabel(MachineKind);

#[derive(Component)]
struct PlaceButton(MachineKind);

#[derive(Component)]
struct StockRow(MachineKind);

#[derive(Component)]
struct StockLabel(MachineKind);

fn panel(title: &str, hint: &str) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            top: percent(12),
            left: percent(50),
            margin: UiRect::left(px(-230)),
            width: px(460),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            padding: UiRect::all(px(18)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(10)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.94)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.16)),
        children![label(title, 24.0, BRIGHT), label(hint, 13.0, DIM)],
    )
}

fn action_button(width: f32) -> Node {
    Node {
        width: px(width),
        padding: UiRect::axes(px(10), px(7)),
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(px(5)),
        flex_shrink: 0.0,
        ..default()
    }
}

fn row(kind: MachineKind, action: impl Bundle) -> impl Bundle {
    let spec = kind.spec();
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: px(12),
            padding: UiRect::vertical(px(6)),
            ..default()
        },
        children![
            (
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                },
                children![
                    label(spec.name, 17.0, BRIGHT),
                    label(spec.blurb, 12.0, DIM),
                ],
            ),
            action,
        ],
    )
}

fn spawn_panels(mut commands: Commands) {
    let store = commands
        .spawn((StorePanel, panel("Store", "B closes. Machines you buy go to your inventory.")))
        .id();
    let inventory = commands
        .spawn((
            InventoryPanel,
            panel("Inventory", "Tab closes. Place puts one down; R rotates, X takes it back."),
        ))
        .id();

    for kind in MachineKind::ALL {
        commands.spawn((
            row(
                kind,
                (
                    Button,
                    BuyButton(kind),
                    action_button(120.0),
                    BackgroundColor(Color::srgb(0.14, 0.16, 0.19)),
                    children![(label("", 14.0, BRIGHT), BuyLabel(kind))],
                ),
            ),
            ChildOf(store),
        ));
        commands.spawn((
            StockRow(kind),
            row(
                kind,
                (
                    Button,
                    PlaceButton(kind),
                    action_button(96.0),
                    BackgroundColor(Color::srgb(0.16, 0.22, 0.17)),
                    children![(label("", 14.0, BRIGHT), StockLabel(kind))],
                ),
            ),
            ChildOf(inventory),
        ));
    }
}

fn toggle_panels(keys: Res<ButtonInput<KeyCode>>, mut panels: ResMut<Panels>) {
    if keys.just_pressed(KeyCode::KeyB) {
        panels.store = !panels.store;
        panels.inventory = false;
    }
    if keys.just_pressed(KeyCode::Tab) {
        panels.inventory = !panels.inventory;
        panels.store = false;
    }
    if keys.just_pressed(KeyCode::Escape) {
        panels.store = false;
        panels.inventory = false;
    }
}

fn sync_pointer(panels: Res<Panels>, mut pointer: ResMut<CursorMode>) {
    pointer.set_if_neq(
        (panels.store || panels.inventory)
            .then_some(CursorMode::Ui)
            .unwrap_or(CursorMode::Look),
    );
}

fn refresh_panels(
    panels: Res<Panels>,
    inventory: Res<Inventory>,
    mut store: Single<&mut Node, (With<StorePanel>, Without<InventoryPanel>)>,
    mut stock: Single<&mut Node, (With<InventoryPanel>, Without<StorePanel>)>,
    mut rows: Query<(&StockRow, &mut Node), (Without<StorePanel>, Without<InventoryPanel>)>,
    mut buy_labels: Query<(&BuyLabel, &mut Text), Without<StockLabel>>,
    mut stock_labels: Query<(&StockLabel, &mut Text), Without<BuyLabel>>,
) {
    let shown = |open: bool| open.then_some(Display::Flex).unwrap_or(Display::None);
    store.display = shown(panels.store);
    stock.display = shown(panels.inventory);

    for (BuyLabel(kind), mut text) in &mut buy_labels {
        let spec = kind.spec();
        **text = spec
            .unlock
            .map(|_| "Locked".to_string())
            .unwrap_or_else(|| format!("Buy  ${:.0}", spec.price));
    }
    for (StockRow(kind), mut node) in &mut rows {
        node.display = shown(inventory.count(*kind) > 0);
    }
    for (StockLabel(kind), mut text) in &mut stock_labels {
        **text = format!("Place  x{}", inventory.count(*kind));
    }
}

fn tint_buttons(
    money: Res<Money>,
    mut buttons: Query<(&BuyButton, &Interaction, &mut BackgroundColor)>,
) {
    for (BuyButton(kind), interaction, mut color) in &mut buttons {
        let spec = kind.spec();
        let affordable = spec.unlock.is_none() && money.0 >= spec.price;
        color.0 = match (affordable, interaction) {
            (false, _) => Color::srgb(0.13, 0.13, 0.15),
            (true, Interaction::Hovered) => Color::srgb(0.22, 0.30, 0.24),
            (true, Interaction::Pressed) => Color::srgb(0.30, 0.55, 0.34),
            (true, Interaction::None) => Color::srgb(0.16, 0.21, 0.18),
        };
    }
}

fn buy_machines(
    buttons: Query<(&BuyButton, &Interaction), Changed<Interaction>>,
    mut money: ResMut<Money>,
    mut inventory: ResMut<Inventory>,
) {
    for (BuyButton(kind), interaction) in &buttons {
        let spec = kind.spec();
        if *interaction == Interaction::Pressed && spec.unlock.is_none() && money.0 >= spec.price {
            money.0 -= spec.price;
            inventory.add(*kind);
        }
    }
}

fn select_machines(
    buttons: Query<(&PlaceButton, &Interaction), Changed<Interaction>>,
    inventory: Res<Inventory>,
    mut mode: ResMut<BuildMode>,
    mut panels: ResMut<Panels>,
) {
    for (PlaceButton(kind), interaction) in &buttons {
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
        .add_systems(Startup, spawn_panels)
        .add_systems(
            Update,
            (
                toggle_panels,
                buy_machines,
                select_machines,
                sync_pointer,
                refresh_panels,
                tint_buttons,
            )
                .chain(),
        );
}

use {crate::{catalog::{MachineKind, MachinePreviews, Tier},
             construction::{BuildMode, Inventory},
             icon,
             machine::Money,
             menu::{ScreenshotButton, playing},
             player::UiHover,
             ui::{Bold, heavy, label}},
     bevy::prelude::*};

const DIM: Color = Color::srgb(0.52, 0.55, 0.60);
const BRIGHT: Color = Color::srgb(0.94, 0.95, 0.97);
const ITEMS_TINT: Color = Color::srgb(0.36, 0.86, 0.99);
const STORE_TINT: Color = Color::srgb(0.46, 0.93, 0.46);
const GRANTED: Color = Color::srgb(0.48, 0.96, 0.52);
const DENIED: Color = Color::srgb(1.0, 0.56, 0.30);
const SEALED: Color = Color::srgb(0.66, 0.72, 0.86);
const SHOT_TINT: Color = Color::srgb(0.99, 0.78, 0.34);
const INK: Color = Color::srgb(0.06, 0.06, 0.07);
const MUTED: Color = Color::srgb(0.42, 0.44, 0.48);
const BUTTON: f32 = 72.0;
const GLYPH_SCALE: f32 = 2.2;
const TILE: f32 = 124.0;
const TOAST_LIFE: f32 = 1.6;

#[derive(Resource, Default)]
pub struct Panels {
  pub store: bool,
  pub inventory: bool
}

#[derive(Component)]
struct StorePanel;

#[derive(Component)]
struct InventoryPanel;

#[derive(Component)]
pub struct HoverInfo {
  pub title: String,
  pub detail: String
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

#[derive(Component)]
struct ToastStack;

#[derive(Component)]
struct Toast(Timer);

fn side_panel(left: Val, right: Val) -> impl Bundle {
  (
    Node {
      position_type: PositionType::Absolute,
      left,
      right,
      top: px(124),
      bottom: px(112),
      width: px(304),
      flex_direction: FlexDirection::Column,
      row_gap: px(12),
      padding: UiRect::all(px(16)),
      border: UiRect::all(px(1)),
      border_radius: BorderRadius::all(px(12)),
      display: Display::None,
      overflow: Overflow::scroll_y(),
      ..default()
    },
    BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.95)),
    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.16))
  )
}

fn header(glyph: impl Bundle, title: &str, hint: &str) -> impl Bundle {
  (
    Node { flex_direction: FlexDirection::Column, row_gap: px(3), ..default() },
    children![
      (
        Node { align_items: AlignItems::Center, column_gap: px(9), ..default() },
        children![glyph, label(title, 23.0, BRIGHT)],
      ),
      label(hint, 12.0, DIM),
    ]
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

fn tile(
  preview: Handle<Image>,
  tier: Tier,
  name: &str,
  locked: bool,
  caption: impl Bundle,
  bold: &Bold
) -> impl Bundle {
  (
    Button,
    Node {
      width: px(TILE),
      height: px(TILE),
      flex_direction: FlexDirection::Column,
      align_items: AlignItems::Center,
      justify_content: JustifyContent::FlexEnd,
      padding: UiRect::bottom(px(7)),
      border_radius: BorderRadius::all(px(14)),
      overflow: Overflow::clip(),
      ..default()
    },
    BackgroundColor(tier.swatch()),
    children![
      (
        Node {
          position_type: PositionType::Absolute,
          left: px(0),
          right: px(0),
          top: px(-16),
          height: px(TILE + 8.0),
          align_items: AlignItems::Center,
          justify_content: JustifyContent::Center,
          ..default()
        },
        ImageNode::new(preview),
        children![icon::lock(
          locked.then_some(Color::srgba(0.08, 0.08, 0.10, 0.85)).unwrap_or(Color::NONE)
        )],
      ),
      (heavy(name, 14.5, INK, bold), TextLayout {
        justify: Justify::Center,
        ..default()
      },),
      caption,
    ]
  )
}

fn spawn_panels(mut commands: Commands, previews: Res<MachinePreviews>, bold: Res<Bold>) {
  let store = commands
    .spawn((StorePanel, side_panel(Val::Auto, px(18)), children![header(
      icon::store(BRIGHT),
      "Store",
      "What you buy goes to your inventory.",
    )]))
    .id();
  let store_grid = commands.spawn((grid(), ChildOf(store))).id();

  let inventory = commands
    .spawn((InventoryPanel, side_panel(px(18), Val::Auto), children![header(
      icon::inventory(BRIGHT),
      "Inventory",
      "Pick one, then click the platform to place it. R rotates, X takes it back.",
    )]))
    .id();
  let inventory_grid = commands.spawn((grid(), ChildOf(inventory))).id();

  for kind in MachineKind::ALL {
    let spec = kind.spec();

    commands.spawn((
      BuyTile(kind),
      HoverInfo {
        title: spec.name.to_string(),
        detail: spec
          .unlock
          .map(|task| format!("{}\nLocked — {task}", spec.blurb))
          .unwrap_or_else(|| spec.blurb.to_string())
      },
      tile(
        previews.image(kind),
        spec.tier,
        spec.name,
        spec.unlock.is_some(),
        (heavy("", 13.0, INK, &bold), PriceLabel(kind)),
        &bold
      ),
      ChildOf(store_grid)
    ));

    commands.spawn((
      StockEntry(kind),
      StockTile(kind),
      HoverInfo { title: spec.name.to_string(), detail: spec.blurb.to_string() },
      tile(
        previews.image(kind),
        spec.tier,
        spec.name,
        false,
        (heavy("", 13.0, INK, &bold), CountLabel(kind)),
        &bold
      ),
      ChildOf(inventory_grid)
    ));
  }
}

fn toggle_button(
  glyph: impl Bundle,
  title: &str,
  caption: &str,
  key: &str,
  tint: Color
) -> impl Bundle {
  (
    Button,
    HoverInfo { title: title.to_string(), detail: format!("Press {key}") },
    Node {
      width: px(BUTTON),
      height: px(BUTTON),
      flex_direction: FlexDirection::Column,
      align_items: AlignItems::Center,
      justify_content: JustifyContent::Center,
      padding: UiRect::all(px(2)),
      border_radius: BorderRadius::all(px(13)),
      ..default()
    },
    BackgroundColor(Color::srgb(0.07, 0.08, 0.11)),
    children![icon::scaled(glyph, GLYPH_SCALE), label(caption, 14.0, tint)]
  )
}

fn spawn_toolbar(mut commands: Commands) {
  commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(18),
            left: percent(50),
            margin: UiRect::left(px(-((BUTTON * 3.0 + 24.0) / 2.0))),
            column_gap: px(12),
            ..default()
        },
        children![
            (
                InventoryToggle,
                toggle_button(
                    icon::inventory(ITEMS_TINT),
                    "Inventory",
                    "ITEMS",
                    "E",
                    ITEMS_TINT,
                ),
            ),
            (
                StoreToggle,
                toggle_button(icon::store(STORE_TINT), "Store", "STORE", "F", STORE_TINT),
            ),
            (
                ScreenshotButton,
                toggle_button(
                    icon::camera(SHOT_TINT),
                    "Screenshot",
                    "PHOTO",
                    "F2",
                    SHOT_TINT,
                ),
            ),
        ],
    ));

  commands.spawn((ToastStack, Node {
    position_type: PositionType::Absolute,
    bottom: percent(30),
    left: percent(50),
    margin: UiRect::left(px(-180)),
    width: px(360),
    flex_direction: FlexDirection::Column,
    align_items: AlignItems::Center,
    row_gap: px(4),
    ..default()
  }));
}

fn toggle_panels(
  keys: Res<ButtonInput<KeyCode>>,
  store: Query<&Interaction, (With<StoreToggle>, Changed<Interaction>)>,
  inventory: Query<&Interaction, (With<InventoryToggle>, Changed<Interaction>)>,
  mut panels: ResMut<Panels>
) {
  let pressed = |state: &Interaction| *state == Interaction::Pressed;
  if keys.just_pressed(KeyCode::KeyF) || store.iter().any(pressed) {
    panels.store = !panels.store;
  }
  if keys.just_pressed(KeyCode::KeyE) || inventory.iter().any(pressed) {
    panels.inventory = !panels.inventory;
  }
}

fn track_ui_hover(buttons: Query<&Interaction>, mut hovering: ResMut<UiHover>) {
  hovering.0 = buttons.iter().any(|state| *state != Interaction::None);
}

fn refresh_panels(
  panels: Res<Panels>,
  inventory: Res<Inventory>,
  money: Res<Money>,
  mut store_panel: Single<&mut Node, (With<StorePanel>, Without<InventoryPanel>)>,
  mut stock_panel: Single<&mut Node, (With<InventoryPanel>, Without<StorePanel>)>,
  mut entries: Query<
    (&StockEntry, &mut Node),
    (Without<StorePanel>, Without<InventoryPanel>)
  >,
  mut prices: Query<(&PriceLabel, &mut Text), Without<CountLabel>>,
  mut counts: Query<(&CountLabel, &mut Text), Without<PriceLabel>>,
  mut tiles: Query<(&BuyTile, &mut BackgroundColor)>
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
  for (BuyTile(kind), mut background) in &mut tiles {
    let spec = kind.spec();
    background.0 = (spec.unlock.is_none() && money.0 >= spec.price)
      .then(|| spec.tier.swatch())
      .unwrap_or_else(|| spec.tier.swatch().mix(&MUTED, 0.6));
  }
}

fn buy_machines(
  tiles: Query<(&BuyTile, &Interaction), Changed<Interaction>>,
  stack: Single<Entity, With<ToastStack>>,
  mut money: ResMut<Money>,
  mut inventory: ResMut<Inventory>,
  mut commands: Commands
) {
  for (BuyTile(kind), interaction) in &tiles {
    let spec = kind.spec();
    if *interaction == Interaction::Pressed {
      let (announcement, tint) = spec
        .unlock
        .map(|task| (format!("Still sealed — {task}"), SEALED))
        .or_else(|| {
          (money.0 < spec.price).then(|| {
            (format!("${:.0} short of a {}", spec.price - money.0, spec.name), DENIED)
          })
        })
        .unwrap_or_else(|| {
          money.0 -= spec.price;
          inventory.add(*kind);
          (format!("{} is yours", spec.name), GRANTED)
        });
      commands.spawn((
        Toast(Timer::from_seconds(TOAST_LIFE, TimerMode::Once)),
        label(&announcement, 17.0, tint),
        ChildOf(*stack)
      ));
    }
  }
}

fn animate_toasts(
  time: Res<Time>,
  mut toasts: Query<(Entity, &mut Toast, &mut UiTransform, &mut TextColor)>,
  mut commands: Commands
) {
  for (entity, mut toast, mut transform, mut color) in &mut toasts {
    toast.0.tick(time.delta());
    let age = toast.0.fraction();
    transform.translation = Val2::px(0.0, -38.0 * age);
    transform.scale = Vec2::splat(1.0 + 0.4 * (1.0 - age).powi(7));
    color.0 = color.0.with_alpha((3.0 - 3.0 * age).min(1.0));
    if toast.0.is_finished() {
      commands.entity(entity).despawn();
    }
  }
}

fn select_machines(
  tiles: Query<(&StockTile, &Interaction), Changed<Interaction>>,
  inventory: Res<Inventory>,
  mut mode: ResMut<BuildMode>
) {
  for (StockTile(kind), interaction) in &tiles {
    if *interaction == Interaction::Pressed && inventory.count(*kind) > 0 {
      *mode = BuildMode::Placing { kind: *kind, turns: 0 };
    }
  }
}

pub fn plugin(app: &mut App) {
  app
    .init_resource::<Panels>()
    .add_systems(Startup, (spawn_panels, spawn_toolbar))
    .add_systems(
      Update,
      (toggle_panels, buy_machines, select_machines, track_ui_hover, refresh_panels)
        .chain()
        .run_if(playing)
    )
    .add_systems(Update, animate_toasts);
}

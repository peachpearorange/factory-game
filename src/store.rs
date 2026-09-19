use {crate::{catalog::{MachineKind, MachinePreviews, Tier},
             construction::{BuildMode, Inventory},
             icon,
             machine::Money,
             menu::{ScreenshotButton, playing},
             player::UiHover,
             style::{self, Bold, heavy, label, tinted}},
     bevy::{input::mouse::{MouseScrollUnit, MouseWheel},
            prelude::*,
            ui::RelativeCursorPosition}};

const BUTTON: f32 = 72.0;
const GLYPH_SCALE: f32 = 2.2;
const TILE: f32 = 132.0;
const COLUMNS: f32 = 3.0;
const GAP: f32 = 6.0;
const PADDING: f32 = 8.0;
const PANEL_WIDTH: f32 = COLUMNS * TILE + (COLUMNS - 1.0) * GAP + 2.0 * PADDING;
const SCROLL_STEP: f32 = 42.0;
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
struct DeleteToggle;

#[derive(Component)]
struct InventoryToggle;

#[derive(Component)]
struct ToastStack;

#[derive(Component)]
struct Toast(Timer);

fn side_panel(left: Val, right: Val) -> impl Bundle {
  (
    ScrollPosition::default(),
    RelativeCursorPosition::default(),
    Node {
      position_type: PositionType::Absolute,
      left,
      right,
      top: px(124),
      bottom: px(112),
      width: px(PANEL_WIDTH),
      flex_direction: FlexDirection::Column,
      row_gap: px(GAP),
      padding: UiRect::all(px(PADDING)),
      border_radius: BorderRadius::all(px(12)),
      display: Display::None,
      overflow: Overflow::scroll_y(),
      ..default()
    },
    BackgroundColor(style::PANEL)
  )
}

fn header(glyph: impl Bundle, title: &str) -> impl Bundle {
  (Node { align_items: AlignItems::Center, column_gap: px(9), ..default() }, children![
    glyph,
    label(title, style::TITLE)
  ])
}

fn grid() -> Node {
  Node {
    flex_direction: FlexDirection::Row,
    flex_wrap: FlexWrap::Wrap,
    column_gap: px(GAP),
    row_gap: px(GAP),
    flex_shrink: 0.0,
    ..default()
  }
}

fn scroll_panels(
  mut wheel: MessageReader<MouseWheel>,
  mut panels: Query<(&ComputedNode, &RelativeCursorPosition, &mut ScrollPosition)>
) {
  let rolled: f32 = wheel
    .read()
    .map(|event| match event.unit {
      MouseScrollUnit::Line => event.y * SCROLL_STEP,
      MouseScrollUnit::Pixel => event.y
    })
    .sum();
  for (computed, cursor, mut scroll) in &mut panels {
    if rolled != 0.0 && cursor.cursor_over() {
      let reach = (computed.content_size().y - computed.size().y).max(0.0)
        * computed.inverse_scale_factor;
      scroll.y = (scroll.y - rolled).clamp(0.0, reach);
    }
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
          top: px(-26),
          height: px(TILE + 16.0),
          align_items: AlignItems::Center,
          justify_content: JustifyContent::Center,
          ..default()
        },
        ImageNode::new(preview),
        children![icon::lock(locked.then_some(style::VEIL).unwrap_or(Color::NONE))],
      ),
      (heavy(name, 14.5, style::INK, bold), TextLayout {
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
      icon::store(style::TEXT),
      "Store"
    )]))
    .id();
  let store_grid = commands.spawn((grid(), ChildOf(store))).id();

  let inventory = commands
    .spawn((InventoryPanel, side_panel(px(18), Val::Auto), children![header(
      icon::inventory(style::TEXT),
      "Inventory"
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
        (heavy("", style::SMALL, style::INK, &bold), PriceLabel(kind)),
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
        (heavy("", style::SMALL, style::INK, &bold), CountLabel(kind)),
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
    BackgroundColor(style::BUTTON),
    children![icon::scaled(glyph, GLYPH_SCALE), tinted(caption, 14.0, tint)]
  )
}

fn spawn_toolbar(mut commands: Commands) {
  commands.spawn((
    Node {
      position_type: PositionType::Absolute,
      bottom: px(18),
      left: percent(50),
      margin: UiRect::left(px(-((BUTTON * 4.0 + 36.0) / 2.0))),
      column_gap: px(12),
      ..default()
    },
    children![
      (
        InventoryToggle,
        toggle_button(
          icon::inventory(style::ITEMS),
          "Inventory",
          "ITEMS",
          "E",
          style::ITEMS,
        ),
      ),
      (
        StoreToggle,
        toggle_button(icon::store(style::STORE), "Store", "STORE", "F", style::STORE),
      ),
      (
        DeleteToggle,
        toggle_button(
          icon::cross(style::DELETE),
          "Delete mode",
          "CLEAR",
          "Q",
          style::DELETE,
        ),
      ),
      (
        ScreenshotButton,
        toggle_button(
          icon::camera(style::SHOT),
          "Screenshot",
          "PHOTO",
          "F2",
          style::SHOT,
        ),
      ),
    ]
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

fn toggle_delete_mode(
  keys: Res<ButtonInput<KeyCode>>,
  toggle: Query<&Interaction, (With<DeleteToggle>, Changed<Interaction>)>,
  mut mode: ResMut<BuildMode>
) {
  if keys.just_pressed(KeyCode::KeyQ)
    || toggle.iter().any(|state| *state == Interaction::Pressed)
  {
    *mode = (*mode == BuildMode::Idle)
      .then_some(BuildMode::Deleting)
      .unwrap_or(BuildMode::Idle);
  }
}

fn track_ui_hover(buttons: Query<&Interaction>, mut hovering: ResMut<UiHover>) {
  hovering.0 = buttons.iter().any(|state| *state != Interaction::None);
}

fn refresh_panels(
  panels: Res<Panels>,
  inventory: Res<Inventory>,
  money: Res<Money>,
  mode: Res<BuildMode>,
  mut delete: Single<&mut BackgroundColor, (With<DeleteToggle>, Without<BuyTile>)>,
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
  delete.0 = (*mode == BuildMode::Deleting)
    .then_some(style::BUTTON_ARMED)
    .unwrap_or(style::BUTTON);

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
      .unwrap_or_else(|| spec.tier.swatch().mix(&style::MUTED, 0.6));
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
        .map(|task| (format!("Still sealed — {task}"), style::SEALED))
        .or_else(|| {
          (money.0 < spec.price).then(|| {
            (
              format!("${:.0} short of a {}", spec.price - money.0, spec.name),
              style::DENIED
            )
          })
        })
        .unwrap_or_else(|| {
          money.0 -= spec.price;
          inventory.add(*kind);
          (format!("{} is yours", spec.name), style::GRANTED)
        });
      commands.spawn((
        Toast(Timer::from_seconds(TOAST_LIFE, TimerMode::Once)),
        tinted(&announcement, 17.0, tint),
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
      (
        toggle_panels,
        toggle_delete_mode,
        scroll_panels,
        buy_machines,
        select_machines,
        track_ui_hover,
        refresh_panels
      )
        .chain()
        .run_if(playing)
    )
    .add_systems(Update, animate_toasts);
}

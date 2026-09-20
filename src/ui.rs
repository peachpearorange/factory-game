use {crate::{construction::{BuildMode, PlacedMachine, aim_ray, aimed_entity,
                            machine_root},
             icon,
             machine::{Money, OreSold},
             ore::{Effects, Ore, OreLimit},
             player::{MainCamera, Player, UiHover},
             store::HoverInfo,
             style::{self, Bold, heavy, label, tinted}},
     avian3d::prelude::*,
     bevy::{diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
            ecs::entity::EntityHashSet,
            prelude::*}};

const METER_WIDTH: f32 = 208.0;
const TAG_RANGE: f32 = 30.0;
const TAG_LIFT: f32 = 0.62;
const SOLD_LIFE: f32 = 1.1;
const SOLD_RISE: f32 = 78.0;

#[derive(Component)]
struct OreMeterFill;

#[derive(Component)]
struct OreMeterLabel;

#[derive(Component)]
struct MoneyLabel;

#[derive(Component)]
struct FpsLabel;

#[derive(Component)]
struct Tooltip;

#[derive(Component)]
struct TooltipTitle;

#[derive(Component)]
struct TooltipBody;

#[derive(Component)]
struct EffectIcon(Effects);

#[derive(Component)]
struct ModeHint;

#[derive(Component)]
struct ModeHintText;

#[derive(Component)]
struct Hint;

#[derive(Component)]
struct HintTitle;

#[derive(Component)]
struct HintDetail;

fn floating(bottom: f32, width: f32) -> impl Bundle {
  (
    GlobalZIndex(20),
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
      border_radius: BorderRadius::all(px(7)),
      display: Display::None,
      ..default()
    },
    BackgroundColor(style::POPUP)
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
    BackgroundColor(style::POPUP),
    children![
      (label("$0", style::READOUT), MoneyLabel),
      (
        Node {
          width: px(METER_WIDTH),
          height: px(22),
          align_items: AlignItems::Center,
          justify_content: JustifyContent::Center,
          border_radius: BorderRadius::all(px(5)),
          overflow: Overflow::clip(),
          ..default()
        },
        BackgroundColor(style::SLOT),
        children![
          (
            Node {
              position_type: PositionType::Absolute,
              left: px(0),
              top: px(0),
              bottom: px(0),
              width: percent(0),
              ..default()
            },
            BackgroundColor(style::ORE),
            OreMeterFill,
          ),
          (label("Ore  0 / 0", style::SMALL), OreMeterLabel),
        ],
      ),
      (tinted("-- fps", style::TINY, style::TEXT_DIM), FpsLabel),
      label(
        "Right-drag to look    Click a machine to move it    Q to delete",
        style::TINY
      ),
    ]
  ));

  commands.spawn((
    ModeHint,
    GlobalZIndex(20),
    Node {
      position_type: PositionType::Absolute,
      bottom: px(100),
      left: percent(50),
      margin: UiRect::left(px(-170)),
      width: px(340),
      justify_content: JustifyContent::Center,
      padding: UiRect::all(px(9)),
      border_radius: BorderRadius::all(px(8)),
      display: Display::None,
      ..default()
    },
    BackgroundColor(style::POPUP),
    children![(label("", style::SMALL), ModeHintText)]
  ));

  commands.spawn((Tooltip, floating(172.0, 330.0), children![
    (
      Node { align_items: AlignItems::Center, column_gap: px(6), ..default() },
      children![
        (icon::flame(style::FIRE), EffectIcon(Effects::FIERY)),
        (icon::droplet(style::WATER), EffectIcon(Effects::WET)),
        (icon::radiation(style::DECAY), EffectIcon(Effects::RADIOACTIVE)),
        (icon::frost(style::FROST), EffectIcon(Effects::FROSTY)),
        (label("", style::BODY), TooltipTitle),
      ],
    ),
    (label("", style::SMALL), TooltipBody),
  ]));

  commands.spawn((Hint, floating(100.0, 272.0), children![
    (label("", style::BODY), HintTitle),
    (label("", style::SMALL), HintDetail),
  ]));
}

fn update_hud(
  money: Res<Money>,
  limit: Res<OreLimit>,
  ores: Query<(), With<Ore>>,
  mut fill: Single<&mut Node, With<OreMeterFill>>,
  mut meter: Single<&mut Text, (With<OreMeterLabel>, Without<MoneyLabel>)>,
  mut cash: Single<&mut Text, (With<MoneyLabel>, Without<OreMeterLabel>)>
) {
  let live = ores.iter().count();
  fill.width = percent(100.0 * live as f32 / limit.0.max(1) as f32);
  ***meter = (live >= limit.0)
    .then(|| format!("Ore  {live} / {}  — at capacity", limit.0))
    .unwrap_or_else(|| format!("Ore  {live} / {}", limit.0));
  ***cash = format!("${:.0}", money.0);
}

fn update_fps(
  diagnostics: Res<DiagnosticsStore>,
  mut text: Single<&mut Text, With<FpsLabel>>
) {
  ***text = diagnostics
    .get(&FrameTimeDiagnosticsPlugin::FPS)
    .and_then(|fps| fps.smoothed())
    .map(|fps| format!("{fps:.0} fps"))
    .unwrap_or_else(|| "-- fps".to_string());
}

fn show_mode_hint(
  mode: Res<BuildMode>,
  mut hint: Single<&mut Node, With<ModeHint>>,
  mut text: Single<&mut Text, With<ModeHintText>>
) {
  let message = match *mode {
    BuildMode::Idle => None,
    BuildMode::Placing { .. } => Some("R to rotate     Q to cancel"),
    BuildMode::Deleting => Some("Click a machine to store it     Q to stop")
  };
  hint.display = message.map(|_| Display::Flex).unwrap_or(Display::None);
  if let Some(line) = message {
    ***text = line.to_string();
  }
}

fn update_hint(
  hovered: Query<(&Interaction, &HoverInfo)>,
  mut panel: Single<&mut Node, With<Hint>>,
  mut title: Single<&mut Text, (With<HintTitle>, Without<HintDetail>)>,
  mut detail: Single<&mut Text, (With<HintDetail>, Without<HintTitle>)>
) {
  let shown =
    hovered.iter().find(|(state, _)| **state != Interaction::None).map(|(_, info)| info);
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
  eye: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
  window: Single<&Window>,
  player: Single<Entity, With<Player>>,
  parents: Query<&ChildOf>,
  machines: Query<&PlacedMachine>,
  ores: Query<&Ore>,
  mut panel: Single<&mut Node, With<Tooltip>>,
  mut icons: Query<(&EffectIcon, &mut Node), Without<Tooltip>>,
  mut title: Single<&mut Text, (With<TooltipTitle>, Without<TooltipBody>)>,
  mut body: Single<&mut Text, (With<TooltipBody>, Without<TooltipTitle>)>
) {
  let (camera, transform) = *eye;
  let deleting = *mode == BuildMode::Deleting;
  let looked_at = (!hovering.0 && (*mode == BuildMode::Idle || deleting))
    .then(|| aim_ray(camera, transform, *window))
    .flatten()
    .and_then(|ray| aimed_entity(&spatial, ray, *player));

  let described = looked_at.and_then(|entity| {
    ores
      .get(entity)
      .ok()
      .map(|ore| {
        (
          format!("{} {}", ore.effects.label(), ore.form.noun()),
          format!("Worth ${:.0}", ore.value),
          ore.effects
        )
      })
      .or_else(|| {
        machine_root(entity, &parents, &machines)
          .and_then(|root| machines.get(root).ok())
          .map(|machine| {
            let spec = machine.kind.spec();
            (
              spec.name.to_string(),
              format!(
                "{}\n{}",
                spec.blurb,
                deleting
                  .then_some("Click to store it.")
                  .unwrap_or("Click to pick it up.")
              ),
              Effects::NONE
            )
          })
      })
  });

  panel.display = described.as_ref().map(|_| Display::Flex).unwrap_or(Display::None);
  let effects =
    described.as_ref().map(|(_, _, effects)| *effects).unwrap_or(Effects::NONE);
  for (EffectIcon(effect), mut node) in &mut icons {
    node.display =
      effects.contains(*effect).then_some(Display::Flex).unwrap_or(Display::None);
  }
  if let Some((heading, detail, _)) = described {
    ***title = heading;
    ***body = detail;
  }
}

pub fn money(value: f32) -> String {
  const STEPS: [(f32, &str); 5] =
    [(1e12, "T"), (1e9, "B"), (1e6, "M"), (1e3, "k"), (1.0, "")];
  let (scale, suffix) =
    STEPS.into_iter().find(|&(step, _)| value.abs() >= step).unwrap_or((1.0, ""));
  let scaled = value / scale;
  (scaled.abs() < 10.0 && !suffix.is_empty())
    .then(|| format!("${scaled:.1}{suffix}"))
    .unwrap_or_else(|| format!("${scaled:.0}{suffix}"))
}

#[derive(Component)]
struct ValueTag(Entity);

fn pinned(at: Vec2) -> impl Bundle {
  (
    Node {
      position_type: PositionType::Absolute,
      left: px(at.x),
      top: px(at.y),
      padding: UiRect::axes(px(3), px(0)),
      border_radius: BorderRadius::all(px(3)),
      ..default()
    },
    UiTransform { translation: Val2::new(percent(-50), percent(-100)), ..default() },
    BackgroundColor(style::TAG)
  )
}

fn track_value_tags(
  eye: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
  ores: Query<(Entity, &Ore, &GlobalTransform)>,
  mut tags: Query<(Entity, &ValueTag, &mut Node, &mut Text)>,
  mut commands: Commands
) {
  let (camera, view) = *eye;
  let watched = view.translation();
  let above = |transform: &GlobalTransform| {
    (transform.translation().distance(watched) < TAG_RANGE)
      .then(|| {
        camera.world_to_viewport(view, transform.translation() + Vec3::Y * TAG_LIFT)
      })
      .and_then(Result::ok)
  };

  let mut tagged = EntityHashSet::default();
  for (tag, &ValueTag(pinned_to), mut node, mut text) in &mut tags {
    if let Ok((_, ore, transform)) = ores.get(pinned_to)
      && let Some(at) = above(transform)
    {
      node.left = px(at.x);
      node.top = px(at.y);
      **text = money(ore.value);
      tagged.insert(pinned_to);
    } else {
      commands.entity(tag).despawn();
    }
  }

  for (entity, ore, transform) in &ores {
    if !tagged.contains(&entity)
      && let Some(at) = above(transform)
    {
      commands.spawn((
        ValueTag(entity),
        pinned(at),
        style::snug(&money(ore.value), style::SMALL)
      ));
    }
  }
}

#[derive(Component)]
struct SoldTag {
  at: Vec3,
  timer: Timer
}

fn pop_sold_ores(
  mut sold: MessageReader<OreSold>,
  bold: Res<Bold>,
  mut commands: Commands
) {
  for &OreSold { at, value } in sold.read() {
    commands.spawn((
      SoldTag { at, timer: Timer::from_seconds(SOLD_LIFE, TimerMode::Once) },
      Node { position_type: PositionType::Absolute, ..default() },
      heavy(&money(value), 21.0, style::CASH, &bold)
    ));
  }
}

fn animate_sold_tags(
  time: Res<Time>,
  eye: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
  mut tags: Query<(Entity, &mut SoldTag, &mut Node, &mut UiTransform, &mut TextColor)>,
  mut commands: Commands
) {
  let (camera, view) = *eye;
  for (entity, mut tag, mut node, mut transform, mut color) in &mut tags {
    tag.timer.tick(time.delta());
    let age = tag.timer.fraction();
    let spot = camera.world_to_viewport(view, tag.at).ok();
    node.display = spot.map(|_| Display::Flex).unwrap_or(Display::None);
    if let Some(at) = spot {
      node.left = px(at.x);
      node.top = px(at.y);
    }
    transform.translation = Val2::new(percent(-50), px(-SOLD_RISE * age));
    transform.scale = Vec2::splat(1.0 + 0.5 * (1.0 - age).powi(7));
    color.0 = style::CASH.with_alpha((2.6 - 2.6 * age).min(1.0));
    if tag.timer.is_finished() {
      commands.entity(entity).despawn();
    }
  }
}

pub fn plugin(app: &mut App) {
  app
    .add_plugins(FrameTimeDiagnosticsPlugin::default())
    .add_systems(Startup, spawn_hud)
    .add_systems(
      Update,
      (
        update_hud,
        update_fps,
        show_mode_hint,
        update_hint,
        update_tooltip,
        track_value_tags,
        pop_sold_ores,
        animate_sold_tags
      )
    );
}

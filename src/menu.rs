use {crate::{player::UiHover,
             store::Panels,
             style::{self, label, tinted}},
     bevy::{app::AppExit,
            prelude::*,
            render::view::screenshot::{Screenshot, save_to_disk},
            window::{CursorGrabMode, CursorOptions, MonitorSelection, WindowMode}},
     std::{fs,
           time::{SystemTime, UNIX_EPOCH}}};

const SHOT_DIR: &str = "screenshots";
const SHOT_SETTLE: f32 = 1.5;
const FULLSCREEN: WindowMode =
  WindowMode::BorderlessFullscreen(MonitorSelection::Current);

#[derive(Resource, Default)]
pub struct Paused(pub bool);

pub fn playing(paused: Res<Paused>) -> bool { !paused.0 }

#[derive(Component)]
pub struct ScreenshotButton;

#[derive(Component)]
struct MenuPanel;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
  Resume,
  Fullscreen,
  Screenshot,
  Quit
}

fn menu_button(action: MenuAction, caption: &str) -> impl Bundle {
  (
    action,
    Button,
    Node {
      width: percent(100),
      padding: UiRect::all(px(11)),
      justify_content: JustifyContent::Center,
      border_radius: BorderRadius::all(px(9)),
      ..default()
    },
    BackgroundColor(style::SLOT),
    children![label(caption, 16.0)]
  )
}

fn spawn_menu(mut commands: Commands) {
  commands.spawn((
        MenuPanel,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::None,
            ..default()
        },
        BackgroundColor(style::SHADE),
        children![(
            Node {
                width: px(364),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(9),
                padding: UiRect::all(px(24)),
                border_radius: BorderRadius::all(px(14)),
                ..default()
            },
            BackgroundColor(style::PANEL),
            children![
                label("Paused", 30.0),
                (
                    Node {
                        margin: UiRect::bottom(px(8)),
                        ..default()
                    },
                    tinted("The belts wait for you.", style::SMALL, style::TEXT_DIM),
                ),
                menu_button(MenuAction::Resume, "Back to the island"),
                menu_button(MenuAction::Fullscreen, "Toggle fullscreen"),
                menu_button(MenuAction::Screenshot, "Take a screenshot"),
                menu_button(MenuAction::Quit, "Quit to desktop"),
                (
                    Node {
                        margin: UiRect::top(px(10)),
                        ..default()
                    },
                    tinted(
                        "WASD move    Space jump    Right-drag look\nE items    F store    Click a machine to move it    X deletes\nR rotates    Q cancels    F2 screenshot",
                        11.0,
                        style::TEXT_DIM,
                    ),
                    TextLayout {
                        justify: Justify::Center,
                        ..default()
                    },
                ),
            ],
        )],
    ));
}

fn auto_shoot(
  time: Res<Time>,
  mut taken: Local<bool>,
  mut commands: Commands,
  mut quit: MessageWriter<AppExit>
) {
  if let Some(due) = crate::env_secs("FACTORY_SHOT") {
    if !*taken && time.elapsed_secs() > due {
      *taken = true;
      shoot(&mut commands);
    } else if *taken && time.elapsed_secs() > due + SHOT_SETTLE {
      quit.write(AppExit::Success);
    }
  }
}

pub fn capture(commands: &mut Commands, name: &str) {
  let path = format!("{SHOT_DIR}/{name}.png");
  if let Some(folder) = std::path::Path::new(&path).parent() {
    fs::create_dir_all(folder).ok();
  }
  commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
}

fn shoot(commands: &mut Commands) {
  let stamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|since| since.as_millis())
    .unwrap_or_default();
  capture(commands, &format!("factory-{stamp}"));
}

fn steer_menu(
  keys: Res<ButtonInput<KeyCode>>,
  mouse: Res<ButtonInput<MouseButton>>,
  hovering: Res<UiHover>,
  buttons: Query<(&MenuAction, &Interaction), Changed<Interaction>>,
  shutter: Query<&Interaction, (With<ScreenshotButton>, Changed<Interaction>)>,
  mut window: Single<&mut Window>,
  mut cursor: Single<&mut CursorOptions>,
  mut panel: Single<&mut Node, With<MenuPanel>>,
  mut paused: ResMut<Paused>,
  mut panels: ResMut<Panels>,
  mut clock: ResMut<Time<Virtual>>,
  mut quit: MessageWriter<AppExit>,
  mut commands: Commands
) {
  let chosen = buttons
    .iter()
    .find(|(_, state)| **state == Interaction::Pressed)
    .map(|(action, _)| *action);

  if keys.just_pressed(KeyCode::Escape) || chosen == Some(MenuAction::Resume) {
    paused.0 = !paused.0;
    panels.store = false;
    panels.inventory = false;
  }
  let snapped = shutter.iter().any(|state| *state == Interaction::Pressed);
  if keys.just_pressed(KeyCode::F2) || snapped || chosen == Some(MenuAction::Screenshot) {
    shoot(&mut commands);
  }
  let windowed = window.mode == WindowMode::Windowed;
  if chosen == Some(MenuAction::Fullscreen) {
    window.mode = windowed.then_some(FULLSCREEN).unwrap_or(WindowMode::Windowed);
  } else if windowed
    && chosen.is_none()
    && !paused.0
    && !hovering.0
    && mouse.just_pressed(MouseButton::Left)
  {
    window.mode = FULLSCREEN;
  }
  if chosen == Some(MenuAction::Quit) {
    quit.write(AppExit::Success);
  }

  if paused.is_changed() {
    panel.display = paused.0.then_some(Display::Flex).unwrap_or(Display::None);
    paused.0.then(|| clock.pause()).unwrap_or_else(|| clock.unpause());
    if paused.0 {
      cursor.visible = true;
      cursor.grab_mode = CursorGrabMode::None;
    }
  }
}

pub fn plugin(app: &mut App) {
  app
    .init_resource::<Paused>()
    .add_systems(Startup, spawn_menu)
    .add_systems(Update, (steer_menu, auto_shoot));
}

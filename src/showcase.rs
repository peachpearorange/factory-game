use {crate::{catalog::{CELL, MachineAssets, MachineKind, place},
             menu::capture,
             player::MainCamera,
             world::DayClock},
     bevy::{app::AppExit, prelude::*}};

const ANGLES: [f32; 4] = [35.0, 125.0, 215.0, 305.0];
const PHASES: [(f32, &str); 3] = [(0.06, "dawn"), (0.25, "noon"), (0.78, "night")];
const SETTLE: f32 = 0.5;
const DRAIN: usize = 3;
const ORBIT: f32 = 6.4;
const RISE: f32 = 3.2;
const FOCUS: Vec3 = Vec3::new(0.0, 1.05, 0.0);

#[derive(Resource)]
pub struct Showcase {
  kind: MachineKind,
  shot: usize,
  timer: Timer
}

impl Showcase {
  const COUNT: usize = ANGLES.len() * PHASES.len();

  fn framing(&self) -> (f32, (f32, &'static str)) {
    let shot = self.shot.min(Self::COUNT - 1);
    (ANGLES[shot % ANGLES.len()], PHASES[shot / ANGLES.len()])
  }
}

pub fn idle(showcase: Option<Res<Showcase>>) -> bool { showcase.is_none() }

fn slug(text: &str) -> String {
  text.chars().filter(char::is_ascii_alphanumeric).collect::<String>().to_lowercase()
}

fn wanted() -> Option<MachineKind> {
  std::env::var("FACTORY_SHOWCASE").ok().and_then(|asked| {
    MachineKind::ALL
      .into_iter()
      .find(|kind| slug(kind.spec().name).contains(&slug(&asked)))
  })
}

fn stage_showcase(assets: Res<MachineAssets>, mut commands: Commands) {
  if let Some(kind) = wanted() {
    let at = |cell: f32| Transform::from_xyz(cell * CELL, 0.0, 0.0);
    place(&mut commands, &assets, kind, at(0.0));
    if kind.carries_belt() {
      place(&mut commands, &assets, MachineKind::Conveyor, at(-1.0));
      place(&mut commands, &assets, MachineKind::Conveyor, at(1.0));
    }
    commands.insert_resource(Showcase {
      kind,
      shot: 0,
      timer: Timer::from_seconds(SETTLE, TimerMode::Repeating)
    });
  }
}

fn hide_hud(
  showcase: Option<Res<Showcase>>,
  mut roots: Query<&mut Node, Without<ChildOf>>
) {
  if showcase.is_some() {
    for mut node in &mut roots {
      node.display = Display::None;
    }
  }
}

fn frame_showcase(
  time: Res<Time>,
  mut showcase: ResMut<Showcase>,
  mut clock: ResMut<DayClock>,
  mut camera: Single<&mut Transform, With<MainCamera>>,
  mut quit: MessageWriter<AppExit>,
  mut commands: Commands
) {
  let (angle, (phase, moment)) = showcase.framing();
  clock.pin(phase, time.elapsed_secs());
  let yaw = angle.to_radians();
  **camera = Transform::from_translation(
    FOCUS + Vec3::new(yaw.sin(), 0.0, yaw.cos()) * ORBIT + Vec3::Y * RISE
  )
  .looking_at(FOCUS, Vec3::Y);

  if showcase.timer.tick(time.delta()).just_finished() {
    if showcase.shot < Showcase::COUNT {
      capture(
        &mut commands,
        &format!("showcase/{}-{moment}-{angle:.0}deg", slug(showcase.kind.spec().name))
      );
    } else if showcase.shot >= Showcase::COUNT + DRAIN {
      quit.write(AppExit::Success);
    }
    showcase.shot += 1;
  }
}

pub fn plugin(app: &mut App) {
  app
    .add_systems(Startup, stage_showcase)
    .add_systems(PostStartup, hide_hud)
    .add_systems(Update, frame_showcase.run_if(resource_exists::<Showcase>));
}

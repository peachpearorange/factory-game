use bevy::{prelude::*, ui::UiTransform};

pub const SIZE: f32 = 20.0;

fn part(left: f32, top: f32, width: f32, height: f32, radius: BorderRadius) -> Node {
  Node {
    position_type: PositionType::Absolute,
    left: px(left),
    top: px(top),
    width: px(width),
    height: px(height),
    border_radius: radius,
    ..default()
  }
}

fn fill(
  left: f32,
  top: f32,
  width: f32,
  height: f32,
  round: f32,
  color: Color
) -> impl Bundle {
  (part(left, top, width, height, BorderRadius::all(px(round))), BackgroundColor(color))
}

fn frame(
  left: f32,
  top: f32,
  width: f32,
  height: f32,
  thickness: f32,
  round: f32,
  color: Color
) -> impl Bundle {
  (
    Node {
      border: UiRect::all(px(thickness)),
      ..part(left, top, width, height, BorderRadius::all(px(round)))
    },
    BorderColor::all(color)
  )
}

fn ring(
  left: f32,
  top: f32,
  size: f32,
  thickness: f32,
  round: f32,
  color: Color
) -> impl Bundle {
  frame(left, top, size, size, thickness, round, color)
}

fn turned(bundle: impl Bundle, degrees: f32) -> impl Bundle {
  (bundle, UiTransform::from_rotation(Rot2::degrees(degrees)))
}

fn glyph(children: impl Bundle) -> impl Bundle {
  (Node { width: px(SIZE), height: px(SIZE), flex_shrink: 0.0, ..default() }, children)
}

pub fn scaled(glyph: impl Bundle, factor: f32) -> impl Bundle {
  (
    Node {
      width: px(SIZE * factor),
      height: px(SIZE * factor),
      flex_shrink: 0.0,
      align_items: AlignItems::Center,
      justify_content: JustifyContent::Center,
      ..default()
    },
    children![(glyph, UiTransform::from_scale(Vec2::splat(factor)))]
  )
}

pub fn camera(color: Color) -> impl Bundle {
  glyph(children![
    fill(11.0, 1.5, 5.5, 3.2, 1.0, color),
    frame(0.5, 4.5, 19.0, 14.0, 1.8, 3.0, color),
    fill(6.5, 8.0, 7.0, 7.0, 3.5, color),
  ])
}

pub fn store(color: Color) -> impl Bundle {
  glyph(children![
    fill(1.0, 2.5, 18.0, 4.0, 1.4, color),
    frame(3.0, 7.0, 14.0, 11.0, 1.7, 2.0, color),
    fill(7.6, 12.0, 4.8, 6.0, 1.0, color),
  ])
}

pub fn inventory(color: Color) -> impl Bundle {
  glyph(children![
    fill(2.0, 2.0, 7.0, 7.0, 2.0, color),
    fill(11.0, 2.0, 7.0, 7.0, 2.0, color),
    fill(2.0, 11.0, 7.0, 7.0, 2.0, color),
    fill(11.0, 11.0, 7.0, 7.0, 2.0, color),
  ])
}

pub fn lock(color: Color) -> impl Bundle {
  glyph(children![
    ring(6.0, 2.0, 8.0, 1.8, 4.0, color),
    fill(3.5, 8.5, 13.0, 9.5, 2.5, color),
  ])
}

pub fn flame(color: Color) -> impl Bundle {
  glyph(children![turned(
    (
      part(5.0, 5.0, 10.0, 10.0, BorderRadius::new(px(1.5), px(9.0), px(9.0), px(9.0)),),
      BackgroundColor(color),
    ),
    45.0,
  )])
}

pub fn droplet(color: Color) -> impl Bundle {
  glyph(children![turned(
    (
      part(5.5, 5.5, 9.0, 9.0, BorderRadius::new(px(1.2), px(8.0), px(8.0), px(8.0)),),
      BackgroundColor(color),
    ),
    -45.0,
  )])
}

pub fn radiation(color: Color) -> impl Bundle {
  glyph(children![
    turned(fill(8.6, 1.5, 2.8, 8.0, 1.4, color), 0.0),
    turned(fill(8.6, 10.5, 2.8, 8.0, 1.4, color), 60.0),
    turned(fill(8.6, 10.5, 2.8, 8.0, 1.4, color), -60.0),
    fill(7.5, 7.5, 5.0, 5.0, 2.5, color),
  ])
}

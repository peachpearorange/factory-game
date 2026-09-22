#![allow(dead_code)]

use {crate::{material::{Coat, Coats, Finish},
             sdf},
     avian3d::prelude::*,
     bevy::{asset::RenderAssetUsages,
            ecs::spawn::SpawnIter,
            math::Affine3A,
            mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
            prelude::*},
     enum_assoc::Assoc,
     std::f32::consts::TAU};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Axis {
  X,
  Y,
  Z
}

impl Axis {
  fn mirror(self) -> Affine3A {
    Affine3A::from_scale(match self {
      Self::X => Vec3::new(-1.0, 1.0, 1.0),
      Self::Y => Vec3::new(1.0, -1.0, 1.0),
      Self::Z => Vec3::new(1.0, 1.0, -1.0)
    })
  }
}

fn wedge_corners(size: Vec3) -> [Vec3; 6] {
  let Vec3 { x, y, z } = size / 2.0;
  [
    Vec3::new(-x, -y, -z),
    Vec3::new(x, -y, -z),
    Vec3::new(x, y, -z),
    Vec3::new(-x, -y, z),
    Vec3::new(x, -y, z),
    Vec3::new(x, y, z)
  ]
}

fn wedge_mesh(size: Vec3) -> Mesh {
  let [back_low, front_low, front_high, near_low, near_front, near_high] =
    wedge_corners(size);
  let slope = Vec3::new(-size.y, size.x, 0.0).normalize();
  let faces = [
    (vec![back_low, front_low, near_front, near_low], Vec3::NEG_Y),
    (vec![front_low, front_high, near_high, near_front], Vec3::X),
    (vec![back_low, near_low, near_high, front_high], slope),
    (vec![near_low, near_front, near_high], Vec3::Z),
    (vec![back_low, front_high, front_low], Vec3::NEG_Z)
  ];
  let mut positions = Vec::new();
  let mut normals = Vec::new();
  let mut indices = Vec::new();
  for (corners, normal) in faces {
    let base = positions.len() as u32;
    for corner in 2..corners.len() as u32 {
      indices.extend([base, base + corner - 1, base + corner]);
    }
    normals.extend(std::iter::repeat_n(normal, corners.len()));
    positions.extend(corners);
  }
  Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

#[derive(Clone, Copy, PartialEq, Eq, Assoc)]
#[func(fn mesh(self, size: Vec3) -> Mesh)]
#[func(fn collider(self, size: Vec3) -> Collider)]
pub enum Form {
  #[assoc(
    mesh = Cuboid::from_size(size.abs()).mesh().build(),
    collider = Collider::cuboid(size.x.abs(), size.y.abs(), size.z.abs())
  )]
  Slab,
  #[assoc(
    mesh = Cylinder::new(size.x.abs() / 2.0, size.y.abs()).mesh().build(),
    collider = Collider::cylinder(size.x.abs() / 2.0, size.y.abs())
  )]
  Rod,
  #[assoc(
    mesh = Sphere::new(size.x.abs() / 2.0).mesh().ico(3).expect("ball mesh"),
    collider = Collider::sphere(size.x.abs() / 2.0)
  )]
  Ball,
  #[assoc(
    mesh = wedge_mesh(size),
    collider = Collider::convex_hull(wedge_corners(size).into()).expect("wedge hull")
  )]
  Wedge
}

#[derive(Clone, Copy)]
pub struct Part {
  form: Form,
  size: Vec3,
  at: Vec3,
  spin: Quat,
  frame: Affine3A,
  finish: Finish,
  solid: bool
}

impl Finish {
  pub const fn slab(self, size: Vec3) -> Part { Part::new(Form::Slab, size, self) }

  pub const fn cube(self, size: f32) -> Part { self.slab(Vec3::splat(size)) }

  pub const fn beam(self, long: f32, thick: f32) -> Part {
    self.slab(Vec3::new(thick, long, thick))
  }

  pub const fn rod(self, across: f32, long: f32) -> Part {
    Part::new(Form::Rod, Vec3::new(across, long, across), self)
  }

  pub const fn ball(self, across: f32) -> Part {
    Part::new(Form::Ball, Vec3::splat(across), self)
  }

  pub const fn wedge(self, size: Vec3) -> Part { Part::new(Form::Wedge, size, self) }
}

impl Part {
  const fn new(form: Form, size: Vec3, finish: Finish) -> Self {
    Self {
      form,
      size,
      at: Vec3::ZERO,
      spin: Quat::IDENTITY,
      frame: Affine3A::IDENTITY,
      finish,
      solid: false
    }
  }

  pub const fn at(self, at: Vec3) -> Self { Self { at, ..self } }

  pub fn on(self, floor: Vec3) -> Self { self.at(floor + Vec3::Y * self.size.y / 2.0) }

  pub fn under(self, ceiling: Vec3) -> Self {
    self.at(ceiling - Vec3::Y * self.size.y / 2.0)
  }

  pub fn span(self, from: Vec3, to: Vec3) -> Self {
    let reach = to - from;
    Self {
      size: self.size.with_y(reach.length()),
      spin: Quat::from_rotation_arc(Vec3::Y, reach.try_normalize().unwrap_or(Vec3::Y))
        * self.spin,
      ..self.at((from + to) / 2.0)
    }
  }

  pub fn spun(self, spin: Quat) -> Self { Self { spin: spin * self.spin, ..self } }

  pub fn tilted(self, angle: f32) -> Self { self.spun(Quat::from_rotation_z(angle)) }

  pub fn rolled(self, angle: f32) -> Self { self.spun(Quat::from_rotation_x(angle)) }

  pub fn turned(self, angle: f32) -> Self { self.spun(Quat::from_rotation_y(angle)) }

  pub const fn solid(self) -> Self { Self { solid: true, ..self } }

  pub const fn finished(self, finish: Finish) -> Self { Self { finish, ..self } }

  pub const fn tinted(self, color: LinearRgba) -> Self {
    self.finished(self.finish.tinted(color))
  }

  pub const fn shaded(self, amount: f32) -> Self {
    self.finished(self.finish.shaded(amount))
  }

  pub const fn lit(self, glow: LinearRgba) -> Self {
    self.finished(self.finish.lit(glow))
  }

  fn framed(self, by: Affine3A) -> Self { Self { frame: by * self.frame, ..self } }

  fn placing(&self) -> Affine3A {
    self.frame * Affine3A::from_rotation_translation(self.spin, self.at)
  }

  fn mesh(&self) -> Mesh {
    let placing = self.placing();
    let mut mesh = self.form.mesh(self.size);
    let normals: Vec<Vec3> = mesh
      .attribute(Mesh::ATTRIBUTE_NORMAL)
      .and_then(VertexAttributeValues::as_float3)
      .expect("part mesh has normals")
      .iter()
      .map(|&normal| (placing.matrix3 * Vec3::from(normal)).normalize())
      .collect();
    let positions: Vec<Vec3> = sdf::points(&mesh)
      .into_iter()
      .map(|corner| placing.transform_point3(corner))
      .collect();
    if placing.matrix3.determinant() < 0.0
      && let Some(Indices::U32(indices)) = mesh.indices_mut()
    {
      for triangle in indices.chunks_exact_mut(3) {
        triangle.swap(0, 2);
      }
    }
    mesh.insert_attribute(
      Mesh::ATTRIBUTE_UV_0,
      positions
        .iter()
        .zip(&normals)
        .map(|(&position, &normal)| sdf::box_uv(position, normal))
        .collect::<Vec<_>>()
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![
      self.finish.color.to_f32_array();
      positions.len()
    ]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(
      Mesh::ATTRIBUTE_NORMAL,
      normals.iter().map(Vec3::to_array).collect::<Vec<_>>()
    );
    mesh
  }

  fn collider(&self) -> (Collider, Transform) {
    let (scale, rotation, translation) = self.placing().to_scale_rotation_translation();
    (
      self.form.collider(self.size * scale),
      Transform::from_translation(translation).with_rotation(rotation)
    )
  }
}

pub struct Assembly(Vec<Part>);

pub fn group(parts: impl IntoIterator<Item = Part>) -> Assembly {
  Assembly(parts.into_iter().collect())
}

impl Assembly {
  fn framed(self, by: Affine3A) -> Self {
    Self(self.0.into_iter().map(|part| part.framed(by)).collect())
  }

  fn copied(self, frames: impl IntoIterator<Item = Affine3A>) -> Self {
    Self(
      frames
        .into_iter()
        .flat_map(|frame| {
          self.0.iter().map(move |&part| part.framed(frame)).collect::<Vec<_>>()
        })
        .collect()
    )
  }

  pub fn at(self, offset: Vec3) -> Self {
    self.framed(Affine3A::from_translation(offset))
  }

  pub fn spun(self, spin: Quat) -> Self { self.framed(Affine3A::from_quat(spin)) }

  pub fn tilted(self, angle: f32) -> Self { self.spun(Quat::from_rotation_z(angle)) }

  pub fn rolled(self, angle: f32) -> Self { self.spun(Quat::from_rotation_x(angle)) }

  pub fn turned(self, angle: f32) -> Self { self.spun(Quat::from_rotation_y(angle)) }

  pub fn mirrored(self, axis: Axis) -> Self {
    self.copied([Affine3A::IDENTITY, axis.mirror()])
  }

  pub fn repeated(self, times: u32, step: Vec3) -> Self {
    self.copied(
      (0..times).map(|step_over| Affine3A::from_translation(step * step_over as f32))
    )
  }

  pub fn ringed(self, times: u32, radius: f32) -> Self {
    self.copied((0..times).map(|spoke| {
      Affine3A::from_rotation_y(spoke as f32 * TAU / times as f32)
        * Affine3A::from_translation(Vec3::X * radius)
    }))
  }

  pub fn solid(self) -> Self { Self(self.0.into_iter().map(Part::solid).collect()) }

  pub fn finished(self, finish: Finish) -> Self {
    Self(self.0.into_iter().map(|part| part.finished(finish)).collect())
  }
}

impl IntoIterator for Assembly {
  type IntoIter = std::vec::IntoIter<Part>;
  type Item = Part;

  fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl FromIterator<Part> for Assembly {
  fn from_iter<I: IntoIterator<Item = Part>>(parts: I) -> Self { group(parts) }
}

impl FromIterator<Assembly> for Assembly {
  fn from_iter<I: IntoIterator<Item = Assembly>>(groups: I) -> Self {
    group(groups.into_iter().flatten())
  }
}

pub fn smooth(mut mesh: Mesh) -> Mesh {
  const MID_BOARD: [f32; 2] = [0.5, 0.12];
  mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![MID_BOARD; sdf::points(&mesh).len()]);
  mesh
}

pub fn painted(mut mesh: Mesh, color: LinearRgba) -> Mesh {
  mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![
    color.to_f32_array();
    sdf::points(&mesh).len()
  ]);
  mesh
}

pub fn merged(mut whole: Mesh, part: Mesh) -> Mesh {
  whole.merge(&part).expect("meshes merge into one mesh");
  whole
}

pub fn colliders(
  parts: impl IntoIterator<Item = Part>
) -> impl Iterator<Item = (Collider, Transform)> {
  parts.into_iter().filter(|part| part.solid).map(|part| part.collider())
}

pub fn coated(mut coats: Vec<(Coat, Mesh)>, coat: Coat, mesh: Mesh) -> Vec<(Coat, Mesh)> {
  if let Some(found) = coats.iter().position(|(each, _)| *each == coat) {
    coats[found].1.merge(&mesh).expect("parts of a coat merge")
  } else {
    coats.push((coat, mesh))
  }
  coats
}

pub fn assembled(parts: impl IntoIterator<Item = Part>) -> Vec<(Coat, Mesh)> {
  parts
    .into_iter()
    .fold(Vec::new(), |coats, part| coated(coats, part.finish.coat(), part.mesh()))
}

pub fn spawned(
  model: Vec<(Coat, Mesh)>,
  meshes: &mut Assets<Mesh>,
  coats: &mut Coats,
  materials: &mut Assets<StandardMaterial>
) -> impl Bundle {
  Children::spawn(SpawnIter(
    model
      .into_iter()
      .map(|(coat, mesh)| {
        (Mesh3d(meshes.add(mesh)), MeshMaterial3d(coats.of(coat, materials)))
      })
      .collect::<Vec<_>>()
      .into_iter()
  ))
}

pub fn whole(coats: &[(Coat, Mesh)]) -> Mesh {
  coats
    .iter()
    .map(|(_, mesh)| mesh.clone())
    .reduce(merged)
    .expect("a model is made of at least one part")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn wedge_faces_point_outwards() {
    let size = Vec3::new(1.4, 0.8, 2.0);
    let mesh = wedge_mesh(size);
    let points = sdf::points(&mesh);
    let normals: Vec<Vec3> = mesh
      .attribute(Mesh::ATTRIBUTE_NORMAL)
      .and_then(VertexAttributeValues::as_float3)
      .expect("wedge normals")
      .iter()
      .map(|&normal| Vec3::from(normal))
      .collect();
    let Some(Indices::U32(indices)) = mesh.indices() else { panic!("wedge indices") };
    let inside = wedge_corners(size).iter().sum::<Vec3>() / 6.0;
    for corners in indices.chunks_exact(3) {
      let [a, b, c] = [0, 1, 2].map(|corner| points[corners[corner] as usize]);
      let facing = (b - a).cross(c - a).normalize();
      assert!(facing.dot(normals[corners[0] as usize]) > 0.99);
      assert!(facing.dot(a - inside) > 0.0);
    }
  }

  #[test]
  fn mirroring_reflects_positions() {
    let leaning = crate::material::STEEL.beam(1.0, 0.2).at(Vec3::X).tilted(0.4);
    let pair: Vec<Part> =
      group([leaning]).mirrored(Axis::Z).at(Vec3::Z * 2.0).into_iter().collect();
    let [near, far] = [0, 1].map(|copy| pair[copy].placing().transform_point3(Vec3::Y));
    assert!((near.x - far.x).abs() < 1e-5);
    assert!((near.z - 2.0 - (2.0 - far.z)).abs() < 1e-5);
  }
}

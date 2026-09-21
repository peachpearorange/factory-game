use {crate::sdf,
     avian3d::prelude::*,
     bevy::{mesh::VertexAttributeValues, prelude::*},
     enum_assoc::Assoc};

#[derive(Clone, Copy, PartialEq, Eq, Assoc)]
#[func(fn mesh(self, half: Vec3) -> Mesh)]
#[func(fn collider(self, half: Vec3) -> Collider)]
pub enum Form {
  #[assoc(
    mesh = Cuboid::from_size(half * 2.0).mesh().build(),
    collider = Collider::cuboid(half.x * 2.0, half.y * 2.0, half.z * 2.0)
  )]
  Slab,
  #[assoc(
    mesh = Cylinder::new(half.x, half.y * 2.0).mesh().build(),
    collider = Collider::cylinder(half.x, half.y * 2.0)
  )]
  Pillar,
  #[assoc(
    mesh = Sphere::new(half.x).mesh().ico(3).expect("ball mesh"),
    collider = Collider::sphere(half.x)
  )]
  Ball
}

pub struct Block {
  form: Form,
  half: Vec3,
  at: Vec3,
  spin: Quat,
  color: LinearRgba,
  solid: bool
}

impl Block {
  pub const fn new(half: Vec3, at: Vec3, color: LinearRgba) -> Self {
    Self { form: Form::Slab, half, at, spin: Quat::IDENTITY, color, solid: false }
  }

  pub const fn shaped(self, form: Form) -> Self { Self { form, ..self } }

  pub fn tilted(self, tilt: f32) -> Self { self.spun(Quat::from_rotation_z(tilt)) }

  pub fn rolled(self, roll: f32) -> Self { self.spun(Quat::from_rotation_x(roll)) }

  pub const fn spun(self, spin: Quat) -> Self { Self { spin, ..self } }

  pub const fn solid(self) -> Self { Self { solid: true, ..self } }

  fn placing(&self) -> Transform {
    Transform::from_translation(self.at).with_rotation(self.spin)
  }

  fn mesh(&self) -> Mesh {
    let mut mesh = self.form.mesh(self.half).transformed_by(self.placing());
    let corners: Vec<(Vec3, Vec3)> = sdf::points(&mesh)
      .into_iter()
      .zip(
        mesh
          .attribute(Mesh::ATTRIBUTE_NORMAL)
          .and_then(VertexAttributeValues::as_float3)
          .expect("block mesh has normals")
          .iter()
          .copied()
          .map(Vec3::from)
      )
      .collect();
    mesh.insert_attribute(
      Mesh::ATTRIBUTE_UV_0,
      corners
        .iter()
        .map(|&(position, normal)| sdf::box_uv(position, normal))
        .collect::<Vec<_>>()
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![
      self.color.to_f32_array();
      corners.len()
    ]);
    mesh
  }
}

pub fn smooth(mut mesh: Mesh) -> Mesh {
  const MID_BOARD: [f32; 2] = [0.5, 0.12];
  mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![MID_BOARD; sdf::points(&mesh).len()]);
  mesh
}

pub fn merged(mut whole: Mesh, part: Mesh) -> Mesh {
  whole.merge(&part).expect("meshes merge into one machine mesh");
  whole
}

pub fn colliders(
  blocks: impl IntoIterator<Item = Block>
) -> impl Iterator<Item = (Collider, Transform)> {
  blocks
    .into_iter()
    .filter(|block| block.solid)
    .map(|block| (block.form.collider(block.half), block.placing()))
}

pub fn assembled(blocks: impl IntoIterator<Item = Block>) -> Mesh {
  blocks
    .into_iter()
    .map(|block| block.mesh())
    .reduce(merged)
    .expect("a machine is made of at least one block")
}

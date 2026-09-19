use {crate::sdf,
     bevy::{mesh::VertexAttributeValues, prelude::*}};

pub struct Block {
  half: Vec3,
  at: Vec3,
  tilt: f32,
  color: LinearRgba
}

impl Block {
  pub const fn new(half: Vec3, at: Vec3, color: LinearRgba) -> Self {
    Self { half, at, tilt: 0.0, color }
  }

  pub const fn tilted(self, tilt: f32) -> Self { Self { tilt, ..self } }

  fn mesh(&self) -> Mesh {
    let mut mesh = Cuboid::from_size(self.half * 2.0).mesh().build().transformed_by(
      Transform::from_translation(self.at)
        .with_rotation(Quat::from_rotation_z(self.tilt))
    );
    let corners: Vec<(Vec3, Vec3)> = sdf::points(&mesh)
      .into_iter()
      .zip(
        mesh
          .attribute(Mesh::ATTRIBUTE_NORMAL)
          .and_then(VertexAttributeValues::as_float3)
          .expect("cuboid mesh has normals")
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

pub fn assembled(blocks: impl IntoIterator<Item = Block>, carved: Mesh) -> Mesh {
  blocks.into_iter().fold(carved, |mut whole, block| {
    whole.merge(&block.mesh()).expect("blocks merge into the machine mesh");
    whole
  })
}

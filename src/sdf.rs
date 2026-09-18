use {bevy::{asset::RenderAssetUsages,
            mesh::{PrimitiveTopology, VertexAttributeValues},
            prelude::*},
     fidget::{context::Tree,
              mesh::{Octree, Settings},
              shape::BoundShape,
              vm::{VmFunction, VmShape}},
     nalgebra::{Matrix4, Vector3}};

pub fn sphere(radius: f32) -> Tree {
  let (x, y, z) = Tree::axes();
  (x.square() + y.square() + z.square()).sqrt() - f64::from(radius)
}

pub fn rounded_box(half: Vec3, radius: f32) -> Tree {
  let (x, y, z) = Tree::axes();
  let r = f64::from(radius);
  let q = [
    x.abs() - (f64::from(half.x) - r),
    y.abs() - (f64::from(half.y) - r),
    z.abs() - (f64::from(half.z) - r)
  ];
  let outside =
    (q[0].max(0.0).square() + q[1].max(0.0).square() + q[2].max(0.0).square()).sqrt();
  outside + q[0].max(q[1].clone()).max(q[2].clone()).min(0.0) - r
}

pub fn cuboid(half: Vec3) -> Tree { rounded_box(half, 0.0) }

pub fn cylinder(radius: f32, half_height: f32) -> Tree {
  let (x, y, z) = Tree::axes();
  let radial = (x.square() + z.square()).sqrt() - f64::from(radius);
  let axial = y.abs() - f64::from(half_height);
  let outside = (radial.max(0.0).square() + axial.max(0.0).square()).sqrt();
  outside + radial.max(axial).min(0.0)
}

pub fn along_z(shape: Tree) -> Tree {
  let (x, y, z) = Tree::axes();
  shape.remap_xyz(x, z, y)
}

pub fn at(shape: Tree, offset: Vec3) -> Tree {
  let (x, y, z) = Tree::axes();
  shape.remap_xyz(
    x - f64::from(offset.x),
    y - f64::from(offset.y),
    z - f64::from(offset.z)
  )
}

pub fn union(shapes: impl IntoIterator<Item = Tree>) -> Tree {
  shapes.into_iter().reduce(|a, b| a.min(b)).expect("union of no shapes")
}

pub fn difference(shape: Tree, cutout: Tree) -> Tree { shape.max(-cutout) }

pub fn smooth_union(a: Tree, b: Tree, radius: f32) -> Tree {
  let r = f64::from(radius);
  a.min(b.clone()) - 1.0 / (4.0 * r) * (r - (a - b).abs()).max(0.0).square()
}

pub struct Bounds {
  pub center: Vec3,
  pub half_extent: f32,
  pub depth: u8
}

impl Bounds {
  pub const fn new(half_extent: f32, depth: u8) -> Self {
    Self::around(Vec3::ZERO, half_extent, depth)
  }

  pub const fn around(center: Vec3, half_extent: f32, depth: u8) -> Self {
    Self { center, half_extent, depth }
  }

  fn world_to_model(&self) -> Matrix4<f32> {
    Matrix4::new_translation(&Vector3::new(self.center.x, self.center.y, self.center.z))
      * Matrix4::new_scaling(self.half_extent)
  }
}

const CREASE: f32 = 0.72;
const SLIVER: f32 = 1e-9;

fn box_uv(position: Vec3, normal: Vec3) -> [f32; 2] {
  let axis = normal.abs();
  if axis.x > axis.y && axis.x > axis.z {
    [position.z, -position.y]
  } else if axis.y >= axis.z {
    [position.x, position.z]
  } else {
    [position.x, -position.y]
  }
}

pub fn bake(shape: Tree, bounds: Bounds) -> Mesh {
  let bound: BoundShape<VmFunction, f32> =
    VmShape::from(shape).try_into().expect("sdf may only use the x, y and z axes");
  let settings = Settings {
    depth: bounds.depth,
    world_to_model: bounds.world_to_model(),
    ..Default::default()
  };
  let dual = Octree::build(&bound, &settings).expect("meshing was cancelled").walk_dual();

  let vertices: Vec<Vec3> =
    dual.vertices.iter().map(|vertex| Vec3::new(vertex.x, vertex.y, vertex.z)).collect();
  let faces: Vec<([usize; 3], Vec3)> = dual
    .triangles
    .iter()
    .map(|corners| {
      let corners = [corners.x, corners.y, corners.z];
      let [a, b, c] = corners.map(|index| vertices[index]);
      (corners, (b - a).cross(c - a))
    })
    .filter(|(_, weighted)| weighted.length_squared() > SLIVER)
    .collect();

  let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); vertices.len()];
  for (index, (corners, _)) in faces.iter().enumerate() {
    for &corner in corners {
      adjacency[corner].push(index);
    }
  }

  let smoothed = |corner: usize, flat: Vec3| {
    adjacency[corner]
      .iter()
      .map(|&neighbour| faces[neighbour].1)
      .filter(|weighted| weighted.normalize().dot(flat) > CREASE)
      .sum::<Vec3>()
      .try_normalize()
      .unwrap_or(flat)
  };

  let corners: Vec<(Vec3, Vec3, [f32; 2])> = faces
    .iter()
    .flat_map(|&(corners, weighted)| {
      let flat = weighted.normalize();
      corners.map(|corner| {
        let position = vertices[corner];
        (position, smoothed(corner, flat), box_uv(position, flat))
      })
    })
    .collect();

  Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
    .with_inserted_attribute(
      Mesh::ATTRIBUTE_POSITION,
      corners.iter().map(|&(position, ..)| position.to_array()).collect::<Vec<_>>()
    )
    .with_inserted_attribute(
      Mesh::ATTRIBUTE_NORMAL,
      corners.iter().map(|&(_, normal, _)| normal.to_array()).collect::<Vec<_>>()
    )
    .with_inserted_attribute(
      Mesh::ATTRIBUTE_UV_0,
      corners.iter().map(|&(.., uv)| uv).collect::<Vec<_>>()
    )
}

pub fn points(mesh: &Mesh) -> Vec<Vec3> {
  mesh
    .attribute(Mesh::ATTRIBUTE_POSITION)
    .and_then(VertexAttributeValues::as_float3)
    .map(|positions| positions.iter().copied().map(Vec3::from).collect())
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn outwardness(mesh: &Mesh) -> f32 {
    let positions = points(mesh);
    let normals: Vec<Vec3> = mesh
      .attribute(Mesh::ATTRIBUTE_NORMAL)
      .and_then(VertexAttributeValues::as_float3)
      .map(|values| values.iter().copied().map(Vec3::from).collect())
      .unwrap_or_default();
    let center = positions.iter().sum::<Vec3>() / positions.len() as f32;
    positions
      .iter()
      .zip(&normals)
      .map(|(position, normal)| (*position - center).normalize_or_zero().dot(*normal))
      .sum::<f32>()
      / positions.len() as f32
  }

  #[test]
  fn normals_point_outwards() {
    let ore = bake(
      smooth_union(
        rounded_box(Vec3::new(0.34, 0.28, 0.30), 0.06),
        at(sphere(0.22), Vec3::new(0.14, 0.12, -0.08)),
        0.10
      ),
      Bounds::new(0.55, 6)
    );
    assert!(outwardness(&ore) > 0.5, "ore {}", outwardness(&ore));

    let ball = bake(sphere(0.5), Bounds::new(0.8, 5));
    assert!(outwardness(&ball) > 0.9, "ball {}", outwardness(&ball));
  }
}

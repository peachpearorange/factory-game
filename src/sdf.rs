use bevy::asset::RenderAssetUsages;
use bevy::mesh::{PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use fidget::{
    context::Tree,
    mesh::{Octree, Settings},
    shape::BoundShape,
    vm::{VmFunction, VmShape},
};
use nalgebra::{Matrix4, Vector3};

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
        z.abs() - (f64::from(half.z) - r),
    ];
    let outside = (q[0].max(0.0).square() + q[1].max(0.0).square() + q[2].max(0.0).square()).sqrt();
    outside + q[0].max(q[1].clone()).max(q[2].clone()).min(0.0) - r
}

pub fn cuboid(half: Vec3) -> Tree {
    rounded_box(half, 0.0)
}

pub fn cylinder(radius: f32, half_height: f32) -> Tree {
    let (x, y, z) = Tree::axes();
    let radial = (x.square() + z.square()).sqrt() - f64::from(radius);
    let axial = y.abs() - f64::from(half_height);
    let outside = (radial.max(0.0).square() + axial.max(0.0).square()).sqrt();
    outside + radial.max(axial).min(0.0)
}

pub fn at(shape: Tree, offset: Vec3) -> Tree {
    let (x, y, z) = Tree::axes();
    shape.remap_xyz(
        x - f64::from(offset.x),
        y - f64::from(offset.y),
        z - f64::from(offset.z),
    )
}

pub fn union(shapes: impl IntoIterator<Item = Tree>) -> Tree {
    shapes
        .into_iter()
        .reduce(|a, b| a.min(b))
        .expect("union of no shapes")
}

pub fn difference(shape: Tree, cutout: Tree) -> Tree {
    shape.max(-cutout)
}

pub fn smooth_union(a: Tree, b: Tree, radius: f32) -> Tree {
    let r = f64::from(radius);
    a.min(b.clone()) - 1.0 / (4.0 * r) * (r - (a - b).abs()).max(0.0).square()
}

pub struct Bounds {
    pub center: Vec3,
    pub half_extent: f32,
    pub depth: u8,
}

impl Bounds {
    pub const fn new(half_extent: f32, depth: u8) -> Self {
        Self {
            center: Vec3::ZERO,
            half_extent,
            depth,
        }
    }

    fn world_to_model(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&Vector3::new(self.center.x, self.center.y, self.center.z))
            * Matrix4::new_scaling(self.half_extent)
    }
}

pub fn bake(shape: Tree, bounds: Bounds) -> Mesh {
    let bound: BoundShape<VmFunction, f32> = VmShape::from(shape)
        .try_into()
        .expect("sdf may only use the x, y and z axes");
    let settings = Settings {
        depth: bounds.depth,
        world_to_model: bounds.world_to_model(),
        ..Default::default()
    };
    let mesh = Octree::build(&bound, &settings)
        .expect("meshing was cancelled")
        .walk_dual();

    let positions: Vec<[f32; 3]> = mesh
        .triangles
        .iter()
        .flatten()
        .map(|&index| {
            let vertex = mesh.vertices[index];
            [vertex.x, vertex.y, vertex.z]
        })
        .collect();

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_computed_flat_normals()
}

pub fn points(mesh: &Mesh) -> Vec<Vec3> {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(VertexAttributeValues::as_float3)
        .map(|positions| positions.iter().copied().map(Vec3::from).collect())
        .unwrap_or_default()
}

use super::{Vertex};
use crate::planet;
use nalgebra::{Point3};
use ncollide::bounding_volume::{AABB, AABB3};
use planet::renderer::node_backing::NodeId;
use planet::renderer::node_backing::NodeBacking;
use planet::quad_tree::HasAABB;

/// Contains geometry information for a single node of a quad tree for a face.
pub struct Node {
    pub node_id: NodeId,

    pub aabb: AABB3<f64>,
    pub origin: Point3<f64>,
}

impl Node {
    pub fn new(
        backing: &mut NodeBacking,
        geometry: &planet::PatchGeometry,
    ) -> Node {
        let id = backing.acquire();
        let slice = backing.vertices.slice_mut(id);
        let mut mapping = slice.map_write();
        assert_eq!(mapping.len(), geometry.positions.len());

        let mut min = geometry.positions[0].clone();
        let mut max = geometry.positions[0].clone();

        let origin = geometry.positions[0].clone();

        for (i, (pos, normal)) in geometry.positions.iter().zip(geometry.normals.iter()).enumerate() {
            min = nalgebra::inf(&min, pos);
            max = nalgebra::sup(&max, pos);

            let rel_pos = Point3::from_coordinates(pos - origin);
            mapping.set(i, Vertex {
                position: [rel_pos.x as f32, rel_pos.y as f32, rel_pos.z as f32],
                normal: [normal.x as f32, normal.y as f32, normal.z as f32],
            });
        }

        Node {
            node_id: id,
            aabb: AABB3::new(min, max),
            origin
        }
    }
}

impl HasAABB<Point3<f64>> for Node where {
    fn bounding_box(&self) -> AABB<Point3<f64>> { self.aabb.clone() }
}
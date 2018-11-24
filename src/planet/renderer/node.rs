use super::{Vertex};
use crate::planet;
use ncollide::bounding_volume::{AABB3};
use planet::renderer::node_backing::NodeId;
use planet::renderer::node_backing::NodeBacking;

/// Contains geometry information for a single node of a quad tree for a face.
pub struct Node {
    pub node_id: NodeId,

    pub aabb: AABB3<f64>,
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

        for (i, (pos, normal)) in geometry.positions.iter().zip(geometry.normals.iter()).enumerate() {
            min = nalgebra::inf(&min, pos);
            max = nalgebra::sup(&max, pos);
            mapping.set(i, Vertex {
                position: [pos.x as f32, pos.y as f32, pos.z as f32],
                normal: [normal.x as f32, normal.y as f32, normal.z as f32],
            });
        }

        Node {
            node_id: id,
            aabb: AABB3::new(min, max),
        }
    }
}

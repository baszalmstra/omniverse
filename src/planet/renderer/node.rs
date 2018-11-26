use super::Vertex;
use core::mem;
use crate::planet;
use nalgebra::{Matrix4, Point3, UnitQuaternion};
use ncollide::bounding_volume::{AABB, AABB3};
use crate::planet::quad_tree::HasAABB;
use crate::planet::renderer::node_backing::NodeBacking;
use crate::planet::renderer::node_backing::NodeId;

/// Contains geometry information for a single node of a quad tree for a face.
pub struct Node {
    pub node_id: NodeId,

    pub aabb: AABB3<f64>,
    pub origin: Point3<f64>,
    pub transform: Matrix4<f64>,
}

impl Node {
    pub fn new(backing: &mut NodeBacking, geometry: &planet::PatchGeometry) -> Node {
        use crate::planet::constants::{NORMALS_PER_PATCH, VERTICES_PER_PATCH};

        let id = backing.acquire();

        let mut min = geometry.positions[0];
        let mut max = geometry.positions[0];

        // Compute the reference frame of the node
        let origin = geometry.positions[0];
        let tangent =
            (geometry.positions[VERTICES_PER_PATCH - 1] - geometry.positions[0]).normalize();
        let geometric_binormal = (&geometry.positions
            [(VERTICES_PER_PATCH * VERTICES_PER_PATCH) - 1]
            - &geometry.positions[0])
            .normalize();
        let normal = tangent.cross(&geometric_binormal).normalize();
        let binormal = normal.cross(&tangent);
        let transform = UnitQuaternion::new_observer_frame(&normal, &binormal);
        let inverse_transform = transform.inverse();

        let mut heights: [f32; VERTICES_PER_PATCH * VERTICES_PER_PATCH];
        let mut vertices: [Vertex; VERTICES_PER_PATCH * VERTICES_PER_PATCH];
        let mut normals: [(f32, f32, f32); NORMALS_PER_PATCH * NORMALS_PER_PATCH];

        unsafe {
            heights = mem::uninitialized();
            vertices = mem::uninitialized();
            normals = mem::uninitialized();

            for (i, pos) in geometry.positions.iter().enumerate() {
                min = nalgebra::inf(&min, pos);
                max = nalgebra::sup(&max, pos);

                let x = i % VERTICES_PER_PATCH;
                let y = (i - x) / VERTICES_PER_PATCH;

                // Compute the vertex index that this vertex will morph to while morphing
                let morph_target_index = i - ((x % 2) * 1) - ((y % 2) * VERTICES_PER_PATCH);

                let rel_pos = inverse_transform * (pos - origin);
                heights[i] = rel_pos.z as f32;

                vertices[i].position = [rel_pos.x as f32, rel_pos.y as f32];
                vertices[i].position_morph_target = vertices[morph_target_index].position;
                vertices[i].local_texcoords = [
                    x as f32 / (VERTICES_PER_PATCH - 1) as f32,
                    y as f32 / (VERTICES_PER_PATCH - 1) as f32,
                ];
            }

            for (i, normal) in geometry.normals.iter().enumerate() {
                normals[i] = (normal.x as f32, normal.y as f32, normal.z as f32);
            }
        }

        backing.normals.write(id, 0, &normals);
        backing.heights.write(id, 0, &heights);
        backing.vertices.write(id, &vertices);

        Node {
            node_id: id,
            aabb: AABB3::new(min, max),
            origin,
            transform: nalgebra::convert(transform),
        }
    }
}

impl HasAABB<Point3<f64>> for Node where {
    fn bounding_box(&self) -> AABB<Point3<f64>> {
        self.aabb.clone()
    }
}

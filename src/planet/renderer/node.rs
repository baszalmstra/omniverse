use super::{Description, Vertex};
use crate::planet;
use glium::backend::Facade;
use glium::VertexBuffer;
use nalgebra::{Point2, Point3, UnitQuaternion};
use ncollide::bounding_volume::{point_cloud_aabb, AABB3};

/// Contains geometry information for a single node of a quad tree for a face.
pub struct Node {
    pub vertex_buffer: VertexBuffer<Vertex>,

    pub aabb: AABB3<f64>,
}

impl Node {
    pub fn new<F: ?Sized + Facade>(
        facade: &F,
        geometry: &planet::PatchGeometry,
    ) -> Result<Node, Box<std::error::Error>> {
        use crate::planet::constants::VERTICES_PER_PATCH;
        let mut vertices =
            Vec::<Vertex>::with_capacity(VERTICES_PER_PATCH * VERTICES_PER_PATCH as usize);

        let mut min = geometry.positions[0].clone();
        let mut max = geometry.positions[0].clone();

        for (pos, normal) in geometry.positions.iter().zip(geometry.normals.iter()) {
            min = nalgebra::inf(&min, pos);
            max = nalgebra::sup(&max, pos);
            vertices.push(Vertex {
                position: [pos.x as f32, pos.y as f32, pos.z as f32],
                normal: [normal.x as f32, normal.y as f32, normal.z as f32],
            });
        }

        Ok(Node {
            vertex_buffer: VertexBuffer::new(facade, &vertices)?,
            aabb: AABB3::new(min, max),
        })
    }
}

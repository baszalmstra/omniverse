use super::{Description, Vertex};
use crate::planet;
use glium::backend::Facade;
use glium::VertexBuffer;
use nalgebra::{Point2, Point3, UnitQuaternion};

/// Contains geometry information for a single node of a quad tree for a face.
pub struct Node {
    pub vertex_buffer: VertexBuffer<Vertex>,
}

impl Node {
    pub fn new<F: ?Sized + Facade>(
        facade: &F,
        geometry: &planet::PatchGeometry,
    ) -> Result<Node, Box<std::error::Error>> {
        use planet::constants::VERTICES_PER_PATCH;
        let mut vertices =  Vec::<Vertex>::with_capacity(VERTICES_PER_PATCH * VERTICES_PER_PATCH as usize);
        for pos in geometry.positions.iter() {
            vertices.push(Vertex {
               position: [
                   pos.x as f32,
                   pos.y as f32,
                   pos.z as f32,
               ]
            });
        }

        Ok(Node {
            vertex_buffer: VertexBuffer::new(facade, &vertices)?,
        })
    }
}


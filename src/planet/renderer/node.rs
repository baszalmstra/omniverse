use glium::VertexBuffer;
use glium::backend::{Facade};
use nalgebra::{UnitQuaternion, Point2, Point3};
use super::{Vertex, Description};

/// Location of a node in the oriented unit quad.
pub struct NodeLocation {
    pub orientation: UnitQuaternion<f64>,
    pub offset: Point2<f64>,
    pub size: f64,
}

/// Contains geometry information for a single node of a quad tree for a face.
pub struct Node {
    pub vertex_buffer: VertexBuffer<Vertex>
}

impl Node {
    pub fn new<F:?Sized+Facade>(facade: &F, description: &Description, location: NodeLocation) -> Result<Node, Box<std::error::Error>> {
        use super::constants::VERTICES_PER_PATCH;

        let mut vertices = Vec::<Vertex>::with_capacity(VERTICES_PER_PATCH*VERTICES_PER_PATCH as usize);
        let step = location.size/(VERTICES_PER_PATCH as f64-1.0);
        for y in 0..VERTICES_PER_PATCH {
            for x in 0..VERTICES_PER_PATCH {
                let local_position = Point3::<f64>::new(x as f64 * step - 0.5,y as f64 * step - 0.5,0.5) * 2.0 * description.radius;
                let oriented_position = location.orientation * local_position;

                vertices.push(Vertex {
                    position: [
                        oriented_position.x as f32,
                        oriented_position.y as f32,
                        oriented_position.z as f32
                    ]
                })
            }
        }

        Ok(Node{
            vertex_buffer: VertexBuffer::new(facade, &vertices)?
        })
    }
}
use std::rc::Rc;
use glium::{Surface, Program, Frame, Display, IndexBuffer};
use glium::backend::{Facade, Context};
use glium::index::{IndicesSource, PrimitiveType};
use frustum::Frustum;
use transform::Transform;
use nalgebra::{Point2, UnitQuaternion};
use super::quad_tree::QuadTree;
use super::Description;
use crate::planet;

mod node;
mod face;
mod vertex;
mod constants;

pub use self::vertex::Vertex;
pub use self::face::Face;
pub use self::node::{NodeLocation, Node};

pub struct Renderer {
    context: Rc<Context>,
    description: Description,
    faces: [Face; 6],

    program: Program,
    index_buffer: IndexBuffer<u16>
}

impl Renderer {
    pub fn new<F:?Sized+Facade>(facade: &F, description:Description) -> Result<Renderer, Box<std::error::Error>> {
        use nalgebra::UnitQuaternion;
        use self::constants::VERTICES_PER_PATCH;

        let program = {
            let vertex_shader_src = r#"
                #version 140

                in vec3 position;

                uniform mat4 viewProjection;

                void main() {
                    gl_Position = viewProjection*vec4(position, 1.0);
                }
            "#;

            let fragment_shader_src = r#"
                #version 140

                out vec4 color;

                void main() {
                    color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "#;

            Program::from_source(facade,
                                 vertex_shader_src,
                                 fragment_shader_src,
                                 None)?
        };

        let index_buffer = {
            let mut indices:Vec<u16> = Vec::with_capacity((VERTICES_PER_PATCH-1)*(VERTICES_PER_PATCH-1)*6);
            {
                let mut add_region = |x_start: usize, y_start: usize, x_end: usize, y_end: usize| {
                    for y in y_start..y_end - 1 {
                        for x in x_start..x_end - 1 {
                            indices.push(((x + 0) + (y + 0) * VERTICES_PER_PATCH) as u16);
                            indices.push(((x + 1) + (y + 1) * VERTICES_PER_PATCH) as u16);
                            indices.push(((x + 0) + (y + 1) * VERTICES_PER_PATCH) as u16);
                            indices.push(((x + 1) + (y + 1) * VERTICES_PER_PATCH) as u16);
                            indices.push(((x + 0) + (y + 0) * VERTICES_PER_PATCH) as u16);
                            indices.push(((x + 1) + (y + 0) * VERTICES_PER_PATCH) as u16);
                        }
                    }
                };

                add_region(0, 0, VERTICES_PER_PATCH / 2 + 1, VERTICES_PER_PATCH / 2 + 1);
                add_region(VERTICES_PER_PATCH / 2, 0, VERTICES_PER_PATCH, VERTICES_PER_PATCH / 2 + 1);
                add_region(0, VERTICES_PER_PATCH / 2, VERTICES_PER_PATCH / 2 + 1, VERTICES_PER_PATCH);
                add_region(VERTICES_PER_PATCH / 2, VERTICES_PER_PATCH / 2, VERTICES_PER_PATCH, VERTICES_PER_PATCH);
            }
            IndexBuffer::new(facade, PrimitiveType::TrianglesList,&indices)?
        };

        Ok(Renderer {
            context: facade.get_context().clone(),
            faces: [
                Renderer::generate_face(facade, planet::Face::Front, &description)?,
                Renderer::generate_face(facade, planet::Face::Back, &description)?,
                Renderer::generate_face(facade, planet::Face::Left, &description)?,
                Renderer::generate_face(facade, planet::Face::Right, &description)?,
                Renderer::generate_face(facade, planet::Face::Top, &description)?,
                Renderer::generate_face(facade, planet::Face::Bottom, &description)?
            ],
            description,
            program,
            index_buffer
        })
    }

    fn generate_face<F:?Sized+Facade>(facade: &F, face:planet::Face, description: &Description) -> Result<Face, Box<std::error::Error>> {
        let root_node = Node::new(facade, description, NodeLocation {
            face,
            size: 1.0,
            offset: Point2::new(0.0, 0.0)
        })?;
        Ok(Face::new(root_node, face))
    }

    /// Draws the planet to the screen from the perspective of the given frustum.
    /// * `frame` - The frame to render to
    /// * `frustum` - The frustum that represents the view to render from in world space.
    /// * `planet_world_transform` - The transformation of the planet relative to the world.
    pub fn draw(&self, frame: &mut Frame, frustum: &Frustum, planet_world_transform:&Transform) {
        let uniforms = uniform! {
            viewProjection: Into::<[[f32; 4]; 4]>::into(frustum.view_projection),
        };

        for face in self.faces.iter() {
            frame.draw(&face.root.content.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
        }
    }

    /// Returns the context corresponding to this Renderer.
    pub fn get_context(&self) -> &Rc<Context> {
        &self.context
    }
}
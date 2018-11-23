use super::quad_tree::QuadTree;
use super::Description;
use crate::planet;
use frustum::Frustum;
use glium::backend::{Context, Facade};
use glium::index::{IndicesSource, PrimitiveType};
use glium::{Display, Frame, IndexBuffer, Program, Surface};
use nalgebra::{Point2, UnitQuaternion};
use std::rc::Rc;
use transform::Transform;

mod face;
mod node;
mod vertex;

pub use self::face::Face;
pub use self::node::Node;
pub use self::vertex::Vertex;
use planet::geometry_provider::PatchLocation;

pub struct Renderer<T: planet::GeometryProvider> {
    /// The OpenGL context
    context: Rc<Context>,

    description: Description,
    geometry_provider: T,

    faces: [Face; 6],

    program: Program,
    index_buffer: IndexBuffer<u16>,
}

impl<T: planet::GeometryProvider> Renderer<T> {
    pub fn new<F: ?Sized + Facade>(
        facade: &F,
        description: Description,
        geometry_provider: T,
    ) -> Result<Renderer<T>, Box<std::error::Error>> {
        use nalgebra::UnitQuaternion;
        use planet::constants::VERTICES_PER_PATCH;

        let program = {
            let vertex_shader_src = r#"
                #version 430 core

                layout(location = 0) in vec3 position;
                layout(location = 1) in vec3 normal;

                layout(location = 0) out vec3 Normal;

                uniform mat4 viewProjection;

                void main() {
                    gl_Position = viewProjection*vec4(position, 1.0);

                    Normal = normal;
                }
            "#;

            let fragment_shader_src = r#"
                #version 430 core

                layout(location = 0) in vec3 normal;

                out vec4 color;

                void main() {
                    float nDotL = max(0, dot(normal, vec3(1,0,0)));
                    color = vec4(nDotL,nDotL,nDotL, 1.0);
                }
            "#;

            Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)?
        };

        let index_buffer = {
            let mut indices: Vec<u16> =
                Vec::with_capacity((VERTICES_PER_PATCH - 1) * (VERTICES_PER_PATCH - 1) * 6);
            {
                let mut add_region =
                    |x_start: usize, y_start: usize, x_end: usize, y_end: usize| {
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
                add_region(
                    VERTICES_PER_PATCH / 2,
                    0,
                    VERTICES_PER_PATCH,
                    VERTICES_PER_PATCH / 2 + 1,
                );
                add_region(
                    0,
                    VERTICES_PER_PATCH / 2,
                    VERTICES_PER_PATCH / 2 + 1,
                    VERTICES_PER_PATCH,
                );
                add_region(
                    VERTICES_PER_PATCH / 2,
                    VERTICES_PER_PATCH / 2,
                    VERTICES_PER_PATCH,
                    VERTICES_PER_PATCH,
                );
            }
            IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices)?
        };

        Ok(Renderer {
            context: facade.get_context().clone(),
            faces: [
                generate_face(facade, planet::Face::Front, &geometry_provider)?,
                generate_face(facade, planet::Face::Back, &geometry_provider)?,
                generate_face(facade, planet::Face::Left, &geometry_provider)?,
                generate_face(facade, planet::Face::Right, &geometry_provider)?,
                generate_face(facade, planet::Face::Top, &geometry_provider)?,
                generate_face(facade, planet::Face::Bottom, &geometry_provider)?,
            ],
            geometry_provider,
            description,
            program,
            index_buffer,
        })
    }



    /// Draws the planet to the screen from the perspective of the given frustum.
    /// * `frame` - The frame to render to
    /// * `frustum` - The frustum that represents the view to render from in world space.
    /// * `planet_world_transform` - The transformation of the planet relative to the world.
    pub fn draw(&self, frame: &mut Frame, frustum: &Frustum, planet_world_transform: &Transform) {
        let uniforms = uniform! {
            viewProjection: Into::<[[f32; 4]; 4]>::into(frustum.view_projection),
        };

        for face in self.faces.iter() {
            frame
                .draw(
                    &face.root.content.vertex_buffer,
                    &self.index_buffer,
                    &self.program,
                    &uniforms,
                    &Default::default(),
                ).unwrap();
        }
    }

    /// Returns the context corresponding to this Renderer.
    pub fn get_context(&self) -> &Rc<Context> {
        &self.context
    }

    pub fn geometry_generator(&self) -> &planet::GeometryProvider {
        &self.geometry_provider
    }
}

fn generate_face<F: ?Sized + Facade>(
    facade: &F,
    face: planet::Face,
    geometry_provider: &planet::GeometryProvider,
) -> Result<Face, Box<std::error::Error>> {
    let root_node = Node::new(
        facade,
        &geometry_provider.provide(PatchLocation {
            face,
            size: 1.0,
            offset: Point2::new(0.0, 0.0),
        }),
    )?;
    Ok(Face::new(root_node, face))
}
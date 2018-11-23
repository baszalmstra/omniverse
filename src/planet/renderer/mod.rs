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

mod node;
mod vertex;

pub use self::node::Node;
pub use self::vertex::Vertex;
use planet::geometry_provider::PatchLocation;

pub struct Renderer<T: planet::GeometryProvider> {
    /// The OpenGL context
    context: Rc<Context>,

    description: Description,
    geometry_provider: T,

    faces: [Face; 6],
    max_lod_level: usize,
    split_distances: Vec<f64>,

    program: Program,
    index_buffer: IndexBuffer<u16>,
}

struct Face {
    pub face: planet::Face,
    pub root: QuadTree<Node>,
}

impl<T: planet::GeometryProvider> Renderer<T> {
    pub fn new<F: ?Sized + Facade>(
        facade: &F,
        description: Description,
        geometry_provider: T,
    ) -> Result<Renderer<T>, Box<std::error::Error>> {
        use nalgebra::UnitQuaternion;
        use planet::constants::VERTICES_PER_PATCH;
        use std::f64::consts::PI;

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
                                indices.push(((x + 0) + (y + 1) * VERTICES_PER_PATCH) as u16);
                                indices.push(((x + 1) + (y + 1) * VERTICES_PER_PATCH) as u16);
                                indices.push(((x + 0) + (y + 0) * VERTICES_PER_PATCH) as u16);
                                indices.push(((x + 1) + (y + 1) * VERTICES_PER_PATCH) as u16);
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

        let max_lod_level = ((0.5 * PI * description.radius)
            .log2()
            .ceil()
            - 1.0).max(1.0) as usize;

        let mut split_distances:Vec<f64> = Vec::with_capacity(max_lod_level);
        split_distances.push(2.0);
        let mut last_value= 2.0;
        for i in 0..max_lod_level-1 {
            split_distances.push(last_value * 2.0);
            last_value = last_value * 2.0;
        }

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
            max_lod_level,
            split_distances
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

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        for face in self.faces.iter() {
            frame
                .draw(
                    &face.root.content.vertex_buffer,
                    &self.index_buffer,
                    &self.program,
                    &uniforms,
                    &params,
                ).unwrap();
        }
    }

    pub fn ensure_resident_patches(
        &mut self,
        frustum: &Frustum,
        planet_world_transform: &Transform,
    ) {
        let frustum_planet = Frustum::new(
            planet_world_transform.inverse() * frustum.transform,
            frustum.projection,
        );

        for face in self.faces.iter_mut() {
            ensure_resident_children(
                frustum,
                face.face,
                &mut face.root,
                Point2::new(0.0, 0.0),
                1.0,
                self.max_lod_level,
                &self.split_distances
            );
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
    Ok(Face {
        face,
        root: QuadTree::new(root_node),
    })
}

fn ensure_resident_children(
    frustum_planet: &Frustum,
    face: planet::Face,
    node: &mut QuadTree<Node>,
    offset: Point2<f64>,
    size: f64,
    depth: usize,
    split_distances: &[f64]
) {

}
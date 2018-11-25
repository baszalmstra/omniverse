#![allow(dead_code)]

use super::quad_tree::QuadTree;
use super::quad_tree;
use super::Description;
use crate::frustum::Frustum;
use crate::planet;
use crate::transform::Transform;
use nalgebra::{Point3, Vector3, Matrix4, Translation3};
use std::rc::Rc;

mod node;
mod vertex;
mod node_backing;

pub use self::node::Node;
pub use self::vertex::Vertex;
use crate::planet::geometry_provider::PatchLocation;
use std::collections::VecDeque;
use planet::renderer::node_backing::NodeBacking;
use std::cell::RefCell;
use glium::{
    VertexBuffer,
    Frame,
    IndexBuffer,
    Program,
    Surface,
    index::{PrimitiveType, IndicesSource},
    backend::{Context, Facade},
    index::DrawCommandsIndicesBuffer
};
use glium::index::DrawCommandIndices;
use planet::quad_tree::HasAABB;

pub struct DrawParameters {
    pub wire_frame: bool,
}

impl Default for DrawParameters {
    fn default() -> Self {
        DrawParameters { wire_frame: false }
    }
}

#[derive(Copy, Clone)]
struct PerNodeInstanceVertex {
    pose_camera: [[f32;4]; 4],
}

implement_vertex!(PerNodeInstanceVertex, pose_camera);

pub struct Renderer<T: planet::GeometryProvider> {
    /// The OpenGL context
    context: Rc<Context>,

    description: Description,
    geometry_provider: T,
    backing: NodeBacking,
    command_buffer: RefCell<DrawCommandsIndicesBuffer>,
    per_visible_node_buffer: RefCell<VertexBuffer<PerNodeInstanceVertex>>,

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
        use crate::planet::constants::{VERTICES_PER_PATCH, MAX_PATCH_COUNT};
        use std::f64::consts::PI;

        let program = {
            let vertex_shader_src = r#"
                #version 330 core

                in vec3 position;
                in vec3 normal;
                in mat4 pose_camera;

                out vec3 Normal;

                uniform mat4 viewProjection;

                void main() {
                    gl_Position = viewProjection*(pose_camera*vec4(position, 1.0));

                    Normal = normal;
                }
            "#;

            let fragment_shader_src = r#"
                #version 330 core

                in vec3 Normal;

                out vec4 color;

                void main() {
                    float nDotL = max(0, dot(Normal, vec3(1,0,0)));
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

        let max_lod_level = ((0.5 * PI * description.radius).log2().ceil() - 1.0).max(1.0) as usize;

        let mut split_distances: Vec<f64> = Vec::with_capacity(max_lod_level);
        split_distances.push(2.0);
        let mut last_value = 2.0;
        for _i in 0..max_lod_level {
            split_distances.push(last_value * 2.0);
            last_value = last_value * 2.0;
        }

        let mut backing = NodeBacking::new(facade)?;

        let faces = [
            generate_face(&mut backing, planet::Face::Front, &geometry_provider),
            generate_face(&mut backing, planet::Face::Back, &geometry_provider),
            generate_face(&mut backing, planet::Face::Left, &geometry_provider),
            generate_face(&mut backing, planet::Face::Right, &geometry_provider),
            generate_face(&mut backing, planet::Face::Top, &geometry_provider),
            generate_face(&mut backing, planet::Face::Bottom, &geometry_provider),
        ];

        Ok(Renderer {
            context: facade.get_context().clone(),
            faces,
            backing,
            geometry_provider,
            description,
            program,
            index_buffer,
            max_lod_level,
            split_distances,
            per_visible_node_buffer: RefCell::new(VertexBuffer::empty_persistent(facade, MAX_PATCH_COUNT)?),
            command_buffer: RefCell::new(DrawCommandsIndicesBuffer::empty_persistent(facade, MAX_PATCH_COUNT)?)
        })
    }

    /// Draws the planet to the screen from the perspective of the given frustum.
    /// * `frame` - The frame to render to
    /// * `frustum` - The frustum that represents the view to render from in world space.
    /// * `planet_world_transform` - The transformation of the planet relative to the world.
    pub fn draw(
        &self,
        frame: &mut Frame,
        frustum: &Frustum,
        planet_world_transform: &Transform,
        draw_parameters: &DrawParameters,
    ) {
        // Construct a new frustum relative to the planet to ease computations
        let frustum_planet = Frustum::new(
            planet_world_transform.inverse() * frustum.transform,
            frustum.projection,
        );

        // Projection frustum is used for the final projection in the shader. This frustum is
        // basically `frustum_planet` but without the translation. This is because the translation
        // of the camera is already encoded in the patches themselves. This way we get maximum
        // precision near the camera position.
        let projection_frustum = Frustum::new(
            Transform::from_parts(Translation3::identity(), frustum_planet.transform.rotation),
            frustum.projection,
        );

        // Query all faces for visible nodes
        let visible_nodes: VecDeque<VisibleNode> = {
            let mut result = VecDeque::new();
            for face in self.faces.iter() {
                lod_select(
                    &frustum_planet,
                    &face.root,
                    face.face.into(),
                    self.max_lod_level,
                    self.max_lod_level,
                    &self.split_distances,
                    false,
                    &mut result,
                );
            }
            result
        };

        // Setup all uniforms for drawing
        let uniforms = uniform! {
            viewProjection: Into::<[[f32; 4]; 4]>::into(projection_frustum.view_projection),
        };

        // Setup render pipeline
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
            polygon_mode: if draw_parameters.wire_frame {
                glium::PolygonMode::Line
            } else {
                glium::PolygonMode::Line
            },
            ..Default::default()
        };

        // Generate per node instance data
        {
            let mut node_instance_data = self.per_visible_node_buffer.borrow_mut();
            let mut mapping = node_instance_data.map_write();
            for (idx, node) in visible_nodes.iter().enumerate() {
                mapping.set(idx, PerNodeInstanceVertex {
                    pose_camera: node.transform_camera.into()
                })
            }
        }

        // Fill the command buffer to execute rendering of all patches on the GPU in a single
        // command
        {
            let mut command_buffer = self.command_buffer.borrow_mut();
            let mut mapping = command_buffer.map_write();
            for (idx, node) in visible_nodes.iter().enumerate() {
                let index_count = self.index_buffer.len() as u32;
                let (first_index, count) = match node.part {
                    VisibleNodePart::Whole => (0, index_count),
                    VisibleNodePart::Child(child) => match child {
                        quad_tree::Child::TopLeft => (0, index_count/4),
                        quad_tree::Child::TopRight => (index_count/4, index_count/4),
                        quad_tree::Child::BottomLeft => (index_count/4*2, index_count/4),
                        quad_tree::Child::BottomRight => (index_count/4*3, index_count/4),
                    }
                };
                mapping.set(idx, DrawCommandIndices {
                    count,
                    instance_count: 1,
                    first_index,
                    base_vertex: self.backing.vertices.base_vertex(node.node.node_id),
                    base_instance: idx as u32,
                })
            }
        }

        let command_buffer = self.command_buffer.borrow();
        let command_buffer_slice = command_buffer.slice(0 .. visible_nodes.len()).unwrap();
        let node_instance_data = self.per_visible_node_buffer.borrow();
        frame
            .draw(
                (&self.backing.vertices.vertex_buffer, node_instance_data.per_instance().unwrap()),
                IndicesSource::MultidrawElement {
                    commands: command_buffer_slice.as_slice_any(),
                    indices: self.index_buffer.as_slice_any(),
                    data_type: self.index_buffer.get_indices_type(),
                    primitives: self.index_buffer.get_primitives_type(),
                },
                &self.program,
                &uniforms,
                &params,
            )
            .unwrap();

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
                &self.context,
                &frustum_planet,
                &self.geometry_provider,
                &mut self.backing,
                &mut face.root,
                face.face.into(),
                self.max_lod_level,
                &self.split_distances,
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

fn generate_face(
    backing: &mut NodeBacking,
    face: planet::Face,
    geometry_provider: &planet::GeometryProvider,
) -> Face {
    Face {
        face,
        root: QuadTree::new(Node::new(
            backing,
            &geometry_provider.provide(face.into()),
        )),
    }
}

fn ensure_resident_children<F: ?Sized + Facade>(
    facade: &F,
    frustum_planet: &Frustum,
    geometry_provider: &planet::GeometryProvider,
    backing: &mut NodeBacking,
    node: &mut QuadTree<Node>,
    location: PatchLocation,
    depth: usize,
    split_distances: &[f64],
) {
    if depth == 0 {
        return;
    }

    // If the node is out of range of it's split distance, remove it's children
    let frustum_pos = Point3::from_coordinates(frustum_planet.transform.translation.vector);
    if !in_range(&node.content.aabb, &frustum_pos, split_distances[depth]) {
        merge(backing, node);
        return;
    }

    // Otherwise; ensure that this node has children resident
    if !node.has_children() {
        node.children = Some(Box::new([
            QuadTree::new(Node::new(backing,&geometry_provider.provide(location.top_left()))),
            QuadTree::new(Node::new(backing,&geometry_provider.provide(location.top_right()))),
            QuadTree::new(Node::new(backing,&geometry_provider.provide(location.bottom_left()))),
            QuadTree::new(Node::new(backing,&geometry_provider.provide(location.bottom_right()))),
        ]))
    }

    if let Some(ref mut children) = node.children {
        for child in quad_tree::Child::values() {
            ensure_resident_children(
                facade,
                frustum_planet,
                geometry_provider,
                backing,
                &mut (*children)[child.index()],
                location.split(*child),
                depth - 1,
                split_distances,
            );
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum VisibleNodePart {
    Whole,
    Child(quad_tree::Child)
}

struct VisibleNode<'a> {
    pub node: &'a Node,
    pub transform_camera: Matrix4<f32>,
    pub part: VisibleNodePart
}

#[derive(Copy, Clone, PartialEq)]
enum LODSelectResult {
    // Undefined value (patch doesn't exist)
    Undefined,

    // The patch is outside of the frustum
    OutOfFrustum,

    // The patch is outside of its lod range
    OutOfRange,

    // The patch was selected
    Selected,
}

impl LODSelectResult {
    fn is_not_selected(&self) -> bool {
        match self {
            LODSelectResult::Undefined => true,
            LODSelectResult::OutOfFrustum => false,
            LODSelectResult::OutOfRange => true,
            LODSelectResult::Selected => false,
        }
    }
}

fn lod_select<'a>(
    frustum_planet: &Frustum,
    node: &'a QuadTree<Node>,
    location: PatchLocation,
    depth: usize,
    max_lod_level: usize,
    split_distances: &[f64],
    parent_completly_in_frustum:bool,
    result: &mut VecDeque<VisibleNode<'a>>,
) -> LODSelectResult {
    use frustum::{Containment, Classify};

    let frustum_containment = if parent_completly_in_frustum { Containment::Inside } else { frustum_planet.classify(&node.bounding_box()) };
    if frustum_containment == Containment::Outside {
        return LODSelectResult::OutOfFrustum;
    }

    if depth == 0 {
        // The last lod level is just always added to the selection list
        add_to_visible_list(
            frustum_planet,
            &node.content,
            depth,
            split_distances,
            result,
            VisibleNodePart::Whole
        );
        return LODSelectResult::Selected;
    }

    // Check if the node is within it's LOD range or if it's children should be selected
    let frustum_pos = Point3::from_coordinates(frustum_planet.transform.translation.vector);
    if !in_range(&node.bounding_box(), &frustum_pos, split_distances[depth]) {
        // No matter what, the highest lod level is always selected
        if depth == max_lod_level {
            add_to_visible_list(
                frustum_planet,
                &node.content,
                depth,
                split_distances,
                result,
                VisibleNodePart::Whole,
            );
            return LODSelectResult::Selected;
        }

        return LODSelectResult::OutOfRange;
    }

    if let Some(ref children) = node.children {
        let node_completely_in_frustum = frustum_containment == Containment::Inside;
        let mut children_selection_results = [LODSelectResult::Undefined; 4];

        for child in quad_tree::Child::values() {
            children_selection_results[child.index()] = lod_select(
                frustum_planet,
                &(*children)[child.index()],
                location.split(*child),
                depth - 1,
                max_lod_level,
                split_distances,
                node_completely_in_frustum,
                result,
            );
        }

        // If non of the nodes was selected because they either lack geometry or where out of range,
        // the entire node is simply selected
        if children_selection_results.iter().all(LODSelectResult::is_not_selected) {
            // If the node has no children, we'll add it anyway
            add_to_visible_list(
                frustum_planet,
                &node.content,
                depth,
                split_distances,
                result,
                VisibleNodePart::Whole,
            );
            return LODSelectResult::Selected
        }

        // If any of the nodes is not selected because it has no geometry or because it's out of
        // range, fill it in with geometry from the parent node
        for child in quad_tree::Child::values()
            .filter(|c| children_selection_results[c.index()].is_not_selected()) {
            add_to_visible_list(frustum_planet,
                                &node.content,
                                depth,
                                split_distances,
                                result,
                                VisibleNodePart::Child(*child))
        }

        if children_selection_results.iter().any(|s| *s == LODSelectResult::Selected) {
            LODSelectResult::Selected
        } else {
            LODSelectResult::OutOfFrustum
        }
    } else {
        // If the node has no children, we'll add it anyway
        add_to_visible_list(
            frustum_planet,
            &node.content,
            depth,
            split_distances,
            result,
            VisibleNodePart::Whole,
        );
        LODSelectResult::Selected
    }
}

fn add_to_visible_list<'a>(
    frustum_planet: &Frustum,
    node: &'a Node,
    _depth: usize,
    _split_distances: &[f64],
    result: &mut VecDeque<VisibleNode<'a>>,
    part: VisibleNodePart
) {
    let node_camera = node.origin - Point3::from_coordinates(frustum_planet.transform.translation.vector);

    result.push_back(VisibleNode {
        node,
        transform_camera: nalgebra::convert(Translation3::from_vector(node_camera).to_homogeneous()),
        part
    })
}

fn merge(backing: &mut NodeBacking, node: &mut QuadTree<Node>) {
    if let Some(ref mut children) = node.children {
        for node in children.iter_mut() {
            backing.release(node.content.node_id);
        }
    }
    node.children = None;
}

fn in_range(
    aabb: &ncollide::bounding_volume::AABB3<f64>,
    position: &Point3<f64>,
    range: f64,
) -> bool {
    let min: Vector3<f64> = aabb.mins() - position;
    let max: Vector3<f64> = position - aabb.maxs();
    let delta = nalgebra::sup(&Vector3::new(0.0, 0.0, 0.0), &nalgebra::sup(&min, &max));
    nalgebra::dot(&delta, &delta) <= range * range
}

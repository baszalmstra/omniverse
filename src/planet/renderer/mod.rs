#![allow(dead_code)]

use super::quad_tree;
use super::quad_tree::QuadTree;
use super::Description;
use crate::frustum::Frustum;
use crate::planet;
use crate::transform::Transform;
use nalgebra::{Matrix4, Point3, Translation3, Vector3};
use std::rc::Rc;

mod horizon_culling;
mod node;
mod node_backing;
mod vertex;

pub use self::node::Node;
pub use self::vertex::Vertex;
use crate::culling::Classify;
use crate::planet::geometry_provider::PatchLocation;
use crate::planet::quad_tree::HasAABB;
use crate::planet::renderer::node::NodeGeometry;
use crate::planet::renderer::node_backing::NodeBacking;
use glium::index::DrawCommandIndices;
use glium::{
    backend::{Context, Facade},
    index::DrawCommandsIndicesBuffer,
    index::{IndicesSource, PrimitiveType},
    IndexBuffer, Program, Surface, VertexBuffer,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::atomic::Ordering;

#[derive(Deserialize)]
#[serde(default)]
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
    pose_camera: [[f32; 4]; 4],
    atlas_index: u32,
    morph_range: (f32, f32),
    lod_level: u16,
}

implement_vertex!(
    PerNodeInstanceVertex,
    pose_camera,
    atlas_index,
    morph_range,
    lod_level
);

type PendingStreamingNodesMap = HashMap<usize, *mut QuadTree<Node>>;

pub struct Renderer<T: planet::AsyncGeometryProvider + planet::GeometryProvider> {
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

    /// Contains a mapping from streaming request id's to the QuadTree node that requested it. This
    /// map is used when results come back from the streaming system and is kept up to date with
    /// nodes being destroyed. This ensures that there are never dangling pointers in this map.
    pending_geometry_requests: PendingStreamingNodesMap,
}

struct Face {
    pub face: planet::Face,
    pub root: QuadTree<Node>,
}

impl<T: planet::AsyncGeometryProvider + planet::GeometryProvider> Renderer<T> {
    pub fn new<F: ?Sized + Facade>(
        facade: &F,
        description: Description,
        geometry_provider: T,
    ) -> Result<Renderer<T>, Box<std::error::Error>> {
        use crate::planet::constants::{MAX_PATCH_COUNT, VERTICES_PER_PATCH};
        use std::f64::consts::PI;

        let program = {
            let vertex_shader_src = r#"
                #version 430 core
                #extension GL_EXT_texture_array : enable

                in vec2 position;
                in vec2 position_morph_target;
                in vec2 local_texcoords;
                in vec2 local_texcoords_morph_target;
                in vec3 color;

                in uint atlas_index;
                in uint lod_level;
                in mat4 pose_camera;
                in vec2 morph_range;

                out vec2 Texcoords;
                out vec4 Color;
                flat out uint AtlasIndex;
                out float MorphFactor;
                out float LogZ;

                uniform mat4 view_projection;
                uniform sampler2DArray height_atlas;
                uniform uint vertices_per_patch;

                uniform float camera_far = 20000000;
                uniform float camera_log_z_constant = 0.01;

                float sample_height(vec2 texcoord) {
                    // Compute the modified texture coordinates
                    ivec2 texture_size = textureSize(height_atlas, 0).xy;
                    vec2 texel_size = 1.0 / texture_size.xy;
                    vec2 height_atlas_texcoords = texcoord * (vec2(1.0, 1.0) - texel_size) + texel_size*0.5;
                    return texture2DArray(height_atlas, vec3(height_atlas_texcoords, atlas_index)).r;
                }

                vec3 random_colors[18] = vec3[18](
                    vec3(230, 25, 75) * (1.0/255.0),
                    vec3(60, 180, 75) * (1.0/255.0),
                    vec3(255, 225, 25) * (1.0/255.0),
                    vec3(0, 130, 200) * (1.0/255.0),
                    vec3(245, 130, 48) * (1.0/255.0),
                    vec3(145, 30, 180) * (1.0/255.0),
                    vec3(70, 240, 240) * (1.0/255.0),
                    vec3(240, 50, 230) * (1.0/255.0),
                    vec3(210, 245, 60) * (1.0/255.0),
                    vec3(250, 190, 190) * (1.0/255.0),
                    vec3(0, 128, 128) * (1.0/255.0),
                    vec3(230, 190, 255) * (1.0/255.0),
                    vec3(170, 110, 40) * (1.0/255.0),
                    vec3(255, 250, 200) * (1.0/255.0),
                    vec3(128, 0, 0) * (1.0/255.0),
                    vec3(170, 255, 195) * (1.0/255.0),
                    vec3(128, 128, 0) * (1.0/255.0),
                    vec3(255, 215, 180) * (1.0/255.0)
                );

                void main() {
                    // Construct the patch local coordinates and transform them to camera space
                    vec3 pos_patch = vec3(position.xy, sample_height(local_texcoords));
                    vec4 pos_camera = pose_camera*vec4(pos_patch, 1.0);

                    // Determine the camera distance
                    float camera_distance = length(pos_camera);
                    float morph_factor = max(0,min(1,(camera_distance-morph_range.x)/(morph_range.y-morph_range.x)));

                    // Determine the actual position and local texcoords of the vertex based on the morph factor
                    vec2 morphed_local_texcoords = local_texcoords - fract(local_texcoords * (vertices_per_patch-1) * 0.5) * 2.0 / vertices_per_patch * morph_factor;
                    vec2 morphed_position = mix(position, position_morph_target, morph_factor);

                    // Construct the patch local coordinates and transform them to camera space
                    vec3 morphed_pos_patch = vec3(morphed_position, sample_height(morphed_local_texcoords));
                    vec4 morphed_pos_camera = pose_camera*vec4(morphed_pos_patch, 1.0);

                    // Project to the screen and apply logarithmic depth buffer
                    // https://outerra.blogspot.com/2012/11/maximizing-depth-buffer-range-and.html
                    gl_Position = view_projection*morphed_pos_camera;
                    const float far_constant = 1.0/log(camera_far*camera_log_z_constant + 1);
                    LogZ = log(gl_Position.w*camera_log_z_constant + 1)*far_constant;
                    gl_Position.z = (2*LogZ - 1)*gl_Position.w;

                    Texcoords = morphed_local_texcoords;
                    AtlasIndex = atlas_index;
                    //Color = vec4(mix(random_colors[lod_level], random_colors[lod_level+1], morph_factor), 1);
                    Color = vec4(color,1);
                    MorphFactor = morph_factor;
                }
            "#;

            let fragment_shader_src = r#"
                #version 430 core
                #extension GL_EXT_texture_array : enable
                #extension GL_ARB_conservative_depth : enable

                in vec3 Normal;
                in vec2 Texcoords;
                in vec4 Color;
                flat in uint AtlasIndex;
                in float MorphFactor;
                in float LogZ;

                uniform sampler2DArray normal_atlas;

                out vec4 color;

                void main() {
                    // Logarithmic depth: https://outerra.blogspot.com/2012/11/maximizing-depth-buffer-range-and.html
	                gl_FragDepth = LogZ;

                    // Compute the modified texture coordinates
                    ivec2 texture_size = textureSize(normal_atlas, 0).xy;
                    ivec2 texture_size_low_detail = textureSize(normal_atlas, 1).xy;
                    vec2 texel_size = 1.0 / texture_size.xy;
                    vec2 texel_size_low_detail = 1.0 / texture_size_low_detail.xy;
                    vec2 normal_atlas_texcoords = Texcoords * (vec2(1.0, 1.0) - 2*texel_size) + texel_size*0.5;
                    vec2 normal_atlas_texcoords_low_detail = Texcoords * (vec2(1.0, 1.0) - texel_size_low_detail) + texel_size_low_detail*0.5;

                    // Sample the normal from the texture atlas
                    vec3 normal_high_detail = texture2DArrayLod(normal_atlas, vec3(normal_atlas_texcoords, AtlasIndex), 0).xyz;
                    vec3 normal_low_detail = texture2DArrayLod(normal_atlas, vec3(normal_atlas_texcoords_low_detail, AtlasIndex), 1).xyz;
                    vec3 normal = normalize(mix(normal_high_detail, normal_low_detail, MorphFactor));

                    float nDotL = max(0, dot(normal, vec3(1,0,0)));
                    color = vec4(vec3(nDotL), 1.0) * Color;
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
        split_distances.push(1.0);
        let mut last_value = 2.0;
        for _i in 0..max_lod_level {
            let split_amount = 2.0;
            split_distances.push(last_value * split_amount);
            last_value *= split_amount;
        }
        split_distances.reverse();

        let mut backing = NodeBacking::new(facade)?;

        fn generate_face(
            backing: &mut NodeBacking,
            face: planet::Face,
            geometry_provider: &planet::GeometryProvider,
        ) -> Face {
            Face {
                face,
                root: QuadTree::new(Node::WithGeometry(NodeGeometry::new(
                    backing,
                    &geometry_provider.compute_geometry(face.into()),
                ))),
            }
        }

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
            per_visible_node_buffer: RefCell::new(VertexBuffer::empty_persistent(
                facade,
                MAX_PATCH_COUNT,
            )?),
            command_buffer: RefCell::new(DrawCommandsIndicesBuffer::empty_persistent(
                facade,
                MAX_PATCH_COUNT,
            )?),
            pending_geometry_requests: PendingStreamingNodesMap::new(),
        })
    }

    /// Draws the planet to the screen from the perspective of the given frustum.
    /// * `frame` - The frame to render to
    /// * `frustum` - The frustum that represents the view to render from in world space.
    /// * `planet_world_transform` - The transformation of the planet relative to the world.
    pub fn draw<S: Surface>(
        &self,
        frame: &mut S,
        frustum: &Frustum,
        planet_world_transform: &Transform,
        draw_parameters: &DrawParameters,
    ) {
        // Construct a new frustum relative to the planet to ease computations
        let frustum_planet = frustum.relative_to(planet_world_transform);

        // Projection frustum is used for the final projection in the shader. This frustum is
        // basically `frustum_planet` but without the translation. This is because the translation
        // of the camera is already encoded in the patches themselves. This way we get maximum
        // precision near the camera position.
        let projection_frustum = frustum.with_transform(Transform::from_parts(
            Translation3::identity(),
            frustum_planet.transform.rotation,
        ));

        // Construct the cone for horizon culling
        let horizon_cone = horizon_culling::Cone::new(
            Point3::from_coordinates(frustum_planet.transform.translation.vector),
            self.description.radius,
        );

        // Query all faces for visible nodes
        let visible_nodes: VecDeque<VisibleNode> = {
            let mut lod_select: LODSelectHelper =
                LODSelectHelper::new(&frustum_planet, &horizon_cone, &self.split_distances);
            for face in self.faces.iter() {
                lod_select.select(&face.root, &face.face.into());
            }
            lod_select.result()
        };

        // Setup all uniforms for drawing
        let uniforms = uniform! {
            view_projection: Into::<[[f32; 4]; 4]>::into(projection_frustum.view_projection),
            vertices_per_patch: planet::constants::VERTICES_PER_PATCH as u32,
            height_atlas: self.backing.heights.texture.sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
            normal_atlas: self.backing.normals.texture.sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
            camera_far: frustum.far_distance
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
                glium::PolygonMode::Fill
            },
            ..Default::default()
        };

        // Generate per node instance data
        {
            let mut node_instance_data = self.per_visible_node_buffer.borrow_mut();
            let mut mapping = node_instance_data.map_write();
            for (idx, node) in visible_nodes.iter().enumerate() {
                mapping.set(
                    idx,
                    PerNodeInstanceVertex {
                        pose_camera: node.transform_camera.into(),
                        atlas_index: self.backing.atlas_index(node.node.node_id),
                        morph_range: node.morph_range,
                        lod_level: node.lod_level,
                    },
                )
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
                        quad_tree::Child::TopLeft => (0, index_count / 4),
                        quad_tree::Child::TopRight => (index_count / 4, index_count / 4),
                        quad_tree::Child::BottomLeft => (index_count / 4 * 2, index_count / 4),
                        quad_tree::Child::BottomRight => (index_count / 4 * 3, index_count / 4),
                    },
                };
                mapping.set(
                    idx,
                    DrawCommandIndices {
                        count,
                        instance_count: 1,
                        first_index,
                        base_vertex: self.backing.vertices.base_vertex(node.node.node_id),
                        base_instance: idx as u32,
                    },
                )
            }
        }

        let command_buffer = self.command_buffer.borrow();
        let command_buffer_slice = command_buffer.slice(0..visible_nodes.len()).unwrap();
        let node_instance_data = self.per_visible_node_buffer.borrow();
        frame
            .draw(
                (
                    &self.backing.vertices.vertex_buffer,
                    node_instance_data.per_instance().unwrap(),
                ),
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

    /// Ensures that all the nodes in range of the frustum are either in a pending state or contain
    /// geometry to be rendered.
    pub fn ensure_resident_patches(
        &mut self,
        frustum: &Frustum,
        planet_world_transform: &Transform,
    ) {
        // Compute the frustum relative to the planet
        let frustum_planet = frustum.relative_to(planet_world_transform);

        // Ensure residency of all faces
        for face in self.faces.iter_mut() {
            ensure_resident_children(
                &mut self.backing,
                &mut self.pending_geometry_requests,
                &frustum_planet,
                &self.geometry_provider,
                &mut face.root,
                face.face.into(),
                true,
                &self.split_distances,
            );
        }

        // Process streaming results
        let backing = &mut self.backing;
        let pending_requests = &mut self.pending_geometry_requests;
        self.geometry_provider.receive_all(|id, data| {
            if let Some(node) = pending_requests.get(&id) {
                let node_geometry = Node::WithGeometry(NodeGeometry::new(backing, &data));
                unsafe {
                    (**node).content = node_geometry;
                }
                pending_requests.remove(&id);
            }
        });
    }

    pub fn set_generator(&mut self, geometry_provider: T,) {
        self.geometry_provider = geometry_provider;
        for face in self.faces.iter_mut() {
            remove_face(
                &mut self.backing,
                &mut face.root,
                &mut self.pending_geometry_requests,
            );
            face.root = QuadTree::new(Node::WithGeometry(NodeGeometry::new(
                &mut self.backing,
                &self.geometry_provider.compute_geometry(face.face.into()),
            )));
        }
    }

    /// Returns the context corresponding to this Renderer.
    pub fn get_context(&self) -> &Rc<Context> {
        &self.context
    }
}

/// Ensures that all children within range of the frustum are either loaded or in a pending state.
/// When a node is not yet present it will be queued for generation.
fn ensure_resident_children<T: planet::AsyncGeometryProvider>(
    backing: &mut NodeBacking,
    pending_requests: &mut PendingStreamingNodesMap,
    frustum_planet: &Frustum,
    geometry_provider: &T,
    node: &mut QuadTree<Node>,
    location: PatchLocation,
    parent_in_frustum: bool,
    split_distances: &[f64],
) {
    // Do not split the last LOD level.
    if location.lod_level >= split_distances.len() {
        return;
    }

    // If this node contains geometry check if it needs to be split.
    if let Node::WithGeometry(ref geometry) = node.content {
        // If the node is out of range of it's split distance, remove it's children.
        let frustum_pos = Point3::from_coordinates(frustum_planet.transform.translation.vector);
        if !in_range(
            &geometry.bounding_box(),
            &frustum_pos,
            split_distances[location.lod_level],
        ) {
            merge(backing, node, pending_requests);
            return;
        }

        let in_frustum = frustum_planet.intersects(&geometry.aabb);

        // Otherwise; ensure that this node has children resident
        if !node.has_children() {
            node.children = Some([
                request_node(geometry_provider, pending_requests, location.top_left()),
                request_node(geometry_provider, pending_requests, location.top_right()),
                request_node(geometry_provider, pending_requests, location.bottom_left()),
                request_node(geometry_provider, pending_requests, location.bottom_right()),
            ])
        }

        if let Some(ref mut children) = node.children {
            for child in quad_tree::Child::values() {
                ensure_resident_children(
                    backing,
                    pending_requests,
                    frustum_planet,
                    geometry_provider,
                    &mut (*children)[child.index()],
                    location.split(*child),
                    in_frustum,
                    split_distances,
                );
            }
        }
    } else if let Node::Pending(_, ref token) = node.content {
        token.priority.store(
            location.lod_level | if parent_in_frustum { 512 } else { 0 },
            Ordering::SeqCst,
        );
    }

    /// A helper method to create a new quad tree node and queue it to the geometry provider.
    fn request_node<T: planet::AsyncGeometryProvider>(
        geometry_provider: &T,
        pending_requests: &mut PendingStreamingNodesMap,
        location: PatchLocation,
    ) -> Box<QuadTree<Node>> {
        let (token, id) = geometry_provider.queue(location);
        token
            .priority
            .store(location.lod_level + 1, Ordering::SeqCst);
        let node_ptr = Box::into_raw(Box::new(QuadTree::new(Node::Pending(id, token))));
        pending_requests.insert(id, node_ptr);
        unsafe { Box::from_raw(node_ptr) }
    }
}

/// Removes all state associated with the planet state
fn remove_face(
    backing: &mut NodeBacking,
    node: &mut QuadTree<Node>,
    pending_requests: &mut PendingStreamingNodesMap,
) {
    merge(backing, node, pending_requests);
    match &node.content {
        Node::Pending(id, token) => {
            token.priority.store(0, Ordering::SeqCst); // Priority 0 = cancelled
            pending_requests.remove(&id);
        }
        Node::WithGeometry(geometry) => {
            backing.release(geometry.node_id);
        }
    }
}

/// Given a QuadTree Node, destroy all its children and clean up after them.
fn merge(
    backing: &mut NodeBacking,
    node: &mut QuadTree<Node>,
    pending_requests: &mut PendingStreamingNodesMap,
) {
    if let Some(ref mut children) = node.children {
        for node in children.iter_mut() {
            remove_face(backing, node, pending_requests);
        }
    }
    node.children = None;
}

/// Determines which part of a node should be rendered.
#[derive(Copy, Clone, PartialEq)]
enum VisibleNodePart {
    /// Render the entire node
    Whole,

    /// Render only a quarter of a node
    Child(quad_tree::Child),
}

/// Holds one node that can be rendered.
struct VisibleNode<'a> {
    pub node: &'a NodeGeometry,
    pub transform_camera: Matrix4<f32>,
    pub part: VisibleNodePart,
    pub morph_range: (f32, f32),
    pub lod_level: u16,
}

/// Defines the result of calling `lod_select` on a node.
#[derive(Copy, Clone, PartialEq)]
enum LODSelectResult {
    /// Undefined value (patch doesn't exist)
    Undefined,

    /// The patch is outside of the frustum
    OutOfFrustum,

    /// The patch is outside of its lod range
    OutOfRange,

    /// The patch was selected
    Selected,

    /// The node has a pending streaming request
    Pending,
}

impl LODSelectResult {
    /// Returns true if the result indicates that the node was not added to the visible list.
    fn is_not_selected(&self) -> bool {
        match self {
            LODSelectResult::Undefined => true,
            LODSelectResult::OutOfFrustum => false,
            LODSelectResult::OutOfRange => true,
            LODSelectResult::Selected => false,
            LODSelectResult::Pending => true,
        }
    }
}

/// A helper struct that is used to select LOD levels of quad tree nodes.
struct LODSelectHelper<'a> {
    frustum_planet: Frustum,
    frustum_pos: Point3<f64>,
    cone: horizon_culling::Cone<f64>,
    split_distances: &'a [f64],
    result: VecDeque<VisibleNode<'a>>,
}

impl<'a> LODSelectHelper<'a> {
    /// Construct a new LOD selection helper
    /// * `frustum_planet` - The frustum relative to the planet
    /// * `cone` - The horizon culling cone relative to the planet
    /// * `split_distances` - For every lod level at which distance its children should be used instead.
    pub fn new(
        frustum_planet: &Frustum,
        cone: &horizon_culling::Cone<f64>,
        split_distances: &'a [f64],
    ) -> LODSelectHelper<'a> {
        LODSelectHelper {
            frustum_planet: frustum_planet.clone(),
            frustum_pos: Point3::from_coordinates(frustum_planet.transform.translation.vector),
            cone: cone.clone(),
            split_distances,
            result: VecDeque::new(),
        }
    }

    /// Returns the result of the LOD selection process consuming the helper.
    pub fn result(self) -> VecDeque<VisibleNode<'a>> {
        self.result
    }

    /// Select the appropriate LOD levels of the specified node.
    pub fn select(&mut self, node: &'a QuadTree<Node>, location: &PatchLocation) {
        self.recurse(node, location, false);
    }

    /// Adds the geometry of the entire node to the result
    fn add_whole(&mut self, node: &'a NodeGeometry, location: &PatchLocation) {
        self.add(node, location, VisibleNodePart::Whole)
    }

    /// Adds only a part of the geometry of the node to the result
    fn add_part(
        &mut self,
        node: &'a NodeGeometry,
        location: &PatchLocation,
        child: quad_tree::Child,
    ) {
        self.add(node, location, VisibleNodePart::Child(child))
    }

    /// Adds a specific part of the geometry of a node to the result. Use the `add_whole` and
    /// `add_part` methods for easier use.
    fn add(&mut self, node: &'a NodeGeometry, location: &PatchLocation, part: VisibleNodePart) {
        let node_camera = Translation3::from_vector(node.origin - self.frustum_pos)
            .to_homogeneous()
            * node.transform;

        let current_split_depth = *self.split_distances.get(location.lod_level).unwrap_or(&0.0);
        let previous_split_depth = *self
            .split_distances
            .get(location.lod_level.wrapping_sub(1))
            .unwrap_or(&current_split_depth);
        let split_depth = current_split_depth + (previous_split_depth - current_split_depth) * 0.9;

        self.result.push_back(VisibleNode {
            node,
            transform_camera: nalgebra::convert(node_camera),
            part,
            morph_range: (split_depth as f32, previous_split_depth as f32),
            lod_level: location.lod_level as u16,
        })
    }

    /// Recurse into the specified node adding all visible nodes to the LOD selection result.
    fn recurse(
        &mut self,
        node: &'a QuadTree<Node>,
        location: &PatchLocation,
        parent_completely_in_frustum: bool,
    ) -> LODSelectResult {
        use crate::culling::{Classify, Containment};

        if let Node::WithGeometry(ref geometry) = node.content {
            // Determine whether this node is at least intersecting the frustum.
            let frustum_containment = if parent_completely_in_frustum {
                Containment::Inside
            } else {
                self.frustum_planet.classify(&geometry.bounding_box())
            };
            if frustum_containment == Containment::Outside {
                // The node completely outside of the frustum.
                return LODSelectResult::OutOfFrustum;
            }

            // Perform horizon culling by checking if the node is outside of the horizon cone.
            if self.cone.contains(&geometry.bounding_box()) {
                return LODSelectResult::OutOfFrustum;
            }

            // Check if the node is within the split distance of its parent lod level. If this is not
            // the case the geometry of the parent is used instead of the high detailed version.
            // Don't do this for the highest (lowest detail) lod level because it doesn't have a parent.
            if location.lod_level > 0
                && !in_range(
                    &geometry.bounding_box(),
                    &self.frustum_pos,
                    self.split_distances[location.lod_level - 1],
                )
            {
                return LODSelectResult::OutOfRange;
            }

            // Check if this node should be split into it's children by checking the distance from the
            // camera to the node. If it is within split distance traverse its children.
            if location.lod_level < self.split_distances.len()
                && in_range(
                    &geometry.bounding_box(),
                    &self.frustum_pos,
                    self.split_distances[location.lod_level],
                )
            {
                let mut children_selection_results = [LODSelectResult::Undefined; 4];

                // Recurse into the children capturing the selection result.
                if let Some(ref children) = node.children {
                    for child in quad_tree::Child::values() {
                        children_selection_results[child.index()] = self.recurse(
                            &(*children)[child.index()],
                            &location.split(*child),
                            frustum_containment == Containment::Inside,
                        );
                    }
                }

                // If non of the nodes was selected because they either lack geometry or where out of
                // range we use the geometry of this node. The entire node is added to the list.
                if children_selection_results
                    .iter()
                    .all(LODSelectResult::is_not_selected)
                {
                    // If the node has no children, we'll add it anyway
                    self.add_whole(geometry, location);
                    return LODSelectResult::Selected;
                }

                // If any of the nodes is not selected because it has no geometry or because it's out of
                // range, fill it in with geometry from the parent node.
                for child in quad_tree::Child::values()
                    .filter(|c| children_selection_results[c.index()].is_not_selected())
                {
                    self.add_part(geometry, location, *child);
                }

                if children_selection_results
                    .iter()
                    .any(|s| *s == LODSelectResult::Selected)
                {
                    LODSelectResult::Selected
                } else {
                    LODSelectResult::OutOfFrustum
                }
            } else {
                // If the node has no children, we'll add it anyway
                self.add_whole(geometry, location);
                LODSelectResult::Selected
            }
        } else {
            LODSelectResult::Pending
        }
    }
}

/// Performs a AABB circle collision check to see if the AABB is within a certain distance of a
/// point.
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

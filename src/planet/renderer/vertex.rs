#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub position_morph_target: [f32; 2],
    pub local_texcoords: [f32; 2],
    pub color: [f32; 3],
}

implement_vertex!(Vertex, position, position_morph_target, local_texcoords, color);

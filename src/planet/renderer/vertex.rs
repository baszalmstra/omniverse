#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}

implement_vertex!(Vertex, position);
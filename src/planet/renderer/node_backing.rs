use glium::{VertexBuffer};
use glium::backend::Facade;
use std::boxed::Box;
use super::Vertex;
use crate::id_arena::{IdGenerator, SimpleIdArena};
use glium::vertex::VertexBufferSlice;
use glium::buffer::BufferMutSlice;

#[derive(Debug, Copy, Clone)]
pub struct NodeId(usize);

/// Stores geometry information for a fixed number of patches
pub struct GeometryBuffer<T:glium::Vertex> {
    pub vertex_buffer: VertexBuffer<T>,
    vertices_per_patch: usize,
}

impl<T:glium::Vertex> GeometryBuffer<T>
where
    [T]: glium::buffer::Content,
    T: Copy {
    fn new<F: ?Sized + Facade>(facade: &F, patch_count: usize, vertices_per_patch: usize) -> Result<GeometryBuffer<T>, Box<std::error::Error>> {
        Ok(GeometryBuffer {
            vertex_buffer: VertexBuffer::empty_dynamic(facade, patch_count*vertices_per_patch)?,
            vertices_per_patch
        })
    }

    pub fn slice(&self, id:NodeId) -> VertexBufferSlice<T> {
        let start = id.0 * self.vertices_per_patch;
        let end = start + self.vertices_per_patch;
        self.vertex_buffer.slice(start .. end )
            .expect("Unable to slice GeometryBuffer")
    }

    pub fn slice_mut(&mut self, id:NodeId) -> BufferMutSlice<[T]> {
        let start = id.0 * self.vertices_per_patch;
        let end = start + self.vertices_per_patch;
        self.vertex_buffer.slice_mut(start .. end )
            .expect("Unable to slice GeometryBuffer")
    }
}

pub struct NodeBacking {
    id_generator: SimpleIdArena,
    pub vertices: GeometryBuffer<Vertex>
}

impl NodeBacking {
    pub fn new<F: ?Sized + Facade>(facade: &F) -> Result<NodeBacking, Box<std::error::Error>> {
        use planet::constants::{VERTICES_PER_PATCH, MAX_PATCH_COUNT};
        Ok(NodeBacking {
            id_generator: SimpleIdArena::with_capacity(MAX_PATCH_COUNT),
            vertices: GeometryBuffer::new(facade, MAX_PATCH_COUNT, VERTICES_PER_PATCH*VERTICES_PER_PATCH)?
        })
    }

    pub fn acquire(&mut self) -> NodeId {
        self.id_generator.acquire()
            .map(NodeId)
            .expect("There are no more node id's available.")
    }

    pub fn release(&mut self, id:NodeId) {
        self.id_generator.release(id.0);
    }
}
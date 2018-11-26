use super::Vertex;
use crate::id_arena::{IdGenerator, SimpleIdArena};
use glium::backend::Facade;
use glium::buffer::BufferMutSlice;
use glium::texture::pixel_buffer::PixelBuffer;
use glium::texture::MipmapsOption;
use glium::texture::UncompressedFloatFormat;
use glium::texture::{PixelValue, Texture2dArray};
use glium::vertex::VertexBufferSlice;
use glium::VertexBuffer;
use std::boxed::Box;

#[derive(Debug, Copy, Clone)]
pub struct NodeId(usize);

pub struct TextureAtlas<P: PixelValue> {
    pub texture: Texture2dArray,
    upload_buffer: PixelBuffer<P>,
    pixels_per_patch: u32,
}

impl<T: PixelValue> TextureAtlas<T> {
    fn new<F: ?Sized + Facade>(
        facade: &F,
        format: UncompressedFloatFormat,
        patch_count: usize,
        pixels_per_patch: usize,
    ) -> Result<TextureAtlas<T>, Box<std::error::Error>> {
        Ok(TextureAtlas {
            texture: Texture2dArray::empty_with_format(
                facade,
                format,
                MipmapsOption::NoMipmap,
                pixels_per_patch as u32,
                pixels_per_patch as u32,
                patch_count as u32,
            )?,
            pixels_per_patch: pixels_per_patch as u32,
            upload_buffer: PixelBuffer::new_empty(facade, pixels_per_patch * pixels_per_patch),
        })
    }

    pub fn write(&mut self, id: NodeId, mip_level: u32, data: &[T]) {
        assert_eq!(
            ((self.pixels_per_patch >> mip_level) * (self.pixels_per_patch >> mip_level)) as usize,
            data.len()
        );
        self.upload_buffer.write(data);
        self.texture
            .mipmap(mip_level)
            .unwrap()
            .raw_upload_from_pixel_buffer(
                self.upload_buffer.as_slice(),
                0..self.pixels_per_patch,
                0..self.pixels_per_patch,
                id.0 as u32..id.0 as u32 + 1,
            );
        self.upload_buffer.invalidate();
    }
}

/// Stores geometry information for a fixed number of patches
pub struct GeometryBuffer<T: glium::Vertex> {
    pub vertex_buffer: VertexBuffer<T>,
    vertices_per_patch: usize,
}

impl<T: glium::Vertex> GeometryBuffer<T>
where
    [T]: glium::buffer::Content,
    T: Copy,
{
    fn new<F: ?Sized + Facade>(
        facade: &F,
        patch_count: usize,
        vertices_per_patch: usize,
    ) -> Result<GeometryBuffer<T>, Box<std::error::Error>> {
        Ok(GeometryBuffer {
            vertex_buffer: VertexBuffer::empty_dynamic(facade, patch_count * vertices_per_patch)?,
            vertices_per_patch,
        })
    }

    pub fn slice(&self, id: NodeId) -> VertexBufferSlice<T> {
        let start = id.0 * self.vertices_per_patch;
        let end = start + self.vertices_per_patch;
        self.vertex_buffer
            .slice(start..end)
            .expect("Unable to slice GeometryBuffer")
    }

    pub fn slice_mut(&mut self, id: NodeId) -> BufferMutSlice<[T]> {
        let start = id.0 * self.vertices_per_patch;
        let end = start + self.vertices_per_patch;
        self.vertex_buffer
            .slice_mut(start..end)
            .expect("Unable to slice GeometryBuffer")
    }

    pub fn base_vertex(&self, id: NodeId) -> u32 {
        (id.0 * self.vertices_per_patch) as u32
    }

    pub fn write(&self, id: NodeId, data: &[T]) {
        self.slice(id).write(data)
    }
}

pub struct NodeBacking {
    id_generator: SimpleIdArena,
    pub vertices: GeometryBuffer<Vertex>,
    pub heights: TextureAtlas<f32>,
    pub normals: TextureAtlas<(f32, f32, f32)>,
}

impl NodeBacking {
    pub fn new<F: ?Sized + Facade>(facade: &F) -> Result<NodeBacking, Box<std::error::Error>> {
        use planet::constants::{MAX_PATCH_COUNT, NORMALS_PER_PATCH, VERTICES_PER_PATCH};
        Ok(NodeBacking {
            id_generator: SimpleIdArena::with_capacity(MAX_PATCH_COUNT),
            vertices: GeometryBuffer::new(
                facade,
                MAX_PATCH_COUNT,
                VERTICES_PER_PATCH * VERTICES_PER_PATCH,
            )?,
            heights: TextureAtlas::new(
                facade,
                UncompressedFloatFormat::F32,
                MAX_PATCH_COUNT,
                VERTICES_PER_PATCH,
            )?,
            normals: TextureAtlas::new(
                facade,
                UncompressedFloatFormat::F32F32F32,
                MAX_PATCH_COUNT,
                NORMALS_PER_PATCH,
            )?,
        })
    }

    pub fn acquire(&mut self) -> NodeId {
        self.id_generator
            .acquire()
            .map(NodeId)
            .expect("There are no more node id's available.")
    }

    pub fn release(&mut self, id: NodeId) {
        self.id_generator.release(id.0);
    }

    pub fn atlas_index(&self, id: NodeId) -> u32 {
        id.0 as u32
    }
}

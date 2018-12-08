pub const VERTICES_PER_PATCH: usize = 32 + 1; // Power of two + 1 for overlap with the next patch
pub const NORMALS_RESOLUTION: usize = 2;      // Must be a power of two and at least 2
pub const NORMALS_PER_PATCH: usize = VERTICES_PER_PATCH * NORMALS_RESOLUTION;
pub const MAX_PATCH_COUNT: usize = 2048;

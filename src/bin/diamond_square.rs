const FACE_SIZE: usize = 1024;
const HALF_FACE_SIZE: usize = FACE_SIZE / 2;
const TWO_FACE_SIZE: usize = FACE_SIZE * 2;

enum FaceTriangle {
    Top = 0,
    Left = 1,
    Bottom = 2,
    Right = 3
}

enum Face {
    Front = 0,
    Right = 1,
    Back = 2,
    Left = 3,
    Top = 4,
    Bottom = 5,
}

const TRI_ROT_MATRICES:[[i32;4];4] = [
    [ 1, 0, 0, 1],
    [ 0,-1, 1, 0],
    [-1, 0, 0,-1],
    [ 0, 1,-1, 0]
];

const INV_TRI_ROT_MATRICES:[[i32;4];4] = [
    [ 1, 0, 0, 1],
    [ 0, 1,-1, 0],
    [-1, 0, 0,-1],
    [ 0,-1, 1, 0],
];

static FACE_TRIANGLES:[[(Face,FaceTriangle);4];6] = [
    [(Face::Top,   FaceTriangle::Bottom),   (Face::Right, FaceTriangle::Left),   (Face::Bottom, FaceTriangle::Top),     (Face::Left,  FaceTriangle::Right)],    // Front
    [(Face::Top,   FaceTriangle::Right),    (Face::Back,  FaceTriangle::Left),   (Face::Bottom, FaceTriangle::Right),   (Face::Front, FaceTriangle::Right)],    // Right
    [(Face::Top,   FaceTriangle::Top),      (Face::Left,  FaceTriangle::Left),   (Face::Bottom, FaceTriangle::Bottom),  (Face::Right, FaceTriangle::Right)],    // Back
    [(Face::Top,   FaceTriangle::Left),     (Face::Front, FaceTriangle::Left),   (Face::Bottom, FaceTriangle::Left),    (Face::Back,  FaceTriangle::Right)],    // Left
    [(Face::Back,  FaceTriangle::Top),      (Face::Right, FaceTriangle::Top),    (Face::Front,  FaceTriangle::Top),     (Face::Left,  FaceTriangle::Top)],      // Top
    [(Face::Front, FaceTriangle::Bottom),   (Face::Right, FaceTriangle::Bottom), (Face::Back,   FaceTriangle::Bottom),  (Face::Left,  FaceTriangle::Bottom)],   // Bottom
];

fn main() {
    let mut faces = [[0.0; FACE_SIZE * FACE_SIZE]; 6];
    let mut face_indirection = [[0_usize; (FACE_SIZE * 2) * (FACE_SIZE * 2)]; 6];

    for face in 0..6 {

        // Fill the center indirection
        for y in 0..FACE_SIZE {
            for x in 0..FACE_SIZE {
                face_indirection[face]
                    [(y + HALF_FACE_SIZE) * (TWO_FACE_SIZE) + x + HALF_FACE_SIZE] =
                    y * FACE_SIZE + x;
            }
        }

        // Top


        // Right

        // Bottom

        // Left
    }
}

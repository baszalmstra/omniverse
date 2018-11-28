use image::ImageBuffer;
use omniverse::planet::Face;

#[derive(Debug, Copy, Clone, PartialEq)]
struct CubeCoord {
    pub x: i32,
    pub y: i32,
    pub face: Face,
}

fn face_to_primitive(face:Face) -> u8 {
    match face {
        Face::Left => 3,
        Face::Right => 1,
        Face::Top => 4,
        Face::Bottom => 5,
        Face::Front => 0,
        Face::Back => 2,
    }
}

fn primitive_to_face(face: i32) -> Face {
    match face {
        3 => Face::Left,
        1 => Face::Right,
        4 => Face::Top,
        5 => Face::Bottom,
        0 => Face::Front,
        2 => Face::Back,
        _ => unreachable!()
    }
}

impl CubeCoord {
    fn new(x: i32, y: i32, face: Face) -> CubeCoord {
        CubeCoord{x, y, face}
    }

    fn add_dx(self, dx: i32, size: i32) -> CubeCoord {
        debug_assert!(dx.abs() <= size);

        let x_new = self.x + dx;
        if x_new >= 0 && x_new < size {
            CubeCoord {
                x: x_new,
                y: self.y,
                face: self.face
            }
        } else {
            match self.face {
                Face::Left | Face::Right | Face::Front | Face::Back => {
                    let face_idx = face_to_primitive(self.face);
                    CubeCoord {
                        x: (x_new + size) % size,
                        y: self.y,
                        face: primitive_to_face(((x_new + 4 * size) / size + face_idx as i32) % 4)
                    }
                },
                Face::Top => {
                    if x_new < 0 {
                        CubeCoord {
                            x: self.y,
                            y: -x_new,
                            face: Face::Left
                        }
                    } else {
                        CubeCoord {
                            x: size - self.y,
                            y: x_new - size,
                            face: Face::Right
                        }
                    }
                },
                Face::Bottom => {
                    if x_new < 0 {
                        CubeCoord {
                            x: size - self.y,
                            y: size + x_new,
                            face: Face::Left
                        }
                    } else {
                        CubeCoord {
                            x: self.y,
                            y: 2 * size - x_new - 1,
                            face: Face::Right
                        }
                    }
                },
            }
        }
    }

    fn add_dy(self, dy: i32, size: i32) -> CubeCoord {
        debug_assert!(dy.abs() <= size);

        let y_new = self.y + dy;
        if y_new >= 0 && y_new < size {
            CubeCoord {
                x: self.x,
                y: y_new,
                face: self.face
            }
        } else {
            match self.face {
                Face::Front => {
                    if y_new < 0 {
                        CubeCoord {
                            x: self.x,
                            y: size + y_new,
                            face: Face::Top,
                        }
                    } else {
                        CubeCoord {
                            x: self.x,
                            y: y_new - size,
                            face: Face::Bottom,
                        }
                    }
                },
                Face::Right => {
                    if y_new < 0 {
                        CubeCoord {
                            x: size + y_new,
                            y: size - self.x,
                            face: Face::Top,
                        }
                    } else {
                        CubeCoord {
                            x: 2 * size - y_new - 1,
                            y: self.x,
                            face: Face::Bottom,
                        }
                    }
                },
                Face::Back => {
                    if y_new < 0 {
                        CubeCoord {
                            x: size - self.x,
                            y: -y_new,
                            face: Face::Top,
                        }
                    } else {
                        CubeCoord {
                            x: size - self.x,
                            y: 2 * size - y_new - 1,
                            face: Face::Bottom,
                        }
                    }
                },
                Face::Left => {
                    if y_new < 0 {
                        CubeCoord {
                            x: -y_new,
                            y: self.x,
                            face: Face::Top,
                        }
                    } else {
                        CubeCoord {
                            x: y_new - size,
                            y: size - self.x,
                            face: Face::Bottom,
                        }
                    }
                },
                Face::Top => {
                    if y_new < 0 {
                        CubeCoord {
                            x: size - self.x,
                            y: -y_new,
                            face: Face::Back,
                        }
                    } else {
                        CubeCoord {
                            x: self.x,
                            y: y_new - size,
                            face: Face::Front,
                        }
                    }
                },
                Face::Bottom => {
                    if y_new < 0 {
                        CubeCoord {
                            x: self.x,
                            y: size + y_new,
                            face: Face::Front,
                        }
                    } else {
                        let result = CubeCoord {
                            x: size - self.x,
                            y: 2 * size - y_new - 1,
                            face: Face::Back,
                        };
                        println!("{:?}", result);
                        result
                    }
                },
            }
        }
    }
}

fn main() {
    //

    let size = 512;

    let mut vec = [
        vec![0.0; size * size],
        vec![0.0; size * size],
        vec![0.0; size * size],
        vec![0.0; size * size],
        vec![0.0; size * size],
        vec![0.0; size * size]];

    let mut p = CubeCoord::new(100, 200,  Face::Top);

    for i in 0..size * 3{
        vec[face_to_primitive(p.face) as usize][(p.y * size as i32 + p.x) as usize] = 1.0;
        p = p.add_dy(1, size as i32);
}


    for i in 0..6 {
        let image = ImageBuffer::from_fn(size as u32, size as u32, |x,y| {
            let idx = (y  * size as u32  +x) as usize;
            if vec[i][idx] > 0.0 {
                image::Rgb([255, 0, 0])
            } else {
                image::Rgb([0, 0, 0])
            }
        });

        image.save(format!("diamong_square_{:?}.png", primitive_to_face( i as i32)) ).unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_horizontal() {
        let size = 5;
        for face in Face::values() {
            let start = CubeCoord::new(1, 3, *face);
            let positive = start.add_dx(4, size);
            let negative = start.add_dx(-4, size);

            assert_eq!(start, positive.add_dx(-4, size));
            assert_eq!(start, negative.add_dx(4, size));
        }
    }

    #[test]
    fn test_vertical() {
        let size = 5;
        for face in Face::values() {
            let start = CubeCoord::new(1, 3, *face);
            let positive = start.add_dy(4, size);
            let negative = start.add_dy(-4, size);

            assert_eq!(start, positive.add_dy(-4, size));
            assert_eq!(start, negative.add_dy(4, size));
        }
    }
}

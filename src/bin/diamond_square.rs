use image::ImageBuffer;
use omniverse::planet::Face;
use rand::Rng;

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
                            x: size - self.y - 1,
                            y: x_new - size,
                            face: Face::Right
                        }
                    }
                },
                Face::Bottom => {
                    if x_new < 0 {
                        CubeCoord {
                            x: size - self.y - 1,
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
                            y: size - self.x - 1,
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
                            x: size - self.x - 1,
                            y: -y_new,
                            face: Face::Top,
                        }
                    } else {
                        CubeCoord {
                            x: size - self.x - 1,
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
                            y: size - self.x - 1,
                            face: Face::Bottom,
                        }
                    }
                },
                Face::Top => {
                    if y_new < 0 {
                        CubeCoord {
                            x: size - self.x - 1,
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
                            x: size - self.x - 1,
                            y: 2 * size - y_new - 1,
                            face: Face::Back,
                        };
                        result
                    }
                },
            }
        }
    }
}

fn diamond_step<D>(vec: &mut [Vec<f64>; 6], step: i32, size: i32, rnd: &D)
    where D: rand::distributions::Distribution<f64>
{
    for i in 0..4 {

        for y in (step..size).step_by(2 * step as usize) {
            for x in (step..size).step_by(2 * step as usize) {
                let p = CubeCoord::new(x, y, primitive_to_face(i));

                let p1 = p.add_dx(-step, size).add_dy(-step, size);
                let p2 = p.add_dx(step, size).add_dy(-step, size);
                let p3 = p.add_dx(-step, size).add_dy(step, size);
                let p4 = p.add_dx(step, size).add_dy(step, size);

                println!("DIAMOND");
                println!("p  = {:?}", p);
                println!("p1 = {:?}", p1);
                println!("p2 = {:?}", p2);
                println!("p3 = {:?}", p3);
                println!("p4 = {:?}", p4);

                let h = rand::thread_rng().sample(rnd) * step as f64;

                vec[i as usize][(p.y * size + p.x) as usize] = (
                    vec[i as usize][(p1.y * size + p1.x) as usize] +
                        vec[i as usize][(p2.y * size + p2.x) as usize] +
                        vec[i as usize][(p3.y * size + p3.x) as usize] +
                        vec[i as usize][(p4.y * size + p4.x) as usize]) / 4.0 + h;
            }
        }
    }

    for i in 4..6 {

        for y in (step..size).step_by(2 * step as usize) {
            for x in (step..size).step_by(2 * step as usize) {
                let p = CubeCoord::new(x, y, primitive_to_face(i));

                let p1 = p.add_dy(-step, size).add_dx(-step, size);
                let p2 = p.add_dy(step, size).add_dx(-step, size);
                let p3 = p.add_dy(-step, size).add_dx(step, size);
                let p4 = p.add_dy(step, size).add_dx(step, size);

                println!("DIAMOND");
                println!("p  = {:?}", p);
                println!("p1 = {:?}", p1);
                println!("p2 = {:?}", p2);
                println!("p3 = {:?}", p3);
                println!("p4 = {:?}", p4);

                let h = rand::thread_rng().sample(rnd) * step as f64;

                vec[i as usize][(p.y * size + p.x) as usize] = (
                    vec[i as usize][(p1.y * size + p1.x) as usize] +
                        vec[i as usize][(p2.y * size + p2.x) as usize] +
                        vec[i as usize][(p3.y * size + p3.x) as usize] +
                        vec[i as usize][(p4.y * size + p4.x) as usize]) / 4.0 + h;
            }
        }
    }
}

fn square_step<D>(vec: &mut [Vec<f64>; 6], step: i32, size: i32, rnd: &D)
    where D: rand::distributions::Distribution<f64>
{
    for i in 0..6 {

        for y in (0..size).step_by(step as usize) {
            for x in (0..size).step_by(step as usize) {
                if (x + y) / step % 2 == 0 {
                    continue;
                }

                let p = CubeCoord::new(x, y, primitive_to_face(i));
                let p1 = p.add_dx(-step, size);
                let p2 = p.add_dx(step, size);
                let p3 = p.add_dy(-step, size);
                let p4 = p.add_dy(step, size);

                println!("SQUARE");
                println!("p  = {:?}", p);
                println!("p1 = {:?}", p1);
                println!("p2 = {:?}", p2);
                println!("p3 = {:?}", p3);
                println!("p4 = {:?}", p4);

                let h = rand::thread_rng().sample(rnd) * step as f64;

                vec[i as usize][(p.y * size + p.x) as usize] = (
                    vec[i as usize][(p1.y * size + p1.x) as usize] +
                        vec[i as usize][(p2.y * size + p2.x) as usize] +
                        vec[i as usize][(p3.y * size + p3.x) as usize] +
                        vec[i as usize][(p4.y * size + p4.x) as usize]) / 4.0 + h;
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

    let rng = rand::distributions::Normal::new(0.0, 1.0);

    let mut step = size / 2;
    loop {
        diamond_step(&mut vec, step as i32, size as i32, &rng);
        square_step(&mut vec, step as i32, size as i32, &rng);
        if step == 1 {
            break;
        }

        break;
//        step = step / 2;
    }

    for i in 0..6 {
        let image = ImageBuffer::from_fn(size as u32, size as u32, |x,y| {
            let idx = (y  * size as u32  +x) as usize;
            let height = vec[i][idx] / 1000.0;
            let color = (height*128.0 + 128.0) as u8;
            image::Rgb([color, color,color])
        });

        image.save(format!("diamong_square_{:?}.png", primitive_to_face( i as i32)) ).unwrap();
    }
}

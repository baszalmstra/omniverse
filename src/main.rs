#[macro_use]
extern crate glium;
extern crate nalgebra;

use crate::camera::Camera;
use nalgebra::Vector3;
use crate::transform::Transformer;
use crate::camera_controller::CameraController;

mod planet;
mod planet_renderer;
mod camera;
mod camera_controller;
mod frustum;
mod transform;
mod timeline;

fn main() {
    use glium::glutin;
    use glium::Surface;

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Omniverse");
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
        }

        implement_vertex!(Vertex, position);

        let vertex1 = Vertex { position: [-0.5, 0.0, 0.0] };
        let vertex2 = Vertex { position: [0.0, 0.5, 0.0] };
        let vertex3 = Vertex { position: [0.5, 0.0, 0.0] };
        let shape = vec![vertex1, vertex2, vertex3];

        glium::VertexBuffer::new(&display, &shape).unwrap()
    };

    let program = {
        let vertex_shader_src = r#"
            #version 140

            in vec3 position;

            uniform mat4 viewProjection;
            uniform mat4 model;

            void main() {
                gl_Position = viewProjection*(model*vec4(position, 1.0));
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;

        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap()
    };

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let camera = Camera::new();
    camera.translate_by(&Vector3::new(0.0, 0.0, -2.0));

    let mut camera_controller = CameraController::new(&camera);

    let mut closed = false;
    let mut timeline = timeline::Timeline::new();
    let mut rotation:f32 = 0.0;
    while !closed {
        timeline.next_frame();

        println!("Delta time: {}", timeline.previous_frame_time()*1000.0);

        rotation += timeline.previous_frame_time();
        camera_controller.tick(timeline.previous_frame_time() );

        let mut frame = display.draw();
        let frame_size = frame.get_dimensions();
        let aspect_ratio = frame_size.0 as f32 / frame_size.1 as f32;
        let frustum = camera.frustum(aspect_ratio);
        frame.clear_color(0.0, 1.0, 0.0, 1.0);

        let triangle_uniforms = uniform! {
            viewProjection: Into::<[[f32; 4]; 4]>::into(frustum.view_projection),
            model: Into::<[[f32; 4]; 4]>::into(
                nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(0.0, 0.0, -2.0))*
                nalgebra::Matrix4::from_euler_angles(0.0,rotation,0.0))
        };

        frame.draw(&vertex_buffer, &indices, &program, &triangle_uniforms, &Default::default()).unwrap();
        frame.finish().unwrap();

        events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => closed = true,
                    glutin::WindowEvent::KeyboardInput { input, .. } => {
                        camera_controller.key_event(&input);
                    }
                    _ => (),
                },
                _ => (),
            }
        })
    }
}

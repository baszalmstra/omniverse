extern crate glium;
extern crate image;
extern crate nalgebra;
extern crate omniverse;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use omniverse::planet;

#[derive(Deserialize)]
struct Screenshots {
    samples: Option<u8>,
    size: Option<(u32, u32)>,
    screenshots: Vec<ScreenshotInfo>,
}

#[derive(Deserialize)]
struct ScreenshotInfo {
    name: String,
    position: (f64, f64, f64),
    rotation: (f64, f64, f64),

    #[serde(default)]
    draw_parameters: planet::DrawParameters,

    #[serde(default)]
    samples: Option<u8>,

    #[serde(default)]
    size: Option<(u32, u32)>,
}

fn main() {
    use glium::glutin;
    use glium::Surface;
    use image::GenericImage;
    use nalgebra::Vector3;
    use omniverse::camera::Camera;
    use omniverse::transform::{Rotation, Transform, Transformable};
    use std::path::Path;

    pretty_env_logger::init();

    let screenshots_content = std::fs::read_to_string(&Path::new("screenshots/screenshots.json"))
        .expect("Could not read screenshots.json");

    let screenshot_infos: Screenshots =
        serde_json::from_str(&screenshots_content).expect("Could not parse screenshots.json");

    /*let context = glutin::HeadlessRendererBuilder::new(1024, 1024)
        .build()
        .unwrap();
    let display = glium::HeadlessRenderer::new(context).unwrap();*/

    let events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_visibility(false);
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // Initialize a planet
    let planet_desc = planet::Description { radius: 1000.0 };
    let planet_transform = Transform::identity();
    let mut planet_renderer =
        planet::Renderer::new(&display, planet_desc, planet::Generator::new(planet_desc))
            .expect("Could not instantiate renderer");

    for screenshot in screenshot_infos.screenshots.iter() {
        let filename = format!("screenshots/{}.png", screenshot.name);
        info!("Creating {}..", filename);

        let samples = screenshot.samples.or(screenshot_infos.samples).unwrap_or(1);
        let size = screenshot
            .size
            .or(screenshot_infos.size)
            .unwrap_or((1280, 800));

        let frame_buffer_size = (size.0 * samples as u32, size.1 * samples as u32);
        let render_texture = glium::texture::SrgbTexture2d::empty(
            &display,
            frame_buffer_size.0,
            frame_buffer_size.1,
        )
        .unwrap();
        let depth_stencil_target = glium::texture::DepthTexture2d::empty(
            &display,
            frame_buffer_size.0,
            frame_buffer_size.1,
        )
        .unwrap();

        let mut frame_buffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
            &display,
            render_texture.main_level(),
            depth_stencil_target.main_level(),
        )
        .unwrap();

        // Initialize a camera
        let mut camera = Camera::new();
        camera.translate_by(&Vector3::new(
            screenshot.position.0,
            screenshot.position.1,
            screenshot.position.2,
        ));
        camera.rotate_by(&Rotation::from_euler_angles(
            screenshot.rotation.0.to_radians(),
            screenshot.rotation.1.to_radians(),
            screenshot.rotation.2.to_radians(),
        ));

        // Construct a frustum
        let frame_size = frame_buffer.get_dimensions();
        let aspect_ratio = frame_size.0 as f32 / frame_size.1 as f32;
        let frustum = camera.frustum(aspect_ratio);

        // Clear the backbuffer (we have to clear with sRGB values here)
        frame_buffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0).from_linear(), 1.0);

        // Draw the planet
        planet_renderer.ensure_resident_patches(&frustum, &planet_transform);
        planet_renderer.draw(
            &mut frame_buffer,
            &frustum,
            &planet_transform,
            &screenshot.draw_parameters,
        );

        // Read the render target into an image
        let image: glium::texture::RawImage2d<u8> = render_texture.read();
        let image =
            image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned())
                .unwrap();
        let mut image = image::DynamicImage::ImageRgba8(image).flipv();
        while image.width() > size.0 {
            image = image.resize(
                image.width() >> 1,
                image.height() >> 1,
                image::FilterType::Triangle,
            )
        }
        let mut output = std::fs::File::create(&Path::new(&filename)).unwrap();
        image.save(&mut output, image::ImageFormat::PNG).unwrap();
    }
}

const GAMMA_EXP: f32 = 1.0 / 2.2;

trait CanGammaCorrect {
    fn from_linear(self) -> Self;
}

impl CanGammaCorrect for (f32, f32, f32) {
    fn from_linear(self) -> Self {
        (
            self.0.powf(GAMMA_EXP),
            self.1.powf(GAMMA_EXP),
            self.2.powf(GAMMA_EXP),
        )
    }
}

impl CanGammaCorrect for (f32, f32, f32, f32) {
    fn from_linear(self) -> Self {
        (
            self.0.powf(GAMMA_EXP),
            self.1.powf(GAMMA_EXP),
            self.2.powf(GAMMA_EXP),
            self.3,
        )
    }
}

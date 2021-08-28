/// RS Raytracer
/// 
/// A ray tracer implemented in Rust based on https://raytracing.github.io/books/RayTracingInOneWeekend.html.
/// 
/// The ray tracer is implemented as a standalone module that can write to a file or copy to an SDL2 texture.
/// An SDL window handles the display and keystrokes for running the ray tracer and saving the image.
/// 
/// Escape: Quit.
/// Space: Run the ray tracer and update the preview with its result.
/// S: Save the current ray tracer buffer to a PPM file. This is local to your terminal CWD or exe if run directly.
/// 
/// Daniel Green <KasumiL5x@gmail.com>

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::keyboard::Keycode;

pub mod math;
pub mod raytracer;

pub fn main() -> Result<(), String> {
    let window_width = raytracer::WIDTH;
    let window_height = raytracer::HEIGHT;

    // Setup SDL and create the video subsystem.
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    // Create the window.
    let window = video_subsys
        .window("RS Raytracer", window_width, window_height)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    // Create the Canvas that we can draw to.
    let mut canvas = window
        .into_canvas() // Creates a new Canvas.
        .target_texture() // Allows rendering to a texture.
        .present_vsync() // Enables vsync.
        .build()
        .map_err(|e| e.to_string())?;
    
    println!("Using SDL_Renderer {}.", canvas.info().name);

    // Clear the Canvas and push it to the window.
    canvas.set_draw_color(Color::RGB(1, 0, 1));
    canvas.clear();
    canvas.present();

    // Create a TextureCreator (the Canvas cannot do so directly due to lifetime issues).
    let texture_creator = canvas.texture_creator();
    // Create the actual texture we'll be splatting to the Canvas.
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, window_width, window_height)
        .map_err(|e| e.to_string())?;

    // Create the ray tracer instance.
    let mut ray_tracer = raytracer::RSRaytracer::new();

    // Setup the scene.
    let mat_ground = ray_tracer.add_lambertian_material(raytracer::Lambertian::new(math::Vec3::new(0.8, 0.8, 0.0)));
    let mat_center = ray_tracer.add_lambertian_material(raytracer::Lambertian::new(math::Vec3::new(0.7, 0.3, 0.3)));
    let mat_left = ray_tracer.add_metal_material(raytracer::Metal::new(math::Vec3::new(0.8, 0.8, 0.8)));
    let mat_right = ray_tracer.add_metal_material(raytracer::Metal::new(math::Vec3::new(0.8, 0.6, 0.2)));
    ray_tracer.add_sphere(
        raytracer::Sphere::new(
            math::Vec3::new(0.0, -100.5, -1.0), 100.0,
            mat_ground
        )
    );
    ray_tracer.add_sphere(
        raytracer::Sphere::new(
            math::Vec3::new(0.0, 0.0, -1.0), 0.5,
            mat_center
        )
    );
    ray_tracer.add_sphere(
        raytracer::Sphere::new(
            math::Vec3::new(-1.0, 0.0, -1.0), 0.5,
            mat_left
        )
    );
    ray_tracer.add_sphere(
        raytracer::Sphere::new(
            math::Vec3::new(1.0, 0.0, -1.0), 0.5,
            mat_right
        )
    );

    // ray_tracer.add_sphere(
    //     math::Sphere::new(math::Vec3::new(0.0, 0.0, -1.0), 0.5)
    // );
    // ray_tracer.add_sphere(
    //     math::Sphere::new(math::Vec3::new(0.0, -100.5, -1.0), 100.0)
    // );

    // Copy the initial raytracer texture over and display it.
    ray_tracer.copy_to(&mut texture);
    copy_texture_to_canvas(&texture, &mut canvas, window_width, window_height);

    // Event loop.
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                // Quit.
                Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape),..} => {
                    break 'running
                }

                // Run ray tracer and update preview.
                Event::KeyDown{keycode: Some(Keycode::Space), repeat: false, ..} => {
                    run_raytracer(&mut ray_tracer, &mut texture, &mut canvas, window_width, window_height);
                }

                // Save ray tracer result to file.
                Event::KeyDown{keycode: Some(Keycode::S), repeat: false, ..} => {
                    ray_tracer.save_as_ppm().expect("Failed to write PPM file.");
                }

                // Update preview from raytracer (if window focus is lost during a ray trace, it may not update).
                Event::KeyDown{keycode: Some(Keycode::U), repeat: false, ..} => {
                    ray_tracer.copy_to(&mut texture);
                    copy_texture_to_canvas(&mut texture, &mut canvas, window_width, window_height);
                }

                // Movement.
                Event::KeyDown{keycode: Some(Keycode::Left), ..} => {
                    let cam = ray_tracer.get_camera();
                    let mut pos = cam.get_position();
                    pos.x -= 0.1;
                    cam.set_position(pos);

                    run_raytracer(&mut ray_tracer, &mut texture, &mut canvas, window_width, window_height);
                }
                Event::KeyDown{keycode: Some(Keycode::Right), ..} => {
                    let cam = ray_tracer.get_camera();
                    let mut pos = cam.get_position();
                    pos.x += 0.1;
                    cam.set_position(pos);

                    run_raytracer(&mut ray_tracer, &mut texture, &mut canvas, window_width, window_height);
                }
                Event::KeyDown{keycode: Some(Keycode::Up), ..} => {
                    let cam = ray_tracer.get_camera();
                    let mut pos = cam.get_position();
                    pos.y += 0.1;
                    cam.set_position(pos);

                    run_raytracer(&mut ray_tracer, &mut texture, &mut canvas, window_width, window_height);
                }
                Event::KeyDown{keycode: Some(Keycode::Down), ..} => {
                    let cam = ray_tracer.get_camera();
                    let mut pos = cam.get_position();
                    pos.y -= 0.1;
                    cam.set_position(pos);

                    run_raytracer(&mut ray_tracer, &mut texture, &mut canvas, window_width, window_height);
                }

                _ => {}
            }
        }
    }


    Ok(())
}

fn run_raytracer(rt: &mut raytracer::RSRaytracer, texture: &mut sdl2::render::Texture, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, width: u32, height: u32) {
    rt.run();
    rt.copy_to(texture);
    copy_texture_to_canvas(texture, canvas, width, height);
}

fn copy_texture_to_canvas(texture: &sdl2::render::Texture, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, width: u32, height: u32) {
    canvas.clear();
    canvas.copy(texture, None, Some(Rect::new(0, 0, width, height))).unwrap();
    canvas.present();
}

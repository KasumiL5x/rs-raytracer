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

extern crate sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::keyboard::Keycode;

mod rs_raytracer {
    use rand::Rng;

    use std::io;
    use std::io::prelude::*;
    use std::io::BufWriter;
    use std::fs::File;

    pub const IMAGE_WIDTH: u32 = 1280;
    pub const IMAGE_HEIGHT: u32 = 720;

    pub const PPM_OUT: &str = "./out.ppm";

    pub struct RSRaytracer {
        //? NOTE: This may change to be an array of f32 at a later time, so the SDL texture copy would require a
        //?       conversion to 0..255. Note that the f32 may go beyond 1, so clipping would still be required.
        pixels: Box<[u32]>
    }

    impl RSRaytracer {
        pub fn new() -> RSRaytracer {
            let mut pixels = vec![255; (IMAGE_WIDTH * IMAGE_HEIGHT * 3) as usize];

            // Start with a gradient texture.
            for y in 0..(IMAGE_HEIGHT as usize) {
                for x in 0..(IMAGE_WIDTH as usize) {
                    let pitch = (IMAGE_WIDTH * 3) as usize;
                    let offset = y * pitch + x * 3;
                    pixels[offset + 0] = (((x as f32) / (IMAGE_WIDTH as f32)) * 255.0) as u32;
                    pixels[offset + 1] = (((y as f32) / (IMAGE_HEIGHT as f32)) * 255.0) as u32;
                    pixels[offset + 2] = 0;
                }
            }

            RSRaytracer {
                pixels: pixels.into_boxed_slice()
            }
        }

        pub fn copy_to(&self, texture: &mut sdl2::render::Texture) {
            // Safety check before copying.
            let query = texture.query();
            if (query.width != IMAGE_WIDTH) || (query.height != IMAGE_HEIGHT) {
                println!("Texture dimensions do not match internal dimensions. Ignoring copy request.");
                return
            }

            // Manual copy per pixel.
            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..(IMAGE_HEIGHT as usize) {
                    for x in 0..(IMAGE_WIDTH as usize) {
                        let offset = y * pitch + x * 3;

                        let r = self.pixels[offset + 0];
                        buffer[offset + 0] = if r > u8::MAX.into() {u8::MAX} else {r as u8};

                        let g = self.pixels[offset + 1];
                        buffer[offset + 1] = if g > u8::MAX.into() {u8::MAX} else {g as u8};

                        let b = self.pixels[offset + 2];
                        buffer[offset + 2] = if b > u8::MAX.into() {u8::MAX} else {b as u8};
                    }
                }
            }).unwrap();

            // Direct memory copy from internal pixels array. Requires pixels to be [u8] format.
            // texture.update(
            //     sdl2::rect::Rect::new(0, 0, IMAGE_WIDTH, IMAGE_HEIGHT),
            //     &self.pixels,
            //     (IMAGE_WIDTH * 3) as usize
            // ).unwrap();
        }

        pub fn run(&mut self) {
            println!("Starting ray tracer...");
            let start_time = std::time::Instant::now();

            // TODO: Ray tracing code. For now it's random noise.
            let mut rng = rand::thread_rng();
            let pitch = (IMAGE_WIDTH * 3) as usize;
            for y in 0..(IMAGE_HEIGHT as usize) {
                print!("Rendering line {}/{}...", y+1, IMAGE_HEIGHT);
                for x in 0..(IMAGE_WIDTH as usize) {
                    let offset = y * pitch + x * 3;

                    let rand_val: u32 = rng.gen::<u8>() as u32;
                    self.pixels[offset + 0] = rand_val;
                    self.pixels[offset + 1] = rand_val;
                    self.pixels[offset + 2] = rand_val;
                }
                println!("done!");
            }

            let end_time = std::time::Instant::now();
            let delta_time = end_time.duration_since(start_time);
            println!("Ray trace complete in {:?}.", delta_time);
        }

        pub fn save_as_ppm(&self) -> io::Result<()> {
            print!("Writing PPM file...");
            let f = File::create(PPM_OUT)?;
            {
                let mut writer = BufWriter::new(f);

                // P3
                // WIDTH HEIGHT
                // MAX_VALUE
                write!(writer, "P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT)?;

                // Pixels (in rows, left to right, top to bottom).
                let pitch = (IMAGE_WIDTH * 3) as usize;
                for y in 0..(IMAGE_HEIGHT as usize) {
                    for x in 0..(IMAGE_WIDTH as usize) {
                        let offset = y * pitch + x * 3;
                        let r = self.pixels[offset + 0];
                        let g = self.pixels[offset + 1];
                        let b = self.pixels[offset + 2];

                        write!(writer, "{} {} {}\n", r, g, b)?;
                    }
                }

            } // Buffer is flushed when it goes out of scope.
            println!("Done!");

            Ok(())
        }
    }
}

pub fn main() -> Result<(), String> {
    let window_width = rs_raytracer::IMAGE_WIDTH;
    let window_height = rs_raytracer::IMAGE_HEIGHT;

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
    let mut ray_tracer = rs_raytracer::RSRaytracer::new();

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
                    ray_tracer.run();
                    ray_tracer.copy_to(&mut texture);
                    copy_texture_to_canvas(&texture, &mut canvas, window_width, window_height);
                }

                // Save ray tracer result to file.
                Event::KeyDown{keycode: Some(Keycode::S), repeat: false, ..} => {
                    ray_tracer.save_as_ppm().expect("Failed to write PPM file.");
                }

                _ => {}
            }
        }
    }


    Ok(())
}

fn copy_texture_to_canvas(texture: &sdl2::render::Texture, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, width: u32, height: u32) {
    canvas.clear();
    canvas.copy(texture, None, Some(Rect::new(0, 0, width, height))).unwrap();
    canvas.present();
}

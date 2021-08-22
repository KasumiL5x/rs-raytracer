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

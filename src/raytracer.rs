use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use rand::prelude::*;

use crate::math::*;

// --------------------------------------------------
// RSRaytracer
// --------------------------------------------------
pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 720;
pub const CHANNELS: u32 = 3;

const SAMPLES_PER_PIXEL: u32 = 20; // 100
const MAX_DEPTH: u32 = 20; // 50


pub const PPM_OUT: &str = "./out.ppm";

pub struct RSRaytracer {
    pixels: Box<[f32]>,
    objects: Vec<Box<dyn Hittable>>,
    cam: Camera
}

impl RSRaytracer {
    pub fn new() -> RSRaytracer {
        let mut pixels = vec![1.0; (WIDTH * HEIGHT * CHANNELS) as usize];

        // Start with a simple gradient.
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let pitch = WIDTH * CHANNELS;
                let offset = (y * pitch + x * CHANNELS) as usize;

                let scale: f32 = SAMPLES_PER_PIXEL as f32;

                pixels[offset + 0] = ((x as f32) / (WIDTH as f32)) * scale;
                pixels[offset + 1] = ((y as f32) / (HEIGHT as f32)) * scale;
                pixels[offset + 2] = 0.0;
            }
        }

        // No objects.
        let empty_objects:  Vec<Box<dyn Hittable>> = Vec::new();

        RSRaytracer {
            pixels: pixels.into_boxed_slice(),
            objects: empty_objects,
            cam: Camera::new()
        }
    }

    pub fn get_camera(&mut self) -> &mut Camera {
        &mut self.cam
    }

    pub fn add_sphere(&mut self, sphere: Sphere) {
        let boxed_obj = Box::new(sphere);
        self.objects.push(boxed_obj)
    }

    fn hit_objects(&self, ray: &Ray, t_min: f32, t_max: f32, out_hit: &mut HitRecord) -> bool {
        let mut tmp_rec: HitRecord = HitRecord::new();
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        for obj in self.objects.as_slice() {
            if obj.hit(ray, t_min, closest_so_far, &mut tmp_rec) {
                hit_anything = true;
                closest_so_far = tmp_rec.t;
                *out_hit = tmp_rec;
            }
        }

        return hit_anything
    }

    pub fn copy_to(&self, texture: &mut sdl2::render::Texture) {
        // Safety check before copying.
        let query = texture.query();
        if (query.width != WIDTH) || (query.height != HEIGHT) {
            println!("Texture dimensions do not match internal dimensions. Ignoring copy request.");
            return
        }

        // Manual copy per pixel.
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let offset = (y * (pitch as u32) + x * CHANNELS) as usize;

                    let pixel_color = Vec3::new(
                        self.pixels[offset + 0],
                        self.pixels[offset + 1],
                        self.pixels[offset + 2]
                    );
                    let (r_value, g_value, b_value) = self.get_final_rgb(&pixel_color);

                    buffer[offset + 0] = r_value;
                    buffer[offset + 1] = g_value;
                    buffer[offset + 2] = b_value;
                }
            }
        }).unwrap();

        // Direct memory copy from internal pixels array. Requires pixels to be [u8] format.
        // texture.update(
        //     sdl2::rect::Rect::new(0, 0, IMAGE_WIDTH, IMAGE_HEIGHT),
        //     &self.pixels,
        //     (IMAGE_WIDTH * CHANNELS) as usize
        // ).unwrap();
    }

    pub fn run(&mut self) {
        println!("Starting ray tracer...");
        let start_time = std::time::Instant::now();

        // Much, much more efficient than thread_rng.
        let mut rng = SmallRng::from_entropy();

        let pitch = WIDTH * CHANNELS;
        for y in 0..HEIGHT {
            print!("Rendering line {}/{}...", y+1, HEIGHT);
            for x in 0..WIDTH {
                let offset = (y * pitch + x * CHANNELS) as usize;

                let mut pixel_color = Vec3::zero();
                for _i in 0..SAMPLES_PER_PIXEL {
                    let r0: f32 = rng.gen();
                    let u = ((x as f32) + r0) / ((WIDTH-1) as f32);

                    let r1: f32 = rng.gen();
                    let v = ((y as f32) + r1) / ((HEIGHT-1) as f32);

                    let r = self.cam.get_ray(u, 1.0 - v);
                    pixel_color += self.ray_color(&r, MAX_DEPTH);
                }
                // let u = (x as f32) / ((WIDTH-1) as f32);
                // let v = 1.0 - ((y as f32) / ((HEIGHT-1) as f32));
                // let r = self.cam.get_ray(u, v);
                // let pixel_color = self.ray_color(&r);

                self.pixels[offset + 0] = pixel_color.x;
                self.pixels[offset + 1] = pixel_color.y;
                self.pixels[offset + 2] = pixel_color.z;
            }
            println!("done!");
        }

        let end_time = std::time::Instant::now();
        let delta_time = end_time.duration_since(start_time);
        println!("Ray trace complete in {:?}.", delta_time);
    }

    fn ray_color(&self, ray: &Ray, depth: u32) -> Vec3 {
        // Exceeded bounce limit, so no more light is gathered.
        if depth <= 0 {
            return Vec3::zero();
        }

        let mut hit_rec = HitRecord::new();
        if self.hit_objects(ray, 0.001, f32::MAX, &mut hit_rec) {
            let target = hit_rec.p + hit_rec.n + random_on_sphere();
            return 0.5 * self.ray_color(
                &Ray::new(hit_rec.p, target - hit_rec.p),
                depth - 1
            )
            // return 0.5 * (hit_rec.n + Vec3::new(1.0, 1.0, 1.0))
        }

        let direction = ray.direction.normalized();
        let t = 0.5 * (direction.y + 1.0);
        (1.0-t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
    }

    fn get_final_rgb(&self, pixel_color: &Vec3) -> (u8, u8, u8) {
        let mut out_color = pixel_color.clone();

        // Divide the color by the number of samples and gamma correct for gamma=2.0.
        let scale = 1.0 / (SAMPLES_PER_PIXEL as f32);
        out_color.x = (out_color.x * scale).sqrt();
        out_color.y = (out_color.y * scale).sqrt();
        out_color.z = (out_color.z * scale).sqrt();

        // Translate RGB to [0, 255] and return.
        (
            (256.0 * out_color.x.clamp(0.0, 0.999)) as u8,
            (256.0 * out_color.y.clamp(0.0, 0.999)) as u8,
            (256.0 * out_color.z.clamp(0.0, 0.999)) as u8
        )
    }

    pub fn save_as_ppm(&self) -> io::Result<()> {
        print!("Writing PPM file...");
        let f = File::create(PPM_OUT)?;
        {
            let mut writer = BufWriter::new(f);

            // P3
            // WIDTH HEIGHT
            // MAX_VALUE
            write!(writer, "P3\n{} {}\n255\n", WIDTH, HEIGHT)?;

            // Pixels (in rows, left to right, top to bottom).
            let pitch = WIDTH * CHANNELS;
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let offset = (y * pitch + x * CHANNELS) as usize;

                    let pixel_color = Vec3::new(
                        self.pixels[offset + 0],
                        self.pixels[offset + 1],
                        self.pixels[offset + 2]
                    );
                    let (r_value, g_value, b_value) = self.get_final_rgb(&pixel_color);

                    write!(writer, "{} {} {}\n", r_value, g_value, b_value)?;
                }
            }

        } // Buffer is flushed when it goes out of scope.
        println!("Done!");

        Ok(())
    }
}


// --------------------------------------------------
// Camera
// --------------------------------------------------
pub struct Camera {
    aspect_ratio: f32,
    viewport_height: f32,
    viewport_width: f32,
    focal_length: f32,
    origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lower_left_corner: Vec3,
    dirty: bool
}

impl Camera {
    pub fn new() -> Camera {
        // Image details.
        let aspect_ratio = (WIDTH as f32) / (HEIGHT as f32);
        // Camera details.
        let viewport_height = 2.0; // Why 2?
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.0;
        //
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        let vertical = Vec3::new(0.0, viewport_height, 0.0);
        let lower_left_corner = origin - (horizontal * 0.5) - (vertical * 0.5) - Vec3::new(0.0, 0.0, focal_length);

        Camera {
            aspect_ratio: aspect_ratio,
            viewport_height: viewport_height,
            viewport_width: viewport_width,
            focal_length: focal_length,
            origin: origin,
            horizontal: horizontal,
            vertical: vertical,
            lower_left_corner: lower_left_corner,
            dirty: true
        }
    }

    fn update(&mut self) {
        self.aspect_ratio = (WIDTH as f32) / (HEIGHT as f32);

        self.viewport_height = 2.0; // Why 2?
        self.viewport_width = self.aspect_ratio * self.viewport_height;
        self.focal_length = 1.0;
        //
        // self.origin = Vec3::new(0.0, 0.0, 0.0);
        self.horizontal = Vec3::new(self.viewport_width, 0.0, 0.0);
        self.vertical = Vec3::new(0.0, self.viewport_height, 0.0);
        self.lower_left_corner = self.origin - (self.horizontal * 0.5) - (self.vertical * 0.5) - Vec3::new(0.0, 0.0, self.focal_length);
    }

    pub fn get_position(&self) -> Vec3 {
        self.origin
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.origin = pos;
        self.dirty = true;
    }

    pub fn get_ray(&mut self, u: f32, v: f32) -> Ray {
        if self.dirty {
            self.update();
            self.dirty = false;
        }

        Ray::new(self.origin, self.lower_left_corner + u*self.horizontal + v*self.vertical - self.origin)
    }
}

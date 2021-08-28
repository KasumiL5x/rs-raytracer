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
    materials: Vec<Box<dyn Material>>,
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

                // Must be multiplied here as there's a conversion using this value when outputting the underlying data.
                let scale: f32 = SAMPLES_PER_PIXEL as f32;

                pixels[offset + 0] = ((x as f32) / (WIDTH as f32)) * scale;
                pixels[offset + 1] = ((y as f32) / (HEIGHT as f32)) * scale;
                pixels[offset + 2] = 0.0;
            }
        }

        // Add a single default material so that default 0 indexes don't fail.
        let mut mats: Vec<Box<dyn Material>> = Vec::new();
        mats.push(Box::new(Lambertian::new(Vec3::one())));

        RSRaytracer {
            pixels: pixels.into_boxed_slice(),
            objects: Vec::<Box<dyn Hittable>>::new(),
            materials: mats,
            cam: Camera::new()
        }
    }

    pub fn get_camera(&mut self) -> &mut Camera {
        &mut self.cam
    }

    pub fn add_lambertian_material(&mut self, mat: Lambertian) -> u32 {
        let boxed_mat = Box::new(mat);
        self.materials.push(boxed_mat);
        return (self.materials.len() - 1) as u32
    }

    pub fn add_metal_material(&mut self, mat: Metal) -> u32 {
        let boxed_mat = Box::new(mat);
        self.materials.push(boxed_mat);
        return (self.materials.len() - 1) as u32
    }

    pub fn get_material(&self, idx: u32) -> &Box<dyn Material> {
        &self.materials[idx as usize]
    }

    pub fn add_sphere(&mut self, sphere: Sphere) {
        let boxed_obj = Box::new(sphere);
        self.objects.push(boxed_obj)
    }

    fn hit_objects(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut best_rec: HitRecord = HitRecord::empty();
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        for obj in self.objects.as_slice() {
            let tmp_rec = obj.hit(ray, t_min, closest_so_far);
            if !tmp_rec.is_none() {
                let tmp_rec = tmp_rec.unwrap();
                hit_anything = true;
                closest_so_far = tmp_rec.t;
                best_rec = tmp_rec;
            }
        }

        return if hit_anything {Some(best_rec)} else {None}
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

        let hit_rec = self.hit_objects(ray, 0.001, f32::MAX);
        if !hit_rec.is_none() {
            let mut scattered: Ray = Ray::new(Vec3::zero(), Vec3::zero());
            let mut attenuation: Vec3 = Vec3::zero();
            let hit_rec = hit_rec.unwrap();
            let mat = self.get_material(hit_rec.mat_id);
            if mat.scatter(ray, &hit_rec, &mut attenuation, &mut scattered) {
                return attenuation * self.ray_color(&scattered, depth - 1)
            }

            return Vec3::zero()
        }

        let direction = ray.direction.normalized();
        let t = 0.5 * (direction.y + 1.0);
        return (1.0-t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
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


// --------------------------------------------------
// Material(s)
// --------------------------------------------------
// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
// pub trait Material: MaterialClone {
//     fn scatter(&self, ray: &Ray, hit_rec: &HitRecord, out_attenuation: &mut Vec3, out_scattered: &mut Ray) -> bool;
// }
// pub trait MaterialClone {
//     fn clone_mat(&self) -> Box<dyn Material>;
// }
// impl<T> MaterialClone for T where T: 'static + Material + Clone, {
//     fn clone_mat(&self) -> Box<dyn Material> {
//         Box::new(self.clone())
//     }
// }
// impl Clone for Box<dyn Material> {
//     fn clone(&self) -> Box<dyn Material> {
//         self.clone_mat()
//     }
// }
// NOTE: The above is no longer needed as materials are now referred to by an index. I'm keeping this around for posterity, though.
pub trait Material {
    fn scatter(&self, ray: &Ray, hit_rec: &HitRecord, out_attenuation: &mut Vec3, out_scattered: &mut Ray) -> bool;
}

pub struct Lambertian {
    albedo: Vec3
}
impl Lambertian {
    pub fn new(albedo: Vec3) -> Lambertian {
        Lambertian {
            albedo: albedo
        }
    }
}
impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit_rec: &HitRecord, out_attenuation: &mut Vec3, out_scattered: &mut Ray) -> bool {
        let mut scatter_dir = hit_rec.n + Vec3::random_on_sphere();

        // Catch degenerate scatter direction.
        if scatter_dir.near_zero() {
            scatter_dir = hit_rec.n;
        }

        out_scattered.origin = hit_rec.p;
        out_scattered.direction = scatter_dir;

        *out_attenuation = self.albedo;

        return true
    }
}

pub struct Metal {
    albedo: Vec3
}
impl Metal {
    pub fn new(albedo: Vec3) -> Metal {
        Metal {
            albedo: albedo
        }
    }
}
impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit_rec: &HitRecord, out_attenuation: &mut Vec3, out_scattered: &mut Ray) -> bool {
        let reflected = ray.direction.normalized().reflect(hit_rec.n);

        out_scattered.origin = hit_rec.p;
        out_scattered.direction = reflected;

        *out_attenuation = self.albedo;

        return out_scattered.direction.dot(&hit_rec.n) > 0.0
    }
}


// --------------------------------------------------
// Hittable / HitRecord
// --------------------------------------------------
#[derive(Clone)]
pub struct HitRecord {
    pub p: Vec3,
    pub n: Vec3,
    pub t: f32,
    pub front_face: bool,
    pub mat_id: u32
}

impl HitRecord {
    pub fn empty() -> HitRecord {
        HitRecord{
            p: Vec3::new(0.0, 0.0, 0.0),
            n: Vec3::new(0.0, 0.0, 0.0),
            t: 0.0,
            front_face: false,
            mat_id: 0
        }
    }

    pub fn new(p: Vec3, n: Vec3, t: f32, front_face: bool, mat_id: u32) -> HitRecord {
        HitRecord {
            p: p,
            n: n,
            t: t,
            front_face: front_face,
            mat_id: mat_id
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vec3) {
        self.front_face = ray.direction.dot(outward_normal) < 0.0;
        self.n = if self.front_face {*outward_normal} else {-*outward_normal};
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}


// --------------------------------------------------
// Sphere
// --------------------------------------------------
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub mat_id: u32
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, mat_id: u32) -> Sphere {
        Sphere {
            center: center,
            radius: radius,
            mat_id: mat_id
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.sqr_length();
        let half_b = oc.dot(&ray.direction);
        let c = oc.sqr_length() - (self.radius * self.radius);

        let discriminant = (half_b * half_b) - (a * c);
        if discriminant < 0.0 {
            return None
        }
        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None
            }
        }

        let mut hr = HitRecord::new(
            ray.at(root), Vec3::zero(), root, false, self.mat_id
        );
        let outward_normal = (hr.p - self.center) / self.radius;
        hr.set_face_normal(ray, &outward_normal);

        return Some(hr)
    }
}

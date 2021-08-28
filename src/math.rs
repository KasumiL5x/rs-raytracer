use std::ops;

use rand::prelude::*;

// --------------------------------------------------
// Vec3
// --------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x: x, y: y, z: z }
    }

    pub fn zero() -> Vec3 {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn random() -> Vec3 {
        let mut rng = SmallRng::from_entropy();
        Vec3 {
            x: rng.gen(),
            y: rng.gen(),
            z: rng.gen()
        }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn sqr_length(&self) -> f32 {
        (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
    }

    pub fn dot(&self, rhs: &Vec3) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }

    pub fn cross(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn normalize(&mut self) -> Vec3 {
        let len = self.length();
        self.x /= len;
        self.y /= len;
        self.z /= len;
        *self
    }

    pub fn normalized(&self) -> Vec3 {
        let len = self.length();
        Vec3 {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }

    pub fn reflect(&self, normal: Vec3) -> Vec3 {
        return (*self) - 2.0 * self.dot(&normal) * normal;
    }

    pub fn near_zero(&self) -> bool {
        let EPS: f32 = 1e-5; // Not sure what the precision is in Rust for f32.
        return (self.x.abs() < EPS) && (self.y.abs() < EPS) && (self.z.abs() < EPS)
    }
}

// Vec3 + Vec3
impl ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

// Vec3 += Vec3
impl ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}

// Vec3 - Vec3
impl ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

// Vec3 * Vec3
impl ops::Mul for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

// Vec3 * f32
impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

// f32 * Vec3
impl ops::Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

// Vec3 / f32
impl ops::Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

// -Vec3
impl ops::Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// --------------------------------------------------
// Ray
// --------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray {
            origin: origin,
            direction: direction,
        }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

// --------------------------------------------------
// Hittable / Geometric Objects
// --------------------------------------------------
#[derive(Clone)]
pub struct HitRecord {
    pub p: Vec3,
    pub n: Vec3,
    pub t: f32,
    pub front_face: bool,
    pub mat: Box<dyn Material>
}

impl HitRecord {
    pub fn new() -> HitRecord {
        HitRecord{
            p: Vec3::new(0.0, 0.0, 0.0),
            n: Vec3::new(0.0, 0.0, 0.0),
            t: 0.0,
            front_face: false,
            mat: Box::new(Lambertian::new(Vec3::zero())) // Dummy material. Sadly cannot be null.
        }
    }

    pub fn new_pop(p: Vec3, n: Vec3, t: f32, front_face: bool, mat: Box<dyn Material>) -> HitRecord {
        HitRecord {
            p: p,
            n: n,
            t: t,
            front_face: front_face,
            mat: mat
        }
    }

    // Manual copy because of the Option<Box>.
    // pub fn copy_to(&self, other: &mut HitRecord) {
    //     other.p = self.p;
    //     other.n = self.n;
    //     other.t = self.t;
    //     other.front_face = self.front_face;
    //     other.mat = Some(self.mat.unwrap());
    // }

    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vec3) {
        self.front_face = ray.direction.dot(outward_normal) < 0.0;
        self.n = if self.front_face {*outward_normal} else {-*outward_normal};
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    mat: Box<dyn Material>
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, mat: Box<dyn Material>) -> Sphere {
        Sphere {
            center: center,
            radius: radius,
            mat: mat
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

        let mut hr = HitRecord::new_pop(
            ray.at(root), Vec3::zero(), root, false, self.mat.clone()
        );
        let outward_normal = (hr.p - self.center) / self.radius;
        hr.set_face_normal(ray, &outward_normal);

        return Some(hr)
    }
}

// --------------------------------------------------
// Intersections
// --------------------------------------------------
pub fn hit_sphere(center: &Vec3, radius: f32, ray: &Ray) -> f32 {
    let oc = ray.origin - (*center);
    let a = ray.direction.sqr_length();
    let half_b = oc.dot(&ray.direction);
    let c = oc.sqr_length() - (radius * radius);
    let discriminant = (half_b * half_b) - (a * c);

    if discriminant < 0.0 {
        return -1.0;
    } else {
        return (-half_b - discriminant.sqrt()) / a;
    }
}

// --------------------------------------------------
// Other
// --------------------------------------------------
pub fn random_on_sphere() -> Vec3 {
    let mut rng = SmallRng::from_entropy();
    Vec3::new(
        rng.gen::<f32>() - 0.5,
        rng.gen::<f32>() - 0.5,
        rng.gen::<f32>() - 0.5
    ).normalize()
}

pub fn random_in_hemisphere(normal: &Vec3) -> Vec3 {
    let rand_on_sphere = random_on_sphere();
    if rand_on_sphere.dot(normal) > 0.0 {
        // In the same hemisphere as the normal.
        return rand_on_sphere
    } else {
        return -rand_on_sphere
    }
}


// --------------------------------------------------
// Material(s)
// --------------------------------------------------
pub trait Material: MaterialClone {
    fn scatter(&self, ray: &Ray, hit_rec: &HitRecord, out_attenuation: &mut Vec3, out_scattered: &mut Ray) -> bool;
}
pub trait MaterialClone {
    fn clone_mat(&self) -> Box<dyn Material>;
}
impl<T> MaterialClone for T where T: 'static + Material + Clone, {
    fn clone_mat(&self) -> Box<dyn Material> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn Material> {
    fn clone(&self) -> Box<dyn Material> {
        self.clone_mat()
    }
}

#[derive(Clone)]
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
    fn scatter(&self, ray: &Ray, hit_rec: &HitRecord, out_attenuation: &mut Vec3, out_scattered: &mut Ray) -> bool {
        let mut scatter_dir = hit_rec.n + random_on_sphere();

        // Catch degenerate scatter direction.
        if scatter_dir.near_zero() {
            scatter_dir = hit_rec.n;
        }

        out_scattered.origin = hit_rec.p;
        out_scattered.direction = scatter_dir;

        out_attenuation.x = self.albedo.x;
        out_attenuation.y = self.albedo.y;
        out_attenuation.z = self.albedo.z;

        return true
    }
}

#[derive(Clone)]
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

        out_attenuation.x = self.albedo.x;
        out_attenuation.y = self.albedo.y;
        out_attenuation.z = self.albedo.z;

        return out_scattered.direction.dot(&hit_rec.n) > 0.0
    }
}

// #[derive(Clone, Copy, Debug)]
// pub struct Material<T, MaterialData> {
//     data: T
// }

// impl Material {
//     pub fn new() -> Material {
//         Material {
//         }
//     }
// }

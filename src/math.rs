use std::ops;
use std::ops::Range;

use rand::prelude::*;

pub struct RandGen {
    rng: SmallRng // Much, much more efficient than thread_rng.
}
impl RandGen {
    pub fn new() -> RandGen {
        RandGen {
            rng: SmallRng::from_entropy()
        }
    }
    
    pub fn next01(&mut self) -> f32 {
        self.rng.gen()
    }

    pub fn next_range(&mut self, r: Range<f32>) -> f32 {
        self.rng.gen_range(r)
    }
}

// --------------------------------------------------
// Vec3
// --------------------------------------------------
#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 {
            x: x,
            y: y,
            z: z
        }
    }

    pub fn zero() -> Vec3 {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    }

    pub fn one() -> Vec3 {
        Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0
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

    pub fn random_range(r: Range<f32>) -> Vec3 {
        let mut rng = SmallRng::from_entropy();
        Vec3 {
            x: rng.gen_range(r.clone()),
            y: rng.gen_range(r.clone()),
            z: rng.gen_range(r.clone())
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

    pub fn normalize(&mut self) -> &mut Vec3 {
        let len = self.length();
        self.x /= len;
        self.y /= len;
        self.z /= len;
        self
    }

    pub fn normalized(&self) -> Vec3 {
        let len = self.length();
        Vec3 {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len
        }
    }

    pub fn reflect(&self, normal: Vec3) -> Vec3 {
        return (*self) - 2.0 * self.dot(&normal) * normal;
    }

    pub fn near_zero(&self) -> bool {
        let eps: f32 = 1e-5; // Not sure what the precision is in Rust for f32.
        return (self.x.abs() < eps) && (self.y.abs() < eps) && (self.z.abs() < eps)
    }

    pub fn random_on_sphere() -> Vec3 {
        let mut rng = SmallRng::from_entropy();
        Vec3::new(
            rng.gen::<f32>() - 0.5,
            rng.gen::<f32>() - 0.5,
            rng.gen::<f32>() - 0.5
        ).normalized()
    }
    
    pub fn random_in_hemisphere(normal: &Vec3) -> Vec3 {
        let rand_on_sphere = Vec3::random_on_sphere();
        if rand_on_sphere.dot(normal) > 0.0 { // In the same hemisphere as the normal.
            return rand_on_sphere
        } else {
            return -rand_on_sphere
        }
    }

    pub fn refract(uv: Vec3, n: Vec3, eta_i_over_eta_t: f32) -> Vec3 {
        // NOTE: Inputs aren't references as you have to re-implement all std::ops for reference types...not worth it yet.
        let cos_theta = (-uv).dot(&n).min(1.0);
        let r_out_perpendicular = eta_i_over_eta_t * (uv + cos_theta * n);
        let r_out_parallel = (-(1.0 - r_out_perpendicular.sqr_length()).abs().sqrt()) * n;
        return r_out_perpendicular + r_out_parallel
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
            z: self.z - rhs.z
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
            z: self.z * rhs.z
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
            z: self.z * rhs
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
            z: self * rhs.z
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
            z: self.z / rhs
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
            z: -self.z
        }
    }
}
// Rust is playing a dangerous game here having separate implementations for references...
// https://stackoverflow.com/questions/28005134/how-do-i-implement-the-add-trait-for-a-reference-to-a-struct
// impl ops::Neg for &Vec3 {
//     type Output = Vec3;
//     fn neg(self) -> Vec3 {
//         Vec3 {
//             x: -self.x,
//             y: -self.y,
//             z: -self.z
//         }
//     }
// }

// --------------------------------------------------
// Ray
// --------------------------------------------------
#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray {
            origin: origin,
            direction: direction
        }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

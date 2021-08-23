use std::ops;

// --------------------------------------------------
// Vec3
// --------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
  pub x: f32,
  pub y: f32,
  pub z: f32
}

impl Vec3 {
  pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3{x: x, y: y, z: z}
  }

  pub fn zero() -> Vec3 {
    Vec3{x: 0.0, y: 0.0, z: 0.0}
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
      z: self.x * rhs.y - self.y * rhs.x
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
      z: self.z / len
    }
  }
}

// Vec3 + Vec3
impl ops::Add for Vec3 {
  type Output = Vec3;
  fn add(self, rhs: Vec3) -> Vec3 {
    Vec3 {
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



// --------------------------------------------------
// Ray
// --------------------------------------------------
#[derive(Clone, Copy, Debug)]
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


// --------------------------------------------------
// Intersections
// --------------------------------------------------
pub fn hit_sphere(center: &Vec3, radius: f32, ray: &Ray) -> f32 {
  let oc = ray.origin - (*center);
  let a = ray.direction.dot(&ray.direction);
  let b = 2.0 * oc.dot(&ray.direction);
  let c = oc.dot(&oc) - (radius * radius);
  let discriminant = (b * b) - (4.0 * a * c);

  if discriminant < 0.0 {
    return -1.0
  } else {
    return (-b - discriminant.sqrt()) / (2.0 * a)
  }
}

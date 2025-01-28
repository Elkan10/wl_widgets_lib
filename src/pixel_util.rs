use std::ops::{Add, Sub};

use rusttype::Point;

#[derive(Copy, Clone)]
pub struct PolarVector2 {
    pub r: f32,
    pub theta: f32,
}

impl PolarVector2 {
    fn new(r: f32, theta: f32) -> Self {
        Self {r, theta}
    }
}

impl From<Vector2> for PolarVector2 {
    fn from(value: Vector2) -> Self {
        let r = value.dist(&Vector2 { x: 0.0, y: 0.0 });
        let theta = value.y.atan2(value.x);
        Self {r, theta}
    }
}

#[derive(Copy, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }

    pub fn square_dist(&self, b: &Vector2) -> f32  {
        let a = self;
        (a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)
    }

    pub fn dist(&self, b: &Vector2) -> f32 {
        self.square_dist(b).sqrt()
    }
}

impl From<Point<f32>> for Vector2 {
    fn from(value: Point<f32>) -> Self {
        Self {x: value.x, y: value.y}
    }
}

impl From<PolarVector2> for Vector2 {
    fn from(value: PolarVector2) -> Self {
        let x = value.r * value.theta.cos();
        let y = value.r * value.theta.sin();
        Self {x, y}
    }
}

impl Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {x: self.x - rhs.x, y: self.y - rhs.y}
    }
}

impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {x: self.x + rhs.x, y: self.y + rhs.y}
    }
}


pub fn dist_to_line(a: Vector2, b: Vector2, other: Vector2) -> f32 {
    let l_squared = a.square_dist(&b);
    let e_a_squared = a.square_dist(&other);
    let e_b_squared = b.square_dist(&other);

    let a_disc = l_squared + e_a_squared - e_b_squared;
    if a_disc < 0.0 {
        return e_a_squared.sqrt();
    }

    let b_disc = l_squared + e_b_squared - e_a_squared;
    if b_disc < 0.0 {
        return e_b_squared.sqrt();
    }

    let a = 0.5 * a_disc * (1.0/l_squared.sqrt()); 


    let dist = (e_a_squared - (a * a)).sqrt();
    if dist.is_nan() {
        0.0
    } else {
        dist
    }
}

pub fn dist_to_arc(center: Vector2, radius: f32, start: f32, end: f32, other: Vector2) -> f32 {
    let p: PolarVector2 = (other - center).into();
    if p.theta > start && p.theta < end {
        (p.r - radius).abs()
    } else if p.theta < start {
        other.dist(&(Into::<Vector2>::into(PolarVector2 {r: radius, theta: start}) + center))
    } else {
        other.dist(&(Into::<Vector2>::into(PolarVector2 {r: radius, theta: end}) + center))
    }
}

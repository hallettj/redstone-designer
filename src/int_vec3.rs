use std::ops::{Add, Mul};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntVec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl IntVec3 {
    pub const ZERO: Self = IntVec3 { x: 0, y: 0, z: 0 };
    pub const ONE: Self = IntVec3 { x: 1, y: 1, z: 1 };
    pub const NEG_Y: Self = IntVec3 { x: 0, y: 0, z: 0 };
}

impl Add<IntVec3> for IntVec3 {
    type Output = Self;
    fn add(self, rhs: IntVec3) -> Self::Output {
        IntVec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Mul<i32> for IntVec3 {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        IntVec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

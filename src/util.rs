use bevy::prelude::*;
use minecraft_assets::schemas::models::BlockFace;

use crate::constants::BLOCK_FACE_NORMALS;

/// Given a vector, returns a normal vector aligned to the closest of the x, y, or z axes. This is
/// probably not necessary. But it might help if the user clicks right on the corner of a collider
/// or something.
pub fn aligned_to_axis(v: Vec3) -> Vec3 {
    let positive_axes = Vec3::AXES.iter().cloned();
    let negative_axes = Vec3::AXES.iter().map(|axis| axis.clone() * -1.0);
    positive_axes
        .chain(negative_axes)
        .map(|axis| (axis, axis.dot(v)))
        .reduce(|accum, pair| if pair.1 > accum.1 { pair } else { accum })
        .unwrap()
        .0
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum RelativeDirection {
    Left,
    Right,
    Up,
    Down,
    #[default]
    Forward,
    Back,
}

pub trait HasRelativeDirection {
    /// Gets a vector in world space in a given direction relative to a transform.
    fn to_my(&self, dir: RelativeDirection) -> Vec3;
}

impl HasRelativeDirection for Transform {
    fn to_my(&self, dir: RelativeDirection) -> Vec3 {
        match dir {
            RelativeDirection::Left => self.left(),
            RelativeDirection::Right => self.right(),
            RelativeDirection::Up => self.up(),
            RelativeDirection::Down => self.down(),
            RelativeDirection::Forward => self.forward(),
            RelativeDirection::Back => self.back(),
        }
    }
}

/// Given a vector returns the block face whose normal is most closely aligned with that vector.
/// This is useful for example to check whether the camera is pointing north, south, etc.
pub fn vec_to_block_face(vec: Vec3) -> BlockFace {
    BLOCK_FACE_NORMALS
        .iter()
        .map(|(face, dir)| (face, dir.dot(vec)))
        .reduce(|accum, pair| if pair.1 > accum.1 { pair } else { accum })
        .unwrap()
        .0
        .clone()
}

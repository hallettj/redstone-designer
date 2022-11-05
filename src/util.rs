use bevy::prelude::*;

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

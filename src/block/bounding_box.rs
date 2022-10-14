use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use minecraft_assets::schemas::models::Element;

use crate::{constants::BLOCKS, lines::LineList};

/// Gets the 3D box used for cursor interaction with a block. This volume is used to calculate
/// whether the cursor is over the block, and is also used to draw a wireframe around the block to
/// show that the cursor is in the right place to interact with it. In many cases the bounding box
/// can be inferred from the block model, but some blocks have ad-hoc bounding boxes.
///
/// The returned value gives two corners for the bounding box. The first value is the minimum x, y,
/// and z coordinates; the second value gives the maximum values.
///
/// TODO: Some bounding boxes are not actually boxes, such as the corner block state for redstone
/// dust.
pub fn bounding_box_for_block_model(model_name: &str, elements: &[Element]) -> (Vec3, Vec3) {
    match model_name {
        "repeater_2tick" => bounding_box_for_elements(&elements[0..1]),
        _ => bounding_box_for_elements(elements),
    }
}

pub fn bounding_box_to_line_list(bounding_box: (Vec3, Vec3)) -> LineList {
    // Move box coordinates outward by delta to put lines outside of the model that they highlight.
    let delta = 0.05;
    let min_x = bounding_box.0.x - delta;
    let min_y = bounding_box.0.y - delta;
    let min_z = bounding_box.0.z - delta;
    let max_x = bounding_box.1.x + delta;
    let max_y = bounding_box.1.y + delta;
    let max_z = bounding_box.1.z + delta;
    LineList::new(vec![
        // lower square
        (
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(min_x, min_y, max_z),
        ),
        (
            Vec3::new(min_x, min_y, max_z),
            Vec3::new(max_x, min_y, max_z),
        ),
        (
            Vec3::new(max_x, min_y, max_z),
            Vec3::new(max_x, min_y, min_z),
        ),
        (
            Vec3::new(max_x, min_y, min_z),
            Vec3::new(min_x, min_y, min_z),
        ),
        // upper square
        (
            Vec3::new(min_x, max_y, min_z),
            Vec3::new(min_x, max_y, max_z),
        ),
        (
            Vec3::new(min_x, max_y, max_z),
            Vec3::new(max_x, max_y, max_z),
        ),
        (
            Vec3::new(max_x, max_y, max_z),
            Vec3::new(max_x, max_y, min_z),
        ),
        (
            Vec3::new(max_x, max_y, min_z),
            Vec3::new(min_x, max_y, min_z),
        ),
        // vertical corner lines
        (
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(min_x, max_y, min_z),
        ),
        (
            Vec3::new(min_x, min_y, max_z),
            Vec3::new(min_x, max_y, max_z),
        ),
        (
            Vec3::new(max_x, min_y, min_z),
            Vec3::new(max_x, max_y, min_z),
        ),
        (
            Vec3::new(max_x, min_y, max_z),
            Vec3::new(max_x, max_y, max_z),
        ),
    ])
}

pub fn bounding_box_to_collider(bounding_box: (Vec3, Vec3)) -> Collider {
    let (
        Vec3 {
            x: min_x,
            y: min_y,
            z: min_z,
        },
        Vec3 {
            x: max_x,
            y: max_y,
            z: max_z,
        },
    ) = bounding_box;
    // It's a cuboid, but translated so that the origin is at one corner instead of in the center
    // of the cuboid.
    Collider::convex_hull(&[
        Vec3::new(min_x, min_y, min_z),
        Vec3::new(max_x, min_y, min_z),
        Vec3::new(min_x, max_y, min_z),
        Vec3::new(min_x, min_y, max_z),
        Vec3::new(max_x, max_y, min_z),
        Vec3::new(min_x, max_y, max_z),
        Vec3::new(max_x, min_y, max_z),
        Vec3::new(max_x, max_y, max_z),
    ]).unwrap()
}

/// Compute a bounding box that encompasses all of the given elements.
fn bounding_box_for_elements(elements: &[Element]) -> (Vec3, Vec3) {
    elements.iter().fold(
        (Vec3::ONE * BLOCKS, Vec3::ZERO),
        |(
            Vec3 {
                x: min_x,
                y: min_y,
                z: min_z,
            },
            Vec3 {
                x: max_x,
                y: max_y,
                z: max_z,
            },
        ),
         element| {
            (
                Vec3::new(
                    f32::min(min_x, element.from[0]),
                    f32::min(min_y, element.from[1]),
                    f32::min(min_z, element.from[2]),
                ),
                Vec3::new(
                    f32::max(max_x, element.to[0]),
                    f32::max(max_y, element.to[1]),
                    f32::max(max_z, element.to[2]),
                ),
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use minecraft_assets::{
        api::{AssetPack, ModelResolver},
        schemas::Model,
    };

    use super::bounding_box_for_block_model;

    fn get_block_model(block_model_name: &str) -> Model {
        let assets = AssetPack::at_path("assets/minecraft/");
        let models = assets.load_block_model_recursive(block_model_name).unwrap();
        ModelResolver::resolve_model(models.iter())
    }

    #[test]
    fn bounding_box_for_repeater() {
        let model = get_block_model("repeater_2tick");
        let actual = bounding_box_for_block_model("repeater_2tick", &(model.elements.unwrap()));
        let expected = (Vec3::new(0.0, 0.0, 0.0), Vec3::new(16.0, 2.0, 16.0));
        assert_eq!(
            actual, expected,
            "bounding box for repeater matches bounds of its first element"
        );
    }
}

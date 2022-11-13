use bevy::{prelude::*, render::camera::RenderTarget};
use bevy_rapier3d::prelude::*;

use crate::{
    camera::MainCamera,
    constants::{BLOCKS, PIXELS},
    util::aligned_to_axis,
};

/// Maximum distance for interacting with a block with the cursor.
const MAX_TOI: f32 = 32.0 * BLOCKS;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Cursor::default())
            .add_system_to_stage(CoreStage::PreUpdate, update_current_block);
    }
}

#[derive(Default, Debug, Resource)]
pub struct Cursor {
    /// Block under the cursor
    pub current_block: Option<Entity>,

    /// If placing a new block at the cursor position, this transform provides the appropriate
    /// translation and rotation for placement.
    pub place_block_transform: Option<Transform>,
}

fn update_current_block(
    mut cursor: ResMut<Cursor>,
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    match get_block_under_cursor(windows, rapier_context, query_camera) {
        Some(hit) => {
            let (entity, transform) = current_block_and_place_block_transform(hit);
            cursor.current_block = Some(entity);
            cursor.place_block_transform = Some(transform);
        }
        None => {
            cursor.current_block = None;
            cursor.place_block_transform = None;
        }
    }
}

fn current_block_and_place_block_transform(hit: EntityHit) -> (Entity, Transform) {
    let EntityHit {
        entity,
        intersection: RayIntersection { point, normal, .. },
        ..
    } = hit;

    // Produce an offset that moves into the space of the next block in the grid in the direction
    // of the vector. This assumes that the smallest possible selection box for a model is at least
    // one pixel (which it is - Minecraft model sizes are given in integer pixel counts).
    let offset = aligned_to_axis(normal) * (15.0 * PIXELS);

    // Translate from the clicked point in the direction of the normal, and snap to an intersection
    // in the block grid.
    let transform = Transform::from_translation(((point + offset) / BLOCKS).round() * BLOCKS);

    (entity, transform)
}

#[derive(Clone, Debug, PartialEq)]
struct EntityHit {
    entity: Entity,
    intersection: RayIntersection,
}

fn get_block_under_cursor(
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<EntityHit> {
    let (camera, camera_transform) = query_camera.single();
    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);
    let (ray_pos, ray_dir) = ray_from_screenspace(
        window_size,
        window.cursor_position()?,
        camera,
        camera_transform,
    );
    let solid = true;
    let groups = InteractionGroups::all();
    let filter = QueryFilter::new().groups(groups);

    let (entity, intersection) =
        rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, MAX_TOI, solid, filter)?;
    Some(EntityHit {
        entity,
        intersection,
    })
}

/// Returns origin and direction for a ray from the camera through the cursor. This involves
/// reversing the camera projection to map the cursor's coordinates in screen space to a set of
/// coordinates in world space.
fn ray_from_screenspace(
    window_size: Vec2,
    cursor_pos_screen: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> (Vec3, Vec3) {
    // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
    let ndc = (cursor_pos_screen / window_size) * 2.0 - Vec2::ONE;

    // matrix for undoing the projection and camera transform
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

    // use it to convert ndc to world-space coordinates
    let cursor_pos_world = ndc_to_world.project_point3(ndc.extend(-1.0));

    let origin = cursor_pos_world;
    let ray_direction = (camera_transform.translation() - cursor_pos_world).normalize();

    (origin, ray_direction)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_rapier3d::{prelude::*, rapier::prelude::FeatureId};

    use crate::constants::BLOCKS;

    use super::{current_block_and_place_block_transform, EntityHit};

    fn test_hit_intersection(point: Vec3, normal: Vec3) -> EntityHit {
        EntityHit {
            entity: Entity::from_raw(1),
            intersection: RayIntersection {
                toi: 10.0,
                point,
                normal,
                feature: FeatureId::Unknown,
            },
        }
    }

    #[test]
    fn computes_transform_for_block_placement() {
        let test_cases = vec![
            (
                "east, full block",
                test_hit_intersection(
                    Vec3::new(0.5 * BLOCKS, 0.1 * BLOCKS, 0.2 * BLOCKS),
                    Vec3::new(1.0, 1e-16, 0.0),
                ),
                Transform::from_xyz(1.0 * BLOCKS, 0.0, 0.0),
            ),
            (
                "west, full block",
                test_hit_intersection(
                    Vec3::new(2.5 * BLOCKS, 0.1 * BLOCKS, 1.2 * BLOCKS),
                    Vec3::new(-1.0, 1e-16, 0.0),
                ),
                Transform::from_xyz(2.0 * BLOCKS, 0.0, 1.0 * BLOCKS),
            ),
            (
                "top, full block",
                test_hit_intersection(
                    Vec3::new(3.25 * BLOCKS, 0.5 * BLOCKS, 2.25 * BLOCKS),
                    Vec3::new(0.0, 1.0, 0.0),
                ),
                Transform::from_xyz(3.0 * BLOCKS, 1.0 * BLOCKS, 2.0 * BLOCKS),
            ),
            (
                "east, non-full block",
                test_hit_intersection(
                    Vec3::new(3.3 * BLOCKS, 0.25 * BLOCKS, 0.2 * BLOCKS),
                    Vec3::new(1.0, 1e-16, 0.0),
                ),
                Transform::from_xyz(4.0 * BLOCKS, 0.0, 0.0),
            ),
            (
                "west, non-full block",
                test_hit_intersection(
                    Vec3::new(2.7 * BLOCKS, 0.25 * BLOCKS, 1.2 * BLOCKS),
                    Vec3::new(-1.0, 1e-16, 0.0),
                ),
                Transform::from_xyz(2.0 * BLOCKS, 0.0, 1.0 * BLOCKS),
            ),
        ];

        for (label, hit, expected) in test_cases {
            let (_, actual) = current_block_and_place_block_transform(hit);
            assert_eq!(actual, expected, "places block: {}", label);
        }
    }
}

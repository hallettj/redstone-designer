use bevy::{prelude::*, render::camera::RenderTarget};
use bevy_rapier3d::prelude::*;

use crate::{camera::MainCamera, constants::BLOCKS, lines::LineMaterial};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<LineMaterial>::default())
            .insert_resource(Cursor::default())
            .add_system_to_stage(CoreStage::PreUpdate, update_current_block);
    }
}

#[derive(Default, Debug)]
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
    // mut query_block_outlines: Query<(&mut Visibility, &Parent), With<BlockOutline>>,
) {
    let hit = get_entity_under_cursor(windows, rapier_context, query_camera);
    let hit_entity = hit.map(|h| h.entity);
    cursor.current_block = hit_entity;
}

#[derive(Clone, Debug, PartialEq)]
struct EntityHit {
    entity: Entity,
    intersection: RayIntersection,
}

fn get_entity_under_cursor(
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
    let max_toi = 16.0 * BLOCKS;
    let solid = true;
    let groups = InteractionGroups::all();
    let filter = QueryFilter::new().groups(groups);

    let (entity, intersection) =
        rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, max_toi, solid, filter)?;
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

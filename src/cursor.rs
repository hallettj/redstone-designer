use bevy::{prelude::*, render::camera::RenderTarget};
use bevy_rapier3d::prelude::*;

use crate::{block::BlockOutline, camera::MainCamera, constants::BLOCKS};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_click)
            .add_system(highlight_block_on_hover);
    }
}

fn handle_click(
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    input_mouse: Res<Input<MouseButton>>,
) {
    if input_mouse.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = query_camera.single();
        let window = if let RenderTarget::Window(id) = camera.target {
            windows.get(id).unwrap()
        } else {
            windows.get_primary().unwrap()
        };
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);
        if let Some(cursor_pos_screen) = window.cursor_position() {
            match get_entity_under_cursor(
                rapier_context,
                window_size,
                cursor_pos_screen,
                camera,
                camera_transform,
            ) {
                Some(hit) => println!("Hit {:?}", hit),
                None => println!("Tried to cast a ray, but got None"),
            }
        }
    }
}

fn highlight_block_on_hover(
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query_block_outlines: Query<(&mut Visibility, &Parent), With<BlockOutline>>,
) {
    let (camera, camera_transform) = query_camera.single();
    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);
    if let Some(cursor_pos_screen) = window.cursor_position() {
        let hit = get_entity_under_cursor(
            rapier_context,
            window_size,
            cursor_pos_screen,
            camera,
            camera_transform,
        );
        let hit_entity = hit.map(|h| h.entity);
        for (mut visibility, parent) in query_block_outlines.iter_mut() {
            visibility.is_visible = hit_entity.contains(&parent.get());
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct EntityHit {
    entity: Entity,
    intersection: RayIntersection,
}

fn get_entity_under_cursor(
    rapier_context: Res<RapierContext>,
    window_size: Vec2,
    cursor_pos_screen: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<EntityHit> {
    let (ray_pos, ray_dir) =
        ray_from_screenspace(window_size, cursor_pos_screen, camera, camera_transform);
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
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

    let origin = camera_transform.translation();
    let ray_direction = (origin - world_pos).normalize();

    (origin, ray_direction)
}

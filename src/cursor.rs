use bevy::{input::mouse::MouseWheel, prelude::*, render::camera::RenderTarget};
use bevy_rapier3d::prelude::*;

use crate::{camera::MainCamera, constants::BLOCKS};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_click);
        // app.add_startup_system(setup_control_plane)
        //     .add_system(update_control_plane_height);
    }
}

fn handle_click(
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    input_mouse: Res<Input<MouseButton>>,
) {
    if input_mouse.just_pressed(MouseButton::Left) {
        match cast_ray(windows, rapier_context, query_camera) {
            Some(_) => (),
            None => println!("Tried to cast a ray, but got None"),
        }
    }
}

fn cast_ray(
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<()> {
    let (camera, camera_transform) = query_camera.single();

    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    let cursor_pos_screen = window.cursor_position()?;
    let (ray_pos, ray_dir) =
        ray_from_screenspace(window, cursor_pos_screen, camera, camera_transform);
    let max_toi = 16.0 * BLOCKS;
    let solid = true;
    let groups = InteractionGroups::all();
    let filter = QueryFilter::new().groups(groups);

    if let Some((entity, toi)) = rapier_context.cast_ray(ray_pos, ray_dir, max_toi, solid, filter) {
        // The first collider hit has the entity `entity` and it hit after
        // the ray travelled a distance equal to `ray_dir * toi`.
        let hit_point = ray_pos + ray_dir * toi;
        println!("Entity {:?} hit at point {}", entity, hit_point);
    }

    if let Some((entity, intersection)) =
        rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, max_toi, solid, filter)
    {
        // This is similar to `QueryPipeline::cast_ray` illustrated above except
        // that it also returns the normal of the collider shape at the hit point.
        let hit_point = intersection.point;
        let hit_normal = intersection.normal;
        println!(
            "Entity {:?} hit at point {} with normal {}",
            entity, hit_point, hit_normal
        );
    }

    rapier_context.intersections_with_ray(
        ray_pos,
        ray_dir,
        max_toi,
        solid,
        filter,
        |entity, intersection| {
            // Callback called on each collider hit by the ray.
            let hit_point = intersection.point;
            let hit_normal = intersection.normal;
            println!(
                "Entity {:?} hit at point {} with normal {}",
                entity, hit_point, hit_normal
            );
            true // Return `false` instead if we want to stop searching for other hits.
        },
    );

    Some(())
}

/// Returns origin and direction for a ray from the camera through the cursor. This involves
/// reversing the camera projection to map the cursor's coordinates in screen space to a set of
/// coordinates in world space.
fn ray_from_screenspace(
    window: &Window,
    cursor_pos_screen: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> (Vec3, Vec3) {
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);

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

#[derive(Component, Default)]
struct ControlPlane {
    /// Height in blocks. This should never be less than -1, or more than 15.
    height: f32,
}

fn update_control_plane_height(
    mut ev_scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut ControlPlane, &mut Transform)>,
) {
    let scroll = ev_scroll.iter().fold(0.0, |accum, ev| accum + ev.y);
    for (mut control_plane, mut transform) in query.iter_mut() {
        if scroll != 0. {
            let height = control_plane.height + scroll.floor();
            control_plane.height = height.clamp(-1.0, 15.0);
            // Add 1 pixel to displayed height so that control plane is not embedded inside blocks
            // on lower layer.
            transform.translation.y = control_plane_render_height(control_plane.into_inner());
        }
    }
}

fn setup_control_plane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let control_plane = ControlPlane {
        height: 0.0,
        ..default()
    };
    let image_handle = asset_server.load("local/control_plane.png");
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 16.0 * BLOCKS,
            })),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(image_handle),
                alpha_mode: AlphaMode::Blend,
                cull_mode: None, // render both sides
                unlit: true,
                ..default()
            }),
            transform: Transform::from_xyz(
                8.0 * BLOCKS,
                control_plane_render_height(&control_plane),
                8.0 * BLOCKS,
            ),
            ..default()
        })
        .insert(control_plane);
}

/// Render control plane slightly above the blocks below so that the mesh is not embedded partially
/// inside meshes below.
fn control_plane_render_height(control_plane: &ControlPlane) -> f32 {
    control_plane.height * BLOCKS + 0.1
}

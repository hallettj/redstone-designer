use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::constants::{BLOCKS, PIXELS};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_control_plane)
            .add_system(update_control_plane_height);
    }
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
            transform.translation.y = control_plane.height * BLOCKS + 1.0 * PIXELS;
        }
    }
}

fn setup_control_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let control_plane = ControlPlane {
        height: 0.0,
        ..default()
    };
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 16.0 * BLOCKS,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.8, 0., 0.8, 0.33).into(),
                alpha_mode: AlphaMode::Blend,
                cull_mode: None, // render both sides
                unlit: true,
                ..default()
            }),
            transform: Transform::from_xyz(
                8.0 * BLOCKS,
                control_plane.height + 1.0 * PIXELS,
                8.0 * BLOCKS,
            ),
            ..default()
        })
        .insert(control_plane);
}

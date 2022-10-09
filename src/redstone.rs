use crate::{
    block::{load_block_material, setup_block},
    constants::BLOCKS,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct RedstonePlugin;

impl Plugin for RedstonePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_floor)
            .add_startup_system(setup_block)
            .add_startup_system(setup_lights);
    }
}

fn setup_floor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sandstone_material = load_block_material(
        &asset_server,
        &mut materials,
        "minecraft/assets/minecraft/textures/block/sandstone_top.png",
    );

    for x in 0..16 {
        for z in 0..16 {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 * BLOCKS })),
                    material: sandstone_material.clone(),
                    transform: Transform::from_xyz(
                        x as f32 * BLOCKS + 0.5 * BLOCKS, // shift by 0.5 to align edge with origin
                        0.0,
                        z as f32 * BLOCKS + 0.5 * BLOCKS,
                    ),
                    ..default()
                })
                .insert(Collider::cuboid(0.5 * BLOCKS, 0.0, 0.5 * BLOCKS));
        }
    }
}

fn setup_lights(mut commands: Commands) {
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    // directional 'sun' light
    const HALF_SIZE: f32 = 10.0 * BLOCKS;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20_000.0,
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(8.0 * BLOCKS, 2.0 * BLOCKS, 8.0 * BLOCKS),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });
}

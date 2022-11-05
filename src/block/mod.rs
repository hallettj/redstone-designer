mod bounding_box;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology, view::RenderLayers},
};
use minecraft_assets::{
    api::{AssetPack, ModelResolver},
    schemas::models::{BlockFace, Element, Texture},
};

use crate::{
    block_picker::SelectedBlockType,
    constants::{BLOCKS, BLOCK_FACES},
    cursor::Cursor,
    lines::LineMaterial,
    user_input::{sent_command, UiCommand},
};

use self::bounding_box::{
    bounding_box_for_block_model, bounding_box_to_collider, bounding_box_to_line_list,
};

#[derive(Component, Clone, Default)]
pub struct Block;

#[derive(Component, Clone, Default)]
pub struct BlockOutline;

#[derive(Bundle, Clone, Default)]
pub struct BlockBundle {
    block: Block,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<LineMaterial>::default())
            .add_startup_system(spawn_test_block)
            .add_system(highlight_block_on_hover)
            .add_system(place_block)
            .add_system(destroy_block);
    }
}

fn highlight_block_on_hover(
    cursor: Res<Cursor>,
    mut query_block_outlines: Query<(&mut Visibility, &Parent), With<BlockOutline>>,
) {
    if !cursor.is_changed() {
        return;
    }
    for (mut visibility, parent) in query_block_outlines.iter_mut() {
        let is_visible = cursor.current_block.contains(&parent.get());
        if visibility.is_visible != is_visible {
            visibility.is_visible = is_visible;
        }
    }
}

fn place_block(
    selected: Res<SelectedBlockType>,
    user_input: EventReader<UiCommand>,
    cursor: Res<Cursor>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    if sent_command(user_input, UiCommand::PlaceBlock) {
        if let Some(transform) = cursor.place_block_transform {
            spawn_block(
                &mut commands,
                &asset_server,
                &mut meshes,
                &mut materials,
                &mut line_materials,
                selected.block_type,
                transform,
            )
        }
    }
}

fn destroy_block(mut commands: Commands, user_input: EventReader<UiCommand>, cursor: Res<Cursor>) {
    if sent_command(user_input, UiCommand::DestroyBlock) {
        if let Some(block) = cursor.current_block {
            commands.entity(block).despawn_recursive();
        }
    }
}

pub fn spawn_test_block(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    spawn_block(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        &mut line_materials,
        "repeater_2tick",
        Transform::from_xyz(8.0 * BLOCKS, 0.0, 8.0 * BLOCKS),
    )
}

pub fn spawn_block(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    mut line_materials: &mut ResMut<Assets<LineMaterial>>,
    block_model: &str,
    transform: Transform,
) {
    // TODO: Get AssetPack as a resource; implement custom loader that uses AssetServer
    let assets = AssetPack::at_path("assets/minecraft/");
    let models = assets.load_block_model_recursive(block_model).unwrap();
    let model = ModelResolver::resolve_model(models.iter());
    let elements = model.elements.unwrap();
    let bounding_box = bounding_box_for_block_model(block_model, &elements);
    commands
        .spawn_bundle(BlockBundle {
            transform,
            ..default()
        })
        .insert(bounding_box_to_collider(bounding_box))
        .with_children(|parent| {
            for element in elements.iter() {
                spawn_element(
                    parent,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    element,
                    None as Option<RenderLayers>,
                );
            }
            spawn_block_outline(parent, &mut meshes, &mut line_materials, bounding_box);
        });
}

/// Spawn a block to display in the block picker, not in the simulation world.
/// TODO: rename this to be less similar to `spawn_block_preview`
/// TODO: reuse this logic in `spawn_block`
pub fn spawn_for_block_preview(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    block_model: &str,
    transform: Transform,

    // Component to insert in the entity, and into children.
    recursive_component: Option<impl Component + Clone + std::fmt::Debug>,
) -> Entity {
    // TODO: Get AssetPack as a resource; implement custom loader that uses AssetServer
    let assets = AssetPack::at_path("assets/minecraft/");
    let models = assets.load_block_model_recursive(block_model).unwrap();
    let model = ModelResolver::resolve_model(models.iter());
    let elements = model.elements.unwrap();
    let mut block = commands.spawn_bundle(BlockBundle {
        transform,
        ..default()
    });
    block.with_children(|parent| {
        for element in elements.iter() {
            spawn_element(
                parent,
                &asset_server,
                &mut meshes,
                &mut materials,
                element,
                recursive_component.clone(),
            );
        }
    });
    if let Some(component) = recursive_component {
        block.insert(component);
    }
    block.id()
}

/// This is the black wireframe that is displayed around a block on hover.
fn spawn_block_outline(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<LineMaterial>>,
    bounding_box: (Vec3, Vec3),
) {
    parent
        .spawn_bundle(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(bounding_box_to_line_list(bounding_box))),
            material: materials.add(LineMaterial::new(Color::BLACK)),
            visibility: Visibility { is_visible: false },
            ..default()
        })
        .insert(BlockOutline);
}

/// Each Minecraft model is made up of one or more "elements" which are boxes that may have
/// different textures on each face.
fn spawn_element(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    element: &Element,
    component: Option<impl Component + Clone>,
) {
    for face in BLOCK_FACES {
        if let Some(mesh) = mesh_for_face(&element, face) {
            // TODO: would there be a benefit to memoizing materials?
            let material = materials.add(material_for_face(asset_server, &element, face));
            let mut element = parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                material,
                ..default()
            });
            if let Some(component) = component.clone() {
                element.insert(component);
            }
        }
    }
}

fn material_for_face(
    asset_server: &Res<AssetServer>,
    element: &Element,
    face: BlockFace,
) -> StandardMaterial {
    if let Some(element_face) = element.faces.get(&face) {
        if let Some(path) = texture_path(&element_face.texture) {
            let image_handle = asset_server.load(&path);
            return StandardMaterial {
                base_color_texture: Some(image_handle),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            };
        }
    }
    Color::rgb(0.8, 0., 0.8).into()
}

fn texture_path(texture: &Texture) -> Option<String> {
    let location = texture.location()?;
    let mut parts = location.split(":").take(2);
    let namespace = parts.next()?;
    let name_option = parts.next();
    Some(match name_option {
        Some(name) => format!("minecraft/assets/{}/textures/{}.png", namespace, name),
        None => {
            let name = namespace;
            let namespace = "minecraft";
            format!("minecraft/assets/{}/textures/{}.png", namespace, name)
        }
    })
}

fn mesh_for_face(element: &Element, face: BlockFace) -> Option<Mesh> {
    let [min_x, min_y, min_z] = element.from;
    let [max_x, max_y, max_z] = element.to;

    let element_face = element.faces.get(&face)?;

    // Minecraft models provide a pair of uv pixel coordinates for each element face in the form,
    //
    //     [min_u inclusive, min_v inclusive, max_u exclusive, max_v exclusive]
    //
    // The coordinates are integers representing pixel coordinates in a texture image that is most
    // often 16×16 pixels. Bevy uses floating point uv coordinates from 0.0 to 1.0. So we have to
    // do some translation.
    //
    // If a model does not specify uv coordinates then the default values are [0, 0, 16, 16].
    let uv = element_face.uv.unwrap_or([0., 0., 16.0, 16.0]);
    // TODO: Check image size instead of assuming that all image files are 16×16 px
    let (min_u, min_v, max_u, max_v) = (uv[0] / 16.0, uv[1] / 16.0, uv[2] / 16.0, uv[3] / 16.0);

    // Minecraft textures are flipped - if you look at the image it looks right-side up, but the
    // origin for uv coordinates is the top-left of the image. So for example to correct vertically
    // on north, south, east, and west faces min_y maps to max_v, and max_y maps to min_v.
    let vertices = match face {
        BlockFace::Up => [
            ([max_x, max_y, min_z], [0., 1.0, 0.], [max_u, min_v]),
            ([min_x, max_y, min_z], [0., 1.0, 0.], [min_u, min_v]),
            ([min_x, max_y, max_z], [0., 1.0, 0.], [min_u, max_v]),
            ([max_x, max_y, max_z], [0., 1.0, 0.], [max_u, max_v]),
        ],
        BlockFace::Down => [
            ([max_x, min_y, max_z], [0., -1.0, 0.], [max_u, min_v]),
            ([min_x, min_y, max_z], [0., -1.0, 0.], [min_u, min_v]),
            ([min_x, min_y, min_z], [0., -1.0, 0.], [min_u, max_v]),
            ([max_x, min_y, min_z], [0., -1.0, 0.], [max_u, max_v]),
        ],
        BlockFace::North => [
            ([min_x, max_y, min_z], [0., 0., -1.0], [max_u, min_v]),
            ([max_x, max_y, min_z], [0., 0., -1.0], [min_u, min_v]),
            ([max_x, min_y, min_z], [0., 0., -1.0], [min_u, max_v]),
            ([min_x, min_y, min_z], [0., 0., -1.0], [max_u, max_v]),
        ],
        BlockFace::South => [
            ([min_x, min_y, max_z], [0., 0., 1.0], [min_u, max_v]),
            ([max_x, min_y, max_z], [0., 0., 1.0], [max_u, max_v]),
            ([max_x, max_y, max_z], [0., 0., 1.0], [max_u, min_v]),
            ([min_x, max_y, max_z], [0., 0., 1.0], [min_u, min_v]),
        ],
        BlockFace::East => [
            ([max_x, min_y, min_z], [1.0, 0., 0.], [max_u, max_v]),
            ([max_x, max_y, min_z], [1.0, 0., 0.], [max_u, min_v]),
            ([max_x, max_y, max_z], [1.0, 0., 0.], [min_u, min_v]),
            ([max_x, min_y, max_z], [1.0, 0., 0.], [min_u, max_v]),
        ],
        BlockFace::West => [
            ([min_x, min_y, max_z], [-1.0, 0., 0.], [max_u, max_v]),
            ([min_x, max_y, max_z], [-1.0, 0., 0.], [max_u, min_v]),
            ([min_x, max_y, min_z], [-1.0, 0., 0.], [min_u, min_v]),
            ([min_x, min_y, min_z], [-1.0, 0., 0.], [min_u, max_v]),
        ],
    };

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let indices = Indices::U32(vec![0, 1, 2, 2, 3, 0]);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    Some(mesh)
}

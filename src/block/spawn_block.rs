use crate::block::BlockBundle;
use crate::{block::block_state::BlockState, util::degrees_to_radians};
use anyhow::{anyhow, Context, Result};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use minecraft_assets::{
    api::{AssetPack, ModelResolver},
    schemas::{
        blockstates::{ModelProperties, Variant},
        models::{BlockFace, Element, Texture},
        Model,
    },
};

use crate::{constants::BLOCK_FACES, lines::LineMaterial};

use super::{
    bounding_box::{
        bounding_box_for_block_model, bounding_box_to_collider, bounding_box_to_line_list,
    },
    BlockOutline,
};

pub fn spawn_block(
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mut line_materials: &mut ResMut<Assets<LineMaterial>>,
    (block_type, initial_state): (&str, BlockState),
    transform: Transform,
) {
    let (model, model_properties) = get_block_model_metadata(block_type, &initial_state).unwrap();
    let bounding_box = {
        let elements = model.elements.as_ref().unwrap();
        bounding_box_for_block_model(block_type, &elements)
    };
    let block = spawn_block_common(
        &mut commands,
        asset_server,
        meshes,
        materials,
        model,
        model_properties,
        transform,
        None as Option<BlockOutline>, // the choice of component type here does not matter
    )
    .unwrap();
    commands
        .entity(block)
        .insert(bounding_box_to_collider(bounding_box))
        .insert(initial_state)
        .with_children(|parent| {
            spawn_block_outline(parent, &mut meshes, &mut line_materials, bounding_box);
        });
}

/// Spawn a block to display in the block picker, not in the simulation world.
/// TODO: rename this to be less similar to `spawn_block_preview`
/// TODO: reuse this logic in `spawn_block`
pub fn spawn_block_preview_for_block_picker(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    block_type: &str,
    transform: Transform,

    // Component to insert in the entity, and into children.
    recursive_component: Option<impl Component + Clone>,
) -> Result<Entity> {
    // TODO: Get AssetPack as a resource; implement custom loader that uses AssetServer
    let asset_pack = AssetPack::at_path("assets/minecraft/");
    let initial_state = BlockState::initial_state_for(asset_pack, block_type)?;
    let (model, model_properties) = get_block_model_metadata(block_type, &initial_state)?;
    spawn_block_common(
        commands,
        asset_server,
        meshes,
        materials,
        model,
        model_properties,
        transform,
        recursive_component,
    )
}

fn spawn_block_common(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    model: Model,
    model_properties: ModelProperties,
    mut transform: Transform,
    // Component to insert in the entity, and into children.
    recursive_component: Option<impl Component + Clone>,
) -> Result<Entity> {
    transform.rotate_y(degrees_to_radians(model_properties.y));
    transform.rotate_x(degrees_to_radians(model_properties.x));
    let elements = model
        .elements
        .ok_or(anyhow!("block model has no elements"))?;
    let mut block = commands.spawn(BlockBundle {
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
    Ok(block.id())
}

fn get_block_model_metadata(
    block_type: &str,
    state: &BlockState,
) -> Result<(Model, ModelProperties)> {
    // TODO: Get AssetPack as a resource; implement custom loader that uses AssetServer
    let assets = AssetPack::at_path("assets/minecraft/");
    let block_states = assets
        .load_blockstates(block_type)
        .with_context(|| format!("no block states found for \"{}\"", block_type))?;
    let variant = state.active_variant(block_states);
    match variant {
        Variant::Single(model_properties) => {
            let models = assets
                .load_block_model_recursive(&model_properties.model)
                .with_context(|| format!("no block model found for \"{}\"", block_type))?;
            let model = ModelResolver::resolve_model(models.iter());
            Ok((model, model_properties))
        }
        Variant::Multiple(_) => Err(anyhow!(
            "multiple variant models are not supported yet; {} @ {:?}",
            block_type,
            state
        )),
    }
}

/// This is the black wireframe that is displayed around a block on hover.
fn spawn_block_outline(
    parent: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<LineMaterial>>,
    bounding_box: (Vec3, Vec3),
) {
    parent
        .spawn(MaterialMeshBundle {
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
        if let Some((mesh, transform)) = mesh_for_face(&element, face) {
            // TODO: would there be a benefit to memoizing materials?
            let material = materials.add(material_for_face(asset_server, &element, face));
            let mut element = parent.spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material,
                transform,
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

/// Minecraft models are made up of elements (boxes) which are made up of faces which can each have
/// a different texture and UV mapping. This function returns a mesh for a single element face
/// paired with a transform to move it into the correct position relative to the rest of the model.
fn mesh_for_face(element: &Element, face: BlockFace) -> Option<(Mesh, Transform)> {
    // Minecraft uses the corner of the block as its coordinate origin. But we want to use the
    // center of the block. To compensate, translate each vertex position by half a block.
    let [min_x, min_y, min_z] = element.from.map(|c| c - 8.0);
    let [max_x, max_y, max_z] = element.to.map(|c| c - 8.0);

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
            ([max_x, 0.0, min_z], [0., 1.0, 0.], [max_u, min_v]),
            ([min_x, 0.0, min_z], [0., 1.0, 0.], [min_u, min_v]),
            ([min_x, 0.0, max_z], [0., 1.0, 0.], [min_u, max_v]),
            ([max_x, 0.0, max_z], [0., 1.0, 0.], [max_u, max_v]),
        ],
        BlockFace::Down => [
            ([max_x, 0.0, max_z], [0., -1.0, 0.], [max_u, min_v]),
            ([min_x, 0.0, max_z], [0., -1.0, 0.], [min_u, min_v]),
            ([min_x, 0.0, min_z], [0., -1.0, 0.], [min_u, max_v]),
            ([max_x, 0.0, min_z], [0., -1.0, 0.], [max_u, max_v]),
        ],
        BlockFace::North => [
            ([min_x, max_y, 0.0], [0., 0., -1.0], [max_u, min_v]),
            ([max_x, max_y, 0.0], [0., 0., -1.0], [min_u, min_v]),
            ([max_x, min_y, 0.0], [0., 0., -1.0], [min_u, max_v]),
            ([min_x, min_y, 0.0], [0., 0., -1.0], [max_u, max_v]),
        ],
        BlockFace::South => [
            ([min_x, min_y, 0.0], [0., 0., 1.0], [min_u, max_v]),
            ([max_x, min_y, 0.0], [0., 0., 1.0], [max_u, max_v]),
            ([max_x, max_y, 0.0], [0., 0., 1.0], [max_u, min_v]),
            ([min_x, max_y, 0.0], [0., 0., 1.0], [min_u, min_v]),
        ],
        BlockFace::East => [
            ([0.0, min_y, min_z], [1.0, 0., 0.], [max_u, max_v]),
            ([0.0, max_y, min_z], [1.0, 0., 0.], [max_u, min_v]),
            ([0.0, max_y, max_z], [1.0, 0., 0.], [min_u, min_v]),
            ([0.0, min_y, max_z], [1.0, 0., 0.], [min_u, max_v]),
        ],
        BlockFace::West => [
            ([0.0, min_y, max_z], [-1.0, 0., 0.], [max_u, max_v]),
            ([0.0, max_y, max_z], [-1.0, 0., 0.], [max_u, min_v]),
            ([0.0, max_y, min_z], [-1.0, 0., 0.], [min_u, min_v]),
            ([0.0, min_y, min_z], [-1.0, 0., 0.], [min_u, max_v]),
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

    let transform = match face {
        BlockFace::Up => Transform::from_translation(Vec3::new(0.0, max_y, 0.0)),
        BlockFace::Down => Transform::from_translation(Vec3::new(0.0, min_y, 0.0)),
        BlockFace::North => Transform::from_translation(Vec3::new(0.0, 0.0, min_z)),
        BlockFace::South => Transform::from_translation(Vec3::new(0.0, 0.0, max_z)),
        BlockFace::East => Transform::from_translation(Vec3::new(max_x, 0.0, 0.0)),
        BlockFace::West => Transform::from_translation(Vec3::new(min_x, 0.0, 0.0)),
    };

    Some((mesh, transform))
}

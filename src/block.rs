use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use minecraft_assets::{
    api::{AssetPack, ModelResolver},
    schemas::models::{BlockFace, Element},
};

/// The model scale in use sets 1.0 unit of distance in the render space to be
/// one Minecraft "pixel". A Minecraft block is 16 pixels.
const BLOCKS: f32 = 16.0;

pub fn setup_block(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = AssetPack::at_path("assets/minecraft/");
    let models = assets.load_block_model_recursive("repeater_2tick").unwrap();
    let model = ModelResolver::resolve_model(models.iter());

    let transform = Transform::from_xyz(8.0 * BLOCKS, 0.0, 8.0 * BLOCKS);

    if let Some(elements) = model.elements {
        for element in elements {
            render_element(
                &mut commands,
                &asset_server,
                &mut meshes,
                &mut materials,
                element,
                transform,
            );
        }
    }
}

fn render_element(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    element: Element,
    transform: Transform,
) {
    let faces = [
        BlockFace::Down,
        BlockFace::Up,
        BlockFace::North,
        BlockFace::South,
        BlockFace::West,
        BlockFace::East,
    ];
    for face in faces {
        if let Some(mesh) = mesh_for_face(&element, face) {
            // TODO: memoize materials
            let material = materials.add(material_for_face(&asset_server, &element, face));
            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                material,
                transform,
                ..default()
            });
        }
    }
}

fn material_for_face(
    asset_server: &Res<AssetServer>,
    element: &Element,
    face: BlockFace,
) -> StandardMaterial {
    if let Some(element_face) = element.faces.get(&face) {
        if let Some(loc) = element_face.texture.location() {
            let image_path = "minecraft/assets/minecraft/textures/".to_owned() + loc + ".png";
            let image_handle = asset_server.load(&image_path);
            return StandardMaterial {
                base_color_texture: Some(image_handle),
                alpha_mode: AlphaMode::Opaque,
                ..default()
            };
        }
    }
    Color::rgb(0.8, 0., 0.8).into()
}

fn mesh_for_face(element: &Element, face: BlockFace) -> Option<Mesh> {
    let [min_x, min_y, min_z] = element.from;
    let [max_x, max_y, max_z] = element.to;

    let element_face = element.faces.get(&face)?;
    let uv = element_face.uv.unwrap_or([0., 0., 16.0, 16.0]);

    let vertices = match face {
        BlockFace::Up => [
            ([min_x, min_y, max_z], [0., 0., 1.0], [0., 0.]),
            ([max_x, min_y, max_z], [0., 0., 1.0], [1.0, 0.]),
            ([max_x, max_y, max_z], [0., 0., 1.0], [1.0, 1.0]),
            ([min_x, max_y, max_z], [0., 0., 1.0], [0., 1.0]),
        ],
        BlockFace::Down => [
            ([min_x, max_y, min_z], [0., 0., -1.0], [1.0, 0.]),
            ([max_x, max_y, min_z], [0., 0., -1.0], [0., 0.]),
            ([max_x, min_y, min_z], [0., 0., -1.0], [0., 1.0]),
            ([min_x, min_y, min_z], [0., 0., -1.0], [1.0, 1.0]),
        ],
        BlockFace::North => [
            ([max_x, min_y, max_z], [0., -1.0, 0.], [0., 0.]),
            ([min_x, min_y, max_z], [0., -1.0, 0.], [1.0, 0.]),
            ([min_x, min_y, min_z], [0., -1.0, 0.], [1.0, 1.0]),
            ([max_x, min_y, min_z], [0., -1.0, 0.], [0., 1.0]),
        ],
        BlockFace::South => [
            ([max_x, max_y, min_z], [0., 1.0, 0.], [1.0, 0.]),
            ([min_x, max_y, min_z], [0., 1.0, 0.], [0., 0.]),
            ([min_x, max_y, max_z], [0., 1.0, 0.], [0., 1.0]),
            ([max_x, max_y, max_z], [0., 1.0, 0.], [1.0, 1.0]),
        ],
        BlockFace::East => [
            ([max_x, min_y, min_z], [1.0, 0., 0.], [0., 0.]),
            ([max_x, max_y, min_z], [1.0, 0., 0.], [1.0, 0.]),
            ([max_x, max_y, max_z], [1.0, 0., 0.], [1.0, 1.0]),
            ([max_x, min_y, max_z], [1.0, 0., 0.], [0., 1.0]),
        ],
        BlockFace::West => [
            ([min_x, min_y, max_z], [-1.0, 0., 0.], [1.0, 0.]),
            ([min_x, max_y, max_z], [-1.0, 0., 0.], [0., 0.]),
            ([min_x, max_y, min_z], [-1.0, 0., 0.], [0., 1.0]),
            ([min_x, min_y, min_z], [-1.0, 0., 0.], [1.0, 1.0]),
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

pub fn load_block_material(
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_path: &str,
) -> Handle<StandardMaterial> {
    let image_handle = asset_server.load(asset_path);
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle.clone()),
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });
    material_handle
}

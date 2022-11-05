use anyhow::Result;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::{Projection, RenderTarget},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};

use crate::{
    block::{block_state::BlockState, spawn_block_preview_for_block_picker},
    constants::{block_from_palette, BLOCKS, BLOCK_PALETTE, BLOCK_PREVIEW_LAYER},
    user_input::{sent_command, UiCommand},
};

const BLOCK_PREVIEW_SIZE: u32 = 100; // px

const PICKER_BACKGROUND_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const NORMAL_BUTTON_COLOR: Color = Color::rgb(0.95, 0.95, 0.95);
const HOVERED_BUTTON_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
const PRESSED_BUTTON_COLOR: Color = Color::rgb(0.98, 0.98, 0.98);

pub struct BlockPickerPlugin;

impl Plugin for BlockPickerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectedBlockType {
            block: block_from_palette(BLOCK_PALETTE[0].0),
        })
        .add_startup_system(spawn_block_picker)
        .add_system(show_block_picker)
        .add_system(hide_block_picker)
        .add_system(button_system.before(hide_block_picker));
    }
}

#[derive(Debug)]
pub struct SelectedBlockType {
    pub block: (&'static str, BlockState),
}

#[derive(Component, Debug, Default)]
pub struct BlockPicker {
    pub is_open: bool,
}

#[derive(Component)]
struct BlockPreview;

#[derive(Component, Clone, Debug, PartialEq)]
struct BlockPickerButton {
    block: (&'static str, BlockState),
}

fn spawn_block_picker(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let block_preview_image_handles = BLOCK_PALETTE
        .iter()
        .enumerate()
        .map(|(index, block_type)| {
            spawn_block_preview(
                &mut commands,
                &asset_server,
                &mut meshes,
                &mut materials,
                &mut images,
                block_type.0,
                index,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                display: Display::None,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(66.0), Val::Percent(66.0)),
                        border: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    color: Color::rgb(0.6, 0.6, 0.6).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                align_items: AlignItems::FlexEnd,
                                justify_content: JustifyContent::FlexStart,
                                ..default()
                            },
                            color: PICKER_BACKGROUND_COLOR.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            for (index, (block_type, _)) in BLOCK_PALETTE.iter().enumerate() {
                                parent
                                    .spawn_bundle(ButtonBundle {
                                        image: block_preview_image_handles[index].clone().into(),
                                        style: Style {
                                            size: Size::new(
                                                Val::Px(BLOCK_PREVIEW_SIZE as f32),
                                                Val::Px(BLOCK_PREVIEW_SIZE as f32),
                                            ),
                                            margin: UiRect {
                                                top: Val::Px(6.0),
                                                left: Val::Px(6.0),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                        color: NORMAL_BUTTON_COLOR.into(),
                                        ..default()
                                    })
                                    .insert(BlockPickerButton {
                                        block: block_from_palette(block_type),
                                    })
                                    .with_children(|parent| {
                                        // It seems that we need to have a child for the button to
                                        // work.
                                        parent
                                            .spawn_bundle(TextBundle::from_section("", default()));
                                    });
                            }
                        });
                });
        })
        .insert(BlockPicker::default());
}

/// Renders a block to a texture, and returns an image handle so that the texture can be displayed
/// in the block picker UI.
fn spawn_block_preview(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    block_model: &str,
    index: usize,
) -> Result<Handle<Image>> {
    // This code for rendering to a texture is taken from one of the Bevy examples,
    // https://github.com/bevyengine/bevy/blob/main/examples/3d/render_to_texture.rs

    let size = Extent3d {
        width: BLOCK_PREVIEW_SIZE,
        height: BLOCK_PREVIEW_SIZE,
        ..default()
    };

    // This is the texture that will be rendered to
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };

    // Fill image.data with zeros
    image.resize(size);

    let image_handle = images.add(image);

    let translate_out_of_the_way_of_other_block_previews =
        Vec3::new(5.0 * BLOCKS * index as f32, 0.0, 0.0);

    let block = spawn_block_preview_for_block_picker(
        commands,
        asset_server,
        meshes,
        materials,
        block_model,
        Transform::from_translation(translate_out_of_the_way_of_other_block_previews),
        // Make the block visible to the camera below, and not to the main camera
        Some(BLOCK_PREVIEW_LAYER),
    )?;
    commands.entity(block).insert(BlockPreview);

    // TODO: If we shift the block 0.5 * BLOCKS over when we spawn it then we won't have to do all
    // of these corrections with cameras.
    let center_of_block = Vec3::new(0.5 * BLOCKS, 0.5 * BLOCKS, 0.5 * BLOCKS)
        + translate_out_of_the_way_of_other_block_previews;

    commands
        .spawn_bundle(Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgba(1.0, 1.0, 1.0, 0.0)),
                ..default()
            },
            camera: Camera {
                // render before the main camera
                priority: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            projection: Projection::Orthographic(OrthographicProjection {
                scale: 24.0 / BLOCK_PREVIEW_SIZE as f32, // smaller numbers here make the block look bigger
                ..default()
            }),
            transform: Transform::from_translation(
                center_of_block + Vec3::new(1.0 * BLOCKS, 0.66 * BLOCKS, 1.0 * BLOCKS),
            )
            .looking_at(center_of_block, Vec3::Y),
            ..default()
        })
        // only render entities that are also in the same layer - block previews are also moved to
        // this layer by the `move_block_previews_to_ui_layer` system
        .insert(BLOCK_PREVIEW_LAYER)
        // avoid recursion due to the camera attempting to render the image that it renders, per
        // https://github.com/bevyengine/bevy/issues/6181
        .insert(UiCameraConfig { show_ui: false });

    Ok(image_handle)
}

fn show_block_picker(
    user_input: EventReader<UiCommand>,
    mut query: Query<(&mut BlockPicker, &mut Style)>,
) {
    if sent_command(user_input, UiCommand::OpenBlockPicker) {
        let (mut picker, mut picker_style) = query.single_mut();
        picker.is_open = true;
        picker_style.display = Display::Flex;
    }
}

// TODO: Do we want to hide/disable block previews? Do they render when the texture output is not
// visible?
fn hide_block_picker(
    user_input: EventReader<UiCommand>,
    mut query: Query<(&mut BlockPicker, &mut Style)>,
) {
    if sent_command(user_input, UiCommand::CloseBlockPicker) {
        let (mut picker, mut picker_style) = query.single_mut();
        picker.is_open = false;
        picker_style.display = Display::None;
    }
}

fn button_system(
    mut button_query: Query<(&Interaction, &BlockPickerButton, &mut UiColor), Changed<Interaction>>,
    mut selected: ResMut<SelectedBlockType>,
    mut ev_ui_command: EventWriter<UiCommand>,
) {
    for (interaction, button, mut color) in &mut button_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON_COLOR.into();
                selected.block = button.block.clone();
                ev_ui_command.send(UiCommand::CloseBlockPicker);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON_COLOR.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON_COLOR.into();
            }
        }
    }
}

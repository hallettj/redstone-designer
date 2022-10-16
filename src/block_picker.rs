use bevy::prelude::*;

use crate::user_input::{UICommand, UserInput};

pub struct BlockPickerPlugin;

impl Plugin for BlockPickerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_block_picker)
            .add_system(show_block_picker)
            .add_system(hide_block_picker);
    }
}

#[derive(Component, Default)]
pub struct BlockPicker {
    pub is_open: bool,
}

fn spawn_block_picker(mut commands: Commands) {
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
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(66.0), Val::Percent(66.0)),
                    justify_content: JustifyContent::SpaceAround,
                    ..default()
                },
                color: Color::rgb(0.8, 0.8, 0.8).into(),
                ..default()
            });
        })
        .insert(BlockPicker::default());
}

fn show_block_picker(user_input: Res<UserInput>, mut query: Query<(&mut BlockPicker, &mut Style)>) {
    if user_input.sent_command(UICommand::OpenBlockPicker) {
        let (mut picker, mut picker_style) = query.single_mut();
        picker.is_open = true;
        picker_style.display = Display::Flex;
    }
}

fn hide_block_picker(user_input: Res<UserInput>, mut query: Query<(&mut BlockPicker, &mut Style)>) {
    if user_input.sent_command(UICommand::CloseBlockPicker) {
        let (mut picker, mut picker_style) = query.single_mut();
        picker.is_open = false;
        picker_style.display = Display::None;
    }
}

use bevy::prelude::*;

use crate::block_picker::BlockPicker;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UICommand {
    PlaceBlock,
    DestroyBlock,
    OpenBlockPicker,
    CloseBlockPicker,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    ActivateTool(Tool),
    Click,
    ExitMode,
    ToggleBlockPicker,
}

#[derive(Clone, Debug)]
pub struct KeyBindings(Vec<(Action, Binding)>);

impl Default for KeyBindings {
    fn default() -> Self {
        KeyBindings(vec![
            (
                Action::Click,
                Binding {
                    key: Key::Mouse(MouseButton::Left),
                    binding_mode: BindingMode::Tap,
                    modes: vec![Mode::Normal],
                },
            ),
            (
                Action::ActivateTool(Tool::Destroy),
                Binding {
                    key: Key::Keyboard(KeyCode::X),
                    binding_mode: BindingMode::Hold,
                    modes: vec![Mode::Normal],
                },
            ),
            (
                Action::ExitMode,
                Binding {
                    key: Key::Keyboard(KeyCode::Escape),
                    binding_mode: BindingMode::Tap,
                    modes: vec![Mode::UI],
                },
            ),
            (
                Action::ToggleBlockPicker,
                Binding {
                    key: Key::Keyboard(KeyCode::P),
                    binding_mode: BindingMode::Tap,
                    modes: vec![Mode::Normal, Mode::UI],
                },
            ),
        ])
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tool {
    Place,
    Destroy,
}

const DEFAULT_TOOL: Tool = Tool::Place;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    key: Key,
    binding_mode: BindingMode,
    modes: Vec<Mode>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BindingMode {
    /// In this mode tapping a key or mouse button activates the binding.
    Tap,

    /// In this mode the binding is active when holding a key or mouse button. For toggleable
    /// states, like switching to the destroy tool, or opening the block picker, the state is
    /// active while the bound key is held, and inactivated when the key is released.
    Hold,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    /// Normal world interaction; e.g. clicking to place blocks against other blocks
    Normal,

    /// A UI window is open, such as the block picker
    UI,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

#[derive(Debug)]
pub struct UserInput {
    pub commands: Vec<UICommand>,
    active_tool: Tool,
    last_active_tool: Tool,
}

impl UserInput {
    pub fn sent_command(&self, command: UICommand) -> bool {
        self.commands.iter().any(|c| c == &command)
    }
}

impl Default for UserInput {
    fn default() -> Self {
        UserInput {
            commands: vec![],
            active_tool: DEFAULT_TOOL,
            last_active_tool: DEFAULT_TOOL,
        }
    }
}

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyBindings::default())
            .insert_resource(UserInput::default())
            .add_system_to_stage(CoreStage::PreUpdate, register_commands);
    }
}

fn register_commands(
    mut user_input: ResMut<UserInput>,
    bindings: Res<KeyBindings>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    query_block_picker: Query<&BlockPicker>,
) {
    if !user_input.commands.is_empty() {
        user_input.commands = vec![];
    }

    let active_tool = user_input.active_tool;

    for (action, binding) in bindings.0.iter() {
        let picker = query_block_picker.single();
        let mode = if !picker.is_open {
            Mode::Normal
        } else {
            Mode::UI
        };
        if is_binding_active(binding, mode, &mouse, &keyboard) {
            match action {
                Action::Click => {
                    let command = match active_tool {
                        Tool::Place => UICommand::PlaceBlock,
                        Tool::Destroy => UICommand::DestroyBlock,
                    };
                    user_input.commands.push(command);
                }
                Action::ActivateTool(tool) => {
                    user_input.active_tool = tool.clone();
                    if tool != &active_tool {
                        user_input.last_active_tool = active_tool;
                    }
                }
                Action::ExitMode => {
                    if picker.is_open {
                        user_input.commands.push(UICommand::CloseBlockPicker);
                    }
                }
                Action::ToggleBlockPicker => {
                    let picker = query_block_picker.single();
                    if !picker.is_open {
                        user_input.commands.push(UICommand::OpenBlockPicker);
                    } else {
                        user_input.commands.push(UICommand::CloseBlockPicker);
                    }
                }
            }
        }
        if hold_type_binding_was_just_active(binding, &mouse, &keyboard) {
            match action {
                Action::ActivateTool(_) => {
                    user_input.active_tool = user_input.last_active_tool;
                }
                Action::ToggleBlockPicker => {
                    user_input.commands.push(UICommand::CloseBlockPicker);
                }
                _ => (),
            }
        }
    }
}

fn is_binding_active(
    binding: &Binding,
    mode: Mode,
    mouse: &Res<Input<MouseButton>>,
    keyboard: &Res<Input<KeyCode>>,
) -> bool {
    if !binding.modes.iter().any(|m| m == &mode) {
        return false;
    }
    match (binding.key, binding.binding_mode) {
        (Key::Keyboard(key_code), BindingMode::Tap) => keyboard.just_released(key_code),
        (Key::Keyboard(key_code), BindingMode::Hold) => keyboard.just_pressed(key_code),
        (Key::Mouse(button), BindingMode::Tap) => mouse.just_released(button),
        (Key::Mouse(button), BindingMode::Hold) => mouse.just_pressed(button),
    }
}

fn hold_type_binding_was_just_active(
    binding: &Binding,
    mouse: &Res<Input<MouseButton>>,
    keyboard: &Res<Input<KeyCode>>,
) -> bool {
    match (binding.key, binding.binding_mode) {
        (Key::Keyboard(key_code), BindingMode::Hold) => keyboard.just_released(key_code),
        (Key::Mouse(button), BindingMode::Hold) => mouse.just_released(button),
        (_, BindingMode::Tap) => false,
    }
}

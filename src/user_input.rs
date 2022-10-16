use bevy::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UICommand {
    PlaceBlock,
    DestroyBlock,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    ApplyTool,
    ActivateTool(Tool),
}

#[derive(Clone, Debug)]
pub struct KeyBindings(Vec<(Action, Binding)>);

impl Default for KeyBindings {
    fn default() -> Self {
        KeyBindings(vec![
            (
                Action::ApplyTool,
                Binding {
                    key: Key::Mouse(MouseButton::Left),
                    mode: BindingMode::Tap,
                },
            ),
            (
                Action::ActivateTool(Tool::Destroy),
                Binding {
                    key: Key::Keyboard(KeyCode::X),
                    mode: BindingMode::Hold,
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
    mode: BindingMode,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BindingMode {
    Tap,
    Hold,
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
) {
    if !user_input.commands.is_empty() {
        user_input.commands = vec![];
    }

    let active_tool = user_input.active_tool;

    for (action, binding) in bindings.0.iter() {
        if is_binding_active(binding, &mouse, &keyboard) {
            match action {
                Action::ApplyTool => {
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
            }
        }
        if binding_was_just_active(binding, &mouse, &keyboard) {
            match action {
                Action::ActivateTool(_) => {
                    user_input.active_tool = user_input.last_active_tool;
                }
                _ => (),
            }
        }
    }
}

fn is_binding_active(
    binding: &Binding,
    mouse: &Res<Input<MouseButton>>,
    keyboard: &Res<Input<KeyCode>>,
) -> bool {
    match (binding.key, binding.mode) {
        (Key::Keyboard(key_code), BindingMode::Tap) => keyboard.just_released(key_code),
        (Key::Keyboard(key_code), BindingMode::Hold) => keyboard.pressed(key_code),
        (Key::Mouse(button), BindingMode::Tap) => mouse.just_released(button),
        (Key::Mouse(button), BindingMode::Hold) => mouse.pressed(button),
    }
}

fn binding_was_just_active(
    binding: &Binding,
    mouse: &Res<Input<MouseButton>>,
    keyboard: &Res<Input<KeyCode>>,
) -> bool {
    match (binding.key, binding.mode) {
        (Key::Keyboard(key_code), BindingMode::Hold) => keyboard.just_released(key_code),
        (Key::Mouse(button), BindingMode::Hold) => mouse.just_released(button),
        (_, BindingMode::Tap) => false,
    }
}

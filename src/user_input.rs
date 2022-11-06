use bevy::{prelude::*, utils::HashSet};

use crate::util::RelativeDirection;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum UiCommand {
    PlaceBlock,
    DestroyBlock,
    OpenBlockPicker,
    CloseBlockPicker,
    RotateBlock(Option<RelativeDirection>),
    ToggleBlockPicker,
}

pub fn sent_command(mut ev_ui_command: EventReader<UiCommand>, command: UiCommand) -> bool {
    ev_ui_command.iter().any(|c| c == &command)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Action {
    ActivateTool(Tool),
    ExitMode,
    RotateBlock(RelativeDirection),
    ToggleBlockPicker,
    UseActiveTool,
}

#[derive(Clone, Debug)]
pub struct KeyBindings(Vec<Binding>);

impl Default for KeyBindings {
    fn default() -> Self {
        KeyBindings(vec![
            // Normal mode
            Binding {
                action: Action::UseActiveTool,
                key: Key::Mouse(MouseButton::Left),
                binding_style: BindingStyle::Hold,
                modes: vec![Mode::Normal],
            },
            Binding {
                action: Action::ActivateTool(Tool::Destroy),
                key: Key::Keyboard(KeyCode::X),
                binding_style: BindingStyle::Hold,
                modes: vec![Mode::Normal],
            },
            Binding {
                action: Action::ExitMode,
                key: Key::Keyboard(KeyCode::Escape),
                binding_style: BindingStyle::Tap,
                modes: vec![Mode::Normal],
            },
            Binding {
                action: Action::ToggleBlockPicker,
                key: Key::Keyboard(KeyCode::P),
                binding_style: BindingStyle::Tap,
                modes: vec![Mode::Normal],
            },
            // PlacingBlock mode
            Binding {
                action: Action::RotateBlock(RelativeDirection::Left),
                key: Key::Keyboard(KeyCode::Left),
                binding_style: BindingStyle::Tap,
                modes: vec![Mode::PlacingBlock],
            },
            Binding {
                action: Action::RotateBlock(RelativeDirection::Right),
                key: Key::Keyboard(KeyCode::Right),
                binding_style: BindingStyle::Tap,
                modes: vec![Mode::PlacingBlock],
            },
            Binding {
                action: Action::RotateBlock(RelativeDirection::Up),
                key: Key::Keyboard(KeyCode::Up),
                binding_style: BindingStyle::Tap,
                modes: vec![Mode::PlacingBlock],
            },
            Binding {
                action: Action::RotateBlock(RelativeDirection::Down),
                key: Key::Keyboard(KeyCode::Down),
                binding_style: BindingStyle::Tap,
                modes: vec![Mode::PlacingBlock],
            },
            Binding {
                action: Action::RotateBlock(RelativeDirection::Forward),
                key: Key::Keyboard(KeyCode::Home),
                binding_style: BindingStyle::Tap,
                modes: vec![Mode::PlacingBlock],
            },
            Binding {
                action: Action::RotateBlock(RelativeDirection::Back),
                key: Key::Keyboard(KeyCode::End),
                binding_style: BindingStyle::Tap,
                modes: vec![Mode::PlacingBlock],
            },
        ])
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum Tool {
    #[default]
    Place,
    Destroy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    action: Action,
    key: Key,
    binding_style: BindingStyle,
    modes: Vec<Mode>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BindingStyle {
    /// In this mode tapping a key or mouse button activates the binding.
    Tap,

    /// In this mode the binding is active when holding a key or mouse button. For toggleable
    /// states, like switching to the destroy tool, or opening the block picker, the state is
    /// active while the bound key is held, and inactivated when the key is released.
    Hold,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Mode {
    /// Normal world interaction; e.g. clicking to place blocks against other blocks
    #[default]
    Normal,

    /// Active mode while holding down left-click (or after tapping if you have ActivateTool set to
    /// `Tap`).
    PlacingBlock,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

#[derive(Debug, Default)]
struct SelectedTool {
    active_tool: Tool,
    last_active_tool: Tool,
}

impl SelectedTool {
    fn push_tool(&mut self, tool: Tool) {
        if tool != self.active_tool {
            self.last_active_tool = self.active_tool;
            self.active_tool = tool;
        }
    }

    fn pop_tool(&mut self) {
        self.active_tool = self.last_active_tool;
    }
}

#[derive(Debug, Default)]
pub struct InputState {
    mode: Mode,
    last_mode: Mode,
    bindings_pressed: HashSet<Action>,
}

impl InputState {
    fn push_mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.last_mode = self.mode;
            self.mode = mode;
        }
    }

    fn pop_mode(&mut self) {
        self.mode = self.last_mode;
    }
}

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyBindings::default())
            .insert_resource(InputState::default())
            .insert_resource(SelectedTool::default())
            .add_event::<UiCommand>()
            .add_system_to_stage(CoreStage::PreUpdate, register_binding_presses);
    }
}

fn register_binding_presses(
    mut ev_ui_command: EventWriter<UiCommand>,
    mut state: ResMut<InputState>,
    mut selected_tool: ResMut<SelectedTool>,
    bindings: Res<KeyBindings>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
) {
    let mode = state.mode;
    for binding in bindings.0.iter() {
        if is_binding_invoked(binding, mode, &state, &mouse, &keyboard) {
            match binding.binding_style {
                BindingStyle::Tap => dispatch_action(
                    &mut ev_ui_command,
                    &mut state,
                    &mut selected_tool,
                    binding.action,
                ),
                BindingStyle::Hold => dispatch_action_start(
                    &mut ev_ui_command,
                    &mut state,
                    &mut selected_tool,
                    binding.action,
                ),
            }
        }
        if is_binding_released(binding, &state, &mouse, &keyboard) {
            dispatch_action_finish(
                &mut ev_ui_command,
                &mut state,
                &mut selected_tool,
                binding.action,
            )
        }
    }
    update_bindings_pressed(&bindings, &mut state, mode, &mouse, &keyboard);
}

// Called when a tap-style binding is pressed and released
fn dispatch_action(
    ev_ui_command: &mut EventWriter<UiCommand>,
    _state: &mut ResMut<InputState>,
    selected_tool: &mut ResMut<SelectedTool>,
    action: Action,
) {
    match action {
        Action::UseActiveTool => {
            let command = match selected_tool.active_tool {
                Tool::Place => UiCommand::PlaceBlock,
                Tool::Destroy => UiCommand::DestroyBlock,
            };
            ev_ui_command.send(command);
        }
        Action::ActivateTool(tool) => {
            selected_tool.push_tool(tool);
        }
        Action::ExitMode => {
            ev_ui_command.send(UiCommand::CloseBlockPicker);
        }
        Action::RotateBlock(dir) => {
            ev_ui_command.send(UiCommand::RotateBlock(Some(dir)));
        }
        Action::ToggleBlockPicker => {
            ev_ui_command.send(UiCommand::ToggleBlockPicker);
        }
    }
}

// Called when a hold-style binding is pressed
fn dispatch_action_start(
    ev_ui_command: &mut EventWriter<UiCommand>,
    state: &mut ResMut<InputState>,
    selected_tool: &mut ResMut<SelectedTool>,
    action: Action,
) {
    match action {
        Action::UseActiveTool => {
            match selected_tool.active_tool {
                Tool::Place => state.push_mode(Mode::PlacingBlock),
                Tool::Destroy => (),
            };
        }
        Action::ActivateTool(tool) => {
            selected_tool.push_tool(tool);
        }
        Action::ExitMode => (),
        Action::RotateBlock(dir) => ev_ui_command.send(UiCommand::RotateBlock(Some(dir))),
        Action::ToggleBlockPicker => {
            ev_ui_command.send(UiCommand::OpenBlockPicker);
        }
    }
}

/// Called when a hold-style binding is released
fn dispatch_action_finish(
    ev_ui_command: &mut EventWriter<UiCommand>,
    state: &mut ResMut<InputState>,
    selected_tool: &mut ResMut<SelectedTool>,
    action: Action,
) {
    match action {
        Action::UseActiveTool => {
            if state.mode == Mode::PlacingBlock {
                state.pop_mode();
            }
            dispatch_action(ev_ui_command, state, selected_tool, action);
        }
        Action::ActivateTool(_) => selected_tool.pop_tool(),
        Action::ExitMode => dispatch_action(ev_ui_command, state, selected_tool, action),
        Action::RotateBlock(_) => ev_ui_command.send(UiCommand::RotateBlock(None)),
        Action::ToggleBlockPicker => {
            ev_ui_command.send(UiCommand::CloseBlockPicker);
        }
    }
}

fn update_bindings_pressed(
    bindings: &Res<KeyBindings>,
    state: &mut ResMut<InputState>,
    mode: Mode,
    mouse: &Res<Input<MouseButton>>,
    keyboard: &Res<Input<KeyCode>>,
) {
    let mut bindings_pressed = state.bindings_pressed.clone();
    for binding in bindings.0.iter() {
        if is_binding_pressed(binding, mode, mouse, keyboard) {
            bindings_pressed.insert(binding.action);
        }
        if just_released(binding.key, mouse, keyboard) {
            bindings_pressed.remove(&binding.action);
        }
    }
    if bindings_pressed != state.bindings_pressed {
        state.bindings_pressed = bindings_pressed;
    }
}

/// The key for a binding has been pressed - the binding has not necessarily been invoked yet.
fn is_binding_pressed(
    binding: &Binding,
    mode: Mode,
    mouse: &Res<Input<MouseButton>>,
    keyboard: &Res<Input<KeyCode>>,
) -> bool {
    matches_mode(mode, binding) && just_pressed(binding.key, mouse, keyboard)
}

/// Returns true if the key for the given binding was pressed and released (for a Tap style
/// binding), or is being held (for a Hold style binding).
fn is_binding_invoked(
    binding: &Binding,
    mode: Mode,
    previously_pressed_bindings: &InputState,
    mouse: &Res<Input<MouseButton>>,
    keyboard: &Res<Input<KeyCode>>,
) -> bool {
    // For tap-style the mode should match on both press and release. For hold-style it only needs
    // to match on press.
    if !matches_mode(mode, binding) {
        return false;
    }
    match binding.binding_style {
        BindingStyle::Tap => {
            just_released(binding.key, mouse, keyboard)
                && previously_pressed_bindings
                    .bindings_pressed
                    .iter()
                    .any(|action| action == &binding.action)
        }
        BindingStyle::Hold => just_pressed(binding.key, mouse, keyboard),
    }
}

/// Hold-style bindings activate a mode while the given key is held. This function detects when the
/// binding is released meaning that mode should be exited.
fn is_binding_released(
    binding: &Binding,
    previously_pressed_bindings: &InputState,
    mouse: &Res<Input<MouseButton>>,
    keyboard: &Res<Input<KeyCode>>,
) -> bool {
    // No mode check!
    match binding.binding_style {
        BindingStyle::Tap => false,
        BindingStyle::Hold => {
            just_released(binding.key, mouse, keyboard)
                && previously_pressed_bindings
                    .bindings_pressed
                    .iter()
                    .any(|action| action == &binding.action)
        }
    }
}

fn just_pressed(key: Key, mouse: &Res<Input<MouseButton>>, keyboard: &Res<Input<KeyCode>>) -> bool {
    match key {
        Key::Keyboard(key_code) => keyboard.just_pressed(key_code),
        Key::Mouse(button) => mouse.just_pressed(button),
    }
}

fn just_released(
    key: Key,
    mouse: &Res<Input<MouseButton>>,
    keyboard: &Res<Input<KeyCode>>,
) -> bool {
    match key {
        Key::Keyboard(key_code) => keyboard.just_released(key_code),
        Key::Mouse(button) => mouse.just_released(button),
    }
}

fn matches_mode(mode: Mode, binding: &Binding) -> bool {
    binding.modes.iter().any(|m| m == &mode)
}

#[cfg(test)]
mod tests {
    use std::{
        hash::Hash,
        marker::{Copy, Send, Sync},
    };

    use bevy::prelude::*;

    use super::*;

    #[test]
    fn opens_the_block_picker() {
        let mut app = initialize_test_app();
        send_key_press(&mut app, KeyCode::P);
        assert_eq!(
            ui_command_events(&app),
            vec![UiCommand::ToggleBlockPicker],
            "open block picker command was sent"
        )
    }

    #[test]
    fn places_a_block() {
        let mut app = initialize_test_app();
        send_key_down(&mut app, MouseButton::Left);
        assert_eq!(
            app.world.resource::<InputState>().mode,
            Mode::PlacingBlock,
            "mode has changed to PlacingBlock"
        );

        send_key_up(&mut app, MouseButton::Left);
        assert_eq!(
            app.world.resource::<InputState>().mode,
            Mode::Normal,
            "mode has changed to Normal"
        );
        assert_eq!(
            ui_command_events(&app),
            vec![UiCommand::PlaceBlock],
            "place block command was sent"
        )
    }

    #[test]
    fn destroys_a_block() {
        let mut app = initialize_test_app();
        send_key_down(&mut app, KeyCode::X);
        send_key_press(&mut app, MouseButton::Left);
        assert_eq!(
            ui_command_events(&app),
            vec![UiCommand::DestroyBlock],
            "destroy block command was sent"
        )
    }

    fn initialize_test_app() -> App {
        let mut app = App::new();
        app.insert_resource(KeyBindings::default())
            .insert_resource(InputState::default())
            .insert_resource(SelectedTool::default())
            .insert_resource(Input::<MouseButton>::default())
            .insert_resource(Input::<KeyCode>::default())
            .add_event::<UiCommand>()
            .add_system_to_stage(CoreStage::PreUpdate, register_binding_presses);
        app
    }

    fn send_key_press<T>(app: &mut App, key: T)
    where
        T: Copy + Eq + Hash + Send + Sync + 'static,
    {
        send_key_down(app, key);
        send_key_up(app, key);
    }

    fn send_key_down<T>(app: &mut App, key: T)
    where
        T: Copy + Eq + Hash + Send + Sync + 'static,
    {
        app.world.resource_mut::<Input<T>>().press(key);
        app.update(); // run systems
        app.world.resource_mut::<Input<T>>().clear(); // clear `just_pressed` status
    }

    fn send_key_up<T>(app: &mut App, key: T)
    where
        T: Copy + Eq + Hash + Send + Sync + 'static,
    {
        app.world.resource_mut::<Input<T>>().release(key);
        app.update(); // run systems
        app.world.resource_mut::<Input<T>>().clear(); // clear `just_released` status
    }

    fn ui_command_events(app: &App) -> Vec<UiCommand> {
        let events = app.world.resource::<Events<UiCommand>>();
        let mut reader = events.get_reader();
        reader.iter(events).cloned().collect()
    }
}

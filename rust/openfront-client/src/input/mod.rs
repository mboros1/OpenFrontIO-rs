use bevy::prelude::*;

pub mod handler;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<handler::GameInputEvent>()
            .add_systems(Update, (
                handler::handle_keyboard_input,
                handler::handle_mouse_input,
                handler::handle_touch_input,
                handler::process_input_events,
            ));
    }
}
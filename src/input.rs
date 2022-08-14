use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<AirplaneAction>::default())
            .add_startup_system(setup_input);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum AirplaneAction {
    Throttle,
    Roll,
    Pitch,
    Yaw,
}

#[derive(Component)]
pub struct AirplaneControls;

fn setup_input(mut commands: Commands) {
    commands
        .spawn_bundle(InputManagerBundle::<AirplaneAction> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (
                    SingleAxis::symmetric(GamepadAxisType::RightStickY, 0.1),
                    AirplaneAction::Pitch,
                ),
                (
                    SingleAxis::symmetric(GamepadAxisType::RightStickX, 0.1),
                    AirplaneAction::Roll,
                ),
                (
                    SingleAxis::symmetric(GamepadAxisType::LeftStickX, 0.1),
                    AirplaneAction::Yaw,
                ),
            ]),
        })
        .insert(AirplaneControls);
}

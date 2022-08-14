use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_dolly::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContext;

use crate::Airplane;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_dolly_component(MainCamera)
            .add_startup_system(setup_camera)
            .add_system(update_camera);
    }
}

#[derive(Component)]
struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 1., 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(MainCamera);

    commands
        .spawn()
        .insert(
            Rig::builder()
                .with(Position::new(Vec3::ZERO))
                .with(Rotation::new(Quat::IDENTITY))
                .with(YawPitch::new().yaw_degrees(45.0).pitch_degrees(-30.0))
                .with(Smooth::new_rotation(1.5))
                .with(Arm::new(Vec3::Z * 25.0))
                .build(),
        )
        .insert(MainCamera);
}

fn update_camera(
    airplane_query: Query<&Transform, With<Airplane>>,
    mut rig_query: Query<&mut Rig>,
    mut egui_context: ResMut<EguiContext>,
    buttons: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    let plane_transform = airplane_query.single().to_owned();

    let mut rig = rig_query.single_mut();

    rig.driver_mut::<Position>().position = plane_transform.translation;
    rig.driver_mut::<Rotation>().rotation = plane_transform.rotation;

    if !egui_context.ctx_mut().wants_pointer_input() && buttons.pressed(MouseButton::Left) {
        for ev in motion_evr.iter() {
            rig.driver_mut::<YawPitch>()
                .rotate_yaw_pitch(-ev.delta.x, -ev.delta.y);
        }
    }
}

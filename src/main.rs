use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;

use std::f32::consts::PI;

mod aerodynamics;
mod camera;
mod input;

use aerodynamics::{
    AeroSurface, AeroSurfaceConfig, AeroSurfaceList, AerodynamicsPlugin, ControlInputType,
};
use camera::CameraPlugin;
use input::InputPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .insert_resource(ClearColor(Color::rgb(0.52, 0.81, 0.92)))
        .insert_resource(RapierConfiguration {
            gravity: Vec3::ZERO,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(AerodynamicsPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(InputPlugin)
        .add_startup_system(setup_terrain)
        .add_startup_system(setup_airplane)
        .add_system(shadowmap_follow_airplane)
        .run();
}

#[derive(Component)]
struct Sun;

fn setup_terrain(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground_size = 25_000.0;

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: ground_size })),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("terrain/mannheim.png")),
                ..default()
            }),
            ..default()
        })
        .insert(Collider::compound(vec![(
            Vec3::new(0.0, -1.0, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(ground_size / 2.0, 1.0, ground_size / 2.0),
        )]))
        .insert(Restitution::coefficient(0.1))
        .insert(ColliderDebugColor(Color::GREEN));

    commands
        .spawn_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 32_000.0,
                shadows_enabled: true,
                shadow_projection: OrthographicProjection {
                    left: -10.0,
                    right: 10.0,
                    bottom: -10.0,
                    top: 10.0,
                    near: -10.0,
                    far: 10.0,
                    ..default()
                },
                ..default()
            },
            transform: Transform::identity().looking_at(Vec3::new(1.0, -1.0, 0.5), Vec3::Y),
            ..default()
        })
        .insert(Sun);
}

#[derive(Component)]
struct Airplane;

fn setup_airplane(mut commands: Commands, asset_server: Res<AssetServer>) {
    let fuselage_collider = load_collider("assets/planes/ask21/ASK21_Fuselage_Collider.csv");
    let left_wing_collider = load_collider("assets/planes/ask21/ASK21_Left_Wing_Collider.csv");
    let right_wing_collider = load_collider("assets/planes/ask21/ASK21_Right_Wing_Collider.csv");
    let horizontal_stabilizer_collider =
        load_collider("assets/planes/ask21/ASK21_Horizontal_Stabilizer_Collider.csv");
    let vertical_stabilizer_collider =
        load_collider("assets/planes/ask21/ASK21_Vertical_Stabilizer_Collider.csv");

    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(0.0, 200.0, 0.0)))
        .insert_bundle(VisibilityBundle::default())
        .insert(Name::new("Airplane"))
        .insert(Airplane)
        .insert(RigidBody::Dynamic)
        .insert(Ccd::enabled())
        .insert(ExternalForce::default())
        .insert(Velocity::linear(Vec3::new(0.0, 0.0, -27.7)))
        // .insert(Velocity::linear(Vec3::new(0.0, 0.0, 0.0)))
        .insert(ReadMassProperties::default())
        .insert(Collider::compound(vec![
            (
                Vec3::ZERO,
                Quat::IDENTITY,
                Collider::convex_hull(&fuselage_collider).unwrap(),
            ),
            (
                Vec3::ZERO,
                Quat::IDENTITY,
                Collider::convex_hull(&left_wing_collider).unwrap(),
            ),
            (
                Vec3::ZERO,
                Quat::IDENTITY,
                Collider::convex_hull(&right_wing_collider).unwrap(),
            ),
            (
                Vec3::ZERO,
                Quat::IDENTITY,
                Collider::convex_hull(&horizontal_stabilizer_collider).unwrap(),
            ),
            (
                Vec3::ZERO,
                Quat::IDENTITY,
                Collider::convex_hull(&vertical_stabilizer_collider).unwrap(),
            ),
        ]))
        .insert(ColliderMassProperties::MassProperties(MassProperties {
            local_center_of_mass: Vec3::new(-0.08496038, 0.86599594, -0.15),
            mass: 530.0,
            principal_inertia_local_frame: Quat::from_xyzw(
                0.44778442,
                -0.008306614,
                0.0017128434,
                0.8941014,
            ),
            principal_inertia: Vec3::new(6293.8193, 5342.917, 5116.539),
        }))
        .insert(AeroSurfaceList {
            surfaces: vec![
                /*(
                    // left wing
                    AeroSurface {
                        config: AeroSurfaceConfig {
                            span: 8.0,
                            chord: 1.2,
                            control_surface_fraction: 0.2,
                            ..default()
                        },
                        input_type: ControlInputType::Roll,
                        input_sensitivity: 0.5,
                        control_surface_angle: 0.0,
                    },
                    Transform::from_xyz(-4.5, 1.0, 0.2).with_rotation(Quat::from_rotation_z(-0.07)),
                ),
                (
                    // right wing
                    AeroSurface {
                        config: AeroSurfaceConfig {
                            span: 8.0,
                            chord: 1.2,
                            control_surface_fraction: 0.2,
                            ..default()
                        },
                        input_type: ControlInputType::Roll,
                        input_sensitivity: -0.5,
                        control_surface_angle: 0.0,
                    },
                    Transform::from_xyz(4.5, 1.0, 0.2).with_rotation(Quat::from_rotation_z(0.07)),
                ),
                (
                    // fuselage
                    AeroSurface {
                        config: AeroSurfaceConfig {
                            span: 0.8,
                            chord: 8.5,
                            ..default()
                        },
                        input_type: ControlInputType::None,
                        input_sensitivity: 0.0,
                        control_surface_angle: 0.0,
                    },
                    Transform::from_xyz(0.0, 0.6, 1.2)
                        .with_rotation(Quat::from_rotation_z(PI * 0.5)),
                ),
                (
                    // vertical stabilizer
                    AeroSurface {
                        config: AeroSurfaceConfig {
                            span: 1.5,
                            chord: 1.0,
                            control_surface_fraction: 0.3,
                            ..default()
                        },
                        input_type: ControlInputType::Yaw,
                        input_sensitivity: 0.5,
                        control_surface_angle: 0.0,
                    },
                    Transform::from_xyz(0.0, 1.3, 4.9)
                        .with_rotation(Quat::from_rotation_z(PI * 0.5)),
                ),*/
                (
                    // horizontal stabilizer
                    AeroSurface {
                        config: AeroSurfaceConfig {
                            span: 3.0,
                            chord: 0.8,
                            control_surface_fraction: 0.3,
                            ..default()
                        },
                        input_type: ControlInputType::Pitch,
                        input_sensitivity: 0.5,
                        control_surface_angle: 0.0,
                    },
                    Transform::from_xyz(0.0, 2.0, 4.9),
                ),
            ],
        })
        .insert(Restitution::coefficient(0.1))
        .with_children(|child_builder| {
            child_builder
                .spawn_bundle(SceneBundle {
                    scene: asset_server.load("planes/ask21/ask21.glb#Scene0"),
                    ..default()
                })
                .insert(Transform::from_rotation(Quat::from_rotation_y(PI)));
        });
}

fn shadowmap_follow_airplane(
    mut sun_query: Query<&mut Transform, With<Sun>>,
    airplane_query: Query<&Transform, (With<Airplane>, Without<Sun>)>,
) {
    sun_query.single_mut().translation = airplane_query.single().translation;
}

fn load_collider(path: &str) -> Vec<Vec3> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| {
            let mut components = line.split(';').map(|s| s.parse::<f32>().unwrap());
            Vec3::new(
                -components.next().unwrap(),
                components.next().unwrap(),
                -components.next().unwrap(),
            )
        })
        .collect::<Vec<_>>()
}

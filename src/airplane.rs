use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use enum_map::EnumMap;

use crate::aerodynamics::AeroSurfaceList;

pub struct AirplanePlugin;

impl Plugin for AirplanePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_flight_metrics);
    }
}

#[derive(Component)]
pub struct Airplane;

#[derive(Reflect, Clone, Copy, PartialEq, Eq, Hash, Default, Enum)]
pub enum FlightMetric {
    #[default]
    Airspeed,
    VerticalSpeed,
    Height,
}

#[derive(Component, Clone)]
pub struct FlightMetrics {
    pub metrics: EnumMap<FlightMetric, f32>,
}

impl Default for FlightMetrics {
    fn default() -> Self {
        Self {
            metrics: EnumMap::default(),
        }
    }
}

#[derive(Bundle)]
pub struct AirplaneBundle {
    pub airplane: Airplane,
    pub rigidbody: RigidBody,
    pub ccd: Ccd,
    pub external_force: ExternalForce,
    pub velocity: Velocity,
    pub collider: Collider,
    pub aero_surface_list: AeroSurfaceList,
    pub read_mass_properties: ReadMassProperties,
    pub metrics: FlightMetrics,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl Default for AirplaneBundle {
    fn default() -> Self {
        AirplaneBundle {
            airplane: Airplane,
            rigidbody: RigidBody::Dynamic,
            ccd: Ccd::enabled(),
            external_force: ExternalForce::default(),
            velocity: Velocity::default(),
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            aero_surface_list: AeroSurfaceList {
                surfaces: Vec::new(),
            },
            read_mass_properties: ReadMassProperties::default(),
            metrics: FlightMetrics::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        }
    }
}

fn update_flight_metrics(
    mut airplane_query: Query<(&mut FlightMetrics, &Transform, &Velocity), With<Airplane>>,
) {
    for (mut metrics, transform, velocity) in &mut airplane_query {
        metrics.metrics[FlightMetric::Airspeed] =
            velocity.linvel.length() * velocity.linvel.normalize().dot(transform.forward());
        metrics.metrics[FlightMetric::VerticalSpeed] = velocity.linvel.y;
        metrics.metrics[FlightMetric::Height] = transform.translation.y;
    }
}

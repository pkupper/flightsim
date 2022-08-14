use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use std::f32::consts::PI;

use crate::input::{AirplaneAction, AirplaneControls};

pub struct AerodynamicsPlugin;

impl Plugin for AerodynamicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<AeroSurfaceList>()
            .add_system(update_control_surface_angle)
            .add_system(simulate_aerodynamics.after(update_control_surface_angle))
            .add_system_to_stage(
                CoreStage::PostUpdate,
                draw_aero_surfaces
                    .before("draw_lines")
                    .after(bevy::transform::transform_propagate_system),
            );
    }
}

fn update_control_surface_angle(
    input_query: Query<&ActionState<AirplaneAction>, With<AirplaneControls>>,
    mut surface_list_query: Query<&mut AeroSurfaceList>,
) {
    let action_state = input_query.single();

    for mut surface_list in &mut surface_list_query {
        for (surface, _) in &mut surface_list.surfaces {
            let input_value = match surface.input_type {
                ControlInputType::None => 0.0,
                ControlInputType::Roll => action_state.clamped_value(AirplaneAction::Roll),
                ControlInputType::Pitch => action_state.clamped_value(AirplaneAction::Pitch),
                ControlInputType::Yaw => action_state.clamped_value(AirplaneAction::Yaw),
                ControlInputType::Flap => 0.0,
            };
            surface.control_surface_angle = surface.input_sensitivity * input_value;
        }
    }
}

fn draw_aero_surfaces(
    mut lines: ResMut<DebugLines>,
    surface_list_query: Query<(&GlobalTransform, &AeroSurfaceList)>,
) {
    for (global_transform, surface_list) in &surface_list_query {
        lines.line_colored(
            global_transform.translation(),
            global_transform.translation() + global_transform.forward(),
            0.0,
            Color::ORANGE,
        );
        for (surface, transform) in &surface_list.surfaces {
            let surface_transform = global_transform.mul_transform(*transform);
            let back = surface_transform.back();
            let right = surface_transform.right();
            let up = surface_transform.up();
            let half_chord = surface.config.chord * 0.5;
            let half_span = surface.config.span * 0.5;
            let leading_edge = surface_transform.translation() - back * half_chord;
            let hinge = leading_edge
                + back * surface.config.chord * (1.0 - surface.config.control_surface_fraction);
            let trailing_edge = hinge
                + (up * -surface.control_surface_angle.sin()
                    + back * surface.control_surface_angle.cos())
                    * surface.config.chord
                    * surface.config.control_surface_fraction;

            let p0 = leading_edge - right * half_span;
            let p1 = leading_edge + right * half_span;
            let p2 = hinge - right * half_span;
            let p3 = hinge + right * half_span;
            let p4 = trailing_edge - right * half_span;
            let p5 = trailing_edge + right * half_span;
            lines.line_colored(p0, p1, 0.0, Color::BLUE);
            lines.line_colored(p2, p3, 0.0, Color::BLUE);
            lines.line_colored(p4, p5, 0.0, Color::RED);
            lines.line_colored(p0, p2, 0.0, Color::BLUE);
            lines.line_colored(p1, p3, 0.0, Color::BLUE);
            lines.line_colored(p2, p4, 0.0, Color::RED);
            lines.line_colored(p3, p5, 0.0, Color::RED);
        }
    }
}

#[derive(Inspectable, Default, Clone, Copy)]
pub enum ControlInputType {
    #[default]
    None,
    Pitch,
    Yaw,
    Roll,
    Flap,
}

#[derive(Inspectable, Clone, Copy)]
pub struct AeroSurfaceConfig {
    pub lift_slope: f32,
    pub skin_friction: f32,
    pub zero_lift_aoa: f32,
    pub stall_angle_high: f32,
    pub stall_angle_low: f32,
    pub chord: f32,
    pub span: f32,
    pub control_surface_fraction: f32,
}

impl Default for AeroSurfaceConfig {
    fn default() -> Self {
        Self {
            lift_slope: 6.28,
            skin_friction: 0.02,
            zero_lift_aoa: 0.0,     // radians
            stall_angle_high: 0.26, // radians (= 15 degrees)
            stall_angle_low: -0.26, // radians (= -15 degrees)
            chord: 1.0,
            span: 2.0,
            control_surface_fraction: 0.0,
        }
    }
}

#[derive(Inspectable, Default, Clone, Copy)]
pub struct AeroSurface {
    pub config: AeroSurfaceConfig,
    pub input_type: ControlInputType,
    pub input_sensitivity: f32,
    pub control_surface_angle: f32,
}

impl AeroSurface {
    pub fn calculate_forces(&self, mut local_air_velocity: Vec3, air_density: f32) -> Vec3 {
        let aspect_ratio = self.config.span / self.config.chord;

        // Accounting for aspect ratio effect on lift coefficient.
        let corrected_lift_slope = self.config.lift_slope * aspect_ratio
            / (aspect_ratio + 2.0 * (aspect_ratio + 4.0) / (aspect_ratio + 2.0));

        // Calculating flap deflection influence on zero lift angle of attack
        // and angles at which stall happens.
        let theta = (2.0 * self.config.control_surface_fraction - 1.0).acos();
        let control_surface_effectiveness = 1.0 - (theta - theta.sin()) / PI;
        let delta_lift = corrected_lift_slope
            * control_surface_effectiveness
            * self.control_surface_effectiveness_correction()
            * self.control_surface_angle;

        let zero_lift_aoa_base = self.config.zero_lift_aoa;
        let zero_lift_aoa = zero_lift_aoa_base - delta_lift / corrected_lift_slope;

        let stall_angle_high_base = self.config.stall_angle_high;
        let stall_angle_low_base = self.config.stall_angle_low;

        let cl_max_high = corrected_lift_slope * (stall_angle_high_base - zero_lift_aoa)
            + delta_lift * self.lift_coefficient_max_fraction();
        let cl_max_low = corrected_lift_slope * (stall_angle_low_base - zero_lift_aoa)
            + delta_lift * self.lift_coefficient_max_fraction();

        let stall_angle_high = zero_lift_aoa + cl_max_high / corrected_lift_slope;
        let stall_angle_low = zero_lift_aoa + cl_max_low / corrected_lift_slope;

        local_air_velocity.x = 0.0;

        let area = self.config.chord * self.config.span;
        let dynamic_pressure = 0.5 * air_density * local_air_velocity.length_squared();
        let angle_of_attack = (-local_air_velocity.y).atan2(local_air_velocity.z);

        let aerodynamic_coefficients = self.calculate_coefficients(
            aspect_ratio,
            angle_of_attack,
            corrected_lift_slope,
            zero_lift_aoa,
            stall_angle_high,
            stall_angle_low,
        );

        let lift = aerodynamic_coefficients.x * dynamic_pressure * area;
        let drag = aerodynamic_coefficients.y * dynamic_pressure * area;
        let torque = aerodynamic_coefficients.z * dynamic_pressure * area * self.config.chord;

        Vec3::new(lift, drag, torque)
    }

    fn calculate_coefficients(
        &self,
        aspect_ratio: f32,
        angle_of_attack: f32,
        corrected_lift_slope: f32,
        zero_lift_aoa: f32,
        stall_angle_high: f32,
        stall_angle_low: f32,
    ) -> Vec3 {
        // Low angles of attack mode and stall mode curves are stitched together by a line segment.
        let padding_angle_high = lerp_clamped(
            15.0,
            5.0,
            (self.control_surface_angle.to_degrees() + 50.0) / 100.0,
        )
        .to_radians();
        let padding_angle_low = lerp_clamped(
            15.0,
            5.0,
            (-self.control_surface_angle.to_degrees() + 50.0) / 100.0,
        )
        .to_radians();
        let padded_stall_angle_high = stall_angle_high + padding_angle_high;
        let padded_stall_angle_low = stall_angle_low - padding_angle_low;

        if angle_of_attack < stall_angle_high && angle_of_attack > stall_angle_low {
            // Low angle of attack mode
            self.calculate_coefficients_low_aoa(
                aspect_ratio,
                angle_of_attack,
                corrected_lift_slope,
                zero_lift_aoa,
            )
        } else if angle_of_attack > padded_stall_angle_high
            || angle_of_attack < padded_stall_angle_low
        {
            self.calculate_coefficients_stall(
                aspect_ratio,
                angle_of_attack,
                corrected_lift_slope,
                zero_lift_aoa,
                stall_angle_high,
                stall_angle_low,
            )
        } else {
            let (coefficients_low, coefficients_stall, lerp_param) =
                if angle_of_attack > stall_angle_high {
                    (
                        self.calculate_coefficients_low_aoa(
                            aspect_ratio,
                            stall_angle_high,
                            corrected_lift_slope,
                            zero_lift_aoa,
                        ),
                        self.calculate_coefficients_stall(
                            aspect_ratio,
                            padded_stall_angle_high,
                            corrected_lift_slope,
                            zero_lift_aoa,
                            stall_angle_high,
                            stall_angle_low,
                        ),
                        (angle_of_attack - stall_angle_high)
                            / (padded_stall_angle_high - stall_angle_high),
                    )
                } else {
                    (
                        self.calculate_coefficients_low_aoa(
                            aspect_ratio,
                            stall_angle_low,
                            corrected_lift_slope,
                            zero_lift_aoa,
                        ),
                        self.calculate_coefficients_stall(
                            aspect_ratio,
                            padded_stall_angle_low,
                            corrected_lift_slope,
                            zero_lift_aoa,
                            stall_angle_high,
                            stall_angle_low,
                        ),
                        (angle_of_attack - stall_angle_low)
                            / (padded_stall_angle_low - stall_angle_low),
                    )
                };
            coefficients_low.lerp(coefficients_stall, lerp_param.clamp(0.0, 1.0))
        }
    }

    fn calculate_coefficients_low_aoa(
        &self,
        aspect_ratio: f32,
        angle_of_attack: f32,
        corrected_lift_slope: f32,
        zero_lift_aoa: f32,
    ) -> Vec3 {
        let lift_coefficent = corrected_lift_slope * (angle_of_attack - zero_lift_aoa);
        let induced_angle = lift_coefficent / (PI * aspect_ratio);
        let effective_angle = angle_of_attack - zero_lift_aoa - induced_angle;

        let tangential_coefficient = self.config.skin_friction * effective_angle.cos();

        let normal_coefficient = (lift_coefficent + effective_angle.sin() * tangential_coefficient)
            / effective_angle.cos();
        let drag_coefficient = normal_coefficient * effective_angle.sin()
            + tangential_coefficient * effective_angle.cos();
        let torque_coefficient =
            -normal_coefficient * Self::torque_coefficient_proportion(effective_angle);

        Vec3::new(lift_coefficent, drag_coefficient, torque_coefficient)
    }

    fn calculate_coefficients_stall(
        &self,
        aspect_ratio: f32,
        angle_of_attack: f32,
        corrected_lift_slope: f32,
        zero_lift_aoa: f32,
        stall_angle_high: f32,
        stall_angle_low: f32,
    ) -> Vec3 {
        let lift_coefficient_low_aoa = if angle_of_attack > stall_angle_high {
            corrected_lift_slope * (stall_angle_high - zero_lift_aoa)
        } else {
            corrected_lift_slope * (stall_angle_low - zero_lift_aoa)
        };
        let induced_angle_low_aoa = lift_coefficient_low_aoa / (PI * aspect_ratio);

        let lerp_param = if angle_of_attack > stall_angle_high {
            (PI / 2.0 - angle_of_attack.clamp(-PI / 2.0, PI / 2.0)) / (PI / 2.0 - stall_angle_high)
        } else {
            (-PI / 2.0 - angle_of_attack.clamp(-PI / 2.0, PI / 2.0)) / (-PI / 2.0 - stall_angle_low)
        };
        let induced_angle = lerp_clamped(0.0, induced_angle_low_aoa, lerp_param);
        let effective_angle = angle_of_attack - zero_lift_aoa - induced_angle;

        let normal_coefficient = self.friction_at_90_degrees()
            * effective_angle.sin()
            * (1.0 / (0.56 + 0.44 * effective_angle.sin().abs())
                - 0.41 * (1.0 - (-17.0 / aspect_ratio).exp()));
        let tangential_coefficient = 0.5 * self.config.skin_friction * effective_angle.cos();

        let lift_coefficent = normal_coefficient * effective_angle.cos()
            - tangential_coefficient * effective_angle.sin();
        let drag_coefficient = normal_coefficient * effective_angle.sin()
            + tangential_coefficient * effective_angle.cos();
        let torque_coefficient =
            -normal_coefficient * Self::torque_coefficient_proportion(effective_angle);

        Vec3::new(lift_coefficent, drag_coefficient, torque_coefficient)
    }

    fn friction_at_90_degrees(&self) -> f32 {
        1.98 - 4.26e-2 * self.control_surface_angle * self.control_surface_angle
            + 2.1e-1 * self.control_surface_angle
    }

    fn control_surface_effectiveness_correction(&self) -> f32 {
        lerp_clamped(
            0.8,
            0.4,
            (self.control_surface_angle.abs().to_degrees() - 10.0) / 50.0,
        )
    }

    fn lift_coefficient_max_fraction(&self) -> f32 {
        (1.0 - 0.5 * (self.config.control_surface_fraction - 0.1) / 0.3).clamp(0.0, 1.0)
    }

    fn torque_coefficient_proportion(effective_angle: f32) -> f32 {
        0.25 - 0.175 * (1.0 - 2.0 * effective_angle.abs() / PI)
    }
}

fn lerp_clamped(a: f32, b: f32, mut t: f32) -> f32 {
    t = t.clamp(0.0, 1.0);
    return a + t * (b - a);
}

#[derive(Inspectable, Default, Component)]
pub struct AeroSurfaceList {
    pub surfaces: Vec<(AeroSurface, Transform)>,
}

impl AeroSurfaceList {
    pub fn calculate_forces(
        &self,
        external_force: &mut ExternalForce,
        world_center_of_mass: Vec3,
        plane_transform: &Transform,
        velocity: &Velocity,
        lines: &mut ResMut<DebugLines>,
    ) {
        for (surface, surface_transform) in &self.surfaces {
            let surface_plane_transform = plane_transform.mul_transform(*surface_transform);

            let world_position = surface_plane_transform.translation;
            let relative_position = world_position - world_center_of_mass;

            let air_velocity = -velocity.linvel - velocity.angvel.cross(relative_position);

            info!(
                "base linvel {:?} rot linvel {:?}",
                velocity.linvel,
                velocity.angvel.cross(relative_position)
            );

            lines.line_colored(
                world_position,
                world_position + velocity.linvel + velocity.angvel.cross(relative_position),
                0.0,
                Color::BLACK,
            );

            let local_air_velocity = surface_plane_transform.rotation.mul_vec3(air_velocity);

            let (surface_lift, surface_drag, surface_torque) =
                surface.calculate_forces(local_air_velocity, 1.2).into();

            let mut drag_direction = air_velocity.normalize();
            if drag_direction.is_nan() {
                drag_direction = Vec3::ZERO;
            }
            let mut lift_direction = drag_direction.cross(surface_plane_transform.right());
            if lift_direction.is_nan() {
                lift_direction = Vec3::ZERO;
            }

            let lift = surface_lift * lift_direction;
            let drag = surface_drag * drag_direction;
            let torque = surface_torque * surface_plane_transform.back();

            lines.line_colored(
                world_position,
                world_position + lift * 0.01,
                0.0,
                Color::GREEN,
            );
            lines.line_colored(
                world_position,
                world_position + drag * 0.01,
                0.0,
                Color::PINK,
            );

            let total_force = lift + drag;

            external_force.force += total_force;
            external_force.torque += relative_position.cross(total_force);
            external_force.torque += torque;
        }

        external_force.force += plane_transform.forward() * 10000.0;
    }
}

fn simulate_aerodynamics(
    mut airplane_query: Query<(
        &AeroSurfaceList,
        &mut ExternalForce,
        &ReadMassProperties,
        &Transform,
        &Velocity,
    )>,
    mut lines: ResMut<DebugLines>,
) {
    for (surface_list, mut external_force, read_mass_properties, transform, velocity) in
        airplane_query.iter_mut()
    {
        let world_center_of_mass = transform.mul_vec3(read_mass_properties.0.local_center_of_mass);

        lines.line_colored(
            world_center_of_mass,
            world_center_of_mass + velocity.linvel,
            0.0,
            Color::YELLOW,
        );
        external_force.force = Vec3::ZERO;
        external_force.torque = Vec3::ZERO;

        surface_list.calculate_forces(
            &mut external_force,
            world_center_of_mass,
            &transform,
            &velocity,
            &mut lines,
        );
    }
}

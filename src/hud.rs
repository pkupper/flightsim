use bevy::prelude::*;

use crate::airplane::{Airplane, FlightMetric, FlightMetrics};

pub struct AirplaneHudPlugin;

#[derive(Component)]
struct MetricText(FlightMetric);

impl Plugin for AirplaneHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_hud).add_system(update_hud);
    }
}

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Auto, Val::Px(70.0)),
                padding: UiRect {
                    left: Val::Px(80.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                },
                ..default()
            },
            color: Color::rgba(0.0, 0.0, 0.0, 0.3).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle({
                    TextBundle::from_section(
                        "0 km/h",
                        TextStyle {
                            font: asset_server.load("fonts/RobotoCondensed-Light.ttf"),
                            font_size: 70.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_style(Style {
                        size: Size::new(Val::Px(350.0), Val::Auto),
                        ..default()
                    })
                })
                .insert(MetricText(FlightMetric::Airspeed));
            parent
                .spawn_bundle({
                    TextBundle::from_section(
                        "0.00 m/s",
                        TextStyle {
                            font: asset_server.load("fonts/RobotoCondensed-Light.ttf"),
                            font_size: 70.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_style(Style {
                        size: Size::new(Val::Px(350.0), Val::Auto),
                        ..default()
                    })
                })
                .insert(MetricText(FlightMetric::VerticalSpeed));
            parent
                .spawn_bundle({
                    TextBundle::from_section(
                        "0 m",
                        TextStyle {
                            font: asset_server.load("fonts/RobotoCondensed-Light.ttf"),
                            font_size: 70.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_style(Style {
                        size: Size::new(Val::Px(350.0), Val::Auto),
                        ..default()
                    })
                })
                .insert(MetricText(FlightMetric::Height));
        });
}

fn update_hud(
    airplane_query: Query<&FlightMetrics, With<Airplane>>,
    mut metric_query: Query<(&mut Text, &MetricText)>,
) {
    let metrics = airplane_query.single();

    for (mut text, metric) in &mut metric_query {
        let value = match metric.0 {
            FlightMetric::Airspeed => format!("{:.0} km/h", metrics.metrics[metric.0] * 3.6),
            FlightMetric::VerticalSpeed => format!("{:.2} m/s", metrics.metrics[metric.0]),
            FlightMetric::Height => format!("{:.0} m", metrics.metrics[metric.0]),
        };
        text.sections[0].value = value;
    }
}

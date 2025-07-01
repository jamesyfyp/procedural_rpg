use bevy::{color::palettes::css, prelude::*};

use crate::player::{Health, Player};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_health_bar);
        app.add_systems(Update, update_health_bar);
    }
}

#[derive(Component)]
struct HealthBarFill;

#[derive(Component)]
struct HealthBarText;

fn spawn_health_bar(mut commands: Commands) {
    // Parent node (background)
    let parent = commands
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Px(24.0),
                position_type: PositionType::Absolute,
                left: Val::Px(200.0), // Move bar horizontally
                top: Val::Px(200.0),  // Move bar vertically
                ..default()
            },
            BackgroundColor(Color::from(css::DARK_GRAY)),
        ))
        .id();

    // Fill node (foreground)
    let fill = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::from(css::GREEN)),
            HealthBarFill,
        ))
        .id();

    // Text node (health value)
    let text = commands
        .spawn((
            Text::new("100"),
            TextFont {
                font_size: 100.0,
                ..default()
            },
            TextColor(Color::from(css::WHITE)),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(5.0),
                right: Val::Px(5.0),
                ..default()
            },
            HealthBarText,
        ))
        .id();

    // Add children to parent
    commands.entity(parent).add_children(&[fill, text]);
}

fn update_health_bar(
    health_query: Query<&Health, With<Player>>,
    mut fill_query: Query<&mut Node, With<HealthBarFill>>,
    mut text_query: Query<&mut Text, With<HealthBarText>>,
) {
    if let Ok(health) = health_query.single() {
        let health_percent = (health.0 / 100.0).clamp(0.0, 1.0);

        // Update fill width
        if let Ok(mut style) = fill_query.single_mut() {
            style.width = Val::Percent(health_percent * 100.0);
        }

        // Update text
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = format!("{:.0}", health.0);
        }
    }
}

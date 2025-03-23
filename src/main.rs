use std::f32::consts::PI;

use bevy::render::camera::ScalingMode;
use bevy::render::view::window;
use bevy::scene::ron::de::Position;
use bevy::window::{PrimaryMonitor, PrimaryWindow, WindowResized};
use bevy::{prelude::*, scene::ron::de};
use rand::{random, random_range};

const HEIGHT: f32 = 100.;
const PLAYER_HEIGHT: f32 = 20.;
const PLAYER_WIDTH: f32 = 3.5;
const PLAYER_SPEED: f32 = 180.;
const PLAYER_X_PADDING: f32 = PLAYER_WIDTH / 2.;
const BALL_SPEED: f32 = 250.;
const BALL_RADIUS: f32 = 2.5;
const FONT_SIZE: f32 = 70.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Ping Pong"),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .add_systems(PreStartup, spawn_camera)
        .add_systems(Startup, spawn_ball)
        .add_systems(Startup, spawn_players)
        .add_systems(Startup, init_points)
        .add_systems(
            Update,
            (
                player_1_input,
                move_ball,
                init_move_ball,
                ball_wall_deflect,
                ball_paddle_deflect,
                handle_resize_for_paddle,
                check_scored,
            ),
        )
        .add_observer(update_score)
        .init_resource::<PlayerCountResource>()
        .run();
}

#[derive(Event)]
struct PlayerScored {
    scorer: PlayerCount,
    current_score: i8,
}

fn update_score(
    scored: Trigger<PlayerScored>,
    mut point_ui: Query<(&mut Text, &PointUI), With<PointUI>>,
) {
    for (mut text, point_ui) in point_ui.iter_mut() {
        if point_ui.player_count == scored.scorer {
            *text = Text::new(format!("{}", scored.current_score));
        }
    }
}

/// This system shows how to respond to a window being resized.
/// Whenever the window is resized, the text will update with the new resolution.
fn handle_resize_for_paddle(
    mut paddle: Query<(&mut Transform, &Player)>,
    mut resize_reader: EventReader<WindowResized>,
) {
    for window in resize_reader.read() {
        for (mut paddle, player) in paddle.iter_mut() {
            let horizontal_adjustment = PLAYER_X_PADDING + PLAYER_WIDTH / 2.;
            // When resolution is being changed
            paddle.translation.x = match player.player_count {
                PlayerCount::One => {
                    (-HEIGHT / 2. + horizontal_adjustment) * window.width / window.height
                }
                PlayerCount::Two => {
                    (HEIGHT / 2. - horizontal_adjustment) * window.width / window.height
                }
            }
        }
    }
}

fn init_points(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: FONT_SIZE,
        ..default()
    };
    commands.spawn((
        Text::new(format!("{}", 0)),
        text_font.clone(),
        TextLayout::new_with_justify(JustifyText::Left),
        PointUI {
            player_count: PlayerCount::One,
        },
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(5.0),
            ..default()
        },
    ));
    commands.spawn((
        Text::new(format!("{}", 0)),
        text_font.clone(),
        TextLayout::new_with_justify(JustifyText::Right),
        PointUI {
            player_count: PlayerCount::Two,
        },
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            right: Val::Px(5.0),
            ..default()
        },
    ));
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum PlayerCount {
    One = 1,
    #[default]
    Two = 2,
}

#[derive(Component, Default)]
#[require(Text)]
struct PointUI {
    player_count: PlayerCount,
}

#[derive(Component, Default)]
#[require(Mesh2d)]
struct Player {
    player_count: PlayerCount,
    points: i8,
}

#[derive(Component)]
#[require(Player)]
struct User;

#[derive(Component)]
#[require(Player)]
struct Computer {}

#[derive(Component, Default)]
#[require(Camera2d)]
struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: HEIGHT,
            },
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        },
    ));
}

fn check_scored(
    mut ball: Query<(&mut Velocity, &mut Transform), With<Ball>>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut player: Query<&mut Player>,
    mut commands: Commands,
) {
    if let Ok(window) = window.get_single() {
        for (mut velocity, mut transform) in ball.iter_mut() {
            if transform.translation.x >= HEIGHT / 2. * window.width() / window.height()
                || transform.translation.x <= -HEIGHT / 2. * window.width() / window.height()
            {
                let player = if transform.translation.x < 0. {
                    player
                        .iter_mut()
                        .find(|x| x.player_count == PlayerCount::Two)
                } else {
                    player
                        .iter_mut()
                        .find(|x| x.player_count == PlayerCount::One)
                };

                if let Some(mut player) = player {
                    player.points += 1;
                    commands.trigger(PlayerScored {
                        current_score: player.points,
                        scorer: player.player_count,
                    });
                }
                velocity.velocity = Vec3::new(0., 0., 0.);
                transform.translation = Vec3::new(0., 0., 0.);
            }
        }
    }
}

fn ball_wall_deflect(mut ball: Query<(&mut Velocity, &Transform), With<Ball>>) {
    if let Ok((mut velocity, transform)) = ball.get_single_mut() {
        if transform.translation.y >= HEIGHT / 2. || transform.translation.y <= -HEIGHT / 2. {
            velocity.velocity.y = -velocity.velocity.y
        }
    }
}

fn ball_paddle_deflect(
    mut ball: Query<(&mut Velocity, &Transform), With<Ball>>,
    player: Query<&Transform, With<Player>>,
) {
    if let Ok((mut velocity, transform)) = ball.get_single_mut() {
        for player in player.iter() {
            let player_rect = Rect {
                min: player.translation.xy() - Vec2::new(PLAYER_WIDTH / 2., PLAYER_HEIGHT / 2.0),
                max: player.translation.xy() + Vec2::new(PLAYER_WIDTH / 2., PLAYER_HEIGHT / 2.0),
            };

            if player_rect.contains(transform.translation.xy()) {
                velocity.velocity.x = velocity.velocity.x.abs() * -player_rect.min.x.signum();
            }
        }
    }
}

fn move_ball(mut ball: Query<(&Velocity, &mut Transform), With<Ball>>, time: Res<Time>) {
    if let Ok((velocity, mut transform)) = ball.get_single_mut() {
        transform.translation = transform.translation.move_towards(
            transform.translation + velocity.velocity,
            time.delta_secs() * BALL_SPEED,
        );
    };
}

fn init_move_ball(
    mut ball: Query<(&mut Velocity, &Transform), With<Ball>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((mut ball, transform)) = ball.get_single_mut() {
        if transform.translation.eq(&Vec3::new(0., 0., 0.)) && keys.just_pressed(KeyCode::Space) {
            let mut velocity = Vec3::new(
                random_range(-1.0..1.0),
                random_range(-1.0..1.0),
                ball.velocity.z,
            )
            .normalize();
            while velocity.angle_between(Vec3::new(0., 1., ball.velocity.z).normalize()) <= PI / 8.
                || velocity.angle_between(Vec3::new(0., -1., ball.velocity.z).normalize())
                    <= PI / 8.
            {
                // Don't allow the velocity to be straight up or straight down.
                velocity = Vec3::new(
                    random_range(-1.0..1.0),
                    random_range(-1.0..1.0),
                    ball.velocity.z,
                )
                .normalize();
            }
            ball.velocity = velocity
        };
    };
}

fn player_1_input(
    mut player: Query<(&mut Transform, &Player), With<User>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut player_transform, player) in player.iter_mut() {
        let mut y_movement = 0.;
        if player.player_count == PlayerCount::One && keys.pressed(KeyCode::KeyW)
            || player.player_count == PlayerCount::Two && keys.pressed(KeyCode::ArrowUp)
        {
            y_movement += 1.;
        };
        if player.player_count == PlayerCount::One && keys.pressed(KeyCode::KeyS)
            || player.player_count == PlayerCount::Two && keys.pressed(KeyCode::ArrowDown)
        {
            y_movement -= 1.;
        };

        if y_movement == 0. {
            continue;
        }

        let target = Vec3::new(
            player_transform.translation.x,
            HEIGHT / 2.0 * y_movement + (PLAYER_HEIGHT / 2. * -y_movement),
            player_transform.translation.z,
        );

        player_transform.translation = player_transform
            .translation
            .move_towards(target, time.delta_secs() * PLAYER_SPEED);
    }
}

#[derive(Component, Default)]
struct Velocity {
    velocity: Vec3,
}

#[derive(Component, Default)]
#[require(Mesh2d, Velocity)]
struct Ball;

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Ball,
        Velocity {
            velocity: Vec3::new(0., 0., 0.),
        },
        Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb_u8(255, 255, 255)))),
    ));
}

/// The number of users for the game all other players will likely be computers
#[derive(Resource, Default, Copy, Clone)]
struct PlayerCountResource {
    player_count: PlayerCount,
    user_count: PlayerCount,
}

fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_count: Res<PlayerCountResource>,
    cameras: Query<&OrthographicProjection, With<MainCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let shape_width = PLAYER_WIDTH;
    let shape_height = PLAYER_HEIGHT;
    // let shape = meshes.add(Circle::new(15.0))
    let shape = meshes.add(Rectangle::new(shape_width, shape_height));

    if let Ok(camera) = cameras.get_single() {
        if let Ok(window) = window_query.get_single() {
            println!("{:?}", camera);

            // the coordinates of the rectangle covered by the viewport
            let rect = camera.area;
            println!("{:?}", rect);
            let color = Color::srgb(0., 0.7, 0.7);

            let padding = PLAYER_X_PADDING;

            let y = rect.min.y + rect.height() / 2.;

            let horizontal_adjustment = PLAYER_X_PADDING + PLAYER_WIDTH / 2.;
            println!("{}", y);
            commands.spawn((
                Player {
                    player_count: PlayerCount::One,
                    ..default()
                },
                User,
                Mesh2d(shape.clone()),
                MeshMaterial2d(materials.add(color)),
                Transform::from_translation(Vec3::new(
                    (-HEIGHT / 2. + horizontal_adjustment) * window.width() / window.height(),
                    y,
                    1.,
                )),
            ));

            commands.spawn((
                Player {
                    player_count: PlayerCount::Two,
                    ..default()
                },
                User,
                Mesh2d(shape.clone()),
                MeshMaterial2d(materials.add(color)),
                Transform::from_translation(Vec3 {
                    x: (HEIGHT / 2. - horizontal_adjustment) * window.width() / window.height(),
                    y,
                    z: 1.,
                }),
            ));
        }
    }
}

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::PrimaryWindow;

const HEIGHT: f32 = 100.;
const PLAYER_HEIGHT: f32 = 10.;
const PLAYER_WIDTH: f32 = 5.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(PreStartup, spawn_camera)
        .add_systems(Startup, spawn_ball)
        .add_systems(Startup, spawn_players)
        .add_systems(Update, player_1_input)
        .init_resource::<PlayerCount>()
        .run();
}

#[derive(Component, Default)]
#[require(Mesh2d)]
struct Player {
    num: u8,
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

fn player_1_input(
    mut player: Query<(&mut Transform, &Player), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut player_transform, player) in player.iter_mut() {
        let mut y_movement = 0.;
        if player.num == 1 && keys.pressed(KeyCode::KeyW)
            || player.num == 2 && keys.pressed(KeyCode::ArrowUp)
        {
            y_movement += 1.;
        };
        if player.num == 1 && keys.pressed(KeyCode::KeyS)
            || player.num == 2 && keys.pressed(KeyCode::ArrowDown)
        {
            y_movement -= 1.;
        };

        if y_movement == 0. {
            continue;
        }

        let target = Vec3::new(
            player_transform.translation.x,
            HEIGHT / 2.0 * y_movement,
            player_transform.translation.z,
        );
        player_transform.translation = player_transform
            .translation
            .lerp(target, time.delta_secs() * 5.);
    }
}

fn player_2_input(keys: Res<ButtonInput<KeyCode>>) {}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(2.5))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb_u8(255, 255, 255)))),
    ));
}

#[derive(Resource, Default, Copy, Clone)]
enum PlayerCount {
    One = 1,
    #[default]
    Two = 2,
}

fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_count: Res<PlayerCount>,
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

            let padding = shape_width / 2.;

            let y = rect.min.y + rect.height() / 2.;

            let horizontal_adjustment = padding + shape_width / 2.;
            println!("{}", y);
            commands.spawn((
                Player { num: 1 },
                User,
                Mesh2d(shape.clone()),
                MeshMaterial2d(materials.add(color)),
                Transform::from_translation(Vec3::new(
                    (-HEIGHT / 2. + horizontal_adjustment) * window.width() / window.height(),
                    y,
                    1.,
                )),
            ));

            let color = Color::srgb(1., 1.0, 1.0);
            commands.spawn((
                Player { num: 2 },
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

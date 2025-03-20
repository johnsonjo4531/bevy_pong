use bevy::{prelude::*, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Bevy Pong"),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(PreStartup, setup)
        .add_systems(PreStartup, spawn_camera)
        .add_systems(Startup, spawn_players)
        .add_systems(Startup, spawn_ball)
        .run();
}

#[derive(Component, Default)]
#[require(Camera2d)]
struct MainCamera;

#[derive(Component, Default)]
#[require(Mesh2d)]
struct Ball;

#[derive(Component, Default)]
#[require(Mesh2d)]
struct Player {
    num: u8,
}

// Define a custom resource to hold the player count
#[derive(Resource)]
struct PlayerCount(u32);

impl Default for PlayerCount {
    fn default() -> Self {
        PlayerCount(2)
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(PlayerCount(2));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((MainCamera));
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cameras: Query<&OrthographicProjection, Added<MainCamera>>,
) {
    // let shape = meshes.add(Circle::new(15.0))
    let circle_radius = 15.;
    let shape = meshes.add(Circle::new(circle_radius));

    let camera = cameras.single();

    // the coordinates of the rectangle covered by the viewport
    let rect = camera.area;

    let color = Color::srgb(1.0, 1.0, 1.0);

    commands.spawn((
        Ball,
        Mesh2d(shape),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(
            rect.min.x + rect.width() / 2.,
            rect.min.y + rect.height() / 2.,
            0.0,
        ),
    ));
}

fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_count: Res<PlayerCount>,
    cameras: Query<&Camera, Added<MainCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let shape_width = 50.;
    let shape_height = 100.;
    // let shape = meshes.add(Circle::new(15.0))
    let shape = meshes.add(Rectangle::new(shape_width, shape_height));
    let window = window.single();
    let camera = cameras.single();
    println!("{:?}", camera);

    // the coordinates of the rectangle covered by the viewport
    let rect = camera.logical_viewport_rect().unwrap_or(Rect {
        min: Vec2::new(-window.width() / 2., -window.height() / 2.),
        max: Vec2::new(window.width() / 2., window.height() / 2.),
    });
    println!("{:?}", rect);
    let color = Color::srgb(1.0, 1.0, 1.0);

    let padding = shape_width / 2.;
    for i in 1..(player_count.0 + 1) {
        let x = match i {
            1 => Some(padding),
            2 => Some(rect.width() - shape_width - padding),
            _ => None,
        };

        if let Some(x) = x {
            let x = rect.min.x + x;
            let y = rect.min.y + rect.height() / 2. + shape_height / 2.;

            println!("{} {}", x, y);
            commands.spawn((
                Player { num: i as u8 },
                Mesh2d(shape.clone()),
                MeshMaterial2d(materials.add(color)),
                Transform::from_translation(Vec3 { x, y, z: 0. }),
            ));
        }
    }
}

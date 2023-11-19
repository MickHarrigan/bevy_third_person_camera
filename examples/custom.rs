use bevy::prelude::*;
use bevy_third_person_camera::*;
use std::f32::consts::{E, PI};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ThirdPersonCameraPlugin))
        .add_systems(Startup, (spawn_player, spawn_camera, spawn_world))
        // .add_systems(Update, (player_movement_keyboard, log))
        .add_systems(Update, player_movement_keyboard)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Speed(f32);

fn log(camera: Query<(&Transform, &ThirdPersonCamera)>) {
    for (location, cam) in &camera {
        // inside has to be negative to make top Pi and bottom 0
        // info!("Angle: {}", cam.focus.angle_between(-location.translation));
        info!("Forward: {}", location.forward().xz().normalize());
    }
}

fn spawn_player(mut commands: Commands, assets: Res<AssetServer>) {
    let model = assets.load("Player2.gltf#Scene0");
    let player = (
        SceneBundle {
            scene: model,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Player,
        ThirdPersonCameraTarget,
        Speed(2.5),
    );
    commands.spawn(player);
}

// TODO: fix the location of the focus in terms of the actual world, otherwise it seems alright
// with the debug info calls it shows that the functions and things are now able to modify both the
// camera focus and the displacement via the data defined in the struct
fn spawn_camera(mut commands: Commands) {
    let camera = (
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.5, 5.0)
                .looking_at(Vec3::new(0., 0.81, 0.), Vec3::Y),
            ..default()
        },
        ThirdPersonCamera {
            true_focus: Vec3::new(0., 0.81, 0.),
            aim_enabled: false,
            aim_zoom: 0.7,
            zoom_enabled: false,
            zoom: Zoom::new(2.0, 5.0),
            focus_modifier: CameraFocusModifier {
                lower_threshold: PI / 2.,
                upper_threshold: 2. * PI / 3.,
                max_forward_displacement: 0.5,
                max_backward_displacement: 0.5,
                // typical logistic function centered at 0.5
                lower_displacement_function: |x| 1. / (1. + E.powf(-15. * (x - 0.5))),
                upper_displacement_function: |x| 1. / (1. + E.powf(-15. * (x - 0.5))),
            },
            ..default()
        },
    );
    commands.spawn(camera);
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,

    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane::from_size(15.0))),
        material: materials.add(Color::DARK_GREEN.into()),
        ..default()
    };

    let light = PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 5.0, 0.0),
        ..default()
    };

    commands.spawn(floor);
    commands.spawn(light);
}

fn player_movement_keyboard(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut player_q: Query<(&mut Transform, &Speed), With<Player>>,
    cam_q: Query<&Transform, (With<Camera3d>, Without<Player>)>,
) {
    for (mut player_transform, player_speed) in player_q.iter_mut() {
        let cam = match cam_q.get_single() {
            Ok(c) => c,
            Err(e) => Err(format!("Error retrieving camera: {}", e)).unwrap(),
        };

        let mut direction = Vec3::ZERO;

        // forward
        if keys.pressed(KeyCode::W) {
            direction += cam.forward();
        }

        // back
        if keys.pressed(KeyCode::S) {
            direction += cam.back();
        }

        // left
        if keys.pressed(KeyCode::A) {
            direction += cam.left();
        }

        // right
        if keys.pressed(KeyCode::D) {
            direction += cam.right();
        }

        direction.y = 0.0;
        let movement = direction.normalize_or_zero() * player_speed.0 * time.delta_seconds();
        player_transform.translation += movement;

        // rotate player to face direction he is currently moving
        if direction.length_squared() > 0.0 {
            player_transform.look_to(direction, Vec3::Y);
        }
    }
}

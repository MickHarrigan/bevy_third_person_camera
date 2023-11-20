use bevy::prelude::*;
use bevy_third_person_camera::*;
use std::f32::consts::{E, PI};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ThirdPersonCameraPlugin))
        .add_systems(Startup, (spawn_player, spawn_camera, spawn_world))
        .add_systems(Update, (player_movement_gamepad, player_movement_keyboard))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Speed(f32);

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
            aim_enabled: true,
            aim_zoom: 0.7,
            zoom_enabled: false,
            zoom: Zoom::new(1.5, 5.0),
            offset_enabled: true,
            offset: Offset::new(0.4, 0.0),
            focus_modifier: CameraFocusModifier {
                lower_threshold: PI / 2.,
                upper_threshold: 2. * PI / 3.,
                max_forward_displacement: 0.5,
                max_backward_displacement: 0.75,
                // typical logistic function centered at 0.5
                lower_displacement_function: |x| 1. / (1. + E.powf(-15. * (x - 0.5))),
                upper_displacement_function: |x| 1. / (1. + E.powf(-15. * (x - 0.5))),
                behind_radius_displacement: 2.0,
                lower_radius_function: |x| 1. - E.powf(-4. * x),
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

        let mut direction = Vec2::ZERO;

        // forward
        if keys.pressed(KeyCode::W) {
            direction += cam.forward().xz().normalize();
        }

        // back
        if keys.pressed(KeyCode::S) {
            direction += cam.back().xz().normalize();
        }

        // left
        if keys.pressed(KeyCode::A) {
            direction += cam.left().xz().normalize();
        }

        // right
        if keys.pressed(KeyCode::D) {
            direction += cam.right().xz().normalize();
        }

        let movement = direction * player_speed.0 * time.delta_seconds();
        player_transform.translation.x += movement.x;
        player_transform.translation.z += movement.y;
        let direction: Vec3 = (direction.x, 0.0, direction.y).into();

        // rotate player to face direction he is currently moving
        if direction.length_squared() > 0.0 {
            player_transform.look_to(direction, Vec3::Y);
        }
    }
}

fn player_movement_gamepad(
    time: Res<Time>,
    axis: Res<Axis<GamepadAxis>>,
    gamepad_res: Option<Res<GamepadResource>>,
    mut player_q: Query<(&mut Transform, &Speed), With<Player>>,
    cam_q: Query<&Transform, (With<Camera3d>, Without<Player>)>,
) {
    let gamepad = if let Some(gp) = gamepad_res {
        gp.0
    } else {
        return;
    };

    let Ok(cam) = cam_q.get_single() else {
        return;
    };

    for (mut player_transform, player_speed) in player_q.iter_mut() {
        let x_axis = GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX);
        let y_axis = GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY);

        let deadzone = 0.5;
        let mut direction = Vec2::ZERO;
        if let (Some(x), Some(y)) = (axis.get(x_axis), axis.get(y_axis)) {
            if x.abs() > deadzone || y.abs() > deadzone {
                if y > deadzone {
                    // north
                    direction += y * cam.forward().xz().normalize();
                }
                if y < deadzone {
                    // south
                    direction -= y * cam.back().xz().normalize();
                }
                if x > deadzone {
                    // east
                    direction += x * cam.right().xz().normalize();
                }
                if x < deadzone {
                    // west
                    direction -= x * cam.left().xz().normalize();
                }
            }

            let movement = direction * player_speed.0 * time.delta_seconds();
            player_transform.translation.x += movement.x;
            player_transform.translation.z += movement.y;
            let direction: Vec3 = (direction.x, 0.0, direction.y).into();

            if direction.length_squared() > 0.0 {
                player_transform.look_to(direction, Vec3::Y);
            }
        }
    }
}

use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{zoom_condition, ThirdPersonCamera};

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                orbit_mouse.run_if(orbit_condition),
                zoom_mouse.run_if(zoom_condition),
            )
                .chain(),
        );
    }
}

// only run the orbit system if the cursor lock is disabled
fn orbit_condition(cam_q: Query<&ThirdPersonCamera>) -> bool {
    let Ok(cam) = cam_q.get_single() else {
        return true;
    };
    return cam.cursor_lock_active;
}

// heavily referenced https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
pub fn orbit_mouse(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut cam_q: Query<(&ThirdPersonCamera, &mut Transform), With<ThirdPersonCamera>>,
    mouse: Res<Input<MouseButton>>,
    mut mouse_evr: EventReader<MouseMotion>,
) {
    let mut rotation = Vec2::ZERO;
    for ev in mouse_evr.read() {
        rotation = ev.delta;
    }

    let Ok((cam, mut cam_transform)) = cam_q.get_single_mut() else {
        return;
    };

    if cam.mouse_orbit_button_enabled && !mouse.pressed(cam.mouse_orbit_button) {
        return;
    }

    rotation *= cam.mouse_sensitivity;

    if rotation.length_squared() > 0.0 {
        let window = window_q.get_single().unwrap();
        let delta_x = rotation.x / window.width() * std::f32::consts::PI;
        let delta_y = rotation.y / window.height() * PI;
        let yaw = Quat::from_rotation_y(-delta_x);
        let pitch = Quat::from_rotation_x(-delta_y);
        cam_transform.rotation = yaw * cam_transform.rotation; // rotate around global y axis

        // Calculate the new rotation without applying it to the camera yet
        let new_rotation = cam_transform.rotation * pitch;

        // check if new rotation will cause camera to go beyond the 180 degree vertical bounds
        let up_vector = new_rotation * Vec3::Y;
        if up_vector.y > 0.0 {
            cam_transform.rotation = new_rotation;
        }
    }
    let aim_btn = mouse.pressed(cam.aim_button);

    let rot_matrix = Mat3::from_quat(cam_transform.rotation);
    // apply the offset if offset_enabled is true and the user is aiming currently
    let mut offset = Vec3::ZERO;
    if cam.offset_enabled && aim_btn {
        offset = rot_matrix.mul_vec3(Vec3::new(cam.offset.offset.0, cam.offset.offset.1, 0.0));
    }

    cam_transform.translation =
        cam.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, cam.zoom.radius)) + offset;
}

fn zoom_mouse(mut scroll_evr: EventReader<MouseWheel>, mut cam_q: Query<&mut ThirdPersonCamera>) {
    let mut scroll = 0.0;
    for ev in scroll_evr.read() {
        scroll += ev.y;
    }

    if let Ok(mut cam) = cam_q.get_single_mut() {
        if scroll.abs() > 0.0 {
            let new_radius =
                cam.zoom.radius - scroll * cam.zoom.radius * 0.1 * cam.zoom_sensitivity;
            cam.zoom.radius = new_radius.clamp(cam.zoom.min, cam.zoom.max);
        }
    }
}

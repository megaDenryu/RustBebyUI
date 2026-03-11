use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use crate::voxel_world::水面高さ;
use crate::collision::is_colliding;
use crate::chunk_system::ChunkManager;

#[derive(Component)]
pub struct UnityCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub speed: f32,
    pub velocity_y: f32,
    pub is_grounded: bool,
}

pub fn handle_unity_camera(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut UnityCamera)>,
    chunk_manager: Res<ChunkManager>,
) {
    let (mut transform, mut cam) = query.single_mut();
    let delta = time.delta_secs();

    // 1. Mouse Look (Right Click)
    if mouse_button.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            cam.yaw -= ev.delta.x * cam.sensitivity;
            cam.pitch -= ev.delta.y * cam.sensitivity;
        }
    }
    cam.pitch = cam.pitch.clamp(-1.5, 1.5);
    transform.rotation = Quat::from_rotation_y(cam.yaw) * Quat::from_rotation_x(cam.pitch);

    // 2. Gravity
    let gravity = -20.0;
    cam.velocity_y += gravity * delta;

    // 3. Horizontal Movement (XZ only)
    let mut move_dir = Vec3::ZERO;
    let forward = (transform.rotation * Vec3::NEG_Z).with_y(0.0).normalize_or_zero();
    let right = (transform.rotation * Vec3::X).with_y(0.0).normalize_or_zero();

    if keys.pressed(KeyCode::KeyW) { move_dir += forward; }
    if keys.pressed(KeyCode::KeyS) { move_dir -= forward; }
    if keys.pressed(KeyCode::KeyA) { move_dir -= right; }
    if keys.pressed(KeyCode::KeyD) { move_dir += right; }

    let move_speed = cam.speed * (if keys.pressed(KeyCode::ShiftLeft) { 1.5 } else { 1.0 });
    let horizontal_movement = move_dir.normalize_or_zero() * move_speed * delta;

    // 4. Jumping
    if cam.is_grounded && keys.pressed(KeyCode::Space) {
        cam.velocity_y = 7.0;
        cam.is_grounded = false;
    }

    // 5. Collision & Movement
    let mut next_pos = transform.translation;

    // --- Vertical Movement ---
    let y_movement = cam.velocity_y * delta;
    let next_y_pos = next_pos + Vec3::new(0.0, y_movement, 0.0);
    if !is_colliding(next_y_pos, &chunk_manager) {
        next_pos.y = next_y_pos.y;
        cam.is_grounded = false;
    } else {
        if cam.velocity_y < 0.0 {
            cam.is_grounded = true;
        }
        cam.velocity_y = 0.0;
    }

    // --- Horizontal Movement (Sliding) ---
    let next_x_pos = next_pos + Vec3::new(horizontal_movement.x, 0.0, 0.0);
    if !is_colliding(next_x_pos, &chunk_manager) {
        next_pos.x = next_x_pos.x;
    }

    let next_z_pos = next_pos + Vec3::new(0.0, 0.0, horizontal_movement.z);
    if !is_colliding(next_z_pos, &chunk_manager) {
        next_pos.z = next_z_pos.z;
    }

    transform.translation = next_pos;

    // 6. Underwater physics
    if transform.translation.y < 水面高さ {
        cam.velocity_y = cam.velocity_y.max(-1.0) + 5.0 * delta; // Buoyancy
        if keys.pressed(KeyCode::Space) {
            cam.velocity_y = 2.0;
        }
    }
}

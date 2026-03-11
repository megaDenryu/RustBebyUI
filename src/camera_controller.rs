use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use crate::voxel_world::{水面高さ, 岩盤の高さ};
use crate::collision::is_colliding;
use crate::chunk_system::ChunkManager;

pub const INITIAL_POS: Vec3 = Vec3::new(8.0, 25.0, 8.0);

#[derive(Component)]
pub struct UnityCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub speed: f32,
    pub velocity_y: f32,
    pub is_grounded: bool,
    pub is_flying: bool,
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

    // R: リセット
    if keys.just_pressed(KeyCode::KeyR) {
        transform.translation = INITIAL_POS;
        cam.velocity_y = 0.0;
        cam.is_flying = false;
        cam.is_grounded = false;
        return;
    }

    // マウスルック (右クリック)
    if mouse_button.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            cam.yaw -= ev.delta.x * cam.sensitivity;
            cam.pitch -= ev.delta.y * cam.sensitivity;
        }
    }
    cam.pitch = cam.pitch.clamp(-1.5, 1.5);
    transform.rotation = Quat::from_rotation_y(cam.yaw) * Quat::from_rotation_x(cam.pitch);

    // Q/E: 飛行モード開始
    if !cam.is_flying && (keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::KeyE)) {
        cam.is_flying = true;
        cam.velocity_y = 5.0;
        cam.is_grounded = false;
    }

    if cam.is_flying {
        // === 飛行モード (Unity Scene View風) ===
        let fly_speed = cam.speed * 2.0 * (if keys.pressed(KeyCode::ShiftLeft) { 2.5 } else { 1.0 });
        let forward = transform.rotation * Vec3::NEG_Z; // 3D方向
        let right = transform.rotation * Vec3::X;

        let mut move_dir = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) { move_dir += forward; }
        if keys.pressed(KeyCode::KeyS) { move_dir -= forward; }
        if keys.pressed(KeyCode::KeyA) { move_dir -= right; }
        if keys.pressed(KeyCode::KeyD) { move_dir += right; }
        if keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::Space) { move_dir += Vec3::Y; }
        if keys.pressed(KeyCode::KeyE) { move_dir -= Vec3::Y; }

        let movement = move_dir.normalize_or_zero() * fly_speed * delta;
        let next_pos = transform.translation + movement;

        // 岩盤制限
        transform.translation = if next_pos.y > 岩盤の高さ + 0.5 { next_pos }
            else { Vec3::new(next_pos.x, 岩盤の高さ + 0.5, next_pos.z) };

        // 着地判定: 地面に触れたら歩行モードに戻る
        if is_colliding(transform.translation + Vec3::new(0.0, -0.4, 0.0), &chunk_manager) {
            cam.is_flying = false;
            cam.is_grounded = true;
            cam.velocity_y = 0.0;
        }
    } else {
        // === 歩行モード ===
        let is_underwater = transform.translation.y < 水面高さ;

        // 重力 / 水中物理
        if is_underwater {
            let buoyancy = 3.0;
            cam.velocity_y = cam.velocity_y * 0.93 + buoyancy * delta;
            if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ControlLeft) {
                cam.velocity_y = -3.0; // 潜水
            }
            if keys.pressed(KeyCode::Space) {
                cam.velocity_y = 3.5; // 浮上
            }
        } else {
            cam.velocity_y += -20.0 * delta;
        }

        // 水平移動
        let mut move_dir = Vec3::ZERO;
        let forward = (transform.rotation * Vec3::NEG_Z).with_y(0.0).normalize_or_zero();
        let right = (transform.rotation * Vec3::X).with_y(0.0).normalize_or_zero();
        if keys.pressed(KeyCode::KeyW) { move_dir += forward; }
        if keys.pressed(KeyCode::KeyS) { move_dir -= forward; }
        if keys.pressed(KeyCode::KeyA) { move_dir -= right; }
        if keys.pressed(KeyCode::KeyD) { move_dir += right; }

        let speed_mult = if is_underwater { 0.6 } else if keys.pressed(KeyCode::ShiftLeft) { 1.5 } else { 1.0 };
        let h_move = move_dir.normalize_or_zero() * cam.speed * speed_mult * delta;

        // ジャンプ
        if cam.is_grounded && !is_underwater && keys.pressed(KeyCode::Space) {
            cam.velocity_y = 7.0;
            cam.is_grounded = false;
        }

        // 衝突判定 + 移動
        let mut next_pos = transform.translation;

        // 垂直
        let y_move = cam.velocity_y * delta;
        let next_y = next_pos + Vec3::new(0.0, y_move, 0.0);
        if next_y.y <= 岩盤の高さ + 0.5 {
            next_pos.y = 岩盤の高さ + 0.5;
            cam.velocity_y = 0.0;
            cam.is_grounded = true;
        } else if !is_colliding(next_y, &chunk_manager) {
            next_pos.y = next_y.y;
            cam.is_grounded = false;
        } else {
            if cam.velocity_y < 0.0 { cam.is_grounded = true; }
            cam.velocity_y = 0.0;
        }

        // 水平 (スライディング)
        let nx = next_pos + Vec3::new(h_move.x, 0.0, 0.0);
        if !is_colliding(nx, &chunk_manager) { next_pos.x = nx.x; }
        let nz = next_pos + Vec3::new(0.0, 0.0, h_move.z);
        if !is_colliding(nz, &chunk_manager) { next_pos.z = nz.z; }

        transform.translation = next_pos;
    }
}

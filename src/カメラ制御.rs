use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use crate::ボクセル世界::{水面高さ, 岩盤の高さ};
use crate::衝突判定::衝突判定処理;
use crate::チャンク管理::チャンク管理者;

// 地形高さ(8,8) ≈ 15.7 + 目の高さ 1.5 = 17.2
pub const 初期位置: Vec3 = Vec3::new(8.0, 17.5, 8.0);

// 物理定数
const 重力加速度: f32 = -20.0;
const ジャンプ力: f32 = 7.0;
const 浮力: f32 = 3.0;
const 水中減衰: f32 = 0.93;
const 潜水速度: f32 = -3.0;
const 浮上速度: f32 = 3.5;
const 飛行上昇速度: f32 = 5.0;
const ダッシュ倍率: f32 = 2.5;
const 水中速度倍率: f32 = 0.6;
const 地上ダッシュ倍率: f32 = 1.5;

#[derive(Component)]
pub struct カメラ操作 {
    pub yaw: f32,
    pub pitch: f32,
    pub 感度: f32,
    pub 速度: f32,
    pub 垂直速度: f32,
    pub 接地中: bool,
    pub 飛行中: bool,
}

pub fn カメラ操作処理(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut カメラ操作)>,
    chunk_manager: Res<チャンク管理者>,
) {
    let (mut transform, mut cam) = query.single_mut();
    let delta = time.delta_secs();

    // R: リセット
    if keys.just_pressed(KeyCode::KeyR) {
        transform.translation = 初期位置;
        cam.垂直速度 = 0.0;
        cam.飛行中 = false;
        cam.接地中 = false;
        return;
    }

    // マウスルック (右クリック)
    if mouse_button.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            cam.yaw -= ev.delta.x * cam.感度;
            cam.pitch -= ev.delta.y * cam.感度;
        }
    }
    cam.pitch = cam.pitch.clamp(-1.5, 1.5);
    transform.rotation = Quat::from_rotation_y(cam.yaw) * Quat::from_rotation_x(cam.pitch);

    // Q/E: 飛行モード開始
    if !cam.飛行中 && (keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::KeyE)) {
        cam.飛行中 = true;
        cam.垂直速度 = 飛行上昇速度;
        cam.接地中 = false;
    }

    if cam.飛行中 {
        飛行モード更新(&mut transform, &mut cam, &keys, delta, &chunk_manager);
    } else {
        歩行モード更新(&mut transform, &mut cam, &keys, delta, &chunk_manager);
    }
}

fn 飛行モード更新(
    transform: &mut Transform,
    cam: &mut カメラ操作,
    keys: &ButtonInput<KeyCode>,
    delta: f32,
    chunk_manager: &チャンク管理者,
) {
    let fly_speed = cam.速度 * 2.0 * (if keys.pressed(KeyCode::ShiftLeft) { ダッシュ倍率 } else { 1.0 });
    let forward = transform.rotation * Vec3::NEG_Z;
    let right = transform.rotation * Vec3::X;

    let mut move_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { move_dir += forward; }
    if keys.pressed(KeyCode::KeyS) { move_dir -= forward; }
    if keys.pressed(KeyCode::KeyA) { move_dir -= right; }
    if keys.pressed(KeyCode::KeyD) { move_dir += right; }
    if keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::Space) { move_dir += Vec3::Y; }
    if keys.pressed(KeyCode::KeyE) { move_dir -= Vec3::Y; }

    let next_pos = transform.translation + move_dir.normalize_or_zero() * fly_speed * delta;

    // 岩盤制限
    transform.translation = if next_pos.y > 岩盤の高さ + 0.5 { next_pos }
        else { Vec3::new(next_pos.x, 岩盤の高さ + 0.5, next_pos.z) };

    // 着地判定
    if 衝突判定処理(transform.translation + Vec3::new(0.0, -0.4, 0.0), chunk_manager) {
        cam.飛行中 = false;
        cam.接地中 = true;
        cam.垂直速度 = 0.0;
    }
}

fn 歩行モード更新(
    transform: &mut Transform,
    cam: &mut カメラ操作,
    keys: &ButtonInput<KeyCode>,
    delta: f32,
    chunk_manager: &チャンク管理者,
) {
    let is_underwater = transform.translation.y < 水面高さ;

    // 重力 / 水中物理
    if is_underwater {
        cam.垂直速度 = cam.垂直速度 * 水中減衰 + 浮力 * delta;
        if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ControlLeft) {
            cam.垂直速度 = 潜水速度;
        }
        if keys.pressed(KeyCode::Space) {
            cam.垂直速度 = 浮上速度;
        }
    } else {
        cam.垂直速度 += 重力加速度 * delta;
    }

    // 水平移動
    let forward = (transform.rotation * Vec3::NEG_Z).with_y(0.0).normalize_or_zero();
    let right = (transform.rotation * Vec3::X).with_y(0.0).normalize_or_zero();
    let mut move_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { move_dir += forward; }
    if keys.pressed(KeyCode::KeyS) { move_dir -= forward; }
    if keys.pressed(KeyCode::KeyA) { move_dir -= right; }
    if keys.pressed(KeyCode::KeyD) { move_dir += right; }

    let speed_mult = if is_underwater { 水中速度倍率 } else if keys.pressed(KeyCode::ShiftLeft) { 地上ダッシュ倍率 } else { 1.0 };
    let h_move = move_dir.normalize_or_zero() * cam.速度 * speed_mult * delta;

    // ジャンプ
    if cam.接地中 && !is_underwater && keys.pressed(KeyCode::Space) {
        cam.垂直速度 = ジャンプ力;
        cam.接地中 = false;
    }

    // 衝突判定 + 移動
    let mut next_pos = transform.translation;

    // 垂直
    let next_y = next_pos + Vec3::new(0.0, cam.垂直速度 * delta, 0.0);
    if next_y.y <= 岩盤の高さ + 0.5 {
        next_pos.y = 岩盤の高さ + 0.5;
        cam.垂直速度 = 0.0;
        cam.接地中 = true;
    } else if !衝突判定処理(next_y, chunk_manager) {
        next_pos.y = next_y.y;
        cam.接地中 = false;
    } else {
        if cam.垂直速度 < 0.0 { cam.接地中 = true; }
        cam.垂直速度 = 0.0;
    }

    // 水平 (スライディング)
    let nx = next_pos + Vec3::new(h_move.x, 0.0, 0.0);
    if !衝突判定処理(nx, chunk_manager) { next_pos.x = nx.x; }
    let nz = next_pos + Vec3::new(0.0, 0.0, h_move.z);
    if !衝突判定処理(nz, chunk_manager) { next_pos.z = nz.z; }

    transform.translation = next_pos;
}

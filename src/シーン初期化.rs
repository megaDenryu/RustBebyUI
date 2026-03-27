// src/シーン初期化.rs
// 3Dシーンのセットアップ (カメラ, ライト, マテリアル, リソース初期化)

use bevy::prelude::*;
use crate::ボクセル世界;
use crate::カメラ制御::カメラ操作;
use crate::チャンク管理::{チャンク管理者, ボクセル素材};
use crate::四面体チャンク管理::四面体チャンク管理者;

// =============================================================================
// コンポーネント
// =============================================================================

#[derive(Component)]
pub struct プレイヤーライト;

// =============================================================================
// ライティング定数
// =============================================================================
const 太陽光強度: f32 = 15_000.0;
const プレイヤーライト強度: f32 = 80_000.0;
const プレイヤーライト範囲: f32 = 25.0;
const 環境光明度: f32 = 800.0;
const フォグ開始チャンク数: f32 = 8.0;
const フォグ終了チャンク数: f32 = 19.0;

// =============================================================================
// 初期化システム
// =============================================================================

pub fn シーン初期化(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // --- リソース初期化 ---
    commands.insert_resource(チャンク管理者 {
        読込済み: std::collections::HashMap::new(),
        読込中: std::collections::HashSet::new(),
    });
    commands.insert_resource(四面体チャンク管理者 {
        読込済み: std::collections::HashMap::new(),
        読込中: std::collections::HashSet::new(),
    });

    let voxel_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: false,
        perceptual_roughness: 0.9,
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });
    commands.insert_resource(ボクセル素材(voxel_mat));

    // --- カメラ + フォグ ---
    let fog_color = Color::srgb(0.53, 0.72, 0.9);
    let chunk_size = ボクセル世界::チャンクのワールドサイズ;

    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(fog_color),
            ..default()
        },
        Transform::from_xyz(8.0, 17.5, 8.0).looking_at(Vec3::new(12.0, 16.5, 12.0), Vec3::Y),
        カメラ操作 {
            yaw: -0.4, pitch: -0.1,
            感度: 0.002, 速度: 5.0,
            垂直速度: 0.0, 接地中: false, 飛行中: false,
        },
        DistanceFog {
            color: fog_color,
            falloff: FogFalloff::Linear {
                start: フォグ開始チャンク数 * chunk_size,
                end: フォグ終了チャンク数 * chunk_size,
            },
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.95, 0.8),
                intensity: プレイヤーライト強度,
                range: プレイヤーライト範囲,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(0.0, -0.5, 0.0),
            プレイヤーライト,
        ));
    });

    // --- 太陽光 ---
    commands.spawn((
        DirectionalLight {
            illuminance: 太陽光強度,
            shadows_enabled: true,
            shadow_depth_bias: 0.1,
            shadow_normal_bias: 0.2,
            color: Color::srgb(1.0, 0.98, 0.9),
            ..default()
        },
        Transform::from_xyz(60.0, 120.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6, 0.7, 0.9),
        brightness: 環境光明度,
    });
}

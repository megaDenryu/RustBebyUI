use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::ボクセル世界::{チャンク, 描画距離, チャンクのワールドサイズ};
use crate::メッシュ生成;
use crate::地形生成;
use crate::カメラ制御::カメラ操作;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

// =============================================================================
// LOD 距離閾値
// =============================================================================
const LOD1距離2乗: i32 = 6 * 6;
const LOD2距離2乗: i32 = 12 * 12;
const LOD4距離2乗: i32 = 17 * 17;
const 毎フレーム最大タスク数: usize = 2;
const チャンク更新移動閾値: f32 = 0.5;

fn LOD決定(距離2乗: i32) -> u32 {
    if 距離2乗 <= LOD1距離2乗 { 1 }
    else if 距離2乗 <= LOD2距離2乗 { 2 }
    else if 距離2乗 <= LOD4距離2乗 { 4 }
    else { 8 }
}

// =============================================================================
// リソース・コンポーネント
// =============================================================================

#[derive(Resource)]
pub struct チャンク管理者 {
    pub 読込済み: HashMap<(i32, i32, i32), (Entity, チャンク, u32)>,
    pub 読込中: HashSet<(i32, i32, i32)>,
}

#[derive(Resource)]
pub struct ボクセル素材(pub Handle<StandardMaterial>);

#[derive(Component)]
pub struct チャンク描画データ {
    pub cx: i32,
    pub cy: i32,
    pub cz: i32,
    pub lod: u32,
}

#[derive(Component)]
pub struct チャンクタスク {
    pub 位置: (i32, i32, i32),
    pub タスク: Task<Option<((i32, i32, i32), チャンク, Mesh, u32)>>,
}

// =============================================================================
// システム
// =============================================================================

pub fn チャンク管理処理(
    mut commands: Commands,
    camera_query: Query<&Transform, With<カメラ操作>>,
    mut manager: ResMut<チャンク管理者>,
    mut last_pos: Local<Option<Vec3>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return };
    let cam_p = camera_transform.translation;

    if let Some(lp) = *last_pos {
        if cam_p.distance(lp) < チャンク更新移動閾値 { return; }
    }
    *last_pos = Some(cam_p);

    let center_cx = (cam_p.x / チャンクのワールドサイズ).floor() as i32;
    let center_cz = (cam_p.z / チャンクのワールドサイズ).floor() as i32;

    let needed = 必要チャンク集合(center_cx, center_cz);

    // 不要チャンクの削除
    manager.読込済み.retain(|&pos, &mut (entity, _, _)| {
        if !needed.contains(&pos) {
            commands.entity(entity).despawn_recursive();
            return false;
        }
        true
    });
    manager.読込中.retain(|pos| needed.contains(pos));

    // 新規タスクの生成
    let mut tasks_to_spawn: Vec<_> = needed.iter()
        .filter(|pos| !manager.読込中.contains(pos))
        .filter_map(|&pos| {
            let dist_sq = (pos.0 - center_cx).pow(2) + (pos.2 - center_cz).pow(2);
            let target_lod = LOD決定(dist_sq);
            let needs = match manager.読込済み.get(&pos) {
                Some(&(_, _, current_lod)) => current_lod != target_lod,
                None => true,
            };
            if needs { Some((pos, dist_sq, target_lod)) } else { None }
        })
        .collect();

    tasks_to_spawn.sort_by_key(|t| t.1);

    let pool = AsyncComputeTaskPool::get();
    for (pos, _, target_lod) in tasks_to_spawn.into_iter().take(毎フレーム最大タスク数) {
        manager.読込中.insert(pos);
        let task = pool.spawn(async move {
            let chunk = 地形生成::チャンク地形生成(pos.0, pos.1, pos.2);
            let mesh = メッシュ生成::メッシュ生成(&chunk, target_lod);
            Some((pos, chunk, mesh, target_lod))
        });
        commands.spawn(チャンクタスク { 位置: pos, タスク: task });
    }
}

pub fn チャンクタスク処理(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut チャンクタスク)>,
    mut manager: ResMut<チャンク管理者>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ボクセル素材>,
) {
    for (task_entity, mut chunk_task) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut chunk_task.タスク)) {
            commands.entity(task_entity).despawn();

            if let Some((pos, chunk, mesh, lod)) = result {
                if !manager.読込中.contains(&pos) { continue; }

                if let Some((old_entity, _, _)) = manager.読込済み.get(&pos) {
                    commands.entity(*old_entity).despawn_recursive();
                }

                let entity = commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material.0.clone()),
                    Transform::from_xyz(
                        pos.0 as f32 * チャンクのワールドサイズ,
                        pos.1 as f32 * チャンクのワールドサイズ,
                        pos.2 as f32 * チャンクのワールドサイズ,
                    ),
                    チャンク描画データ { cx: pos.0, cy: pos.1, cz: pos.2, lod },
                )).id();

                manager.読込済み.insert(pos, (entity, chunk, lod));
                manager.読込中.remove(&pos);
                break; // 1フレーム1メッシュ登録 (パフォーマンス)
            }
        }
    }
}

// =============================================================================
// ユーティリティ
// =============================================================================

fn 必要チャンク集合(center_cx: i32, center_cz: i32) -> HashSet<(i32, i32, i32)> {
    let mut set = HashSet::new();
    for dx in -描画距離..=描画距離 {
        for dz in -描画距離..=描画距離 {
            if dx * dx + dz * dz > 描画距離 * 描画距離 { continue; }
            set.insert((center_cx + dx, 0, center_cz + dz));
        }
    }
    set
}

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::ボクセル世界::{チャンク, 描画距離, チャンクのワールドサイズ};
use crate::メッシュ生成;
use crate::カメラ制御::カメラ操作;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

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

pub fn チャンク管理処理(
    mut commands: Commands,
    camera_query: Query<&Transform, With<カメラ操作>>,
    mut chunk_manager: ResMut<チャンク管理者>,
    mut last_pos: Local<Option<Vec3>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return };
    let cam_p = camera_transform.translation;

    // チャンク更新頻度を落とす (0.5ユニット移動するか、一定時間経過)
    if let Some(lp) = *last_pos {
        if cam_p.distance(lp) < 0.5 {
            return;
        }
    }
    *last_pos = Some(cam_p);

    let center_cx = (cam_p.x / チャンクのワールドサイズ).floor() as i32;
    let center_cz = (cam_p.z / チャンクのワールドサイズ).floor() as i32;
    let center_cy = 0;

    let mut needed_chunks = HashSet::new();
    for dx in -描画距離..=描画距離 {
        for dz in -描画距離..=描画距離 {
            if dx * dx + dz * dz > 描画距離 * 描画距離 { continue; }
            needed_chunks.insert((center_cx + dx, center_cy, center_cz + dz));
        }
    }

    chunk_manager.読込済み.retain(|&pos, &mut (entity, _, _)| {
        if !needed_chunks.contains(&pos) {
            commands.entity(entity).despawn_recursive();
            return false;
        }
        true
    });

    // 進行中フラグのクリーンアップ
    chunk_manager.読込中.retain(|pos| needed_chunks.contains(pos));

    // 必要なタスクの優先順位付けと収集
    let mut tasks_to_spawn = Vec::new();
    for &pos in &needed_chunks {
        if chunk_manager.読込中.contains(&pos) { continue; }

        let dx = pos.0 - center_cx;
        let dz = pos.2 - center_cz;
        let dist_sq = dx * dx + dz * dz;

        // ターゲットLODの決定 (4段階)
        let target_lod = if dist_sq <= 6*6 { 1 }
            else if dist_sq <= 12*12 { 2 }
            else if dist_sq <= 17*17 { 4 }
            else { 8 };

        let needs_load = if let Some(&(_, _, current_lod)) = chunk_manager.読込済み.get(&pos) {
            current_lod != target_lod
        } else {
            true
        };

        if needs_load {
            tasks_to_spawn.push((pos, dist_sq, target_lod));
        }
    }

    // 距離が近い順にソート
    tasks_to_spawn.sort_by_key(|t| t.1);

    // スポーンの制限 (1フレームに最大2つまで)
    let pool = AsyncComputeTaskPool::get();
    let spawn_limit = 2;
    for (pos, _, target_lod) in tasks_to_spawn.into_iter().take(spawn_limit) {
        chunk_manager.読込中.insert(pos);

        let task = pool.spawn(async move {
            let chunk = チャンク::丘陵地形生成(pos.0, pos.1, pos.2);
            let mesh = メッシュ生成::メッシュ生成(&chunk, target_lod);
            Some((pos, chunk, mesh, target_lod))
        });

        commands.spawn(チャンクタスク { 位置: pos, タスク: task });
    }
}

// 非同期タスクの結果を回収して描画Entityを生成するシステム
pub fn チャンクタスク処理(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut チャンクタスク)>,
    mut chunk_manager: ResMut<チャンク管理者>,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_material: Res<ボクセル素材>,
) {
    for (task_entity, mut chunk_task) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut chunk_task.タスク)) {
            // タスク管理用のEntityを削除
            commands.entity(task_entity).despawn();

            if let Some((pos, chunk, mesh, lod)) = result {
                // 既にロード対象外なら無視
                if !chunk_manager.読込中.contains(&pos) {
                    continue;
                }

                // シームレスな差し替え: 古いEntityがあれば削除
                if let Some((old_entity, _, _)) = chunk_manager.読込済み.get(&pos) {
                    commands.entity(*old_entity).despawn_recursive();
                }

                let render_entity = commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(voxel_material.0.clone()),
                    Transform::from_xyz(
                        pos.0 as f32 * チャンクのワールドサイズ,
                        pos.1 as f32 * チャンクのワールドサイズ,
                        pos.2 as f32 * チャンクのワールドサイズ,
                    ),
                    チャンク描画データ { cx: pos.0, cy: pos.1, cz: pos.2, lod },
                )).id();

                chunk_manager.読込済み.insert(pos, (render_entity, chunk, lod));
                chunk_manager.読込中.remove(&pos);

                // 1フレームに1つだけ処理する (パフォーマンス最適化)
                break;
            }
        }
    }
}

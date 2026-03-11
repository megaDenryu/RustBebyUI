use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::voxel_world::{Chunk, 描画距離, チャンクのワールドサイズ};
use crate::meshing;
use crate::camera_controller::UnityCamera;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

#[derive(Resource)]
pub struct ChunkManager {
    pub loaded_chunks: HashMap<(i32, i32, i32), (Entity, Chunk, u32)>, // Entity, Domain, CurrentLOD
    pub loading_chunks: HashSet<(i32, i32, i32)>, 
}

#[derive(Resource)]
pub struct VoxelMaterial(pub Handle<StandardMaterial>);

#[derive(Component)]
pub struct ChunkRenderData {
    pub cx: i32,
    pub cy: i32,
    pub cz: i32,
    pub lod: u32,
}

#[derive(Component)]
pub struct ChunkTask {
    pub pos: (i32, i32, i32),
    pub task: Task<Option<((i32, i32, i32), Chunk, Mesh, u32)>>,
}

pub fn manage_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<UnityCamera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut last_pos: Local<Option<Vec3>>, // 前回の位置を記録
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

    chunk_manager.loaded_chunks.retain(|&pos, &mut (entity, _, _)| {
        if !needed_chunks.contains(&pos) {
            commands.entity(entity).despawn_recursive();
            return false;
        }
        true
    });

    // 進行中フラグのクリーンアップ
    chunk_manager.loading_chunks.retain(|pos| needed_chunks.contains(pos));

    // 2. 必要なタスクの優先順位付けと収集
    let mut tasks_to_spawn = Vec::new();
    for &pos in &needed_chunks {
        if chunk_manager.loading_chunks.contains(&pos) { continue; }

        let dx = pos.0 - center_cx;
        let dz = pos.2 - center_cz;
        let dist_sq = dx * dx + dz * dz;
        
        // ターゲットLODの決定 (3段階)
        // 8チャンクまでLOD1（高品質）, 12チャンクまでLOD2, 以降LOD4
        // 霧が十分に濃くなる位置（約8チャンク）でLODを切り替える
        let target_lod = if dist_sq <= 8*8 { 1 } else if dist_sq <= 12*12 { 2 } else { 4 };

        let needs_load = if let Some(&(_, _, current_lod)) = chunk_manager.loaded_chunks.get(&pos) {
            current_lod != target_lod // LOD更新が必要
        } else {
            true // 新規ロード
        };

        if needs_load {
            tasks_to_spawn.push((pos, dist_sq, target_lod));
        }
    }

    // 距離が近い順にソート
    tasks_to_spawn.sort_by_key(|t| t.1);

    // 3. スポーンの制限 (1フレームに最大1つまで: スムーズさを優先)
    let pool = AsyncComputeTaskPool::get();
    let spawn_limit = 1;
    for (pos, _, target_lod) in tasks_to_spawn.into_iter().take(spawn_limit) {
        chunk_manager.loading_chunks.insert(pos);
        
        let task = pool.spawn(async move {
            let chunk = Chunk::new_hilly_terrain(pos.0, pos.1, pos.2);
            let mesh = meshing::generate_naive_mesh(&chunk, target_lod);
            Some((pos, chunk, mesh, target_lod))
        });
        
        commands.spawn(ChunkTask { pos, task });
    }
}

// 非同期タスクの結果を回収して描画Entityを生成するシステム
pub fn process_chunk_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkTask)>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_material: Res<VoxelMaterial>,
) {
    for (task_entity, mut chunk_task) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut chunk_task.task)) {
            // タスク管理用のEntityを削除
            commands.entity(task_entity).despawn();

            if let Some((pos, chunk, mesh, lod)) = result {
                // 既にロード対象外なら無視
                if !chunk_manager.loading_chunks.contains(&pos) {
                    continue;
                }

                // シームレスな差し替え: 古いEntityがあれば削除
                if let Some((old_entity, _, _)) = chunk_manager.loaded_chunks.get(&pos) {
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
                    ChunkRenderData { cx: pos.0, cy: pos.1, cz: pos.2, lod },
                )).id();

                chunk_manager.loaded_chunks.insert(pos, (render_entity, chunk, lod));
                chunk_manager.loading_chunks.remove(&pos);

                // **CRITICAL FOR PERFORMANCE**: 1フレームに1つだけ処理する
                // 複数のメッシュを一度に登録(AssetServer::add)するとメインスレッドが止まるため
                break;
            }
        }
    }
}

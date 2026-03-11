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

#[derive(Component)]
pub struct ChunkRenderData {
    pub cx: i32,
    pub cy: i32,
    pub cz: i32,
    pub lod: u32,
}

#[derive(Component)]
pub struct ChunkTask(pub Task<Option<((i32, i32, i32), Chunk, Mesh, u32)>>);

pub fn manage_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<UnityCamera>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return };
    
    let cam_p = camera_transform.translation;
    let center_cx = (cam_p.x / チャンクのワールドサイズ).floor() as i32;
    let center_cy = 0; 
    let center_cz = (cam_p.z / チャンクのワールドサイズ).floor() as i32;

    let mut needed_chunks = HashSet::new();
    for dx in -描画距離..=描画距離 {
        for dz in -描画距離..=描画距離 {
            if dx * dx + dz * dz > 描画距離 * 描画距離 { continue; }
            needed_chunks.insert((center_cx + dx, center_cy, center_cz + dz));
        }
    }

    // アンロード & LOD変更チェック
    chunk_manager.loaded_chunks.retain(|&pos, &mut (entity, _, current_lod)| {
        if !needed_chunks.contains(&pos) {
            commands.entity(entity).despawn_recursive();
            return false;
        }

        // LODが適切かチェック
        let dx = pos.0 - center_cx;
        let dz = pos.2 - center_cz;
        let dist_sq = dx * dx + dz * dz;
        let target_lod = if dist_sq <= 4*4 { 1 } else if dist_sq <= 8*8 { 2 } else { 4 };

        if target_lod != current_lod {
            // LODが変わったので一旦削除して再生成
            commands.entity(entity).despawn_recursive();
            return false;
        }

        true
    });

    // 進行中フラグのクリーンアップ
    chunk_manager.loading_chunks.retain(|pos| needed_chunks.contains(pos));

    let pool = AsyncComputeTaskPool::get();

    for &pos in &needed_chunks {
        if !chunk_manager.loaded_chunks.contains_key(&pos) && !chunk_manager.loading_chunks.contains(&pos) {
            chunk_manager.loading_chunks.insert(pos);
            
            let dx = pos.0 - center_cx;
            let dz = pos.2 - center_cz;
            let dist_sq = dx * dx + dz * dz;
            let target_lod = if dist_sq <= 4*4 { 1 } else if dist_sq <= 8*8 { 2 } else { 4 };

            // バックグラウンドで地形生成とメッシュ化を実行
            let task = pool.spawn(async move {
                let chunk = Chunk::new_hilly_terrain(pos.0, pos.1, pos.2);
                let mesh = meshing::generate_naive_mesh(&chunk, target_lod);
                Some((pos, chunk, mesh, target_lod))
            });
            
            commands.spawn(ChunkTask(task));
        }
    }
}

// 非同期タスクの結果を回収して描画Entityを生成するシステム
pub fn process_chunk_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkTask)>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: false, 
        perceptual_roughness: 0.9,
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    for (entity, mut task) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity).despawn();

            if let Some((pos, chunk, mesh, lod)) = result {
                if chunk_manager.loaded_chunks.contains_key(&pos) || !chunk_manager.loading_chunks.contains(&pos) {
                    chunk_manager.loading_chunks.remove(&pos);
                    continue;
                }

                let render_entity = commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(
                        pos.0 as f32 * チャンクのワールドサイズ,
                        pos.1 as f32 * チャンクのワールドサイズ,
                        pos.2 as f32 * チャンクのワールドサイズ,
                    ),
                    ChunkRenderData { cx: pos.0, cy: pos.1, cz: pos.2, lod },
                )).id();

                chunk_manager.loaded_chunks.insert(pos, (render_entity, chunk, lod));
                chunk_manager.loading_chunks.remove(&pos);
            }
        }
    }
}

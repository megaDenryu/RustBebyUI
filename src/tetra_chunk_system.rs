// src/tetra_chunk_system.rs
// 四面体ワールドのチャンク管理システム

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::voxel_world::{描画距離, チャンクのワールドサイズ};
use crate::tetra_world::TetraChunk;
use crate::tetra_meshing;
use crate::camera_controller::UnityCamera;
use crate::chunk_system::VoxelMaterial;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

#[derive(Resource)]
pub struct TetraChunkManager {
    pub loaded_chunks: HashMap<(i32, i32, i32), (Entity, u32)>,
    pub loading_chunks: HashSet<(i32, i32, i32)>,
}

#[derive(Component)]
pub struct TetraChunkTask {
    pub pos: (i32, i32, i32),
    pub task: Task<Option<((i32, i32, i32), Mesh, u32)>>,
}

#[derive(Component)]
pub struct TetraChunkMarker;

pub fn manage_tetra_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<UnityCamera>>,
    mut manager: ResMut<TetraChunkManager>,
    mut last_pos: Local<Option<Vec3>>,
) {
    let Ok(cam_transform) = camera_query.get_single() else { return };
    let cam_p = cam_transform.translation;

    if let Some(lp) = *last_pos {
        if cam_p.distance(lp) < 0.5 { return; }
    }
    *last_pos = Some(cam_p);

    let center_cx = (cam_p.x / チャンクのワールドサイズ).floor() as i32;
    let center_cz = (cam_p.z / チャンクのワールドサイズ).floor() as i32;

    let mut needed = HashSet::new();
    for dx in -描画距離..=描画距離 {
        for dz in -描画距離..=描画距離 {
            if dx * dx + dz * dz > 描画距離 * 描画距離 { continue; }
            needed.insert((center_cx + dx, 0, center_cz + dz));
        }
    }

    // アンロード
    manager.loaded_chunks.retain(|&pos, &mut (entity, _)| {
        if !needed.contains(&pos) {
            commands.entity(entity).despawn_recursive();
            return false;
        }
        true
    });
    manager.loading_chunks.retain(|pos| needed.contains(pos));

    // タスク生成
    let mut tasks = Vec::new();
    for &pos in &needed {
        if manager.loading_chunks.contains(&pos) || manager.loaded_chunks.contains_key(&pos) { continue; }
        let dx = pos.0 - center_cx;
        let dz = pos.2 - center_cz;
        let dist_sq = dx * dx + dz * dz;
        let lod = if dist_sq <= 6*6 { 1 } else if dist_sq <= 12*12 { 2 } else { 4 };
        tasks.push((pos, dist_sq, lod));
    }
    tasks.sort_by_key(|t| t.1);

    let pool = AsyncComputeTaskPool::get();
    for (pos, _, lod) in tasks.into_iter().take(2) {
        manager.loading_chunks.insert(pos);
        let task = pool.spawn(async move {
            let chunk = TetraChunk::new_hilly_terrain(pos.0, pos.1, pos.2);
            let mesh = tetra_meshing::generate_tetra_mesh(&chunk, lod);
            Some((pos, mesh, lod))
        });
        commands.spawn(TetraChunkTask { pos, task });
    }
}

pub fn process_tetra_chunk_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut TetraChunkTask)>,
    mut manager: ResMut<TetraChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_material: Res<VoxelMaterial>,
) {
    for (task_entity, mut chunk_task) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut chunk_task.task)) {
            commands.entity(task_entity).despawn();
            if let Some((pos, mesh, lod)) = result {
                if !manager.loading_chunks.contains(&pos) { continue; }

                if let Some((old_entity, _)) = manager.loaded_chunks.get(&pos) {
                    commands.entity(*old_entity).despawn_recursive();
                }

                let entity = commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(voxel_material.0.clone()),
                    Transform::from_xyz(
                        pos.0 as f32 * チャンクのワールドサイズ,
                        pos.1 as f32 * チャンクのワールドサイズ,
                        pos.2 as f32 * チャンクのワールドサイズ,
                    ),
                    TetraChunkMarker,
                )).id();

                manager.loaded_chunks.insert(pos, (entity, lod));
                manager.loading_chunks.remove(&pos);
                break;
            }
        }
    }
}

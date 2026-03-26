// src/四面体チャンク管理.rs
// 四面体ワールドのチャンク管理システム

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::ボクセル世界::{描画距離, チャンクのワールドサイズ};
use crate::四面体世界::四面体チャンク;
use crate::四面体メッシュ生成;
use crate::カメラ制御::カメラ操作;
use crate::チャンク管理::ボクセル素材;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

#[derive(Resource)]
pub struct 四面体チャンク管理者 {
    pub 読込済み: HashMap<(i32, i32, i32), (Entity, u32)>,
    pub 読込中: HashSet<(i32, i32, i32)>,
}

#[derive(Component)]
pub struct 四面体チャンクタスク {
    pub 位置: (i32, i32, i32),
    pub タスク: Task<Option<((i32, i32, i32), Mesh, u32)>>,
}

#[derive(Component)]
pub struct 四面体チャンクマーカー;

pub fn 四面体チャンク管理処理(
    mut commands: Commands,
    camera_query: Query<&Transform, With<カメラ操作>>,
    mut manager: ResMut<四面体チャンク管理者>,
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
    manager.読込済み.retain(|&pos, &mut (entity, _)| {
        if !needed.contains(&pos) {
            commands.entity(entity).despawn_recursive();
            return false;
        }
        true
    });
    manager.読込中.retain(|pos| needed.contains(pos));

    // タスク生成
    let mut tasks = Vec::new();
    for &pos in &needed {
        if manager.読込中.contains(&pos) || manager.読込済み.contains_key(&pos) { continue; }
        let dx = pos.0 - center_cx;
        let dz = pos.2 - center_cz;
        let dist_sq = dx * dx + dz * dz;
        let lod = if dist_sq <= 6*6 { 1 } else if dist_sq <= 12*12 { 2 } else { 4 };
        tasks.push((pos, dist_sq, lod));
    }
    tasks.sort_by_key(|t| t.1);

    let pool = AsyncComputeTaskPool::get();
    for (pos, _, lod) in tasks.into_iter().take(2) {
        manager.読込中.insert(pos);
        let task = pool.spawn(async move {
            let chunk = 四面体チャンク::丘陵地形生成(pos.0, pos.1, pos.2);
            let mesh = 四面体メッシュ生成::四面体メッシュ生成(pos, &chunk, lod);
            Some((pos, mesh, lod))
        });
        commands.spawn(四面体チャンクタスク { 位置: pos, タスク: task });
    }
}

pub fn 四面体チャンクタスク処理(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut 四面体チャンクタスク)>,
    mut manager: ResMut<四面体チャンク管理者>,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_material: Res<ボクセル素材>,
) {
    for (task_entity, mut chunk_task) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut chunk_task.タスク)) {
            commands.entity(task_entity).despawn();
            if let Some((pos, mesh, lod)) = result {
                if !manager.読込中.contains(&pos) { continue; }

                if let Some((old_entity, _)) = manager.読込済み.get(&pos) {
                    commands.entity(*old_entity).despawn_recursive();
                }

                let entity = commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(voxel_material.0.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.0), // メッシュ自体がワールド座標で生成されている
                    四面体チャンクマーカー,
                )).id();

                manager.読込済み.insert(pos, (entity, lod));
                manager.読込中.remove(&pos);
                break;
            }
        }
    }
}

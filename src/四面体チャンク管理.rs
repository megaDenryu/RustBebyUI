// src/四面体チャンク管理.rs

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::ボクセル世界::{描画距離, チャンクのワールドサイズ};
use crate::四面体メッシュ生成;
use crate::地形生成::{self};
use crate::カメラ制御::カメラ操作;
use crate::チャンク管理::ボクセル素材;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

// LOD閾値 (チャンク管理と同じ値)
const LOD1距離2乗: i32 = 6 * 6;
const LOD2距離2乗: i32 = 12 * 12;
const 毎フレーム最大タスク数: usize = 16;
const チャンク更新移動閾値: f32 = 0.5;

fn LOD決定(距離2乗: i32) -> u32 {
    if 距離2乗 <= LOD1距離2乗 { 1 }
    else if 距離2乗 <= LOD2距離2乗 { 2 }
    else { 4 }
}

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
        if cam_p.distance(lp) < チャンク更新移動閾値 { return; }
    }
    *last_pos = Some(cam_p);

    let center_cx = (cam_p.x / チャンクのワールドサイズ).floor() as i32;
    let center_cy = (cam_p.y / チャンクのワールドサイズ).floor() as i32;
    let center_cz = (cam_p.z / チャンクのワールドサイズ).floor() as i32;

    let needed = 必要チャンク集合(center_cx, center_cy, center_cz);

    manager.読込済み.retain(|&pos, &mut (entity, _)| {
        if !needed.contains(&pos) {
            commands.entity(entity).despawn_recursive();
            return false;
        }
        true
    });
    manager.読込中.retain(|pos| needed.contains(pos));

    let mut tasks: Vec<_> = needed.iter()
        .filter(|pos| !manager.読込中.contains(pos) && !manager.読込済み.contains_key(pos))
        .map(|&pos| {
            let dist_sq = (pos.0 - center_cx).pow(2) + (pos.2 - center_cz).pow(2);
            (pos, dist_sq, LOD決定(dist_sq))
        })
        .collect();

    tasks.sort_by_key(|t| t.1);

    let pool = AsyncComputeTaskPool::get();
    for (pos, _, lod) in tasks.into_iter().take(毎フレーム最大タスク数) {
        manager.読込中.insert(pos);
        let task = pool.spawn(async move {
            let chunk = 地形生成::四面体チャンク地形生成(pos.0, pos.1, pos.2);
            let mesh = 四面体メッシュ生成::四面体メッシュ生成(pos, &chunk, lod);
            Some((pos, mesh, lod))
        });
        commands.spawn(四面体チャンクタスク { 位置: pos, タスク: task });
    }
}

const メッシュ登録上限: usize = 8;

pub fn 四面体チャンクタスク処理(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut 四面体チャンクタスク)>,
    mut manager: ResMut<四面体チャンク管理者>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ボクセル素材>,
) {
    let mut 登録数 = 0;
    for (task_entity, mut chunk_task) in &mut tasks {
        if 登録数 >= メッシュ登録上限 { break; }
        if let Some(result) = future::block_on(future::poll_once(&mut chunk_task.タスク)) {
            commands.entity(task_entity).despawn();
            if let Some((pos, mesh, lod)) = result {
                if !manager.読込中.contains(&pos) { continue; }

                if let Some((old_entity, _)) = manager.読込済み.get(&pos) {
                    commands.entity(*old_entity).despawn_recursive();
                }

                // 空メッシュスキップ
                let has_vertices = mesh.count_vertices() > 0;
                if has_vertices {
                    let entity = commands.spawn((
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(material.0.clone()),
                        Transform::IDENTITY,
                        四面体チャンクマーカー,
                    )).id();
                    manager.読込済み.insert(pos, (entity, lod));
                } else {
                    let entity = commands.spawn_empty().id();
                    manager.読込済み.insert(pos, (entity, lod));
                }

                manager.読込中.remove(&pos);
                登録数 += 1;
            }
        }
    }
}

fn 必要チャンク集合(center_cx: i32, _center_cy: i32, center_cz: i32) -> HashSet<(i32, i32, i32)> {
    let mut set = HashSet::new();
    for dx in -描画距離..=描画距離 {
        for dz in -描画距離..=描画距離 {
            if dx * dx + dz * dz > 描画距離 * 描画距離 { continue; }

            let cx = center_cx + dx;
            let cz = center_cz + dz;

            let wx = cx as f32 * チャンクのワールドサイズ;
            let wz = cz as f32 * チャンクのワールドサイズ;
            let half = チャンクのワールドサイズ * 0.5;
            let mut max_h: f32 = -100.0;
            let mut min_h: f32 = 100.0;
            for &sx in &[wx, wx + half, wx + チャンクのワールドサイズ] {
                for &sz in &[wz, wz + half, wz + チャンクのワールドサイズ] {
                    let h = 地形生成::地形高さ(sx, sz);
                    max_h = max_h.max(h);
                    min_h = min_h.min(h);
                }
            }

            let cy_surface_min = (min_h / チャンクのワールドサイズ).floor() as i32;
            let cy_surface_max = (max_h / チャンクのワールドサイズ).floor() as i32;
            let cy_bottom = (cy_surface_min - 1).max(-1);
            let cy_top = cy_surface_max + 1;

            for cy in cy_bottom..=cy_top {
                set.insert((cx, cy, cz));
            }
        }
    }
    set
}

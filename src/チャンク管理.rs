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
const 毎フレーム最大タスク数: usize = 16;
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

#[derive(Resource)]
pub struct 水素材(pub Handle<StandardMaterial>);

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
    pub タスク: Task<Option<((i32, i32, i32), チャンク, メッシュ生成::メッシュ生成結果, u32)>>,
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
    let center_cy = (cam_p.y / チャンクのワールドサイズ).floor() as i32;
    let center_cz = (cam_p.z / チャンクのワールドサイズ).floor() as i32;

    let needed = 必要チャンク集合(center_cx, center_cy, center_cz);

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
            let result = メッシュ生成::メッシュ生成(&chunk, target_lod);
            Some((pos, chunk, result, target_lod))
        });
        commands.spawn(チャンクタスク { 位置: pos, タスク: task });
    }
}

const メッシュ登録上限: usize = 8;

pub fn チャンクタスク処理(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut チャンクタスク)>,
    mut manager: ResMut<チャンク管理者>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ボクセル素材>,
    water_material: Res<水素材>,
) {
    let mut 登録数 = 0;
    for (task_entity, mut chunk_task) in &mut tasks {
        if 登録数 >= メッシュ登録上限 { break; }
        if let Some(result) = future::block_on(future::poll_once(&mut chunk_task.タスク)) {
            commands.entity(task_entity).despawn();

            if let Some((pos, chunk, mesh_result, lod)) = result {
                if !manager.読込中.contains(&pos) { continue; }

                if let Some((old_entity, _, _)) = manager.読込済み.get(&pos) {
                    commands.entity(*old_entity).despawn_recursive();
                }

                let chunk_transform = Transform::from_xyz(
                    pos.0 as f32 * チャンクのワールドサイズ,
                    pos.1 as f32 * チャンクのワールドサイズ,
                    pos.2 as f32 * チャンクのワールドサイズ,
                );

                let has_solid = mesh_result.固体.count_vertices() > 0;
                let has_water = mesh_result.水.count_vertices() > 0;

                if has_solid || has_water {
                    let entity = if has_solid {
                        commands.spawn((
                            Mesh3d(meshes.add(mesh_result.固体)),
                            MeshMaterial3d(material.0.clone()),
                            chunk_transform,
                            チャンク描画データ { cx: pos.0, cy: pos.1, cz: pos.2, lod },
                        )).id()
                    } else {
                        commands.spawn_empty().id()
                    };

                    // 水メッシュは別Entityとして子に追加
                    if has_water {
                        commands.spawn((
                            Mesh3d(meshes.add(mesh_result.水)),
                            MeshMaterial3d(water_material.0.clone()),
                            chunk_transform,
                            チャンク描画データ { cx: pos.0, cy: pos.1, cz: pos.2, lod },
                        ));
                    }

                    manager.読込済み.insert(pos, (entity, chunk, lod));
                } else {
                    let entity = commands.spawn_empty().id();
                    manager.読込済み.insert(pos, (entity, chunk, lod));
                }

                manager.読込中.remove(&pos);
                登録数 += 1;
            }
        }
    }
}

// =============================================================================
// ユーティリティ
// =============================================================================

/// 各XZ位置で地形高さをサンプリングし、地表面を含むY層のみ生成する。
/// 「チャンクのY範囲が地形高さを跨ぐ」場合にのみそのY層を含める。
/// 加えて、地表チャンクの上下1層ずつも含める(隣接面カリング用)。
fn 必要チャンク集合(center_cx: i32, _center_cy: i32, center_cz: i32) -> HashSet<(i32, i32, i32)> {
    let mut set = HashSet::new();
    for dx in -描画距離..=描画距離 {
        for dz in -描画距離..=描画距離 {
            if dx * dx + dz * dz > 描画距離 * 描画距離 { continue; }

            let cx = center_cx + dx;
            let cz = center_cz + dz;

            // チャンク内の9点で地形高さをサンプリング
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

            // 地表面を含むY層: min_h のチャンクから max_h のチャンクまで
            let cy_surface_min = (min_h / チャンクのワールドサイズ).floor() as i32;
            let cy_surface_max = (max_h / チャンクのワールドサイズ).floor() as i32;

            // 地表チャンク + 上下1層(隣接面カリング用)
            let cy_bottom = (cy_surface_min - 1).max(-1);
            let cy_top = cy_surface_max + 1;

            for cy in cy_bottom..=cy_top {
                set.insert((cx, cy, cz));
            }
        }
    }
    set
}

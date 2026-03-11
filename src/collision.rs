// src/collision.rs
use bevy::prelude::*;
use crate::voxel_world::{チャンク解像度, ボクセルスケール, チャンクのワールドサイズ};
use crate::chunk_system::ChunkManager;

pub fn is_colliding(
    pos: Vec3,
    chunk_manager: &ChunkManager,
) -> bool {
    let check_offsets = [
        Vec3::ZERO,
        Vec3::new(0.15, 0.0, 0.15),
        Vec3::new(-0.15, 0.0, 0.15),
        Vec3::new(0.15, 0.0, -0.15),
        Vec3::new(-0.15, 0.0, -0.15),
        Vec3::new(0.0, 0.3, 0.0),  // 頭
        Vec3::new(0.0, -0.3, 0.0), // 足元
    ];

    for offset in check_offsets {
        let p = pos + offset;
        let cx = (p.x / チャンクのワールドサイズ).floor() as i32;
        let cy = 0; // 地面レイヤーのみ管理
        let cz = (p.z / チャンクのワールドサイズ).floor() as i32;

        if let Some((_entity, chunk, _lod)) = chunk_manager.loaded_chunks.get(&(cx, cy, cz)) {
            let lx = ((p.x - cx as f32 * チャンクのワールドサイズ) / ボクセルスケール).floor() as i32;
            let ly = ((p.y - cy as f32 * チャンクのワールドサイズ) / ボクセルスケール).floor() as i32;
            let lz = ((p.z - cz as f32 * チャンクのワールドサイズ) / ボクセルスケール).floor() as i32;

            if lx >= 0 && lx < チャンク解像度 as i32 && ly >= 0 && ly < チャンク解像度 as i32 && lz >= 0 && lz < チャンク解像度 as i32 {
                let voxel = chunk.get(lx as usize, ly as usize, lz as usize);
                if !voxel.は空気か() && !voxel.は水か() {
                    return true;
                }
            }
        }
    }

    false
}

// src/衝突判定.rs
use bevy::prelude::*;
use crate::ボクセル世界::{チャンク解像度, ボクセルスケール, チャンクのワールドサイズ};
use crate::チャンク管理::チャンク管理者;

// プレイヤー当たり判定のチェックポイント
const 衝突チェック点: [Vec3; 7] = [
    Vec3::ZERO,
    Vec3::new( 0.15, 0.0,  0.15),
    Vec3::new(-0.15, 0.0,  0.15),
    Vec3::new( 0.15, 0.0, -0.15),
    Vec3::new(-0.15, 0.0, -0.15),
    Vec3::new(0.0,  0.3, 0.0),  // 頭
    Vec3::new(0.0, -0.3, 0.0),  // 足元
];

pub fn 衝突判定処理(pos: Vec3, chunk_manager: &チャンク管理者) -> bool {
    for &offset in &衝突チェック点 {
        let p = pos + offset;
        let cx = (p.x / チャンクのワールドサイズ).floor() as i32;
        let cy = (p.y / チャンクのワールドサイズ).floor() as i32;
        let cz = (p.z / チャンクのワールドサイズ).floor() as i32;

        if let Some((_entity, chunk, _lod)) = chunk_manager.読込済み.get(&(cx, cy, cz)) {
            let lx = ((p.x - cx as f32 * チャンクのワールドサイズ) / ボクセルスケール).floor() as i32;
            let ly = ((p.y - cy as f32 * チャンクのワールドサイズ) / ボクセルスケール).floor() as i32;
            let lz = ((p.z - cz as f32 * チャンクのワールドサイズ) / ボクセルスケール).floor() as i32;

            if lx >= 0 && lx < チャンク解像度 as i32
                && ly >= 0 && ly < チャンク解像度 as i32
                && lz >= 0 && lz < チャンク解像度 as i32
            {
                let voxel = chunk.取得(lx as usize, ly as usize, lz as usize);
                if !voxel.は透過か() {
                    return true;
                }
            }
        }
    }
    false
}

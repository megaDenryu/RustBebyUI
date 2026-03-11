// src/tetra_world.rs
// Layer 1: 四面体ボクセルのドメインロジック
// 各立方体セルを5つの四面体に分割し、空間を完全充填する。

use crate::voxel_world::{ボクセル, ボクセル種別, チャンク解像度, ボクセルスケール, チャンクのワールドサイズ, 水面高さ, 岩盤の高さ};

/// 四面体チャンク: 各セルが5つの四面体を持つ
#[derive(Clone)]
pub struct TetraChunk {
    /// 各セル(x,y,z)に5つの四面体の種別を格納
    /// index = (x + y*S + z*S*S) * 5 + tetra_id (0..5)
    pub tetras: Vec<ボクセル種別>,
}

impl TetraChunk {
    pub fn new_empty() -> Self {
        Self {
            tetras: vec![ボクセル種別::空気; チャンク解像度 * チャンク解像度 * チャンク解像度 * 5],
        }
    }

    fn idx(x: usize, y: usize, z: usize, t: usize) -> usize {
        (x + y * チャンク解像度 + z * チャンク解像度 * チャンク解像度) * 5 + t
    }

    pub fn get(&self, x: usize, y: usize, z: usize, t: usize) -> ボクセル種別 {
        if x < チャンク解像度 && y < チャンク解像度 && z < チャンク解像度 && t < 5 {
            self.tetras[Self::idx(x, y, z, t)]
        } else {
            ボクセル種別::空気
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, t: usize, kind: ボクセル種別) {
        if x < チャンク解像度 && y < チャンク解像度 && z < チャンク解像度 && t < 5 {
            let idx = Self::idx(x, y, z, t);
            self.tetras[idx] = kind;
        }
    }

    /// 地形生成: 立方体ワールドと同じノイズ関数を使い、各四面体の種別を決定
    pub fn new_hilly_terrain(cx: i32, cy: i32, cz: i32) -> Self {
        let mut chunk = Self::new_empty();
        let wo_x = cx as f32 * チャンクのワールドサイズ;
        let wo_y = cy as f32 * チャンクのワールドサイズ;
        let wo_z = cz as f32 * チャンクのワールドサイズ;

        for x in 0..チャンク解像度 {
            for z in 0..チャンク解像度 {
                let gx = wo_x + (x as f32 * ボクセルスケール);
                let gz = wo_z + (z as f32 * ボクセルスケール);

                let h_base = (gx * 0.08).sin() * 5.0 + (gz * 0.08).cos() * 5.0;
                let h_hills = (gx * 0.2).sin() * (gz * 0.15).cos() * 3.0;
                let h_detail = (gx * 0.4).sin() * 1.5 + (gz * 0.35).cos() * 1.5;
                let h_micro = (gx * 0.8).sin() * 0.5 + (gz * 0.7).cos() * 0.5;
                let mountain = ((gx * 0.03).sin() * (gz * 0.04).cos()).abs() * 12.0;
                let ground_y = 6.0 + h_base + h_hills + h_detail + h_micro + mountain;

                for y in 0..チャンク解像度 {
                    let gy = wo_y + (y as f32 * ボクセルスケール);

                    // 各四面体の中心位置に基づいて種別を決定
                    // 5つの四面体はセル内の異なる位置にある
                    for t in 0..5 {
                        let kind = determine_tetra_type(gx, gy, gz, t, ground_y);
                        chunk.set(x, y, z, t, kind);
                    }
                }
            }
        }
        chunk
    }
}

/// 四面体の種別を地形情報から決定
fn determine_tetra_type(gx: f32, gy: f32, gz: f32, _t: usize, ground_y: f32) -> ボクセル種別 {
    // 岩盤
    if gy <= 岩盤の高さ { return ボクセル種別::岩盤; }

    // 地上
    if gy > ground_y {
        if gy <= 水面高さ { return ボクセル種別::水; }
        return ボクセル種別::空気;
    }

    // 洞窟
    let cave = (gx * 0.35).sin() * (gy * 0.4).cos() * (gz * 0.35).sin();
    let tunnel_h = ((gx * 0.08).cos() * (gz * 0.08).sin()).abs();
    let tunnel_y = 5.0 + (gx * 0.05).sin() * 2.0;
    let is_tunnel = tunnel_h < 0.15 && (gy - tunnel_y).abs() < 1.5 && gy < ground_y - 1.0;

    if cave > 0.55 || is_tunnel {
        if gy <= 水面高さ { return ボクセル種別::水; }
        return ボクセル種別::空気;
    }

    // 地層
    let depth = ground_y - gy;
    if depth < 0.3 { ボクセル種別::草 }
    else if depth < 1.5 { ボクセル種別::土 }
    else { ボクセル種別::石 }
}

/// 立方体セルを5つの四面体に分割する頂点定義
/// 各四面体は4つの頂点インデックスで定義 (0-7はセルの8頂点)
/// セルの頂点:
///   0=(0,0,0) 1=(1,0,0) 2=(1,1,0) 3=(0,1,0)
///   4=(0,0,1) 5=(1,0,1) 6=(1,1,1) 7=(0,1,1)
pub const TETRA_VERTICES: [[usize; 4]; 5] = [
    [0, 1, 3, 4], // 底面-左前
    [1, 2, 3, 6], // 上面-右前
    [1, 4, 5, 6], // 底面-右奥
    [3, 4, 6, 7], // 上面-左奥
    [1, 3, 4, 6], // 中央
];

/// セル頂点の相対座標（0.0 or 1.0）
pub const CELL_CORNERS: [[f32; 3]; 8] = [
    [0.0, 0.0, 0.0], // 0
    [1.0, 0.0, 0.0], // 1
    [1.0, 1.0, 0.0], // 2
    [0.0, 1.0, 0.0], // 3
    [0.0, 0.0, 1.0], // 4
    [1.0, 0.0, 1.0], // 5
    [1.0, 1.0, 1.0], // 6
    [0.0, 1.0, 1.0], // 7
];

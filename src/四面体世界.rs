// src/四面体世界.rs
// レイヤー1: 四面体ボクセルのドメインロジック
// 各立方体セルを5つの四面体に分割し、空間を完全充填する。

use bevy::prelude::Vec3;
use crate::ボクセル世界::{ボクセル, ボクセル種別, チャンク解像度, ボクセルスケール, チャンクのワールドサイズ, 水面高さ, 岩盤の高さ};

/// 四面体チャンク: 各セルが5つの四面体を持つ
#[derive(Clone)]
pub struct 四面体チャンク {
    /// 各セル(x,y,z)に5つの四面体の種別を格納
    /// index = (x + y*S + z*S*S) * 5 + tetra_id (0..5)
    pub 四面体群: Vec<ボクセル種別>,
}

pub fn 歪み頂点取得(gx: i32, gy: i32, gz: i32) -> Vec3 {
    let base = Vec3::new(gx as f32, gy as f32, gz as f32) * ボクセルスケール;
    let nx = (gx as f32 * 0.312 + gy as f32 * 0.123).sin() * (gz as f32 * 0.221).cos();
    let ny = (gy as f32 * 0.281 + gz as f32 * 0.145).sin() * (gx as f32 * 0.252).cos();
    let nz = (gz as f32 * 0.354 + gx as f32 * 0.177).sin() * (gy as f32 * 0.288).cos();
    base + Vec3::new(nx, ny, nz) * ボクセルスケール * 0.5
}

impl 四面体チャンク {
    pub fn 空で生成() -> Self {
        Self {
            四面体群: vec![ボクセル種別::空気; チャンク解像度 * チャンク解像度 * チャンク解像度 * 5],
        }
    }

    fn 添字(x: usize, y: usize, z: usize, t: usize) -> usize {
        (x + y * チャンク解像度 + z * チャンク解像度 * チャンク解像度) * 5 + t
    }

    pub fn 取得(&self, x: usize, y: usize, z: usize, t: usize) -> ボクセル種別 {
        if x < チャンク解像度 && y < チャンク解像度 && z < チャンク解像度 && t < 5 {
            self.四面体群[Self::添字(x, y, z, t)]
        } else {
            ボクセル種別::空気
        }
    }

    pub fn 設定(&mut self, x: usize, y: usize, z: usize, t: usize, kind: ボクセル種別) {
        if x < チャンク解像度 && y < チャンク解像度 && z < チャンク解像度 && t < 5 {
            let idx = Self::添字(x, y, z, t);
            self.四面体群[idx] = kind;
        }
    }

    /// 地形生成: 立方体ワールドと同じノイズ関数を使い、各四面体の種別を決定
    pub fn 丘陵地形生成(cx: i32, cy: i32, cz: i32) -> Self {
        let mut chunk = Self::空で生成();

        let ofs_x = cx * チャンク解像度 as i32;
        let ofs_y = cy * チャンク解像度 as i32;
        let ofs_z = cz * チャンク解像度 as i32;

        for x in 0..チャンク解像度 {
            for z in 0..チャンク解像度 {
                for y in 0..チャンク解像度 {
                    let gx = ofs_x + x as i32;
                    let gy = ofs_y + y as i32;
                    let gz = ofs_z + z as i32;

                    let cell_verts = [
                        歪み頂点取得(gx,   gy,   gz),
                        歪み頂点取得(gx+1, gy,   gz),
                        歪み頂点取得(gx+1, gy+1, gz),
                        歪み頂点取得(gx,   gy+1, gz),
                        歪み頂点取得(gx,   gy,   gz+1),
                        歪み頂点取得(gx+1, gy,   gz+1),
                        歪み頂点取得(gx+1, gy+1, gz+1),
                        歪み頂点取得(gx,   gy+1, gz+1),
                    ];

                    let is_even = (x + y + z) % 2 == 0;
                    let tetra_defs = if is_even { 四面体頂点_偶数 } else { 四面体頂点_奇数 };

                    // 各四面体の中心位置に基づいて種別を決定
                    for t in 0..5 {
                        let tv = tetra_defs[t];
                        let mut center = bevy::math::Vec3::ZERO;
                        for &vi in &tv {
                            center += cell_verts[vi];
                        }
                        center /= 4.0;

                        let wx = center.x;
                        let wy = center.y;
                        let wz = center.z;

                        let h_base = (wx * 0.08).sin() * 5.0 + (wz * 0.08).cos() * 5.0;
                        let h_hills = (wx * 0.2).sin() * (wz * 0.15).cos() * 3.0;
                        let h_detail = (wx * 0.4).sin() * 1.5 + (wz * 0.35).cos() * 1.5;
                        let h_micro = (wx * 0.8).sin() * 0.5 + (wz * 0.7).cos() * 0.5;
                        let mountain = ((wx * 0.03).sin() * (wz * 0.04).cos()).abs() * 12.0;
                        let ground_y = 6.0 + h_base + h_hills + h_detail + h_micro + mountain;

                        let kind = 四面体種別判定(wx, wy, wz, ground_y);
                        chunk.設定(x, y, z, t, kind);
                    }
                }
            }
        }
        chunk
    }
}

/// 四面体の種別を地形情報から決定
fn 四面体種別判定(gx: f32, gy: f32, gz: f32, ground_y: f32) -> ボクセル種別 {
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
/// セルの頂点:
///   0=(0,0,0) 1=(1,0,0) 2=(1,1,0) 3=(0,1,0)
///   4=(0,0,1) 5=(1,0,1) 6=(1,1,1) 7=(0,1,1)

pub const 四面体頂点_偶数: [[usize; 4]; 5] = [
    [0, 1, 3, 4], // 0: 角 (0,0,0)
    [1, 2, 3, 6], // 1: 角 (1,1,0)
    [1, 4, 5, 6], // 2: 角 (1,0,1)
    [3, 4, 6, 7], // 3: 角 (0,1,1)
    [1, 3, 4, 6], // 4: 中央
];

pub const 四面体頂点_奇数: [[usize; 4]; 5] = [
    [0, 1, 2, 5], // 0: 角 (1,0,0)
    [0, 2, 3, 7], // 1: 角 (0,1,0)
    [0, 4, 5, 7], // 2: 角 (0,0,1)
    [2, 5, 6, 7], // 3: 角 (1,1,1)
    [0, 2, 5, 7], // 4: 中央
];

/// 隣接四面体テーブル
/// (dx, dy, dz, neighbor_t)
pub fn 隣接四面体取得(is_even: bool, t: usize, face: usize) -> (i32, i32, i32, usize) {
    if is_even {
        match t {
            0 => [(0,0,-1, 2), (0,-1,0, 1), (-1,0,0, 0), (0,0,0, 4)][face],
            1 => [(0,0,-1, 3), (1,0,0, 1), (0,1,0, 0), (0,0,0, 4)][face],
            2 => [(0,-1,0, 3), (1,0,0, 2), (0,0,1, 0), (0,0,0, 4)][face],
            3 => [(-1,0,0, 3), (0,1,0, 2), (0,0,1, 1), (0,0,0, 4)][face],
            4 => [(0,0,0, 0), (0,0,0, 1), (0,0,0, 2), (0,0,0, 3)][face],
            _ => (0,0,0,0),
        }
    } else {
        match t {
            0 => [(0,0,-1, 2), (0,-1,0, 1), (1,0,0, 0), (0,0,0, 4)][face],
            1 => [(0,0,-1, 3), (-1,0,0, 1), (0,1,0, 0), (0,0,0, 4)][face],
            2 => [(0,-1,0, 3), (-1,0,0, 2), (0,0,1, 0), (0,0,0, 4)][face],
            3 => [(1,0,0, 3), (0,1,0, 2), (0,0,1, 1), (0,0,0, 4)][face],
            4 => [(0,0,0, 0), (0,0,0, 1), (0,0,0, 2), (0,0,0, 3)][face],
            _ => (0,0,0,0),
        }
    }
}

/// セル頂点の相対座標（0.0 or 1.0）
pub const セル頂点座標: [[f32; 3]; 8] = [
    [0.0, 0.0, 0.0], // 0
    [1.0, 0.0, 0.0], // 1
    [1.0, 1.0, 0.0], // 2
    [0.0, 1.0, 0.0], // 3
    [0.0, 0.0, 1.0], // 4
    [1.0, 0.0, 1.0], // 5
    [1.0, 1.0, 1.0], // 6
    [0.0, 1.0, 1.0], // 7
];

// src/地形生成.rs
// 立方体・四面体で共有する地形ノイズと種別判定

use crate::ボクセル世界::*;
use crate::四面体世界::四面体チャンク;

// =============================================================================
// 地形ノイズ関数
// =============================================================================

/// ワールド座標 (gx, gz) における地表の高さを返す
pub fn 地形高さ(gx: f32, gz: f32) -> f32 {
    let h_base = (gx * 0.08).sin() * 5.0 + (gz * 0.08).cos() * 5.0;
    let h_hills = (gx * 0.2).sin() * (gz * 0.15).cos() * 3.0;
    let h_detail = (gx * 0.4).sin() * 1.5 + (gz * 0.35).cos() * 1.5;
    let h_micro = (gx * 0.8).sin() * 0.5 + (gz * 0.7).cos() * 0.5;
    let mountain = ((gx * 0.03).sin() * (gz * 0.04).cos()).abs() * 12.0;
    6.0 + h_base + h_hills + h_detail + h_micro + mountain
}

/// 洞窟・トンネル・縦穴の判定
fn 洞窟か(gx: f32, gy: f32, gz: f32, ground_y: f32) -> bool {
    // 大洞窟
    let cave_noise = (gx * 0.35).sin() * (gy * 0.4).cos() * (gz * 0.35).sin();
    // トンネル (2系統)
    let tunnel_h1 = ((gx * 0.08).cos() * (gz * 0.08).sin()).abs();
    let tunnel_h2 = ((gx * 0.12 + 1.0).sin() * (gz * 0.06).cos()).abs();
    let tunnel_y_center = 5.0 + (gx * 0.05).sin() * 2.0;
    let is_tunnel = (tunnel_h1 < 0.15 || tunnel_h2 < 0.12)
        && (gy - tunnel_y_center).abs() < 1.5
        && gy < ground_y - 1.0;
    // 小洞窟
    let cave_small = (gx * 0.5).sin() * (gy * 0.6).cos() * (gz * 0.5).sin()
        + (gx * 0.3 + 2.0).cos() * (gy * 0.3).sin() * (gz * 0.4 + 1.0).cos() * 0.5;
    // 縦穴
    let vertical_shaft = ((gx * 0.15).sin() * (gz * 0.15).cos()).abs();
    let is_shaft = vertical_shaft < 0.05 && gy < ground_y - 2.0 && gy > 岩盤の高さ + 0.5;

    cave_noise > 0.55 || is_tunnel || cave_small > 0.85 || is_shaft
}

/// ワールド座標からボクセル種別を決定する統一関数
pub fn ボクセル種別判定(gx: f32, gy: f32, gz: f32) -> ボクセル種別 {
    let ground_y = 地形高さ(gx, gz);

    // 岩盤層
    if gy <= 岩盤の高さ {
        return ボクセル種別::岩盤;
    }

    // 虹の道
    let rainbow_y = 25.0 + (gx * 0.05).sin() * 5.0;
    if (gy - rainbow_y).abs() < 0.2 && (gz - 10.0).abs() < 3.0 {
        return ボクセル種別::虹;
    }

    // 橋
    if (gz - 2.0).abs() < 1.0 && (gy - 6.0).abs() < 0.2 && (gx % 40.0).abs() < 20.0 {
        return ボクセル種別::橋;
    }

    // 地上（空気 or 水）
    if gy > ground_y {
        return if gy <= 水面高さ { ボクセル種別::水 } else { ボクセル種別::空気 };
    }

    // 洞窟
    if 洞窟か(gx, gy, gz, ground_y) {
        return if gy <= 水面高さ { ボクセル種別::水 } else { ボクセル種別::空気 };
    }

    // 地層
    let depth = ground_y - gy;
    let is_road = (gx - 5.0).abs() < 1.5 || (gz - 5.0).abs() < 1.5;
    if is_road && depth < 0.3 { ボクセル種別::道路 }
    else if depth < 0.3 { ボクセル種別::草 }
    else if depth < 1.5 { ボクセル種別::土 }
    else { ボクセル種別::石 }
}

// =============================================================================
// 立方体チャンク地形生成
// =============================================================================

pub fn チャンク地形生成(cx: i32, cy: i32, cz: i32) -> チャンク {
    let mut chunk = チャンク::空で生成();
    let wo_x = cx as f32 * チャンクのワールドサイズ;
    let wo_y = cy as f32 * チャンクのワールドサイズ;
    let wo_z = cz as f32 * チャンクのワールドサイズ;

    for x in 0..チャンク解像度 {
        for z in 0..チャンク解像度 {
            for y in 0..チャンク解像度 {
                let gx = wo_x + (x as f32 * ボクセルスケール);
                let gy = wo_y + (y as f32 * ボクセルスケール);
                let gz = wo_z + (z as f32 * ボクセルスケール);
                let 種別 = ボクセル種別判定(gx, gy, gz);
                chunk.設定(x, y, z, ボクセル { 種別 });
            }
        }
    }
    chunk
}

// =============================================================================
// 四面体チャンク地形生成
// =============================================================================

pub fn 四面体チャンク地形生成(cx: i32, cy: i32, cz: i32) -> 四面体チャンク {
    use crate::四面体世界::{四面体頂点_偶数, 四面体頂点_奇数, 歪み頂点取得};

    let mut chunk = 四面体チャンク::空で生成();
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

                for t in 0..5 {
                    // 四面体の4頂点の重心で種別を判定
                    let tv = tetra_defs[t];
                    let center = (cell_verts[tv[0]] + cell_verts[tv[1]]
                        + cell_verts[tv[2]] + cell_verts[tv[3]]) / 4.0;
                    let 種別 = ボクセル種別判定(center.x, center.y, center.z);
                    chunk.設定(x, y, z, t, 種別);
                }
            }
        }
    }
    chunk
}

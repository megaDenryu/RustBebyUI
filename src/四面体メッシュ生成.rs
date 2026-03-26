// src/四面体メッシュ生成.rs
// 四面体メッシュ生成: 各四面体の三角形面を描画

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::ボクセル世界::{ボクセル種別, チャンク解像度};
use crate::四面体世界::{四面体チャンク, 四面体頂点_偶数, 四面体頂点_奇数, 隣接四面体取得, 歪み頂点取得};

/// 四面体の色
fn 四面体色(kind: ボクセル種別) -> [f32; 4] {
    match kind {
        ボクセル種別::草  => [0.35, 0.75, 0.35, 1.0],
        ボクセル種別::土  => [0.55, 0.38, 0.2, 1.0],
        ボクセル種別::石  => [0.65, 0.65, 0.65, 1.0],
        ボクセル種別::水  => [0.2, 0.5, 0.9, 0.6],
        ボクセル種別::道路 => [0.3, 0.3, 0.35, 1.0],
        ボクセル種別::橋  => [0.5, 0.3, 0.1, 1.0],
        ボクセル種別::岩盤 => [0.12, 0.1, 0.1, 1.0],
        ボクセル種別::虹  => [0.8, 0.4, 0.9, 1.0],
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

/// 四面体の面が隣接する四面体によって隠されているかを判定
fn 面は見えるか(chunk: &四面体チャンク, x: usize, y: usize, z: usize, t: usize, face: usize) -> bool {
    let is_even = (x + y + z) % 2 == 0;
    let (dx, dy, dz, nt) = 隣接四面体取得(is_even, t, face);

    let nx = x as i32 + dx;
    let ny = y as i32 + dy;
    let nz = z as i32 + dz;

    // チャンク境界外は常に描画
    if nx < 0 || ny < 0 || nz < 0
        || nx >= チャンク解像度 as i32 || ny >= チャンク解像度 as i32 || nz >= チャンク解像度 as i32
    {
        return true;
    }

    let nk = chunk.取得(nx as usize, ny as usize, nz as usize, nt);
    nk == ボクセル種別::空気 || nk == ボクセル種別::水
}

// 面定義: 四面体の4つの三角形面
// fi=0: [0,2,1] missing tv[3]
// fi=1: [0,1,3] missing tv[2]
// fi=2: [0,3,2] missing tv[1]
// fi=3: [1,2,3] missing tv[0]
const 面定義: [[usize; 3]; 4] = [
    [0, 2, 1],
    [0, 1, 3],
    [0, 3, 2],
    [1, 2, 3],
];

/// 四面体チャンクからメッシュを生成
pub fn 四面体メッシュ生成(pos: (i32, i32, i32), chunk: &四面体チャンク, lod: u32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut vert_count: u32 = 0;

    let step = lod as usize;
    let ofs_x = pos.0 * チャンク解像度 as i32;
    let ofs_y = pos.1 * チャンク解像度 as i32;
    let ofs_z = pos.2 * チャンク解像度 as i32;

    for x in (0..チャンク解像度).step_by(step) {
        for y in (0..チャンク解像度).step_by(step) {
            for z in (0..チャンク解像度).step_by(step) {
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
                    let kind = chunk.取得(x, y, z, t);
                    if kind == ボクセル種別::空気 || kind == ボクセル種別::水 { continue; }

                    let color = 四面体色(kind);
                    let tv = tetra_defs[t];

                    for (fi, face) in 面定義.iter().enumerate() {
                        if !面は見えるか(chunk, x, y, z, t, fi) { continue; }

                        // 4番目の頂点(面の反対側)を使って法線の向きを決める
                        let missing_idx = 6 - (face[0] + face[1] + face[2]);
                        let p3 = cell_verts[tv[missing_idx]];

                        let mut p0 = cell_verts[tv[face[0]]];
                        let mut p1 = cell_verts[tv[face[1]]];
                        let mut p2 = cell_verts[tv[face[2]]];

                        let mut normal = (p1 - p0).cross(p2 - p0);
                        if normal.dot(p3 - p0) > 0.0 {
                            // 法線が内側なら表裏反転
                            std::mem::swap(&mut p1, &mut p2);
                            normal = (p1 - p0).cross(p2 - p0);
                        }
                        let normal = normal.normalize_or_zero();

                        positions.push(p0.into());
                        positions.push(p1.into());
                        positions.push(p2.into());

                        for _ in 0..3 {
                            normals.push(normal.into());
                            uvs.push([0.0, 0.0]);
                            colors.push(color);
                        }

                        indices.extend_from_slice(&[vert_count, vert_count + 1, vert_count + 2]);
                        vert_count += 3;
                    }
                }
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

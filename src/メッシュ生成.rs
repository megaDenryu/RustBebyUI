// src/メッシュ生成.rs
// レイヤー2: 立方体メッシュ変換

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::ボクセル世界::{チャンク, ボクセル種別, チャンク解像度, ボクセルスケール};

/// ボクセル種別ごとの色
fn ボクセル色(種別: ボクセル種別, x: usize, y: usize, z: usize) -> [f32; 4] {
    match 種別 {
        ボクセル種別::草  => [0.4, 0.8, 0.4, 1.0],
        ボクセル種別::土  => [0.6, 0.4, 0.2, 1.0],
        ボクセル種別::石  => [0.7, 0.7, 0.7, 1.0],
        ボクセル種別::水  => [0.2, 0.5, 0.9, 0.6],
        ボクセル種別::道路 => [0.3, 0.3, 0.35, 1.0],
        ボクセル種別::橋  => [0.5, 0.3, 0.1, 1.0],
        ボクセル種別::虹  => {
            let r = (x as f32 / チャンク解像度 as f32).fract();
            let g = (y as f32 / チャンク解像度 as f32).fract();
            let b = (z as f32 / チャンク解像度 as f32).fract();
            [r, g, b, 1.0]
        },
        ボクセル種別::岩盤 => [0.12, 0.1, 0.1, 1.0],
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

/// メッシュ生成エントリポイント (LOD対応)
pub fn メッシュ生成(chunk: &チャンク, lod: u32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut vertex_count: u32 = 0;

    let step = lod as usize;
    let scale = ボクセルスケール * lod as f32;
    let hs = 0.5 * ボクセルスケール; // base voxel half-size

    for x in (0..チャンク解像度).step_by(step) {
        for y in (0..チャンク解像度).step_by(step) {
            for z in (0..チャンク解像度).step_by(step) {
                // LOD: step内の代表ボクセルを探す
                let voxel = match 代表ボクセル探索(chunk, x, y, z, step) {
                    Some(v) => v,
                    None => continue,
                };

                let color = ボクセル色(voxel, x, y, z);
                let voxel_pos = Vec3::new(x as f32, y as f32, z as f32) * ボクセルスケール;

                // 6方向の面を確認
                const 方向: [(i32,i32,i32, Vec3); 6] = [
                    ( 0, 1, 0, Vec3::Y),
                    ( 0,-1, 0, Vec3::NEG_Y),
                    ( 1, 0, 0, Vec3::X),
                    (-1, 0, 0, Vec3::NEG_X),
                    ( 0, 0, 1, Vec3::Z),
                    ( 0, 0,-1, Vec3::NEG_Z),
                ];

                for &(dx, dy, dz, normal) in &方向 {
                    if !面は露出か(chunk, x, y, z, dx, dy, dz, lod) { continue; }

                    let face_verts = 面の頂点(dx, dy, dz, hs, scale);
                    for v in &face_verts {
                        positions.push((voxel_pos + *v).into());
                    }
                    for _ in 0..4 {
                        normals.push(normal.into());
                        uvs.push([0.0, 0.0]);
                        colors.push(color);
                    }
                    indices.extend_from_slice(&[
                        vertex_count, vertex_count + 1, vertex_count + 2,
                        vertex_count + 2, vertex_count + 3, vertex_count,
                    ]);
                    vertex_count += 4;
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

/// LODステップ内で最初に見つかった非空気ボクセルの種別を返す
fn 代表ボクセル探索(chunk: &チャンク, x: usize, y: usize, z: usize, step: usize) -> Option<ボクセル種別> {
    for sx in 0..step {
        for sy in 0..step {
            for sz in 0..step {
                if x + sx < チャンク解像度 && y + sy < チャンク解像度 && z + sz < チャンク解像度 {
                    let v = chunk.取得(x + sx, y + sy, z + sz);
                    if !v.は空気か() {
                        return Some(v.種別);
                    }
                }
            }
        }
    }
    None
}

/// 指定方向の隣接ボクセルが透過(空気or水)かどうか
fn 面は露出か(chunk: &チャンク, x: usize, y: usize, z: usize, dx: i32, dy: i32, dz: i32, lod: u32) -> bool {
    let nx = x as i32 + dx * lod as i32;
    let ny = y as i32 + dy * lod as i32;
    let nz = z as i32 + dz * lod as i32;
    if nx < 0 || ny < 0 || nz < 0
        || nx >= チャンク解像度 as i32 || ny >= チャンク解像度 as i32 || nz >= チャンク解像度 as i32
    {
        return true; // チャンク境界は常に描画
    }
    chunk.取得(nx as usize, ny as usize, nz as usize).は透過か()
}

/// 面方向に応じた4頂点を返す
fn 面の頂点(dx: i32, dy: i32, dz: i32, hs: f32, scale: f32) -> [Vec3; 4] {
    let s = scale - hs;
    match (dx, dy, dz) {
        (0, 1, 0) => [ // +Y
            Vec3::new(-hs, scale - hs, -hs), Vec3::new(-hs, scale - hs, s),
            Vec3::new(s, scale - hs, s), Vec3::new(s, scale - hs, -hs),
        ],
        (0, -1, 0) => [ // -Y
            Vec3::new(-hs, -hs, s), Vec3::new(-hs, -hs, -hs),
            Vec3::new(s, -hs, -hs), Vec3::new(s, -hs, s),
        ],
        (1, 0, 0) => [ // +X
            Vec3::new(s, -hs, -hs), Vec3::new(s, s, -hs),
            Vec3::new(s, s, s), Vec3::new(s, -hs, s),
        ],
        (-1, 0, 0) => [ // -X
            Vec3::new(-hs, -hs, s), Vec3::new(-hs, s, s),
            Vec3::new(-hs, s, -hs), Vec3::new(-hs, -hs, -hs),
        ],
        (0, 0, 1) => [ // +Z
            Vec3::new(s, -hs, s), Vec3::new(s, s, s),
            Vec3::new(-hs, s, s), Vec3::new(-hs, -hs, s),
        ],
        _ => [ // -Z
            Vec3::new(-hs, -hs, -hs), Vec3::new(-hs, s, -hs),
            Vec3::new(s, s, -hs), Vec3::new(s, -hs, -hs),
        ],
    }
}

// src/tetra_meshing.rs
// 四面体メッシュ生成: 各四面体の三角形面を描画

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::voxel_world::{ボクセル種別, チャンク解像度};
use crate::tetra_world::{TetraChunk, TETRA_VERTICES_EVEN, TETRA_VERTICES_ODD, get_neighbor_tetra, get_warped_vertex};

/// 四面体の色（立方体と同じ配色）
fn tetra_color(kind: ボクセル種別) -> [f32; 4] {
    match kind {
        ボクセル種別::草 => [0.35, 0.75, 0.35, 1.0],
        ボクセル種別::土  => [0.55, 0.38, 0.2, 1.0],
        ボクセル種別::石 => [0.65, 0.65, 0.65, 1.0],
        ボクセル種別::水 => [0.2, 0.5, 0.9, 0.6],
        ボクセル種別::道路  => [0.3, 0.3, 0.35, 1.0],
        ボクセル種別::橋 => [0.5, 0.3, 0.1, 1.0],
        ボクセル種別::岩盤 => [0.12, 0.1, 0.1, 1.0],
        ボクセル種別::虹 => [0.8, 0.4, 0.9, 1.0],
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

/// 四面体の4面それぞれについて、隣接する四面体が空気かどうかを判定
fn is_face_visible(chunk: &TetraChunk, x: usize, y: usize, z: usize, t: usize, face: usize) -> bool {
    let is_even = (x + y + z) % 2 == 0;
    let (dx, dy, dz, nt) = get_neighbor_tetra(is_even, t, face);

    let nx = x as i32 + dx;
    let ny = y as i32 + dy;
    let nz = z as i32 + dz;

    // チャンク境界外は常に描画する
    if nx < 0 || ny < 0 || nz < 0 || nx >= チャンク解像度 as i32 || ny >= チャンク解像度 as i32 || nz >= チャンク解像度 as i32 {
        return true;
    }

    let nk = chunk.get(nx as usize, ny as usize, nz as usize, nt);
    if nk != ボクセル種別::空気 && nk != ボクセル種別::水 {
        return false; // 隣接がソリッドなら非表示
    }
    true
}

/// 四面体チャンクからメッシュを生成
pub fn generate_tetra_mesh(pos: (i32, i32, i32), chunk: &TetraChunk, lod: u32) -> Mesh {
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

    // Translation for local chunk rendering
    // `get_warped_vertex` returns world positions, so we need to subtract the chunk's base world pos
    // because the Mesh is attached to an Entity that has its own Transform.
    // wait, we can just spawn the entity at (0,0,0) and use world positions directly!
    // We'll adjust tetra_chunk_system.rs to spawn at 0,0,0.

    for x in (0..チャンク解像度).step_by(step) {
        for y in (0..チャンク解像度).step_by(step) {
            for z in (0..チャンク解像度).step_by(step) {
                let gx = ofs_x + x as i32;
                let gy = ofs_y + y as i32;
                let gz = ofs_z + z as i32;

                let cell_verts = [
                    get_warped_vertex(gx,   gy,   gz),
                    get_warped_vertex(gx+1, gy,   gz),
                    get_warped_vertex(gx+1, gy+1, gz),
                    get_warped_vertex(gx,   gy+1, gz),
                    get_warped_vertex(gx,   gy,   gz+1),
                    get_warped_vertex(gx+1, gy,   gz+1),
                    get_warped_vertex(gx+1, gy+1, gz+1),
                    get_warped_vertex(gx,   gy+1, gz+1),
                ];

                let is_even = (x + y + z) % 2 == 0;
                let tetra_defs = if is_even { TETRA_VERTICES_EVEN } else { TETRA_VERTICES_ODD };

                for t in 0..5 {
                    let kind = chunk.get(x, y, z, t);
                    if kind == ボクセル種別::空気 || kind == ボクセル種別::水 { continue; }

                    let color = tetra_color(kind);
                    let tv = tetra_defs[t];

                    let faces: [[usize; 3]; 4] = [
                        [0, 2, 1], // 面0
                        [0, 1, 3], // 面1
                        [0, 3, 2], // 面2
                        [1, 2, 3], // 面3
                    ];

                    for (fi, face) in faces.iter().enumerate() {
                        if !is_face_visible(chunk, x, y, z, t, fi) { continue; }

                        let missing_idx = 6 - (face[0] + face[1] + face[2]);
                        let p3 = cell_verts[tv[missing_idx]];

                        let mut p0 = cell_verts[tv[face[0]]];
                        let mut p1 = cell_verts[tv[face[1]]];
                        let mut p2 = cell_verts[tv[face[2]]];

                        let mut normal = (p1 - p0).cross(p2 - p0);
                        if normal.dot(p3 - p0) > 0.0 {
                            // 法線が内側に向いているため表裏をひっくり返す
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

                        indices.push(vert_count);
                        indices.push(vert_count + 1);
                        indices.push(vert_count + 2);
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

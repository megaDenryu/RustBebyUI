// src/tetra_meshing.rs
// 四面体メッシュ生成: 各四面体の三角形面を描画

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::voxel_world::{ボクセル種別, チャンク解像度, ボクセルスケール};
use crate::tetra_world::{TetraChunk, TETRA_VERTICES, CELL_CORNERS};

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
/// 簡易版: チャンク境界は常に描画する
fn is_face_visible(chunk: &TetraChunk, x: usize, y: usize, z: usize, t: usize, face: usize) -> bool {
    // 同セル内の他の四面体、または隣接セルの四面体を確認
    // 簡易版: すべての非空気四面体の全面を描画（隣接判定は複雑なので省略）
    // ただし同セル内の中央四面体(t=4)との接触面は消す
    let _ = face;
    
    // 同セル内の隣接四面体チェック (簡易)
    // 各四面体t=0..3は中央四面体t=4と1面を共有
    if t < 4 {
        // 中央四面体がソリッドなら、共有面は非表示
        let center = chunk.get(x, y, z, 4);
        if center != ボクセル種別::空気 && center != ボクセル種別::水 {
            if face == 3 { return false; } // 4番目の面が中央との共有面
        }
    } else {
        // 中央四面体の各面は周辺四面体と共有
        let neighbor_t = face; // face 0..3 は tetra 0..3 と共有
        if neighbor_t < 4 {
            let nk = chunk.get(x, y, z, neighbor_t);
            if nk != ボクセル種別::空気 && nk != ボクセル種別::水 { return false; }
        }
    }
    true
}

/// 四面体チャンクからメッシュを生成
pub fn generate_tetra_mesh(chunk: &TetraChunk, lod: u32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut vert_count: u32 = 0;

    let step = lod as usize;
    let scale = ボクセルスケール * lod as f32;

    for x in (0..チャンク解像度).step_by(step) {
        for y in (0..チャンク解像度).step_by(step) {
            for z in (0..チャンク解像度).step_by(step) {
                for t in 0..5 {
                    let kind = chunk.get(x, y, z, t);
                    if kind == ボクセル種別::空気 || kind == ボクセル種別::水 { continue; }

                    let color = tetra_color(kind);
                    let cell_origin = Vec3::new(x as f32, y as f32, z as f32) * ボクセルスケール;

                    // 四面体の4頂点を計算
                    let tv = TETRA_VERTICES[t];
                    let verts: [Vec3; 4] = [
                        cell_origin + Vec3::from_array(CELL_CORNERS[tv[0]]) * scale,
                        cell_origin + Vec3::from_array(CELL_CORNERS[tv[1]]) * scale,
                        cell_origin + Vec3::from_array(CELL_CORNERS[tv[2]]) * scale,
                        cell_origin + Vec3::from_array(CELL_CORNERS[tv[3]]) * scale,
                    ];

                    // 四面体の4つの三角形面
                    // 面: (0,1,2), (0,1,3), (0,2,3), (1,2,3)
                    let faces: [[usize; 3]; 4] = [
                        [0, 2, 1], // 面0
                        [0, 1, 3], // 面1
                        [0, 3, 2], // 面2
                        [1, 2, 3], // 面3
                    ];

                    for (fi, face) in faces.iter().enumerate() {
                        if !is_face_visible(chunk, x, y, z, t, fi) { continue; }

                        let v0 = verts[face[0]];
                        let v1 = verts[face[1]];
                        let v2 = verts[face[2]];

                        // 法線計算
                        let edge1 = v1 - v0;
                        let edge2 = v2 - v0;
                        let normal = edge1.cross(edge2).normalize_or_zero();

                        positions.push(v0.into());
                        positions.push(v1.into());
                        positions.push(v2.into());

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

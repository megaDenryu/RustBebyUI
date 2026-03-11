// src/meshing.rs
// Layer 2: Meshing Adapter
// Greedy Meshing: 同じ色・向きの隣接面を結合し、ポリゴン数を劇的に削減する。

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::voxel_world::{Chunk, ボクセル種別, チャンク解像度, ボクセルスケール};

/// ボクセル種別ごとの色を返す
fn voxel_color(種別: ボクセル種別, x: usize, y: usize, z: usize) -> [f32; 4] {
    match 種別 {
        ボクセル種別::草 => [0.4, 0.8, 0.4, 1.0],
        ボクセル種別::土  => [0.6, 0.4, 0.2, 1.0],
        ボクセル種別::石 => [0.7, 0.7, 0.7, 1.0],
        ボクセル種別::水 => [0.2, 0.5, 0.9, 0.6],
        ボクセル種別::道路  => [0.3, 0.3, 0.35, 1.0],
        ボクセル種別::橋 => [0.5, 0.3, 0.1, 1.0],
        ボクセル種別::虹 => {
            let r = (x as f32 / チャンク解像度 as f32).fract();
            let g = (y as f32 / チャンク解像度 as f32).fract();
            let b = (z as f32 / チャンク解像度 as f32).fract();
            [r, g, b, 1.0]
        },
        ボクセル種別::岩盤 => [0.12, 0.1, 0.1, 1.0],
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

/// メッシュ生成のエントリポイント
/// TODO: LOD1でGreedy Meshingを導入予定（ワインディングオーダーの修正待ち）
pub fn generate_naive_mesh(chunk: &Chunk, lod: u32) -> Mesh {
    generate_lod_mesh(chunk, lod)
}

/// Greedy Meshing: 同じ種別の隣接する面を1つの大きな矩形に結合
fn generate_greedy_mesh(chunk: &Chunk) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut vertex_count: u32 = 0;
    
    let s = チャンク解像度;
    let vs = ボクセルスケール;
    let half = 0.5 * vs;
    
    // 6方向について、各スライスでGreedyを実行
    // direction: 0=+Y, 1=-Y, 2=+X, 3=-X, 4=+Z, 5=-Z
    for dir in 0..6 {
        let (axis, u_axis, v_axis, normal, flip) = match dir {
            0 => (1, 0, 2, Vec3::Y,     false), // +Y (Top)
            1 => (1, 0, 2, Vec3::NEG_Y,  true),  // -Y (Bottom)
            2 => (0, 2, 1, Vec3::X,      false), // +X (Right)
            3 => (0, 2, 1, Vec3::NEG_X,  true),  // -X (Left)
            4 => (2, 0, 1, Vec3::Z,      false), // +Z (Front)
            _ => (2, 0, 1, Vec3::NEG_Z,  true),  // -Z (Back)
        };
        
        // 各スライス（axis方向の各層）
        for d in 0..s {
            // この層のマスクを構築: 面が見えるかどうか + ボクセル種別
            let mut mask: Vec<Option<ボクセル種別>> = vec![None; s * s];
            
            for v in 0..s {
                for u in 0..s {
                    let mut coord = [0usize; 3];
                    coord[axis] = d;
                    coord[u_axis] = u;
                    coord[v_axis] = v;
                    
                    let voxel = chunk.get(coord[0], coord[1], coord[2]);
                    if voxel.は空気か() || voxel.は水か() { continue; }
                    
                    // 隣接ボクセルの確認
                    let mut neighbor_coord = coord;
                    if !flip {
                        // 正方向の面: axis+1に空気があるか
                        if d + 1 < s {
                            neighbor_coord[axis] = d + 1;
                            let neighbor = chunk.get(neighbor_coord[0], neighbor_coord[1], neighbor_coord[2]);
                            if !neighbor.は空気か() && !neighbor.は水か() { continue; }
                        }
                    } else {
                        // 負方向の面: axis-1に空気があるか
                        if d > 0 {
                            neighbor_coord[axis] = d - 1;
                            let neighbor = chunk.get(neighbor_coord[0], neighbor_coord[1], neighbor_coord[2]);
                            if !neighbor.は空気か() && !neighbor.は水か() { continue; }
                        }
                    }
                    
                    mask[u + v * s] = Some(voxel.種別);
                }
            }
            
            // Greedy: マスクを走査し、同色の矩形を結合
            let mut visited = vec![false; s * s];
            for v in 0..s {
                for u in 0..s {
                    let idx = u + v * s;
                    if visited[idx] { continue; }
                    let Some(kind) = mask[idx] else { continue; };
                    
                    // 横(u方向)の最大幅を求める
                    let mut w = 1;
                    while u + w < s && !visited[idx + w] && mask[idx + w] == Some(kind) {
                        w += 1;
                    }
                    
                    // 縦(v方向)の最大高さを求める
                    let mut h = 1;
                    'outer: while v + h < s {
                        for du in 0..w {
                            let check = (u + du) + (v + h) * s;
                            if visited[check] || mask[check] != Some(kind) {
                                break 'outer;
                            }
                        }
                        h += 1;
                    }
                    
                    // マスクを訪問済みに
                    for dv in 0..h {
                        for du in 0..w {
                            visited[(u + du) + (v + dv) * s] = true;
                        }
                    }
                    
                    // 面の生成
                    let color = voxel_color(kind, u, d, v);
                    
                    // 面の位置計算
                    let face_d = if flip { d as f32 } else { (d + 1) as f32 };
                    
                    let mut p = [0.0f32; 3];
                    p[axis] = (face_d - 0.5) * vs;
                    
                    // u, v 方向の開始点  
                    let u_start = u as f32 * vs - half;
                    let v_start = v as f32 * vs - half;
                    let u_end = (u + w) as f32 * vs - half;
                    let v_end = (v + h) as f32 * vs - half;
                    
                    // 4頂点を生成
                    let mut make_pos = |u_val: f32, v_val: f32| -> [f32; 3] {
                        let mut pos = p;
                        pos[u_axis] = u_val;
                        pos[v_axis] = v_val;
                        pos
                    };
                    
                    let (p0, p1, p2, p3) = if !flip {
                        (
                            make_pos(u_start, v_start),
                            make_pos(u_start, v_end),
                            make_pos(u_end, v_end),
                            make_pos(u_end, v_start),
                        )
                    } else {
                        (
                            make_pos(u_start, v_start),
                            make_pos(u_end, v_start),
                            make_pos(u_end, v_end),
                            make_pos(u_start, v_end),
                        )
                    };
                    
                    positions.push(p0);
                    positions.push(p1);
                    positions.push(p2);
                    positions.push(p3);
                    
                    for _ in 0..4 {
                        normals.push(normal.into());
                        uvs.push([0.0, 0.0]);
                        colors.push(color);
                    }
                    
                    indices.push(vertex_count);
                    indices.push(vertex_count + 1);
                    indices.push(vertex_count + 2);
                    indices.push(vertex_count + 2);
                    indices.push(vertex_count + 3);
                    indices.push(vertex_count);
                    
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

/// 従来のNaive Mesh (LOD2以上で使用, 計算コスト優先)
fn generate_lod_mesh(chunk: &Chunk, lod: u32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut vertex_count = 0;
    let step = lod as usize;
    let scale_factor = ボクセルスケール * lod as f32;
    let half_size = 0.5 * scale_factor;

    for x in (0..チャンク解像度).step_by(step) {
        for y in (0..チャンク解像度).step_by(step) {
            for z in (0..チャンク解像度).step_by(step) {
                let mut representative_voxel = None;
                'sample: for sx in 0..step {
                    for sy in 0..step {
                        for sz in 0..step {
                            if x + sx < チャンク解像度 && y + sy < チャンク解像度 && z + sz < チャンク解像度 {
                                let v = chunk.get(x + sx, y + sy, z + sz);
                                if !v.は空気か() {
                                    representative_voxel = Some(v);
                                    break 'sample;
                                }
                            }
                        }
                    }
                }

                let Some(voxel) = representative_voxel else { continue; };
                let color = voxel_color(voxel.種別, x, y, z);

                let check_face = |dx: i32, dy: i32, dz: i32| -> bool {
                    let nx = x as i32 + dx * lod as i32;
                    let ny = y as i32 + dy * lod as i32;
                    let nz = z as i32 + dz * lod as i32;
                    if nx < 0 || ny < 0 || nz < 0 || nx >= チャンク解像度 as i32 || ny >= チャンク解像度 as i32 || nz >= チャンク解像度 as i32 {
                        true 
                    } else {
                        let neighbor = chunk.get(nx as usize, ny as usize, nz as usize);
                        neighbor.は空気か() || neighbor.は水か()
                    }
                };

                let voxel_pos = Vec3::new(x as f32, y as f32, z as f32) * ボクセルスケール;
                let hs = 0.5 * ボクセルスケール; // half of base voxel scale

                let mut add_face = |normal: Vec3, p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3| {
                    positions.push((voxel_pos + p0).into());
                    positions.push((voxel_pos + p1).into());
                    positions.push((voxel_pos + p2).into());
                    positions.push((voxel_pos + p3).into());
                    for _ in 0..4 {
                        normals.push(normal.into());
                        uvs.push([0.0, 0.0]);
                        colors.push(color);
                    }
                    indices.push(vertex_count);
                    indices.push(vertex_count + 1);
                    indices.push(vertex_count + 2);
                    indices.push(vertex_count + 2);
                    indices.push(vertex_count + 3);
                    indices.push(vertex_count);
                    vertex_count += 4;
                };

                if check_face(0, 1, 0) {
                    add_face(Vec3::Y, Vec3::new(-hs, half_size, -hs), Vec3::new(-hs, half_size, scale_factor - hs), Vec3::new(scale_factor - hs, half_size, scale_factor - hs), Vec3::new(scale_factor - hs, half_size, -hs));
                }
                if check_face(0, -1, 0) {
                    add_face(Vec3::NEG_Y, Vec3::new(-hs, -hs, scale_factor - hs), Vec3::new(-hs, -hs, -hs), Vec3::new(scale_factor - hs, -hs, -hs), Vec3::new(scale_factor - hs, -hs, scale_factor - hs));
                }
                if check_face(1, 0, 0) {
                    add_face(Vec3::X, Vec3::new(scale_factor - hs, -hs, -hs), Vec3::new(scale_factor - hs, scale_factor - hs, -hs), Vec3::new(scale_factor - hs, scale_factor - hs, scale_factor - hs), Vec3::new(scale_factor - hs, -hs, scale_factor - hs));
                }
                if check_face(-1, 0, 0) {
                    add_face(Vec3::NEG_X, Vec3::new(-hs, -hs, scale_factor - hs), Vec3::new(-hs, scale_factor - hs, scale_factor - hs), Vec3::new(-hs, scale_factor - hs, -hs), Vec3::new(-hs, -hs, -hs));
                }
                if check_face(0, 0, 1) {
                    add_face(Vec3::Z, Vec3::new(scale_factor - hs, -hs, scale_factor - hs), Vec3::new(scale_factor - hs, scale_factor - hs, scale_factor - hs), Vec3::new(-hs, scale_factor - hs, scale_factor - hs), Vec3::new(-hs, -hs, scale_factor - hs));
                }
                if check_face(0, 0, -1) {
                    add_face(Vec3::NEG_Z, Vec3::new(-hs, -hs, -hs), Vec3::new(-hs, scale_factor - hs, -hs), Vec3::new(scale_factor - hs, scale_factor - hs, -hs), Vec3::new(scale_factor - hs, -hs, -hs));
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

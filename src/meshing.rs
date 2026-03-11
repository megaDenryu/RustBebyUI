// src/meshing.rs
// Layer 2: Meshing Adapter
// 純粋なDomainデータ(Chunk)を受け取り、描画フレームワーク(Bevy Mesh)へ変換する。

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::domain::{Chunk, VoxelType, CHUNK_SIZE, VOXEL_SCALE};

// ナイーブ・メッシュ生成（各ボクセルの見えている面だけをポリゴン化）
// 後にGreedy Meshingなどの最適化関数に置き換え可能
pub fn generate_naive_mesh(chunk: &Chunk) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut vertex_count = 0;
    // ボクセルをスケールに合わせて小さくする
    let half_size = 0.5 * VOXEL_SCALE;

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let voxel = chunk.get(x, y, z);
                if voxel.is_empty() {
                    continue;
                }

                // ボクセル種別ごとの色（明るめ）
                let color = match voxel.v_type {
                    VoxelType::Grass => [0.4, 0.8, 0.4, 1.0], 
                    VoxelType::Dirt  => [0.6, 0.4, 0.2, 1.0], 
                    VoxelType::Stone => [0.7, 0.7, 0.7, 1.0], 
                    VoxelType::Water => [0.2, 0.5, 0.9, 0.6], // Transparent blue
                    VoxelType::Road  => [0.3, 0.3, 0.35, 1.0], // Dark road
                    VoxelType::Bridge => [0.5, 0.3, 0.1, 1.0], // Wood bridge
                    VoxelType::Rainbow => {
                        // Rainbow gradient based on local position
                        let r = (x as f32 / CHUNK_SIZE as f32).fract();
                        let g = (y as f32 / CHUNK_SIZE as f32).fract();
                        let b = (z as f32 / CHUNK_SIZE as f32).fract();
                        [r, g, b, 1.0]
                    },
                    _ => [1.0, 1.0, 1.0, 1.0],
                };

                // 隣接ボクセルの確認（自身がブロックされている面は描画しない）
                let check_face = |dx: i32, dy: i32, dz: i32| -> bool {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let nz = z as i32 + dz;
                    // Chunk境界外はとりあえず描画する
                    if nx < 0 || ny < 0 || nz < 0 || nx >= CHUNK_SIZE as i32 || ny >= CHUNK_SIZE as i32 || nz >= CHUNK_SIZE as i32 {
                        true 
                    } else {
                        // 空気なら面を描画
                        chunk.get(nx as usize, ny as usize, nz as usize).is_empty()
                    }
                };

                // ボクセルごとの中心位置をスケールに合わせて配置
                let voxel_pos = Vec3::new(x as f32, y as f32, z as f32) * VOXEL_SCALE;

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

                // Y (Top)
                if check_face(0, 1, 0) {
                    add_face(Vec3::Y, Vec3::new(-half_size, half_size, -half_size), Vec3::new(-half_size, half_size, half_size), Vec3::new(half_size, half_size, half_size), Vec3::new(half_size, half_size, -half_size));
                }
                // -Y (Bottom)
                if check_face(0, -1, 0) {
                    add_face(Vec3::NEG_Y, Vec3::new(-half_size, -half_size, half_size), Vec3::new(-half_size, -half_size, -half_size), Vec3::new(half_size, -half_size, -half_size), Vec3::new(half_size, -half_size, half_size));
                }
                // X (Right)
                if check_face(1, 0, 0) {
                    add_face(Vec3::X, Vec3::new(half_size, -half_size, -half_size), Vec3::new(half_size, half_size, -half_size), Vec3::new(half_size, half_size, half_size), Vec3::new(half_size, -half_size, half_size));
                }
                // -X (Left)
                if check_face(-1, 0, 0) {
                    add_face(Vec3::NEG_X, Vec3::new(-half_size, -half_size, half_size), Vec3::new(-half_size, half_size, half_size), Vec3::new(-half_size, half_size, -half_size), Vec3::new(-half_size, -half_size, -half_size));
                }
                // Z (Front)
                if check_face(0, 0, 1) {
                    add_face(Vec3::Z, Vec3::new(half_size, -half_size, half_size), Vec3::new(half_size, half_size, half_size), Vec3::new(-half_size, half_size, half_size), Vec3::new(-half_size, -half_size, half_size));
                }
                // -Z (Back)
                if check_face(0, 0, -1) {
                    add_face(Vec3::NEG_Z, Vec3::new(-half_size, -half_size, -half_size), Vec3::new(-half_size, half_size, -half_size), Vec3::new(half_size, half_size, -half_size), Vec3::new(half_size, -half_size, -half_size));
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

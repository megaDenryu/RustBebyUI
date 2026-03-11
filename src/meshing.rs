// src/meshing.rs
// Layer 2: Meshing Adapter
// 純粋なDomainデータ(Chunk)を受け取り、描画フレームワーク(Bevy Mesh)へ変換する。

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::voxel_world::{Chunk, ボクセル種別, チャンク解像度, ボクセルスケール};

// ナイーブ・メッシュ生成（各ボクセルの見えている面だけをポリゴン化）
// lod: 1(最高) 2(1/2) 4(1/4)...
pub fn generate_naive_mesh(chunk: &Chunk, lod: u32) -> Mesh {
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
                // LOD用のサンプリング: ブロック内のいずれかがソリッドなら代表として描画
                let mut representative_voxel = None;
                'sample: for sx in 0..step {
                    for sy in 0..step {
                        for sz in 0..step {
                            let v = chunk.get(x + sx, y + sy, z + sz);
                            if !v.は空気か() {
                                representative_voxel = Some(v);
                                break 'sample;
                            }
                        }
                    }
                }

                let Some(voxel) = representative_voxel else {
                    continue;
                };

                // ボクセル種別ごとの色
                let color = match voxel.種別 {
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
                    _ => [1.0, 1.0, 1.0, 1.0],
                };

                // 隣接ボクセルの確認 (LOD考慮)
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
                    add_face(Vec3::Y, Vec3::new(-0.5 * ボクセルスケール, half_size, -0.5 * ボクセルスケール), Vec3::new(-0.5 * ボクセルスケール, half_size, scale_factor - 0.5 * ボクセルスケール), Vec3::new(scale_factor - 0.5 * ボクセルスケール, half_size, scale_factor - 0.5 * ボクセルスケール), Vec3::new(scale_factor - 0.5 * ボクセルスケール, half_size, -0.5 * ボクセルスケール));
                }
                // -Y (Bottom)
                if check_face(0, -1, 0) {
                    add_face(Vec3::NEG_Y, Vec3::new(-0.5 * ボクセルスケール, -0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール), Vec3::new(-0.5 * ボクセルスケール, -0.5 * ボクセルスケール, -0.5 * ボクセルスケール), Vec3::new(scale_factor - 0.5 * ボクセルスケール, -0.5 * ボクセルスケール, -0.5 * ボクセルスケール), Vec3::new(scale_factor - 0.5 * ボクセルスケール, -0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール));
                }
                // X (Right)
                if check_face(1, 0, 0) {
                    add_face(Vec3::X, Vec3::new(scale_factor - 0.5 * ボクセルスケール, -0.5 * ボクセルスケール, -0.5 * ボクセルスケール), Vec3::new(scale_factor - 0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール, -0.5 * ボクセルスケール), Vec3::new(scale_factor - 0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール), Vec3::new(scale_factor - 0.5 * ボクセルスケール, -0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール));
                }
                // -X (Left)
                if check_face(-1, 0, 0) {
                    add_face(Vec3::NEG_X, Vec3::new(-0.5 * ボクセルスケール, -0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール), Vec3::new(-0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール), Vec3::new(-0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール, -0.5 * ボクセルスケール), Vec3::new(-0.5 * ボクセルスケール, -0.5 * ボクセルスケール, -0.5 * ボクセルスケール));
                }
                // Z (Front)
                if check_face(0, 0, 1) {
                    add_face(Vec3::Z, Vec3::new(scale_factor - 0.5 * ボクセルスケール, -0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール), Vec3::new(scale_factor - 0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール), Vec3::new(-0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール), Vec3::new(-0.5 * ボクセルスケール, -0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール));
                }
                // -Z (Back)
                if check_face(0, 0, -1) {
                    add_face(Vec3::NEG_Z, Vec3::new(-0.5 * ボクセルスケール, -0.5 * ボクセルスケール, -0.5 * ボクセルスケール), Vec3::new(-0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール, -0.5 * ボクセルスケール), Vec3::new(scale_factor - 0.5 * ボクセルスケール, scale_factor - 0.5 * ボクセルスケール, -0.5 * ボクセルスケール), Vec3::new(scale_factor - 0.5 * ボクセルスケール, -0.5 * ボクセルスケール, -0.5 * ボクセルスケール));
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

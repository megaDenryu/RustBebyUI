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

/// メッシュ生成の結果 (固体メッシュと水メッシュを分離)
pub struct メッシュ生成結果 {
    pub 固体: Mesh,
    pub 水: Mesh,
}

/// メッシュ生成エントリポイント (LOD対応、固体/水を分離)
pub fn メッシュ生成(chunk: &チャンク, lod: u32) -> メッシュ生成結果 {
    let mut 固体ビルダー = メッシュビルダー::new();
    let mut 水ビルダー = メッシュビルダー::new();

    let step = lod as usize;
    let scale = ボクセルスケール * lod as f32;
    let hs = 0.5 * ボクセルスケール;

    for x in (0..チャンク解像度).step_by(step) {
        for y in (0..チャンク解像度).step_by(step) {
            for z in (0..チャンク解像度).step_by(step) {
                let voxel = match 代表ボクセル探索(chunk, x, y, z, step) {
                    Some(v) => v,
                    None => continue,
                };

                let is_water = voxel == ボクセル種別::水;
                let builder = if is_water { &mut 水ビルダー } else { &mut 固体ビルダー };

                let color = ボクセル色(voxel, x, y, z);
                let voxel_pos = Vec3::new(x as f32, y as f32, z as f32) * ボクセルスケール;

                const 方向: [(i32,i32,i32, Vec3); 6] = [
                    ( 0, 1, 0, Vec3::Y),
                    ( 0,-1, 0, Vec3::NEG_Y),
                    ( 1, 0, 0, Vec3::X),
                    (-1, 0, 0, Vec3::NEG_X),
                    ( 0, 0, 1, Vec3::Z),
                    ( 0, 0,-1, Vec3::NEG_Z),
                ];

                for &(dx, dy, dz, normal) in &方向 {
                    // 水の場合: 隣が空気のときだけ面を描画 (水同士の境界は不要)
                    // 固体の場合: 隣が透過(空気or水)のときに面を描画
                    let should_draw = if is_water {
                        隣は空気か(chunk, x, y, z, dx, dy, dz, lod)
                    } else {
                        面は露出か(chunk, x, y, z, dx, dy, dz, lod)
                    };
                    if !should_draw { continue; }

                    let face_verts = 面の頂点(dx, dy, dz, hs, scale);
                    builder.面を追加(&voxel_pos, &face_verts, normal, color);
                }
            }
        }
    }

    メッシュ生成結果 {
        固体: 固体ビルダー.build(),
        水: 水ビルダー.build(),
    }
}

struct メッシュビルダー {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
    vertex_count: u32,
}

impl メッシュビルダー {
    fn new() -> Self {
        Self { positions: Vec::new(), normals: Vec::new(), uvs: Vec::new(), colors: Vec::new(), indices: Vec::new(), vertex_count: 0 }
    }

    fn 面を追加(&mut self, voxel_pos: &Vec3, face_verts: &[Vec3; 4], normal: Vec3, color: [f32; 4]) {
        for v in face_verts {
            self.positions.push((*voxel_pos + *v).into());
        }
        for _ in 0..4 {
            self.normals.push(normal.into());
            self.uvs.push([0.0, 0.0]);
            self.colors.push(color);
        }
        self.indices.extend_from_slice(&[
            self.vertex_count, self.vertex_count + 1, self.vertex_count + 2,
            self.vertex_count + 2, self.vertex_count + 3, self.vertex_count,
        ]);
        self.vertex_count += 4;
    }

    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
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

/// 指定方向の隣接ボクセルが空気かどうか (水面の描画判定用)
fn 隣は空気か(chunk: &チャンク, x: usize, y: usize, z: usize, dx: i32, dy: i32, dz: i32, lod: u32) -> bool {
    let nx = x as i32 + dx * lod as i32;
    let ny = y as i32 + dy * lod as i32;
    let nz = z as i32 + dz * lod as i32;
    if nx < 0 || ny < 0 || nz < 0
        || nx >= チャンク解像度 as i32 || ny >= チャンク解像度 as i32 || nz >= チャンク解像度 as i32
    {
        return true;
    }
    chunk.取得(nx as usize, ny as usize, nz as usize).は空気か()
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

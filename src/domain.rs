// src/domain.rs
// Layer 1: Core Domain Logic
// Bevyや描画エンジンに一切依存しない「純粋関数（Pure Function）」と型定義。

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoxelType {
    Empty,
    Dirt,
    Grass,
    Stone,
    Water,
    Road,
    Bridge,
    Rainbow,
}

#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    pub v_type: VoxelType,
}

impl Voxel {
    pub const EMPTY: Self = Self { v_type: VoxelType::Empty };
    pub const DIRT: Self  = Self { v_type: VoxelType::Dirt };
    pub const GRASS: Self = Self { v_type: VoxelType::Grass };
    pub const STONE: Self = Self { v_type: VoxelType::Stone };
    pub const WATER: Self = Self { v_type: VoxelType::Water };
    pub const ROAD: Self  = Self { v_type: VoxelType::Road };
    pub const BRIDGE: Self = Self { v_type: VoxelType::Bridge };
    pub const RAINBOW: Self = Self { v_type: VoxelType::Rainbow };
    
    pub fn is_empty(&self) -> bool {
        self.v_type == VoxelType::Empty
    }

    pub fn is_water(&self) -> bool {
        self.v_type == VoxelType::Water
    }
}

pub const CHUNK_SIZE: usize = 32;     // 1チャンクあたりのボクセル数（解像度アップ）
pub const VOXEL_SCALE: f32 = 0.25;    // ボクセルの表示サイズ（マイクラ感を減らし、詳細化）
pub const CHUNK_WORLD_SIZE: f32 = CHUNK_SIZE as f32 * VOXEL_SCALE;
pub const WATER_LEVEL: f32 = 4.0;

#[derive(Clone)]
pub struct Chunk {
    pub voxels: Vec<Voxel>, // フラットな3D配列 (x + y*S + z*S*S)
}

impl Chunk {
    pub fn new_empty() -> Self {
        Self {
            voxels: vec![Voxel::EMPTY; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    // チャンク座標 (cx, cy, cz) に基づき、滑らかな世界を生成する純粋関数
    pub fn new_hilly_terrain(cx: i32, cy: i32, cz: i32) -> Self {
        let mut chunk = Self::new_empty();
        
        let world_offset_x = cx as f32 * CHUNK_WORLD_SIZE;
        let world_offset_y = cy as f32 * CHUNK_WORLD_SIZE;
        let world_offset_z = cz as f32 * CHUNK_WORLD_SIZE;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let gx = world_offset_x + (x as f32 * VOXEL_SCALE);
                let gz = world_offset_z + (z as f32 * VOXEL_SCALE);
                
                // より滑らかで複雑な地形 (擬似Sinノイズ合成)
                let h_base = (gx * 0.1).sin() * 4.0 + (gz * 0.1).cos() * 4.0;
                let h_detail = (gx * 0.4).sin() * 1.5 + (gz * 0.35).cos() * 1.5;
                let h_micro = (gx * 1.2).sin() * 0.5 + (gz * 1.1).cos() * 0.5;
                let ground_y = 5.0 + h_base + h_detail + h_micro; // 地面の高さ（World Y）

                for y in 0..CHUNK_SIZE {
                    let gy = world_offset_y + (y as f32 * VOXEL_SCALE);
                    
                    // --- 1. Rainbow Road (In the sky) ---
                    let rainbow_y = 25.0 + (gx * 0.05).sin() * 5.0;
                    if (gy - rainbow_y).abs() < 0.2 && (gz - 10.0).abs() < 3.0 {
                        chunk.set(x, y, z, Voxel::RAINBOW);
                        continue;
                    }

                    // --- 2. Bridges ---
                    let bridge_z = 2.0;
                    if (gz - bridge_z).abs() < 1.0 && (gy - 6.0).abs() < 0.2 && (gx % 20.0).abs() < 10.0 {
                        chunk.set(x, y, z, Voxel::BRIDGE);
                        continue;
                    }

                    // --- 3. Roads ---
                    let is_road = (gx - 5.0).abs() < 1.5 || (gz - 5.0).abs() < 1.5;

                    if gy > ground_y {
                        // 水面下の空間を水で満たす
                        if gy <= WATER_LEVEL {
                            chunk.set(x, y, z, Voxel::WATER);
                        }
                        continue; 
                    }

                    // --- 4. Caves (3D Noise mask) ---
                    let cave_noise = (gx * 0.5).sin() * (gy * 0.5).cos() * (gz * 0.5).sin();
                    if cave_noise > 0.7 {
                        // 水面下なら水を入れる、それ以外は空気
                        if gy <= WATER_LEVEL {
                            chunk.set(x, y, z, Voxel::WATER);
                        } else {
                            chunk.set(x, y, z, Voxel::EMPTY);
                        }
                        continue;
                    }
                    
                    let depth = ground_y - gy;
                    let v_type = if is_road && depth < 0.3 {
                        VoxelType::Road
                    } else if depth < 0.3 {
                        VoxelType::Grass
                    } else if depth < 1.5 {
                        VoxelType::Dirt
                    } else {
                        VoxelType::Stone
                    };
                    
                    chunk.set(x, y, z, Voxel { v_type });
                }
            }
        }
        chunk
    }

    fn idx(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.voxels[Self::idx(x, y, z)]
        } else {
            Voxel::EMPTY
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            let idx = Self::idx(x, y, z);
            self.voxels[idx] = voxel;
        }
    }
}

// 穴を掘る（球状のくり抜き）純粋関数
// 既存のChunkと破壊位置・半径を受け取り、副作用なく新しいChunkを返す
pub fn carve_sphere(chunk: &Chunk, cx: f32, cy: f32, cz: f32, radius: f32) -> Chunk {
    let mut new_chunk = chunk.clone();
    let r2 = radius * radius;

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dz = z as f32 - cz;
                if dx * dx + dy * dy + dz * dz <= r2 {
                    new_chunk.set(x, y, z, Voxel::EMPTY);
                }
            }
        }
    }
    new_chunk
}

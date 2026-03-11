// src/voxel_world.rs
// Layer 1: Core Domain Logic
// Bevyや描画エンジンに一切依存しない「純粋関数（Pure Function）」と型定義。

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ボクセル種別 {
    空気,
    土,
    草,
    石,
    水,
    道路,
    橋,
    虹,
}

#[derive(Clone, Copy, Debug)]
pub struct ボクセル {
    pub 種別: ボクセル種別,
}

impl ボクセル {
    pub const 空気: 自自身 = 自自身 { 種別: ボクセル種別::空気 };
    pub const 土: 自自身  = 自自身 { 種別: ボクセル種別::土 };
    pub const 草: 自自身 = 自自身 { 種別: ボクセル種別::草 };
    pub const 石: 自自身 = 自自身 { 種別: ボクセル種別::石 };
    pub const 水: 自自身 = 自自身 { 種別: ボクセル種別::水 };
    pub const 道路: 自自身  = 自自身 { 種別: ボクセル種別::道路 };
    pub const 橋: 自自身 = 自自身 { 種別: ボクセル種別::橋 };
    pub const 虹: 自自身 = 自自身 { 種別: ボクセル種別::虹 };
    
    pub fn は空気か(&self) -> bool {
        self.種別 == ボクセル種別::空気
    }

    pub fn は水か(&self) -> bool {
        self.種別 == ボクセル種別::水
    }
}

// 内部的なエイリアス
type 自自身 = ボクセル;

pub const チャンク解像度: usize = 32;     // 1チャンクあたりのボクセル数
pub const ボクセルスケール: f32 = 0.25;    // ボクセルの表示サイズ
pub const チャンクのワールドサイズ: f32 = チャンク解像度 as f32 * ボクセルスケール;
pub const 水面高さ: f32 = 4.0;
pub const 描画距離: i32 = 7;

#[derive(Clone)]
pub struct Chunk {
    pub voxels: Vec<ボクセル>, // フラットな3D配列 (x + y*S + z*S*S)
}

impl Chunk {
    pub fn new_empty() -> Self {
        Self {
            voxels: vec![ボクセル::空気; チャンク解像度 * チャンク解像度 * チャンク解像度],
        }
    }

    // チャンク座標 (cx, cy, cz) に基づき、滑らかな世界を生成する純粋関数
    pub fn new_hilly_terrain(cx: i32, cy: i32, cz: i32) -> Self {
        let mut chunk = Self::new_empty();
        
        let world_offset_x = cx as f32 * チャンクのワールドサイズ;
        let world_offset_y = cy as f32 * チャンクのワールドサイズ;
        let world_offset_z = cz as f32 * チャンクのワールドサイズ;

        for x in 0..チャンク解像度 {
            for z in 0..チャンク解像度 {
                let gx = world_offset_x + (x as f32 * ボクセルスケール);
                let gz = world_offset_z + (z as f32 * ボクセルスケール);
                
                // より滑らかで複雑な地形 (擬似Sinノイズ合成)
                let h_base = (gx * 0.1).sin() * 4.0 + (gz * 0.1).cos() * 4.0;
                let h_detail = (gx * 0.4).sin() * 1.5 + (gz * 0.35).cos() * 1.5;
                let h_micro = (gx * 0.8).sin() * 0.5 + (gz * 0.7).cos() * 0.5; // 少し周波数を下げて滑らかに
                let ground_y = 6.0 + h_base + h_detail + h_micro;

                for y in 0..チャンク解像度 {
                    let gy = world_offset_y + (y as f32 * ボクセルスケール);
                    
                    // --- 1. Rainbow Road (In the sky) ---
                    let rainbow_y = 25.0 + (gx * 0.05).sin() * 5.0;
                    if (gy - rainbow_y).abs() < 0.2 && (gz - 10.0).abs() < 3.0 {
                        chunk.set(x, y, z, ボクセル::虹);
                        continue;
                    }

                    // --- 2. Bridges ---
                    let bridge_z = 2.0;
                    if (gz - bridge_z).abs() < 1.0 && (gy - 6.0).abs() < 0.2 && (gx % 40.0).abs() < 20.0 {
                        chunk.set(x, y, z, ボクセル::橋);
                        continue;
                    }

                    // --- 3. Roads ---
                    let is_road = (gx - 5.0).abs() < 1.5 || (gz - 5.0).abs() < 1.5;

                    if gy > ground_y {
                        if gy <= 水面高さ {
                            chunk.set(x, y, z, ボクセル::水);
                        }
                        continue; 
                    }

                    // --- 4. Caves (Connected structures) ---
                    // 3Dノイズの閾値を下げて洞窟を広げ、接続しやすくする
                    let cave_noise = (gx * 0.4).sin() * (gy * 0.4).cos() * (gz * 0.4).sin();
                    let cave_tunnel = (gx * 0.1).cos() * (gz * 0.1).sin(); // 縦方向の大きな空洞
                    
                    if cave_noise > 0.6 || (cave_tunnel > 0.8 && gy < ground_y - 2.0) {
                        if gy <= 水面高さ {
                            chunk.set(x, y, z, ボクセル::水);
                        } else {
                            chunk.set(x, y, z, ボクセル::空気);
                        }
                        continue;
                    }
                    
                    let depth = ground_y - gy;
                    let v_type = if is_road && depth < 0.3 {
                        ボクセル種別::道路
                    } else if depth < 0.3 {
                        ボクセル種別::草
                    } else if depth < 1.5 {
                        ボクセル種別::土
                    } else {
                        ボクセル種別::石
                    };
                    
                    chunk.set(x, y, z, ボクセル { 種別: v_type });
                }
            }
        }
        chunk
    }

    fn idx(x: usize, y: usize, z: usize) -> usize {
        x + y * チャンク解像度 + z * チャンク解像度 * チャンク解像度
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> ボクセル {
        if x < チャンク解像度 && y < チャンク解像度 && z < チャンク解像度 {
            self.voxels[Self::idx(x, y, z)]
        } else {
            ボクセル::空気
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel: ボクセル) {
        if x < チャンク解像度 && y < チャンク解像度 && z < チャンク解像度 {
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

    for x in 0..チャンク解像度 {
        for y in 0..チャンク解像度 {
            for z in 0..チャンク解像度 {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dz = z as f32 - cz;
                if dx * dx + dy * dy + dz * dz <= r2 {
                    new_chunk.set(x, y, z, ボクセル::空気);
                }
            }
        }
    }
    new_chunk
}

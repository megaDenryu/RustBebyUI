// src/voxel_world.rs
// Layer 1: Core Domain Logic

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ボクセル種別 {
    空気, 土, 草, 石, 水, 道路, 橋, 虹, 岩盤,
}

#[derive(Clone, Copy, Debug)]
pub struct ボクセル { pub 種別: ボクセル種別 }

impl ボクセル {
    pub const 空気: Self = Self { 種別: ボクセル種別::空気 };
    pub const 土: Self  = Self { 種別: ボクセル種別::土 };
    pub const 草: Self = Self { 種別: ボクセル種別::草 };
    pub const 石: Self = Self { 種別: ボクセル種別::石 };
    pub const 水: Self = Self { 種別: ボクセル種別::水 };
    pub const 道路: Self = Self { 種別: ボクセル種別::道路 };
    pub const 橋: Self = Self { 種別: ボクセル種別::橋 };
    pub const 虹: Self = Self { 種別: ボクセル種別::虹 };
    pub const 岩盤: Self = Self { 種別: ボクセル種別::岩盤 };
    
    pub fn は空気か(&self) -> bool { self.種別 == ボクセル種別::空気 }
    pub fn は水か(&self) -> bool { self.種別 == ボクセル種別::水 }
}

pub const チャンク解像度: usize = 32;
pub const ボクセルスケール: f32 = 0.25;
pub const チャンクのワールドサイズ: f32 = チャンク解像度 as f32 * ボクセルスケール; // = 8.0
pub const 水面高さ: f32 = 4.0;
pub const 岩盤の高さ: f32 = 0.5;
pub const 描画距離: i32 = 20;

#[derive(Clone)]
pub struct Chunk {
    pub voxels: Vec<ボクセル>,
}

impl Chunk {
    pub fn new_empty() -> Self {
        Self { voxels: vec![ボクセル::空気; チャンク解像度 * チャンク解像度 * チャンク解像度] }
    }

    pub fn new_hilly_terrain(cx: i32, cy: i32, cz: i32) -> Self {
        let mut chunk = Self::new_empty();
        let wo_x = cx as f32 * チャンクのワールドサイズ;
        let wo_y = cy as f32 * チャンクのワールドサイズ;
        let wo_z = cz as f32 * チャンクのワールドサイズ;

        for x in 0..チャンク解像度 {
            for z in 0..チャンク解像度 {
                let gx = wo_x + (x as f32 * ボクセルスケール);
                let gz = wo_z + (z as f32 * ボクセルスケール);
                
                let h_base = (gx * 0.08).sin() * 5.0 + (gz * 0.08).cos() * 5.0;
                let h_hills = (gx * 0.2).sin() * (gz * 0.15).cos() * 3.0;
                let h_detail = (gx * 0.4).sin() * 1.5 + (gz * 0.35).cos() * 1.5;
                let h_micro = (gx * 0.8).sin() * 0.5 + (gz * 0.7).cos() * 0.5;
                let mountain = ((gx * 0.03).sin() * (gz * 0.04).cos()).abs() * 12.0;
                let ground_y = 6.0 + h_base + h_hills + h_detail + h_micro + mountain;

                for y in 0..チャンク解像度 {
                    let gy = wo_y + (y as f32 * ボクセルスケール);

                    // 岩盤層
                    if gy <= 岩盤の高さ {
                        chunk.set(x, y, z, ボクセル::岩盤);
                        continue;
                    }

                    // 虹の道
                    let rainbow_y = 25.0 + (gx * 0.05).sin() * 5.0;
                    if (gy - rainbow_y).abs() < 0.2 && (gz - 10.0).abs() < 3.0 {
                        chunk.set(x, y, z, ボクセル::虹); continue;
                    }
                    // 橋
                    if (gz - 2.0).abs() < 1.0 && (gy - 6.0).abs() < 0.2 && (gx % 40.0).abs() < 20.0 {
                        chunk.set(x, y, z, ボクセル::橋); continue;
                    }
                    // 道路
                    let is_road = (gx - 5.0).abs() < 1.5 || (gz - 5.0).abs() < 1.5;

                    if gy > ground_y {
                        if gy <= 水面高さ { chunk.set(x, y, z, ボクセル::水); }
                        continue;
                    }

                    // 洞窟 & トンネル
                    let cave_noise = (gx * 0.35).sin() * (gy * 0.4).cos() * (gz * 0.35).sin();
                    let tunnel_h1 = ((gx * 0.08).cos() * (gz * 0.08).sin()).abs();
                    let tunnel_h2 = ((gx * 0.12 + 1.0).sin() * (gz * 0.06).cos()).abs();
                    let tunnel_y_center = 5.0 + (gx * 0.05).sin() * 2.0;
                    let is_tunnel = (tunnel_h1 < 0.15 || tunnel_h2 < 0.12)
                        && (gy - tunnel_y_center).abs() < 1.5
                        && gy < ground_y - 1.0;
                    let cave_small = (gx * 0.5).sin() * (gy * 0.6).cos() * (gz * 0.5).sin()
                        + (gx * 0.3 + 2.0).cos() * (gy * 0.3).sin() * (gz * 0.4 + 1.0).cos() * 0.5;
                    let vertical_shaft = ((gx * 0.15).sin() * (gz * 0.15).cos()).abs();
                    let is_shaft = vertical_shaft < 0.05 && gy < ground_y - 2.0 && gy > 岩盤の高さ + 0.5;

                    if cave_noise > 0.55 || is_tunnel || cave_small > 0.85 || is_shaft {
                        if gy <= 水面高さ { chunk.set(x, y, z, ボクセル::水); }
                        else { chunk.set(x, y, z, ボクセル::空気); }
                        continue;
                    }

                    let depth = ground_y - gy;
                    let v_type = if is_road && depth < 0.3 { ボクセル種別::道路 }
                        else if depth < 0.3 { ボクセル種別::草 }
                        else if depth < 1.5 { ボクセル種別::土 }
                        else { ボクセル種別::石 };
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
        if x < チャンク解像度 && y < チャンク解像度 && z < チャンク解像度 { self.voxels[Self::idx(x, y, z)] }
        else { ボクセル::空気 }
    }
    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel: ボクセル) {
        if x < チャンク解像度 && y < チャンク解像度 && z < チャンク解像度 {
            self.voxels[Self::idx(x, y, z)] = voxel;
        }
    }
}

pub fn carve_sphere(chunk: &Chunk, cx: f32, cy: f32, cz: f32, radius: f32) -> Chunk {
    let mut new_chunk = chunk.clone();
    let r2 = radius * radius;
    for x in 0..チャンク解像度 { for y in 0..チャンク解像度 { for z in 0..チャンク解像度 {
        let dx = x as f32 - cx; let dy = y as f32 - cy; let dz = z as f32 - cz;
        if dx*dx + dy*dy + dz*dz <= r2 { new_chunk.set(x, y, z, ボクセル::空気); }
    }}}
    new_chunk
}

// src/ボクセル世界.rs
// レイヤー1: コアドメイン型定義

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ボクセル種別 {
    空気, 土, 草, 石, 水, 道路, 橋, 虹, 岩盤,
}

#[derive(Clone, Copy, Debug)]
pub struct ボクセル {
    pub 種別: ボクセル種別,
}

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
    pub fn は透過か(&self) -> bool { self.は空気か() || self.は水か() }
}

// =============================================================================
// 定数
// =============================================================================
pub const チャンク解像度: usize = 32;
pub const ボクセルスケール: f32 = 0.25;
pub const チャンクのワールドサイズ: f32 = チャンク解像度 as f32 * ボクセルスケール; // = 8.0
pub const 水面高さ: f32 = 4.0;
pub const 岩盤の高さ: f32 = 0.5;
pub const 描画距離: i32 = 20;

// =============================================================================
// チャンク (立方体ボクセル格納)
// =============================================================================
#[derive(Clone)]
pub struct チャンク {
    pub ボクセル群: Vec<ボクセル>,
}

impl チャンク {
    pub fn 空で生成() -> Self {
        Self {
            ボクセル群: vec![ボクセル::空気; チャンク解像度 * チャンク解像度 * チャンク解像度],
        }
    }

    fn 添字(x: usize, y: usize, z: usize) -> usize {
        x + y * チャンク解像度 + z * チャンク解像度 * チャンク解像度
    }

    pub fn 取得(&self, x: usize, y: usize, z: usize) -> ボクセル {
        if x < チャンク解像度 && y < チャンク解像度 && z < チャンク解像度 {
            self.ボクセル群[Self::添字(x, y, z)]
        } else {
            ボクセル::空気
        }
    }

    pub fn 設定(&mut self, x: usize, y: usize, z: usize, voxel: ボクセル) {
        if x < チャンク解像度 && y < チャンク解像度 && z < チャンク解像度 {
            self.ボクセル群[Self::添字(x, y, z)] = voxel;
        }
    }
}

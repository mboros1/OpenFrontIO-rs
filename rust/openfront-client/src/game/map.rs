// Direct port of OpenFront's TileRef system
pub type TileRef = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Land,
    Water,
    Mountain,
    Forest,
    Desert,
}

/// Direct port of OpenFront's GameMap
/// Uses a flat array where each tile is a single pixel
#[derive(bevy::prelude::Resource)]
pub struct GameMap {
    width: usize,
    height: usize,
    terrain: Vec<u8>,  // Terrain data (immutable)
    state: Vec<u16>,   // Game state (mutable)
    num_land_tiles: usize,
    
    // Lookup tables for fast coordinate conversion
    ref_to_x: Vec<usize>,
    ref_to_y: Vec<usize>,
    y_to_ref: Vec<TileRef>,
}

impl GameMap {
    // Terrain bits (matching JS implementation)
    const IS_LAND_BIT: u8 = 7;
    const SHORELINE_BIT: u8 = 6;
    const OCEAN_BIT: u8 = 5;
    const MAGNITUDE_MASK: u8 = 0x1f;
    
    // State bits
    const PLAYER_ID_MASK: u16 = 0xfff;
    const FALLOUT_BIT: u16 = 13;
    const DEFENSE_BONUS_BIT: u16 = 14;
    
    pub fn new(width: usize, height: usize, terrain_data: Vec<u8>) -> Self {
        let mut ref_to_x = Vec::with_capacity(width * height);
        let mut ref_to_y = Vec::with_capacity(width * height);
        let mut y_to_ref = Vec::with_capacity(height);
        
        // Build lookup tables
        let mut ref_idx = 0;
        for y in 0..height {
            y_to_ref.push(ref_idx);
            for x in 0..width {
                ref_to_x.push(x);
                ref_to_y.push(y);
                ref_idx += 1;
            }
        }
        
        let num_land_tiles = terrain_data.iter()
            .filter(|&&b| b & (1 << Self::IS_LAND_BIT) != 0)
            .count();
        
        Self {
            width,
            height,
            terrain: terrain_data,
            state: vec![0; width * height],
            num_land_tiles,
            ref_to_x,
            ref_to_y,
            y_to_ref,
        }
    }
    
    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    
    pub fn ref_from_xy(&self, x: usize, y: usize) -> TileRef {
        self.y_to_ref[y] + x
    }
    
    pub fn x(&self, tile_ref: TileRef) -> usize {
        self.ref_to_x[tile_ref]
    }
    
    pub fn y(&self, tile_ref: TileRef) -> usize {
        self.ref_to_y[tile_ref]
    }
    
    pub fn is_valid_coord(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }
    
    pub fn is_land(&self, tile_ref: TileRef) -> bool {
        self.terrain[tile_ref] & (1 << Self::IS_LAND_BIT) != 0
    }
    
    pub fn is_water(&self, tile_ref: TileRef) -> bool {
        !self.is_land(tile_ref)
    }
    
    pub fn is_shore(&self, tile_ref: TileRef) -> bool {
        self.terrain[tile_ref] & (1 << Self::SHORELINE_BIT) != 0
    }
    
    pub fn owner_id(&self, tile_ref: TileRef) -> Option<u16> {
        let id = self.state[tile_ref] & Self::PLAYER_ID_MASK;
        if id == 0 { None } else { Some(id) }
    }
    
    pub fn set_owner_id(&mut self, tile_ref: TileRef, player_id: u16) {
        self.state[tile_ref] = (self.state[tile_ref] & !Self::PLAYER_ID_MASK) | player_id;
    }
    
    pub fn has_fallout(&self, tile_ref: TileRef) -> bool {
        self.state[tile_ref] & (1 << Self::FALLOUT_BIT) != 0
    }
    
    pub fn set_fallout(&mut self, tile_ref: TileRef, value: bool) {
        if value {
            self.state[tile_ref] |= 1 << Self::FALLOUT_BIT;
        } else {
            self.state[tile_ref] &= !(1 << Self::FALLOUT_BIT);
        }
    }
    
    pub fn is_border(&self, tile_ref: TileRef) -> bool {
        if !self.is_land(tile_ref) {
            return false;
        }
        
        let owner = self.owner_id(tile_ref);
        if owner.is_none() {
            return false;
        }
        
        // Check if any neighbor has different owner
        for neighbor in self.neighbors(tile_ref) {
            if self.owner_id(neighbor) != owner {
                return true;
            }
        }
        false
    }
    
    pub fn neighbors(&self, tile_ref: TileRef) -> Vec<TileRef> {
        let mut result = Vec::with_capacity(4);
        let x = self.x(tile_ref) as i32;
        let y = self.y(tile_ref) as i32;
        
        // 4-connected neighbors (up, down, left, right)
        for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            let nx = x + dx;
            let ny = y + dy;
            if self.is_valid_coord(nx, ny) {
                result.push(self.ref_from_xy(nx as usize, ny as usize));
            }
        }
        result
    }
    
    pub fn for_each_tile<F>(&self, mut f: F)
    where
        F: FnMut(TileRef),
    {
        for tile_ref in 0..self.width * self.height {
            f(tile_ref);
        }
    }
}
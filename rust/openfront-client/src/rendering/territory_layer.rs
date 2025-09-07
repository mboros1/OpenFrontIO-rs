use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::game::map::{GameMap, TileRef};

/// Direct port of OpenFront's TerritoryLayer
/// Renders territories as a texture where each pixel represents one tile
pub struct TerritoryLayerPlugin;

impl Plugin for TerritoryLayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TerritoryTexture>()
            .add_systems(Startup, setup_territory_layer)
            .add_systems(Update, (
                update_territory_texture,
                apply_territory_updates,
            ));
    }
}

#[derive(Resource)]
pub struct TerritoryTexture {
    pub image_data: Vec<u8>,
    pub alternative_image_data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub texture_handle: Handle<Image>,
    pub alternative_view: bool,
    pub dirty: bool,
}

impl Default for TerritoryTexture {
    fn default() -> Self {
        Self {
            image_data: Vec::new(),
            alternative_image_data: Vec::new(),
            width: 0,
            height: 0,
            texture_handle: Handle::default(),
            alternative_view: false,
            dirty: true,
        }
    }
}

#[derive(Component)]
pub struct TerritoryLayer;

#[derive(Debug, Clone, Copy)]
pub struct PlayerColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PlayerColor {
    pub fn from_id(player_id: u16) -> Self {
        // Generate player color from ID (similar to JS theme system)
        let hue = ((player_id as f32 * 137.5) % 360.0) / 360.0;
        let (r, g, b) = hsl_to_rgb(hue, 0.7, 0.5);
        Self { r, g, b }
    }
    
    pub fn border_color(&self) -> Self {
        // Darker version for borders
        Self {
            r: (self.r as f32 * 0.7) as u8,
            g: (self.g as f32 * 0.7) as u8,
            b: (self.b as f32 * 0.7) as u8,
        }
    }
    
    pub fn territory_color(&self) -> Self {
        // Lighter version for territory interior
        Self {
            r: self.r.saturating_add(30),
            g: self.g.saturating_add(30),
            b: self.b.saturating_add(30),
        }
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    
    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    ((255.0 * (r + m)) as u8, (255.0 * (g + m)) as u8, (255.0 * (b + m)) as u8)
}

pub fn paint_tile(
    image_data: &mut [u8],
    tile_ref: TileRef,
    color: PlayerColor,
    alpha: u8,
) {
    let offset = tile_ref * 4;
    if offset + 3 < image_data.len() {
        image_data[offset] = color.r;
        image_data[offset + 1] = color.g;
        image_data[offset + 2] = color.b;
        image_data[offset + 3] = alpha;
    }
}

pub fn clear_tile(image_data: &mut [u8], tile_ref: TileRef) {
    let offset = tile_ref * 4;
    if offset + 3 < image_data.len() {
        image_data[offset + 3] = 0; // Set alpha to 0
    }
}

pub fn paint_territory(
    image_data: &mut [u8],
    map: &GameMap,
    tile_ref: TileRef,
) {
    // Check if tile has owner
    if let Some(owner_id) = map.owner_id(tile_ref) {
        let player_color = PlayerColor::from_id(owner_id);
        
        if map.is_border(tile_ref) {
            // Paint border with full opacity
            paint_tile(image_data, tile_ref, player_color.border_color(), 255);
        } else {
            // Paint interior with partial opacity
            paint_tile(image_data, tile_ref, player_color.territory_color(), 150);
        }
    } else if map.has_fallout(tile_ref) {
        // Paint fallout tiles
        let fallout_color = PlayerColor { r: 128, g: 64, b: 0 }; // Brown
        paint_tile(image_data, tile_ref, fallout_color, 150);
    } else {
        // Clear unowned tiles
        clear_tile(image_data, tile_ref);
    }
}

fn setup_territory_layer(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut territory_texture: ResMut<TerritoryTexture>,
) {
    // Initialize with a test map size (will be replaced with actual map data)
    let width = 512;
    let height = 512;
    
    // Create RGBA texture data
    let image_data = vec![0u8; width * height * 4];
    let alternative_image_data = vec![0u8; width * height * 4];
    
    // Create Bevy Image from raw data
    let image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        image_data.clone(),
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );
    
    let texture_handle = images.add(image);
    
    // Update resource
    territory_texture.image_data = image_data;
    territory_texture.alternative_image_data = alternative_image_data;
    territory_texture.width = width;
    territory_texture.height = height;
    territory_texture.texture_handle = texture_handle.clone();
    
    // Create sprite entity for the territory layer
    commands.spawn((
        SpriteBundle {
            texture: texture_handle,
            transform: Transform::from_xyz(0.0, 0.0, 0.0), // Background layer
            sprite: Sprite {
                custom_size: Some(Vec2::new(width as f32, height as f32)),
                ..default()
            },
            ..default()
        },
        TerritoryLayer,
    ));
    
    info!("Territory layer initialized with {}x{} texture", width, height);
}

fn update_territory_texture(
    mut territory_texture: ResMut<TerritoryTexture>,
    map: Option<Res<GameMap>>,
) {
    if !territory_texture.dirty {
        return;
    }
    
    if let Some(map) = map {
        // Update all tiles
        map.for_each_tile(|tile_ref| {
            paint_territory(&mut territory_texture.image_data, &map, tile_ref);
            // Also update alternative view if needed
            // paint_alternative_territory(&mut territory_texture.alternative_image_data, &map, tile_ref);
        });
        
        territory_texture.dirty = false;
    }
}

fn apply_territory_updates(
    territory_texture: Res<TerritoryTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    if !territory_texture.is_changed() {
        return;
    }
    
    if let Some(image) = images.get_mut(&territory_texture.texture_handle) {
        // Update the texture data
        let data = if territory_texture.alternative_view {
            &territory_texture.alternative_image_data
        } else {
            &territory_texture.image_data
        };
        
        image.data = data.clone();
    }
}
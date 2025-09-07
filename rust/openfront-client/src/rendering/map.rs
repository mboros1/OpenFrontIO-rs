use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;

pub struct MapRenderingPlugin;

impl Plugin for MapRenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_map_rendering)
            .add_systems(Update, (
                update_territory_colors,
                update_terrain_tiles,
            ));
    }
}

// Hexagonal tile component
#[derive(Component)]
pub struct HexTile {
    pub q: i32,
    pub r: i32,
    pub terrain_type: TerrainType,
    pub owner: Option<u32>, // Player ID
}

#[derive(Debug, Clone, Copy)]
pub enum TerrainType {
    Land,
    Water,
    Mountain,
    Forest,
    Desert,
}

// Creates a hexagon mesh for rendering
fn create_hex_mesh(radius: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    
    // Center vertex
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([0.5, 0.5]);
    
    // Generate 6 vertices for hexagon corners
    for i in 0..6 {
        let angle = (60.0 * i as f32).to_radians();
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        
        positions.push([x, y, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        
        // UV coordinates
        let u = (x / radius + 1.0) / 2.0;
        let v = (y / radius + 1.0) / 2.0;
        uvs.push([u, v]);
    }
    
    // Create triangles (6 triangles from center to edges)
    let mut indices = Vec::new();
    for i in 0..6 {
        indices.push(0);
        indices.push(i + 1);
        indices.push(if i == 5 { 1 } else { i + 2 });
    }
    
    Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

// Convert hex coordinates to world position
pub fn hex_to_world(q: i32, r: i32, radius: f32) -> Vec3 {
    let x = radius * (3.0_f32.sqrt() * q as f32 + 3.0_f32.sqrt() / 2.0 * r as f32);
    let y = radius * (3.0 / 2.0 * r as f32);
    Vec3::new(x, y, 0.0)
}

fn setup_map_rendering(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let hex_radius = 30.0;
    let hex_mesh = meshes.add(create_hex_mesh(hex_radius));
    
    // Create a small test map (will be replaced with actual map data)
    for q in -5..=5 {
        for r in -5..=5 {
            // Skip tiles outside hex grid bounds
            if q + r < -5 || q + r > 5 {
                continue;
            }
            
            let terrain_type = if (q + r) % 3 == 0 {
                TerrainType::Water
            } else if q == 0 && r == 0 {
                TerrainType::Mountain
            } else {
                TerrainType::Land
            };
            
            let color = match terrain_type {
                TerrainType::Land => Color::srgb(0.4, 0.7, 0.3),
                TerrainType::Water => Color::srgb(0.2, 0.4, 0.8),
                TerrainType::Mountain => Color::srgb(0.6, 0.5, 0.4),
                TerrainType::Forest => Color::srgb(0.2, 0.5, 0.2),
                TerrainType::Desert => Color::srgb(0.9, 0.8, 0.6),
            };
            
            let position = hex_to_world(q, r, hex_radius);
            
            commands.spawn((
                ColorMesh2dBundle {
                    mesh: hex_mesh.clone().into(),
                    material: materials.add(color),
                    transform: Transform::from_translation(position),
                    ..default()
                },
                HexTile {
                    q,
                    r,
                    terrain_type,
                    owner: None,
                },
            ));
        }
    }
    
    info!("Map rendering initialized");
}

fn update_territory_colors(
    mut tiles: Query<(&HexTile, &mut Handle<ColorMaterial>), Changed<HexTile>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (tile, mut material_handle) in tiles.iter_mut() {
        if let Some(owner_id) = tile.owner {
            // Generate player color based on ID (will be replaced with actual player colors)
            let hue = (owner_id as f32 * 137.5) % 360.0;
            let color = Color::hsl(hue, 0.7, 0.5);
            *material_handle = materials.add(color);
        }
    }
}

fn update_terrain_tiles(
    // This will handle terrain texture updates when we add texture support
) {
    // Placeholder for terrain texture updates
}
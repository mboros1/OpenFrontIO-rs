use bevy::prelude::*;

pub struct UnitsRenderingPlugin;

impl Plugin for UnitsRenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, load_unit_sprites)
            .add_systems(Update, (
                spawn_units,
                update_unit_positions,
                animate_units,
            ));
    }
}

#[derive(Component)]
pub struct Unit {
    pub id: u32,
    pub unit_type: UnitType,
    pub owner: u32,
    pub position: Vec2,
    pub troops: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitType {
    Capital,
    City,
    Camp,
    Tower,
    Fort,
    Port,
    Airport,
    Radar,
    Bridge,
}

#[derive(Component)]
pub struct UnitSprite;

#[derive(Component)]
pub struct TroopCount;

#[derive(Resource)]
pub struct UnitAssets {
    pub capital_texture: Handle<Image>,
    pub city_texture: Handle<Image>,
    pub camp_texture: Handle<Image>,
    pub tower_texture: Handle<Image>,
    pub fort_texture: Handle<Image>,
    pub port_texture: Handle<Image>,
    pub airport_texture: Handle<Image>,
    pub radar_texture: Handle<Image>,
    pub bridge_texture: Handle<Image>,
    pub font: Handle<Font>,
}

fn load_unit_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // In production, these would load actual sprite assets
    // For now, we'll create placeholder colored squares
    
    let unit_assets = UnitAssets {
        capital_texture: asset_server.load("sprites/capital.png"),
        city_texture: asset_server.load("sprites/city.png"),
        camp_texture: asset_server.load("sprites/camp.png"),
        tower_texture: asset_server.load("sprites/tower.png"),
        fort_texture: asset_server.load("sprites/fort.png"),
        port_texture: asset_server.load("sprites/port.png"),
        airport_texture: asset_server.load("sprites/airport.png"),
        radar_texture: asset_server.load("sprites/radar.png"),
        bridge_texture: asset_server.load("sprites/bridge.png"),
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
    };
    
    commands.insert_resource(unit_assets);
    
    info!("Unit sprites loaded");
}

fn spawn_units(
    mut commands: Commands,
    units_to_spawn: Query<&Unit, Added<Unit>>,
    unit_assets: Res<UnitAssets>,
) {
    for unit in units_to_spawn.iter() {
        let texture = match unit.unit_type {
            UnitType::Capital => unit_assets.capital_texture.clone(),
            UnitType::City => unit_assets.city_texture.clone(),
            UnitType::Camp => unit_assets.camp_texture.clone(),
            UnitType::Tower => unit_assets.tower_texture.clone(),
            UnitType::Fort => unit_assets.fort_texture.clone(),
            UnitType::Port => unit_assets.port_texture.clone(),
            UnitType::Airport => unit_assets.airport_texture.clone(),
            UnitType::Radar => unit_assets.radar_texture.clone(),
            UnitType::Bridge => unit_assets.bridge_texture.clone(),
        };
        
        // Spawn unit sprite
        let unit_entity = commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(unit.position.x, unit.position.y, 1.0)
                    .with_scale(Vec3::splat(get_unit_scale(unit.unit_type))),
                ..default()
            },
            UnitSprite,
        )).id();
        
        // Add troop count text
        if unit.troops > 0 {
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        format!("{}", unit.troops),
                        TextStyle {
                            font: unit_assets.font.clone(),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    ),
                    transform: Transform::from_xyz(unit.position.x, unit.position.y - 20.0, 2.0),
                    ..default()
                },
                TroopCount,
            ));
        }
    }
}

fn get_unit_scale(unit_type: UnitType) -> f32 {
    match unit_type {
        UnitType::Capital => 1.5,
        UnitType::City => 1.2,
        UnitType::Fort | UnitType::Airport => 1.0,
        _ => 0.8,
    }
}

fn update_unit_positions(
    mut units: Query<(&Unit, &mut Transform), Changed<Unit>>,
) {
    for (unit, mut transform) in units.iter_mut() {
        transform.translation.x = unit.position.x;
        transform.translation.y = unit.position.y;
    }
}

fn animate_units(
    time: Res<Time>,
    mut units: Query<&mut Transform, With<UnitSprite>>,
) {
    // Simple idle animation - gentle bobbing
    for mut transform in units.iter_mut() {
        let offset = (time.elapsed_seconds() * 2.0).sin() * 2.0;
        transform.translation.y += offset * time.delta_seconds();
    }
}

// Attack animation component
#[derive(Component)]
pub struct AttackAnimation {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub duration: f32,
    pub elapsed: f32,
}

pub fn animate_attacks(
    mut commands: Commands,
    time: Res<Time>,
    mut animations: Query<(Entity, &mut AttackAnimation, &mut Transform)>,
) {
    for (entity, mut animation, mut transform) in animations.iter_mut() {
        animation.elapsed += time.delta_seconds();
        
        let progress = (animation.elapsed / animation.duration).min(1.0);
        
        // Interpolate position
        let position = animation.start_pos.lerp(animation.end_pos, progress);
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        
        // Remove animation when complete
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}
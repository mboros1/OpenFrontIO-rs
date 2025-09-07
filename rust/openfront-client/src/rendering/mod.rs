use bevy::prelude::*;

pub mod map;
pub mod units;
pub mod ui;
pub mod territory_layer;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(territory_layer::TerritoryLayerPlugin)
            .add_plugins(units::UnitsRenderingPlugin)
            .add_plugins(ui::UIRenderingPlugin);
    }
}
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub const FLORA_DATA_CORE: &str = include_str!("../../assets/progression/flora.json");

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "player/player.png")]
    pub player: Handle<Image>,

    // --- UI ---
    #[asset(
        paths(
            "ui/icons/potatoe_icon.png",
            "ui/icons/raddish_icon.png",
            "ui/icons/carrot_icon.png",
            "ui/icons/sunflower_icon.png",
        ),
        collection(typed)
    )]
    pub flora_icons: Vec<Handle<Image>>,

    // --- MAP ---
    #[asset(path = "map/grass.png")]
    pub grass: Handle<Image>,

    #[asset(
        paths(
            "map/potatoe.png",
            "map/raddish.png",
            "map/carrot.png",
            "map/sunflower.png"
        ),
        collection(typed)
    )]
    pub flora_images: Vec<Handle<Image>>,

    #[asset(path = "map/tree.png")]
    pub tree: Handle<Image>,

    // --- FONTS ---
    #[asset(path = "fonts/PressStart2P.ttf")]
    pub pixel_font: Handle<Font>,
}

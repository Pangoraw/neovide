use crate::settings::*;

pub use super::keyboard::KeyboardSettings;

#[derive(Clone, SettingGroup)]
pub struct WindowSettings {
    pub refresh_rate: u64,
    pub transparency: f32,
    pub no_idle: bool,
    pub fullscreen: bool,
    pub iso_layout: bool,
    pub remember_dimensions: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            refresh_rate: 60,
            transparency: 1.0,
            no_idle: SETTINGS
                .neovim_arguments
                .contains(&String::from("--noIdle")),
            fullscreen: false,
            iso_layout: false,
            remember_dimensions: false,
        }
    }
}

#[derive(Debug, Clone, Default, SettingGroup)]
pub struct CacheSettings {
    pub stdpath: String,
}

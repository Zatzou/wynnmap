use std::{collections::BTreeMap, sync::Arc};

use gloo_storage::Storage;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::coordfmt::CoordFmt;

#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    pub terrs: TerrSettings,
    pub map: MapSettings,
    pub sidebar: SidebarSettings,

    pub copy_coordfmt: RwSignal<CoordFmt>,

    pub gather: GatherSettings,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct TerrSettings {
    pub show_gtags: RwSignal<bool>,
    pub show_resicons: RwSignal<bool>,
    pub show_timers: RwSignal<bool>,
}

impl Default for TerrSettings {
    fn default() -> Self {
        Self {
            show_gtags: RwSignal::new(true),
            show_resicons: RwSignal::new(true),
            show_timers: RwSignal::new(true),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct MapSettings {
    pub show_terrs: RwSignal<bool>,
    pub show_conns: RwSignal<bool>,
    pub show_non_main_areas: RwSignal<bool>,
}

impl Default for MapSettings {
    fn default() -> Self {
        Self {
            show_terrs: RwSignal::new(true),
            show_conns: RwSignal::new(true),
            show_non_main_areas: RwSignal::new(false),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct SidebarSettings {
    pub show_gleaderboard: RwSignal<bool>,
}

impl Default for SidebarSettings {
    fn default() -> Self {
        Self {
            show_gleaderboard: RwSignal::new(true),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GatherSettings {
    pub show_res: RwSignal<BTreeMap<Arc<str>, RwSignal<bool>>>,
}

#[derive(Clone)]
pub struct SettingsCtx(pub Arc<Settings>);

/// Function for loading the settings from local storage and providing them to the context. This function should be called once at the start of the application.
pub fn provide_settings() {
    // delete old settings variables
    gloo_storage::LocalStorage::delete("settings");

    let settings: Settings = gloo_storage::LocalStorage::get("settings_v1").unwrap_or_default();

    let settings = Arc::new(settings);

    let signal = RwSignal::new(settings.clone());

    Effect::new(move || {
        gloo_storage::LocalStorage::set("settings_v1", signal).expect("failed to save settings");
    });

    provide_context(SettingsCtx(settings));
}

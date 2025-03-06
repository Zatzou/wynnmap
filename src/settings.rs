use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use gloo_storage::Storage;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    toggles: HashMap<Arc<str>, bool>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            toggles: Default::default(),
        }
    }
}

#[derive(Clone)]
struct SettingsContext(RwSignal<Settings>);

pub fn provide_settings() {
    let settings: Settings = gloo_storage::LocalStorage::get("settings").unwrap_or_default();

    let signal = RwSignal::new(settings);

    Effect::new(move || {
        update_settings(signal.get());
    });

    provide_context(SettingsContext(signal));
}

fn update_settings(settings: Settings) {
    gloo_storage::LocalStorage::set("settings", settings).expect("failed to save settings");
}

static TOGGLES: LazyLock<Mutex<HashMap<Arc<str>, RwSignal<bool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn use_toggle(name: &'static str, default: bool) -> RwSignal<bool> {
    let mut toggles = TOGGLES.lock().unwrap();

    if let Some(signal) = toggles.get(name) {
        let signal = signal.clone();
        drop(toggles);
        return signal;
    } else {
        let settings = use_context::<SettingsContext>()
            .expect("attempted to use toggle outside of settings context")
            .0;

        let option = settings
            .read()
            .toggles
            .get(name)
            .copied()
            .unwrap_or(default);

        let signal = RwSignal::new(option);

        toggles.insert(name.into(), signal.clone());
        drop(toggles);

        Effect::new(move || {
            settings.write().toggles.insert(name.into(), signal.get());
        });

        signal
    }
}

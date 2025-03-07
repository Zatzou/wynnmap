use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use gloo_storage::Storage;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Settings {
    toggles: HashMap<Arc<str>, bool>,
}

#[derive(Clone)]
struct SettingsContext(RwSignal<Settings>);

/// Function for loading the settings from local storage and providing them to the context. This function should be called once at the start of the application.
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

/// The static variable for storing the toggle signals which are currently in use.
static TOGGLES: LazyLock<Mutex<HashMap<Arc<str>, RwSignal<bool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// A function that retrieves a signal for a given toggle setting. If the signal or setting doesn't yet exist it will be created with the given default value.
///
/// # Arguments
///
/// * `name` - The name of the toggle setting.
/// * `default` - The default value of the toggle setting.
pub fn use_toggle(name: &'static str, default: bool) -> RwSignal<bool> {
    let mut toggles = TOGGLES.lock().unwrap();

    // check if the signal already exists
    if let Some(signal) = toggles.get(name) {
        let signal = *signal;

        // check that the signal hasn't been disposed and if it has been then generate a new one
        if !signal.is_disposed() {
            drop(toggles);
            return signal;
        }
    }

    // get the settings context
    let settings = use_context::<SettingsContext>()
        .expect("attempted to use toggle outside of settings context")
        .0;

    // get the toggle value from the settings and fall back to the default if it doesn't exist
    let option = settings
        .read_untracked()
        .toggles
        .get(name)
        .copied()
        .unwrap_or(default);

    // create a new signal with the value from the settings
    let signal = RwSignal::new(option);

    // insert the signal into the toggles map
    toggles.insert(name.into(), signal);
    drop(toggles);

    // create an effect to update the settings when the signal changes
    Effect::new(move || {
        settings.write().toggles.insert(name.into(), signal.get());
    });

    signal
}

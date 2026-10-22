use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub enum MapTileFilter {
    #[default]
    Normal,
    LowContrast,
    Grayscale,
    GrayScaleLowcontrast,
}

impl MapTileFilter {
    pub fn get_filter(&self) -> Option<&'static str> {
        match self {
            MapTileFilter::Normal => None,
            MapTileFilter::LowContrast => Some("contrast(0.8) brightness(0.4)"),
            MapTileFilter::Grayscale => Some("grayscale(1)"),
            MapTileFilter::GrayScaleLowcontrast => {
                Some("grayscale(1) contrast(0.8) brightness(0.4)")
            }
        }
    }
}

#[component]
pub fn MapTileFilterSelector(filt: RwSignal<MapTileFilter>) -> impl IntoView {
    view! {
        <select
            class="select"
            on:change:target=move |ev| {
                filt.set(match ev.target().value().as_str() {
                    "n" => MapTileFilter::Normal,
                    "lc" => MapTileFilter::LowContrast,
                    "g" => MapTileFilter::Grayscale,
                    "lcg" => MapTileFilter::GrayScaleLowcontrast,
                    _ => unreachable!()
                });
            }
            prop:value=move || match filt.get() {
                MapTileFilter::Normal => "n",
                MapTileFilter::LowContrast => "lc",
                MapTileFilter::Grayscale => "g",
                MapTileFilter::GrayScaleLowcontrast => "lcg"
            }
        >
            <option value="n">"Normal"</option>
            <option value="lc">"Low contrast"</option>
            <option value="g">"Grayscale"</option>
            <option value="lcg">"Low contrast grayscale"</option>
        </select>
    }
}

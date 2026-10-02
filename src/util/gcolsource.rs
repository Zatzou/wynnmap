use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wynnmap_types::{color::Color, terr::TerrState};

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub enum GColSource {
    #[default]
    Guild,
    Treasury,
    Defences,
}

impl GColSource {
    pub fn get_color(self, state: &TerrState) -> Color {
        match self {
            GColSource::Guild => state.guild.get_color(),
            GColSource::Treasury => state.treasury.color(),
            GColSource::Defences => state.defences.color(),
        }
    }
}

#[component]
pub fn GColSourceSelector(src: RwSignal<GColSource>) -> impl IntoView {
    view! {
        <select
            class="select"
            on:change:target=move |ev| {
                src.set(match ev.target().value().as_str() {
                    "Guild" => GColSource::Guild,
                    "Treasury" => GColSource::Treasury,
                    "Defences" => GColSource::Defences,
                    _ => unreachable!()
                });
            }
            prop:value=move || match src.get() {
                GColSource::Guild => "Guild",
                GColSource::Treasury => "Treasury",
                GColSource::Defences => "Defences",
            }
        >
            <option value="Guild">"Guild"</option>
            <option value="Treasury">"Treasury"</option>
            <option value="Defences">"Defences"</option>
        </select>
    }
}

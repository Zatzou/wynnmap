use std::{collections::HashMap, sync::Arc};

use leptos::prelude::*;
use uuid::Uuid;
use wynnmap_types::Guild;

use crate::{
    components::{
        sidebar::Sidebar,
        sidecard::{
            SideCardHover,
            terr::{GuildName, TerrInfo},
        },
    },
    datasource,
    settings::use_toggle,
    wynnmap::{WynnMap, conns::Connections, maptile::DefaultMapTiles, terrs::TerrView},
};

#[component]
pub fn PlanningMap() -> impl IntoView {
    let show_conns = use_toggle("conns", true);

    let extradata =
        LocalResource::new(async move || datasource::get_extra_terr_info().await.unwrap());

    let terrs = LocalResource::new(async move || datasource::get_terrs().await.unwrap());

    let extradata = move || extradata.get().map_or_else(HashMap::new, |t| t.take());

    let terrs = Memo::new(move |_| terrs.get().map_or_else(HashMap::new, |t| t.take()));

    let guilds: RwSignal<Vec<Guild>> = RwSignal::new(Vec::new());
    let owned: RwSignal<HashMap<Arc<str>, u8>> = RwSignal::new(HashMap::new());

    let mapterrs = move || {
        let mut terrs = terrs.get();

        for (name, terr) in terrs.iter_mut() {
            if let Some(owner) = owned.with(|o| o.get(name).copied()) {
                if let Some(guild) = guilds.with(|g| g.get(usize::from(owner)).cloned()) {
                    terr.guild = guild;
                    continue;
                }
            }

            terr.guild = Guild::default();
        }

        terrs
    };

    let hovered = RwSignal::new(None);
    let selected = RwSignal::new(None);

    view! {
        <WynnMap>
            <DefaultMapTiles />

            // conns
            <Show when={move || show_conns.get()}>
                <Connections terrs={terrs} extradata={Signal::derive(extradata)} />
            </Show>

            // territories
            <TerrView terrs={Signal::derive(mapterrs)} extradata={Signal::derive(extradata)} hovered=hovered selected=selected hide_timers=true />
        </WynnMap>

        // hover box
        {move || if let Some(hovered) = hovered.get() {
            if selected.get().is_some() {
                return None;
            }

            let hovered = Signal::derive(move || hovered.clone());

            Some(view! {
                <SideCardHover>
                    <TerrInfo
                        name={hovered}
                        extradata={Signal::derive(extradata)}
                    />
                    <hr class="border-neutral-600" />
                    <GuildName
                        guild={Signal::derive(move || {
                            guilds.read().get(usize::from(owned.with(|o| o.get(&hovered.get()).copied().unwrap_or(255)))).cloned().unwrap_or_default()
                        })}
                    />
                </SideCardHover>
            })
        } else {None}}

        <Sidebar>

        </Sidebar>
    }
}

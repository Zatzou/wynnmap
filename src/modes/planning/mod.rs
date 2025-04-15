use std::{collections::HashMap, sync::Arc};

use leptos::prelude::*;
use uuid::Uuid;
use wynnmap_types::Guild;

use crate::{
    components::sidebar::Sidebar,
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

            terr.guild = Guild {
                uuid: Uuid::nil(),
                name: Arc::from("None"),
                prefix: Arc::from("None"),
                color: Some(Arc::from("#FFFFFF")),
            }
        }

        terrs
    };

    view! {
        <WynnMap>
            <DefaultMapTiles />

            // conns
            <Show when={move || show_conns.get()}>
                <Connections terrs={terrs} extradata={Signal::derive(extradata)} />
            </Show>

            // territories
            <TerrView terrs={Signal::derive(mapterrs)} extradata={Signal::derive(extradata)} hide_timers=true />
        </WynnMap>

        <Sidebar>

        </Sidebar>
    }
}

use std::{collections::HashMap, sync::Arc};

use leptos::prelude::*;
use wynnmap_types::Guild;

use crate::{
    components::{
        sidebar::Sidebar,
        sidecard::{
            SideCard, SideCardHover,
            terr::{GuildName, TerrInfo},
        },
    },
    datasource,
    dialog::{self, show_dialog},
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

    let guilds: RwSignal<Vec<Guild>> = RwSignal::new(vec![Guild::default()]);
    // key = terr name, value = guild tag
    let owned: RwSignal<HashMap<Arc<str>, Arc<str>>> = RwSignal::new(HashMap::new());

    let mapterrs = move || {
        let mut terrs = terrs.get();

        for (name, terr) in terrs.iter_mut() {
            if let Some(owner) = owned.with(|o| o.get(name).cloned()) {
                if let Some(guild) = guilds
                    .read()
                    .iter()
                    .find(|g| g.prefix == Some(owner.clone()))
                    .cloned()
                {
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
                            guilds.read().iter().find(|g| owned.with(|o| g.prefix == o.get(&hovered.get()).cloned())).cloned().unwrap_or_default()
                        })}
                    />
                </SideCardHover>
            })
        } else {None}}

        <Sidebar>
            <button class="p-2 m-2 border-neutral-600 border rounded-md hover:bg-neutral-700" on:click={
                let owner = Owner::new();
                move |_| {
                    owner.with(move || {
                        show_dialog(move || dialog::planning::manage_guilds(guilds));
                    });
                }
            }>
                "Manage Guilds"
            </button>

            <button class="p-2 m-2 border-neutral-600 border rounded-md hover:bg-neutral-700" on:click={
                let owner = Owner::new();
                move |_| {
                    owner.with(move || {
                        // show_dialog(move || dialog::planning::manage_terrs(guilds));
                    });
                }
            }>
                "Import/Export"
            </button>

        </Sidebar>

        // selected terr info
        {move || selected.get().map(|sel| {
            let sel = Signal::derive(move || sel.clone());

            Some(view! {
                <SideCard closefn={move || selected.set(None)}>
                    <TerrInfo
                        name={sel}
                        extradata={Signal::derive(extradata)}
                    />
                    <hr class="border-neutral-600" />
                    <GuildSelect
                        terr_name={sel}
                        terr_owners={owned}
                        guilds={guilds}
                    />
                </SideCard>
            })
        })}
    }
}

#[component]
pub fn GuildSelect(
    terr_name: Signal<Arc<str>>,
    terr_owners: RwSignal<HashMap<Arc<str>, Arc<str>>>,
    #[prop(into)] guilds: RwSignal<Vec<Guild>>,
) -> impl IntoView {
    let owner = move || {
        // find the tag of the guild which owns the terr
        terr_owners.with(|o| o.get(&terr_name.get()).cloned())
    };

    let onselect = move |sel: String| {
        terr_owners.update(|o| {
            o.insert(terr_name.get().clone(), Arc::from(sel));
        });
    };

    view! {
        <div class="p-2">
            <select class="text-xl p-1 rounded border-1 border-neutral-600" on:input:target=move |ev| onselect(ev.target().value())>
                <ForEnumerate
                    each=move || guilds.get()
                    key=|guild| guild.clone()
                    children=move |_, guild| {
                        let prefix = guild.prefix.clone();

                        view! {
                            <option value={prefix.clone()} selected={move || owner() == prefix.clone()}>
                                {guild.name} " ["{guild.prefix}"]"
                            </option>
                        }
                    }
                />
            </select>
        </div>
    }
}

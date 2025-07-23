use std::{collections::HashMap, sync::Arc};

use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_router::hooks::use_location;
use wynnmap_types::{
    guild::Guild,
    terr::{TerrOwner, Territory},
};

use crate::{
    components::{
        loader::loader,
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
    let terrs = LocalResource::new(async move || datasource::get_terrs().await);

    move || loader(terrs, |terrs| planningmap_inner(terrs).into_any())
}

fn planningmap_inner(terrs: HashMap<Arc<str>, Territory>) -> impl IntoView {
    let location = use_location();
    // read a share string from the url
    let sharedata = move || {
        // get the hash part of the url
        let hash = Some(location.hash.get()).filter(|s| !s.is_empty());

        // remove the # from the start
        let hash = hash.map(|h| h.replace('#', ""));

        console_log(&format!("{hash:?}"));

        // decode the data
        hash.map(dialog::planning::formats::urlshare::ShareUrlData::decode_string)
    };

    // handle errors
    let sharedata = move || {
        if let Some(data) = sharedata() {
            match data {
                Ok(data) => Some(data),
                Err(err) => {
                    show_dialog(move || {
                        dialog::info::info(
                            String::from("Failed to read share URL"),
                            view! {
                                <pre>{format!("{err}")}</pre>
                            },
                        )
                    });

                    let _ = window().location().set_hash("");
                    None
                }
            }
        } else {
            None
        }
    };

    // console_log(&format!("{:?}", sharedata));

    let show_conns = use_toggle("conns", true);

    let terrs = RwSignal::new(terrs);

    let guilds: RwSignal<Vec<ArcRwSignal<Guild>>> =
        RwSignal::new(vec![ArcRwSignal::new(Guild::default())]);
    let owned: RwSignal<HashMap<Arc<str>, ArcRwSignal<Guild>>> = RwSignal::new(HashMap::new());

    // apply the share string when territories have loaded
    Effect::new(move || {
        if !terrs.read().is_empty()
            && let Some(data) = sharedata()
        {
            if data.verify_terrhash(&terrs.read()) {
                let (gu, ow) = data.into_data(&terrs.read());

                guilds.set(gu);
                owned.set(ow);

                // ensure that the sharedata is only decoded once
                let _ = window().location().set_hash("");
            } else {
                show_dialog(move || {
                    dialog::info::info(
                        String::from("Territory hash mismatch"),
                        view! {
                            <p>"Territories have changed since the creation of this share URL. This URL can no longer be decoded."</p>
                        },
                    )
                });
            }
        }
    });

    let mapowneds = Memo::new(move |_| {
        let mut owners = HashMap::new();

        for (tname, owner) in &*owned.read() {
            owners.insert(
                tname.clone(),
                TerrOwner {
                    guild: owner.get(),
                    acquired: None,
                },
            );
        }

        owners
    });

    let hovered = RwSignal::new(None);
    let selected = RwSignal::new(None);

    view! {
        <WynnMap>
            <DefaultMapTiles />

            // conns
            <Show when={move || show_conns.get()}>
                <Connections terrs={terrs} />
            </Show>

            // territories
            <TerrView terrs={terrs} owners={mapowneds} hovered=hovered selected=selected hide_timers=true />
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
                        terrs={terrs}
                    />
                    <hr class="border-neutral-600" />
                    <GuildName
                        guild={Signal::derive(move || {
                            owned.read().get(&*hovered.read()).map(|g| g.get()).unwrap_or_default()
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
                        show_dialog(move || dialog::planning::save_dialog(terrs.into(), guilds, owned));
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
                        terrs={terrs}
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
    terr_owners: RwSignal<HashMap<Arc<str>, ArcRwSignal<Guild>>>,
    #[prop(into)] guilds: RwSignal<Vec<ArcRwSignal<Guild>>>,
) -> impl IntoView {
    // find the index of the current owner if any otherwise default to none
    let owner = move || {
        // get the owner value of the territory
        terr_owners
            .with(|o| o.get(&terr_name.get()).cloned())
            .and_then(|e| {
                // if we have an owner value find the index of the owner in the guilds list
                guilds
                    .read()
                    .iter()
                    .enumerate()
                    .find(|(_, g)| *g == &e)
                    .map(|(i, _)| i)
            })
            .unwrap_or(0)
    };

    let onselect = move |sel: String| {
        if let Ok(idx) = sel.parse::<usize>() {
            if let Some(guild) = guilds.read().get(idx) {
                terr_owners.update(|o| {
                    o.insert(terr_name.get(), guild.clone());
                });
            }
        }
    };

    view! {
        <div class="p-2">
            <select class="text-xl p-1 rounded border-1 border-neutral-600" on:input:target=move |ev| onselect(ev.target().value())>
                <ForEnumerate
                    each=move || guilds.get()
                    key=|guild| guild.get()
                    children=move |idx, guild| {
                        view! {
                            <option value={idx} selected={move || owner() == idx.get()}>
                                {guild.get().name} " ["{guild.get().prefix}"]"
                            </option>
                        }
                    }
                />
            </select>
        </div>
    }
}

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use jiff::SignedDuration;
use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::terr::{MapState, TerrTimestamps, Territory};

use crate::{
    components::{
        checkbox::Checkbox,
        gleaderboard::Gleaderboard,
        sidebar::Sidebar,
        sidecard::{SideCard, terr::TerrStats},
    },
    datasource,
    dialog::{Dialogs, info::info_dialog},
    modes::war::calc::TerrCalc,
    sectimer::SecondTimer,
    settings::SettingsCtx,
    util::{GColSourceSelector, fmt_time_short},
    wynnmap::{
        OnCtxMenu, WynnMap, conns::Connections, maptile::WithDefaultMapTiles, terrs::TerrView,
    },
};

mod calc;

#[component]
pub fn WarMap() -> impl IntoView {
    let SettingsCtx(settings) = expect_context();
    let dialogs = use_context::<Dialogs>().expect("Dialogs context not found");

    let show_terrs = settings.map.show_terrs;
    let show_conns = settings.map.show_conns;
    let show_res = settings.terrs.show_resicons;
    let show_timers = settings.terrs.show_timers;
    let show_guild_leaderboard = settings.sidebar.show_gleaderboard;

    let terrs = RwSignal::new(BTreeMap::new());
    let state = RwSignal::new(BTreeMap::new());
    let last_updated = RwSignal::new(TerrTimestamps::default());

    let load_terrs = move |terrs: RwSignal<_>| async move {
        match datasource::load_json::<BTreeMap<Arc<str>, Territory>>("/api/v3/terr/list").await {
            Ok(data) => terrs.set(data),
            Err(err) => {
                dialogs.add_if_not_exist("err_maptiles", move || {
                    info_dialog(
                        String::from("Failed to load territory data"),
                        view! {
                            <p>"An error occured while loading api data"</p>
                            <pre class="p-2 bg-neutral-800 rounded my-1">{format!("{err:?}")}</pre>
                        },
                    )
                });
            }
        }
    };

    spawn_local(load_terrs(terrs));

    let load_owners = move || async move {
        match datasource::load_json::<MapState>("/api/v3/terr/state").await {
            Ok(data) => {
                state.set(data.terrs);
                last_updated.set(data.timestamps);
            }
            Err(err) => {
                dialogs.add_if_not_exist("err_maptiles", move || {
                    info_dialog(
                        String::from("Failed to load territory data"),
                        view! {
                            <p>"An error occured while loading api data"</p>
                            <pre class="p-2 bg-neutral-800 rounded my-1">{format!("{err:?}")}</pre>
                        },
                    )
                });
            }
        }
    };

    spawn_local(load_owners());

    datasource::sse_terr_updates(state, last_updated);

    let hovered = RwSignal::new(None);
    let selected = RwSignal::new(None);

    let SecondTimer(now) = expect_context();
    let data_age = Memo::new(move |_| {
        last_updated
            .read()
            .updated
            .map_or(SignedDuration::ZERO, |updated| {
                now.read().duration_since(updated)
            })
    });

    // Update the territory data every 10 minutes to ensure the map stays up to date
    let terr_data_updater = set_interval_with_handle(
        move || {
            spawn_local(load_terrs(terrs));
        },
        Duration::from_mins(10),
    )
    .ok();

    on_cleanup(move || {
        if let Some(i) = terr_data_updater {
            i.clear();
        }
    });

    // update the selected territory on click
    let onclick = Callback::new(move |pos| {
        selected.set(
            terrs
                .read()
                .iter()
                .find(|(_, t)| t.location.contains(pos))
                .map(|(n, _)| n.clone()),
        );
    });

    let onctx: OnCtxMenu = (move |pos: RwSignal<[i32; 2]>, close: Callback<()>| {
        let under = terrs
            .read()
            .iter()
            .find(|(_, t)| t.location.contains(pos.get()))
            .and_then(|(n, _)| state.read().get(n).cloned());

        under
            .map(|terr| {
                let name = terr.guild.name.clone();
                let link = move || format!("https://wynncraft.com/stats/guild/{}", name);

                view! {
                    <a href=link on:click=move |_| close.run(()) class="ctxmenu-btn" target="_blank">
                        <icons::ExternalLink/>
                        "Open "{terr.guild.name}" on Wynncraft"
                    </a>
                }
            })
            .into_any()
    })
    .into();

    view! {
        <WynnMap onclick=onclick onctxmenu=onctx>
            <WithDefaultMapTiles />

            // conns
            <Show when={move || show_conns.get()}>
                <Connections terrs />
            </Show>

            // territories
            <Show when={move || show_terrs.get()}>
                <TerrView terrs state hovered />
            </Show>
        </WynnMap>

        // hover box
        {move || if let Some(hovered) = hovered.get() {
            if selected.get().is_some() {
                return None;
            }

            Some(view! {
                <SideCard hover=true>
                    <TerrStats
                        name={hovered}
                        terrs
                        state
                    />
                </SideCard>
            })
        } else {None}}

        // outdated data warning
        {move || if *data_age.read() > SignedDuration::from_mins(10) {
            Some(view! {
                <div class="fixed bottom-4 right-4 bg-neutral-900 text-white rounded-md w-sm p-2">
                    <h2 class="text-2xl">"Warning: territory data is outdated"</h2>
                    <p>"Data was last updated " {move || {
                        fmt_time_short(data_age.get())
                    }} " ago"</p>
                    <p>"If this issue does not resolve within an hour and Wynn isn't having api issues contact the developer."</p>
                </div>
            })
        } else {None}}

        <Sidebar>
            // checkboxes
            <div class="flex-1 flex flex-col gap-2 p-2">
                <div class="flex flex-col gap-2">
                    <Checkbox id="terrs" checked={show_terrs}>"Territories"</Checkbox>
                    <div class="flex flex-col gap-2 ml-6" class:hidden={move || !show_terrs.get()}>
                        <Checkbox id="resico" checked={show_res}>"Resource icons"</Checkbox>
                        <Checkbox id="timers" checked={show_timers}>"Timers"</Checkbox>
                    </div>
                </div>
                <Checkbox id="conns" checked={show_conns}>"Connections"</Checkbox>
                <div>
                    "Territory color: "
                    <GColSourceSelector src=settings.terrs.terr_color/>
                </div>
                <div>
                    "Timer color: "
                    <GColSourceSelector src=settings.terrs.timer_color/>
                </div>
            </div>

            // guild leaderboard
            <Gleaderboard state show_guild_leaderboard/>
        </Sidebar>

        // selected terr info
        {move || selected.get().map(|sel| {
            Some(view! {
                <SideCard on_close=move |_| selected.set(None)>
                    <TerrStats name={sel.clone()} terrs state />

                    <TerrCalc name={sel} terrs state />
                </SideCard>
            })
        })}
    }
}

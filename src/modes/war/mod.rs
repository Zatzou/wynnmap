use std::{collections::BTreeMap, time::Duration};

use chrono::TimeDelta;
use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::terr::TerrTimestamps;

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
    settings::use_toggle,
    util::fmt_time_short,
    wynnmap::{WynnMap, conns::Connections, maptile::DefaultMapTiles, terrs::TerrView},
};

mod calc;

#[component]
pub fn WarMap() -> impl IntoView {
    let dialogs = use_context::<Dialogs>().expect("Dialogs context not found");

    let show_terrs = use_toggle("terrs", true);
    let show_conns = use_toggle("conns", true);
    let show_res = use_toggle("resico", true);
    let show_timers = use_toggle("timers", true);
    let show_guild_leaderboard = use_toggle("gleaderboard", true);

    let terrs = RwSignal::new(BTreeMap::new());
    let state = RwSignal::new(BTreeMap::new());
    let last_updated = RwSignal::new(TerrTimestamps::default());

    let load_terrs = move |terrs: RwSignal<_>| async move {
        match datasource::get_terrs().await {
            Ok(data) => terrs.set(data),
            Err(err) => {
                if !dialogs.contains("err_maptiles") {
                    dialogs.add("err_maptiles", move || {
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
        }
    };

    spawn_local(load_terrs(terrs));

    let load_owners = move || async move {
        match datasource::get_state().await {
            Ok(data) => {
                state.set(data.terrs);
                last_updated.set(data.timestamps);
            }
            Err(err) => {
                if !dialogs.contains("err_maptiles") {
                    dialogs.add("err_maptiles", move || {
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
        }
    };

    spawn_local(load_owners());

    datasource::ws_terr_updates(state, last_updated);

    let hovered = RwSignal::new(None);
    let selected = RwSignal::new(None);

    let SecondTimer(now) = expect_context();
    let data_age = Memo::new(move |_| {
        last_updated
            .read()
            .updated
            .map_or_else(TimeDelta::zero, |updated| {
                now.read().signed_duration_since(updated)
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

    view! {
        <WynnMap onclick=onclick>
            <DefaultMapTiles />

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
        {move || if *data_age.read() > TimeDelta::minutes(10) {
            Some(view! {
                <div class="fixed bottom-4 right-4 bg-neutral-900 text-white rounded-md w-sm p-2">
                    <h1 class="text-2xl">"Warning: territory data is outdated"</h1>
                    <p>"Data was last updated " {move || {
                        fmt_time_short(data_age.get())
                    }} " ago"</p>
                    <p>"If this issue does not resolve within an hour and Wynn isn't having api issues contact the developer."</p>
                </div>
            })
        } else {None}}

        <Sidebar>
            // checkboxes
            <div class="flex-1 flex flex-col gap-2 p-2 text-lg">
                <div>
                    <Checkbox id="terrs" checked={show_terrs}>"Territories"</Checkbox>
                    <div class="flex flex-col gap-1 ml-6" class:hidden={move || !show_terrs.get()}>
                        <Checkbox id="resico" checked={show_res}>"Resource icons"</Checkbox>
                        <Checkbox id="timers" checked={show_timers}>"Timers"</Checkbox>
                    </div>
                </div>
                <Checkbox id="conns" checked={show_conns}>"Connections"</Checkbox>
            </div>

            // guild leaderboard
            <div class="flex flex-col min-h-0">
                <hr class="border-neutral-600" />
                <div class="flex justify-between items-center text-xl p-2 py-1 cursor-pointer" on:click={move |_| show_guild_leaderboard.set(!show_guild_leaderboard.get())}>
                    <h2>"Guild leaderboard"</h2>
                    <Show when=move || !show_guild_leaderboard.get()><lucide_leptos::ChevronUp size=24/></Show>
                    <Show when=move || show_guild_leaderboard.get()><lucide_leptos::ChevronDown size=24/></Show>
                </div>
                <div class="overflow-y-auto shrink min-h-0" class:hidden={move || !show_guild_leaderboard.get()}>
                    <hr class="border-neutral-600"/>
                    <Gleaderboard state/>
                </div>
            </div>
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

use std::{collections::HashMap, sync::Arc, time::Duration};

use leptos::prelude::*;
use wynnmap_types::{ExTerrInfo, Territory};

use crate::{
    components::{checkbox::Checkbox, gleaderboard::Gleaderboard, sidebar::Sidebar},
    datasource,
    settings::use_toggle,
    wynnmap::{WynnMap, conns::Connections, maptile::DefaultMapTiles, terrs::TerrView},
};

#[component]
pub fn WarMap() -> impl IntoView {
    let show_terrs = use_toggle("terrs", true);
    let show_conns = use_toggle("conns", true);
    let show_res = use_toggle("resico", true);
    let show_timers = use_toggle("timers", true);
    let show_guild_leaderboard = use_toggle("gleaderboard", true);

    let extradata =
        LocalResource::new(async move || datasource::get_extra_terr_info().await.unwrap());

    let extradata = move || extradata.get().map_or_else(HashMap::new, |t| t.take());

    let terrs = RwSignal::new(HashMap::new());

    datasource::ws_terr_changes(terrs).unwrap();

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
            <Show when={move || show_terrs.get()}>
                <TerrView terrs={terrs} extradata={Signal::derive(extradata)} hovered=hovered selected=selected />
            </Show>
        </WynnMap>

        <Sidebar>
            // checkboxes
            <div class="flex-1 flex flex-col p-2 text-lg">
                <Checkbox id="terrs" checked={show_terrs}>"Territories"</Checkbox>
                <div class="ml-6" class:hidden={move || !show_terrs.get()}>
                    <Checkbox id="resico" checked={show_res}>"Resource icons"</Checkbox>
                    <Checkbox id="timers" checked={show_timers}>"Timers"</Checkbox>
                </div>
                <Checkbox id="conns" checked={show_conns}>"Connections"</Checkbox>
            </div>

            // guild leaderboard
            <div class="flex flex-col min-h-0">
                <hr class="border-neutral-600" />
                <div class="flex justify-between items-center text-xl p-2 py-1" on:click={move |_| show_guild_leaderboard.set(!show_guild_leaderboard.get())}>
                    <h2>"Guild leaderboard"</h2>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6 cursor-pointer" >
                        <path stroke-linecap="round" stroke-linejoin="round" d="m4.5 15.75 7.5-7.5 7.5 7.5" class:hidden={move || show_guild_leaderboard.get()} />
                        <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" class:hidden={move || !show_guild_leaderboard.get()} />
                    </svg>
                </div>
                <hr class="border-neutral-600" class:hidden={move || !show_guild_leaderboard.get()} />
                <div class="overflow-scroll shrink min-h-0">
                    <Gleaderboard terrs={terrs} class="w-full" class:hidden={move || !show_guild_leaderboard.get()} />
                </div>
            </div>
        </Sidebar>

        <HoverView hovered={hovered} terrs={terrs} extradata={Signal::derive(extradata)} />
    }
}

#[component]
fn HoverView(
    #[prop(into)] hovered: Signal<Option<Arc<str>>>,
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
    extradata: Signal<HashMap<Arc<str>, ExTerrInfo>>,
) -> impl IntoView {
    move || {
        if let Some(hovered) = hovered.get() {
            let h2 = hovered.clone();
            let exdata = Memo::new(move |_| extradata.get().get(&h2).unwrap().clone());
            let h2 = hovered.clone();
            let t = Memo::new(move |_| terrs.get().get(&h2).unwrap().clone());

            let now = chrono::Utc::now();
            let time = now.signed_duration_since(t.read().acquired).num_seconds();

            let (time, set_time) = signal(time);

            let i = set_interval_with_handle(
                move || {
                    let now = chrono::Utc::now();

                    let time = now.signed_duration_since(t.read().acquired).num_seconds();

                    set_time.set(time);
                },
                Duration::from_millis(1000),
            )
            .ok();

            on_cleanup(move || {
                if let Some(i) = i {
                    i.clear();
                }
            });

            let timestr = move || {
                let time = time.get();

                let days = time / 86400;
                let hours = (time % 86400) / 3600;
                let minutes = (time % 3600) / 60;
                let seconds = time % 60;

                if days > 0 {
                    format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
                } else if hours > 0 {
                    format!("{}h {}m {}s", hours, minutes, seconds)
                } else if minutes > 0 {
                    format!("{}m {}s", minutes, seconds)
                } else {
                    format!("{}s", seconds)
                }
            };

            let treas_col = move || {
                let time = time.get();

                // times based on treasury
                if time < 3600 {
                    "oklch(0.723 0.219 149.579)"
                } else if time < (3600 * 24) {
                    "oklch(0.768 0.233 130.85)"
                } else if time < (3600 * 24 * 5) {
                    "oklch(0.795 0.184 86.047)"
                } else if time < (3600 * 24 * 12) {
                    "oklch(0.705 0.213 47.604)"
                } else {
                    "oklch(0.637 0.237 25.331)"
                }
            };

            let treas_text = move || {
                let time = time.get();

                // times based on treasury
                if time < 3600 {
                    "Very Low"
                } else if time < (3600 * 24) {
                    "Low"
                } else if time < (3600 * 24 * 5) {
                    "Medium"
                } else if time < (3600 * 24 * 12) {
                    "High"
                } else {
                    "Very High"
                }
            };

            Some(view! {
                <div class="fixed top-4 right-4 bg-neutral-900 text-white p-2 rounded-md z-50 w-sm terrinfo-hoverbox">
                    <h1 class="text-xl">{hovered}</h1>

                    <div class="p-2">
                        <Show when={move || exdata.get().resources.emeralds > 0}>
                            <div class="flex gap-1 items-center">
                                <div class="icon-emerald"></div>
                                <p>"+"{exdata.get().resources.emeralds}" emeralds per hour"</p>
                            </div>
                        </Show>
                        <Show when={move || exdata.get().resources.ore > 0}>
                            <div class="flex gap-1 items-center">
                                <div class="icon-ores"></div>
                                <p>"+"{exdata.get().resources.ore}" ore per hour"</p>
                            </div>
                        </Show>
                        <Show when={move || exdata.get().resources.wood > 0}>
                            <div class="flex gap-1 items-center">
                                <div class="icon-wood"></div>
                                <p>"+"{exdata.get().resources.wood}" wood per hour"</p>
                            </div>
                        </Show>
                        <Show when={move || exdata.get().resources.fish > 0}>
                            <div class="flex gap-1 items-center">
                                <div class="icon-fish"></div>
                                <p>"+"{exdata.get().resources.fish}" fish per hour"</p>
                            </div>
                        </Show>
                        <Show when={move || exdata.get().resources.crops > 0}>
                            <div class="flex gap-1 items-center">
                                <div class="icon-crops"></div>
                                <p>"+"{exdata.get().resources.crops}" crops per hour"</p>
                            </div>
                        </Show>
                    </div>

                    <h1 class="text-xl">{t.get().guild.name}" ["{t.get().guild.prefix}"]"</h1>

                    <div class="p-2">
                        <h2>"Time held: "{move || timestr()}</h2>
                        <h2>"Treasury: "<span style:color=move || treas_col()>{move || treas_text()}</span></h2>
                    </div>
                </div>
            })
        } else {
            None
        }
    }
}

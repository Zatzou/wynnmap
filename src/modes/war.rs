use std::collections::HashMap;

use leptos::prelude::*;

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

    view! {
        <WynnMap>
            <DefaultMapTiles />

            // conns
            <Show when={move || show_conns.get()}>
                <Connections terrs={terrs} extradata={Signal::derive(extradata)} />
            </Show>

            // territories
            <Show when={move || show_terrs.get()}>
                <TerrView terrs={terrs} extradata={Signal::derive(extradata)} />
            </Show>
        </WynnMap>

        <Sidebar>
            // checkboxes
            <div class="flex-1 flex flex-col p-2 text-lg">
                <Checkbox id="terrs" checked={show_terrs}>"Territories"</Checkbox>
                <Checkbox id="conns" checked={show_conns}>"Connections"</Checkbox>
                <Checkbox id="resico" checked={show_res}>"Resource icons"</Checkbox>
                <Checkbox id="timers" checked={show_timers}>"Timers"</Checkbox>
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
    }
}

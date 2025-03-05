use components::{checkbox::Checkbox, gleaderboard::Gleaderboard};
use leptos::prelude::*;
use std::{collections::HashMap, time::Duration};
use wynnmap::{WynnMap, conns::Connections, maptile::DefaultMapTiles, terrs::TerrView};

mod components;
mod datasource;
mod wynnmap;

fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    mount_to_body(|| view! { <App /> });
}

#[component]
pub fn App() -> impl IntoView {
    let (show_sidebar, set_show_sidebar) = signal(false);
    let show_terrs = RwSignal::new(true);
    let show_res = RwSignal::new(true);
    let show_timers = RwSignal::new(true);
    let show_guild_leaderboard = RwSignal::new(true);

    let (tupd, set_tupd) = signal(());

    let extradata =
        LocalResource::new(async move || datasource::get_extra_terr_info().await.unwrap());
    let terrs = LocalResource::new(move || {
        tupd.track();
        let e = async || datasource::get_wynntils_terrs().await.unwrap();
        e()
    });

    // auto update the map every 30 seconds
    set_interval(
        move || {
            set_tupd.notify();
        },
        Duration::from_secs(10),
    );

    let extradata = move || extradata.get().map(|t| t.take()).unwrap_or(HashMap::new());
    let terrs = Memo::new(move |_| terrs.get().map(|t| t.take()).unwrap_or(HashMap::new()));

    view! {
        <WynnMap>
            <DefaultMapTiles />

            // conns
            <Connections terrs={terrs} extradata={extradata} class:hidden={move || !toggles.get("show_conns")} />

            // territories
            <TerrView terrs={terrs} extradata={extradata} class:hidden={move || !show_terrs.get()} />
        </WynnMap>

        // sidebar open button
        <div on:click={move |_| show_sidebar.set(!show_sidebar.get())} class="fixed top-0 left-0 p-2 cursor-pointer z-50 bg-neutral-900 rounded-e-full mt-2 p-2">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 text-white">
                <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
            </svg>
        </div>
        // class="-translate-x-full"
        <div class="flex flex-col bg-neutral-900 w-full max-w-full h-screen z-50 absolute top-0 md:max-w-sm transition-transform text-white" class:-translate-x-full={move || !show_sidebar.get()}>
            // top text
            <div>
                <div class="flex justify-between p-2 items-center">
                    <h1 class="text-4xl">Wynnmap</h1>

                    // close button
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 cursor-pointer" on:click={move |_| show_sidebar.set(!show_sidebar.get())}>
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                    </svg>
                </div>
                <hr class="border-neutral-600" />
            </div>

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

            // bottom text
            <div>
                <hr class="border-neutral-600" />
                <h2 class="text-neutral-500 p-1 px-2"><a class="underline" href="https://github.com/Zatzou/wynnmap" target="_blank">"Wynnmap"</a>" "{env!("CARGO_PKG_VERSION")}</h2>
            </div>
        </div>
    }
}

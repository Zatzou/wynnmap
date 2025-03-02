use std::{collections::HashMap, time::Duration};
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use shitmap::ShitMap;

mod datasource;
mod shitmap;
mod types;
mod paths;

fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    mount_to_body(|| view! { <App />});
}

#[component]
pub fn App() -> impl IntoView {
    let (tupd, set_tupd) = signal(());

    let tiles = LocalResource::new(async move || datasource::load_map_tiles().await.unwrap());
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
        Duration::from_secs(30),
    );

    let tiles = move || tiles.get().map(|t| t.take()).unwrap_or(Vec::new());
    let extradata = move || extradata.get().map(|t| t.take()).unwrap_or(HashMap::new());
    let terrs = Memo::new(move |_| terrs.get().map(|t| t.take()).unwrap_or(HashMap::new()));


    let conn_path = move || paths::create_route_paths(terrs.get(), extradata());

    view! {
        <ShitMap>
            // map tiles
            <div>
            {move || tiles().iter().map(|tile| {
                let url = tile.url.as_ref().to_string();
                view! {
                    <img
                        src={url}
                        class="shitmap-tile"
                        style={format!("width: {}px; height: {}px; transform: translate({}px, {}px);", tile.width() + 1.0, tile.height() + 1.0, tile.left_side(), tile.top_side())}
                    />
                }
            }).collect_view()}
            </div>

            // conns
            <svg style="position:absolute; overflow:visible;"><path
                d={move || conn_path()}
                style="\
                fill:none;\
                stroke:black;\
                stroke-width:3;\
                stroke-linecap:round;"/>
            </svg>

            // territories
            <div>
                <For
                    each=move || terrs.get().into_iter()
                    key=|(k, v)| (k.clone(), v.guild.clone())
                    children=move |(k, v)| {
                        let width = v.location.width();
                        let height = v.location.height();
                        let left = v.location.left_side();
                        let top = v.location.top_side();
                        let col = v.get_color();
                        let col = format!("{}, {}, {}", col.0, col.1, col.2);

                        let now = chrono::Utc::now();
                        let time = now.signed_duration_since(v.acquired).num_seconds();

                        let (time, set_time) = signal(time);

                        set_interval(move || {
                            let now = chrono::Utc::now();

                            let time = now.signed_duration_since(v.acquired).num_seconds();

                            set_time.set(time);
                        }, Duration::from_millis(1000));

                        let timestr = move || {
                            let time = time.get();

                            let days = time / 86400;
                            let hours = (time % 86400) / 3600;
                            let minutes = (time % 3600) / 60;
                            let seconds = time % 60;

                            if days > 0 {
                                format!("{}d {}h", days, hours)
                            } else if hours > 0 {
                                format!("{}h {}m", hours, minutes)
                            } else if minutes > 0 {
                                format!("{}m {}s", minutes, seconds)
                            } else {
                                format!("{}s", seconds)
                            }
                        };

                        let color = move || {
                            let time = time.get();

                            // times based on treasury
                            if time < 3600 {
                                "background-color: oklch(0.723 0.219 149.579 / .6)"
                            } else if time < (3600 * 24) {
                                "background-color: oklch(0.768 0.233 130.85 / .6);"
                            } else if time < (3600 * 24 * 5) {
                                "background-color: oklch(0.795 0.184 86.047 / .6);"
                            } else if time < (3600 * 24 * 12) {
                                "background-color: oklch(0.705 0.213 47.604 / .6);"
                            } else {
                                "background-color: oklch(0.637 0.237 25.331 / .6);"
                            }
                        };

                        let tkey = k.clone();
                        let extra = Memo::new(move |_| extradata().get(&tkey).cloned());

                        let res = Memo::new(move |_| {
                            if let Some(e) = extra.get() {
                                e.resources.has_res()
                            } else {
                                (false, false, false, false, false)
                            }
                        });

                        view! {
                            <div class="shitmap-item guildterr" style={format!("width: {}px; height: {}px; transform: translate({}px, {}px); background-color: rgba({}, 0.35); border-color: rgb({});", width, height, left, top, col, col)}>
                                    <h3 class="font-bold text-3xl text-white textshadow">{v.guild_prefix.clone()}</h3>
                                    <div class="flex pb-1">
                                        // this is here so that tailwinds cli realizes that this class is used
                                        // class="hidden"
                                        <div class="icon-emerald" class:hidden={move || !res.get().0}></div>
                                        <div class="icon-crops" class:hidden={move || !res.get().1}></div>
                                        <div class="icon-fish" class:hidden={move || !res.get().2}></div>
                                        <div class="icon-ores" class:hidden={move || !res.get().3}></div>
                                        <div class="icon-wood" class:hidden={move || !res.get().4}></div>
                                    </div>
                                    <h4 class="px-2 rounded-2xl text-sm text-center whitespace-nowrap" style={move || color}>{timestr}</h4>
                            </div>
                        }
                    }
                />
            </div>
        </ShitMap>
    }
}

use std::{collections::HashMap, time::Duration};

use leptos::prelude::*;
use shitmap::ShitMap;

mod shitmap;
mod types;
mod wynntils_map;

fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    leptos::mount::mount_to_body(|| view! { <App />});
}

#[component]
pub fn App() -> impl IntoView {
    let (tupd, set_tupd) = signal(());

    let tiles = LocalResource::new(async move || wynntils_map::load_map_tiles().await.unwrap());
    let terrs = LocalResource::new(move || {
        tupd.track();
        let e = async || wynntils_map::get_wynntils_terrs().await.unwrap();
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
    let terrs = Memo::new(move |_| terrs.get().map(|t| t.take()).unwrap_or(HashMap::new()));

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

            // territories
            <div>
                <For
                    each=move || terrs.get().into_iter()
                    key=|(k, _)| k.clone()
                    children=move |(_, v)| {
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
                            set_time.update(|t| *t += 1);
                        }, Duration::from_millis(1000));

                        let timestr = move || {
                            let time = time.get();

                            let days = time / 86400;
                            let hours = (time % 86400) / 3600;
                            let minutes = (time % 3600) / 60;
                            let seconds = time % 60;

                            if days > 0 {
                                format!("{}d {}h {}m", days, hours, minutes)
                            } else if hours > 0 {
                                format!("{}h {}m {}s", hours, minutes, seconds)
                            } else if minutes > 0 {
                                format!("{}m {}s", minutes, seconds)
                            } else {
                                format!("{}s", seconds)
                            }
                        };

                        let color = move || {
                            let time = time.get();

                            if time < 600 {
                                "background-color: oklch(0.637 0.237 25.331 / .6);"
                            } else if time < 1800 {
                                "background-color: oklch(0.705 0.213 47.604 / .6);"
                            } else if time < 3600 {
                                "background-color: oklch(0.769 0.188 70.08 / .6);"
                            } else if time < 36000 {
                                "background-color: oklch(0.795 0.184 86.047 / .6);"
                            } else {
                                "background-color: oklch(0.768 0.233 130.85 / .6);"
                            }
                        };

                        view! {
                            <div class="shitmap-item guildterr" style={format!("width: {}px; height: {}px; transform: translate({}px, {}px); background-color: rgba({}, 0.35); border-color: rgb({});", width, height, left, top, col, col)}>
                                    <h3 class="font-bold text-3xl text-white textshadow">{v.guild_prefix.clone()}</h3>
                                    <h4 class="px-2 rounded-2xl text-sm text-center" style={move || color}>{timestr}</h4>
                            </div>
                        }
                    }
                />
            </div>
        </ShitMap>
    }
}

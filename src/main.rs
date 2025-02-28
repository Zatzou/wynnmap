use std::collections::HashMap;

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
    let tiles = LocalResource::new(async move || wynntils_map::load_map_tiles().await.unwrap());
    let terrs = LocalResource::new(async move || wynntils_map::get_wynntils_terrs().await.unwrap());

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
                        // bounds={Bounds::new(
                        //     Position::new(-tile.z1 as f64, tile.x1 as f64),
                        //     Position::new(-tile.z2 as f64 - 1.0, tile.x2 as f64 + 1.0),
                        // )}
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

                        view! {
                            <div class="shitmap-item guildterr" style={format!("width: {}px; height: {}px; transform: translate({}px, {}px); background-color: rgba({}, 0.35); border: 3px solid rgb({}); border-radius: 3px", width, height, left, top, col, col)}>
                                    <h3 class="font-bold text-3xl text-white textshadow">{v.guild_prefix}</h3>
                            </div>
                        }
                    }
                />
            </div>
        </ShitMap>
    }
}

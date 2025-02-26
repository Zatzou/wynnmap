use std::collections::HashMap;

use custom_mapcontainer::MapContainerSCRS;
use leptos::prelude::*;
use leptos_leaflet::prelude::*;

mod custom_mapcontainer;
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
        <MapContainerSCRS style="height: 100vh;" class="bg-neutral-950" center={Position::new(2000.0, 0.0)} min_zoom=-3.0 zoom=0.0 set_view=true enable_high_accuracy=true>
            // <TileLayer url="https://tile.openstreetmap.org/{z}/{x}/{y}.png" />

            {move || tiles().iter().map(|tile| {
                view! {
                    <ImageOverlay
                        url={tile.url.as_ref()}
                        bounds={Bounds::new(
                            Position::new(-tile.z1 as f64, tile.x1 as f64),
                            Position::new(-tile.z2 as f64 - 1.0, tile.x2 as f64 + 1.0),
                        )}
                        attribution="&copy; <a href=\"https://wynntils.com\">Wynntils</a>"
                    />
                }
            }).collect_view()}

            <For
                each=move || terrs.get().into_iter()
                key=|(k, _)| k.clone()
                children=move |(k, v)| {
                    let pos = v.location.into_posvec();
                    let middle = v.location.middle();
                    let col = v.get_color();

                    view! {
                        <Polygon positions={pos} color={col} />
                        <Tooltip position={middle} permanent=true direction="center">
                            <h3 class="font-bold text-3xl text-white">{v.guild_prefix}</h3>
                        </Tooltip>
                    }
                }
            />
        </MapContainerSCRS>
    }
}

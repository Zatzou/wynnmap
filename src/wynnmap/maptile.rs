use leptos::prelude::*;
use wynnmap_types::WynntilsMapTile;

use crate::datasource;

#[component]
pub fn MapTile(tile: Signal<WynntilsMapTile>) -> impl IntoView {
    view! {
        <img
            src=tile.get().url
            class="wynnmap-tile"
            style:width=move || format!("{}px", tile.read().width() + 1)
            style:height=move || format!("{}px", tile.read().height() + 1)
            style:transform=move || {
                format!(
                    "translate({}px, {}px)",
                    tile.read().left_side(),
                    tile.read().top_side(),
                )
            }
        />
    }
}

#[component]
pub fn MapTiles(tiles: Signal<Vec<WynntilsMapTile>>) -> impl IntoView {
    view! {
        <div class="wynnmap-tiles">
            {move || {
                tiles.get()
                    .into_iter()
                    .map(|tile| view! { <MapTile tile=tile.into() /> })
                    .collect_view()
            }}
        </div>
    }
}

/// A component that displays the default map tiles fetched from the server.
#[component]
pub fn DefaultMapTiles() -> impl IntoView {
    let tiles = LocalResource::new(async move || datasource::load_map_tiles().await.unwrap());

    let tiles = move || tiles.get().map_or(Vec::new(), |t| t.take());

    view! { <MapTiles tiles=Signal::derive(tiles) /> }
}

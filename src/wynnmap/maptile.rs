use leptos::prelude::*;
use wynnmap_types::WynntilsMapTile;

#[component]
pub fn MapTile(tile: Signal<WynntilsMapTile>) -> impl IntoView {
    view! {
        <img
            src=tile.get().url
            class="wynnmap-tile"
            style:width=move || format!("{}px", tile.read().width() + 1.0)
            style:height=move || format!("{}px", tile.read().height() + 1.0)
            style:transform=move || {
                format!(
                    "translate3D({}px, {}px, 0)",
                    tile.read().left_side(),
                    tile.read().top_side(),
                )
            }
        />
    }
}

#[component]
pub fn MapTiles(tiles: impl Fn() -> Vec<WynntilsMapTile> + Send + Sync + 'static) -> impl IntoView {
    view! {
        <div class="wynnmap-tiles">
            {move || {
                tiles()
                    .into_iter()
                    .map(|tile| view! { <MapTile tile=tile.into() /> })
                    .collect_view()
            }}
        </div>
    }
}

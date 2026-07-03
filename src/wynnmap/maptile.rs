use leptos::prelude::*;
use wynnmap_types::maptile::MapTile;

use crate::{settings::use_toggle, util::as_px, wynnmap::context::DefaultMapTiles};

#[component]
pub fn MapTile(
    #[prop(into)] tile: Signal<MapTile>,
    #[prop(default = false.into(), into)] grayscale: Signal<bool>,
) -> impl IntoView {
    let location = move || tile.read().location;

    view! {
        <img
            src=tile.get().url
            class="wynnmap-tile"
            class:grayscale=grayscale
            style:width=move || as_px(location().width())
            style:height=move || as_px(location().height())
            style:top=move || as_px(location().top_side())
            style:left=move || as_px(location().left_side())
        />
    }
}

#[component]
pub fn MapTiles(
    #[prop(into)] tiles: Signal<Vec<MapTile>>,
    #[prop(default = false.into(), into)] grayscale: Signal<bool>,
) -> impl IntoView {
    let show_non_main = use_toggle("show_non_main_maps", false);

    view! {
        <div class="wynnmap-tiles">
            {move || {
                tiles.get()
                    .into_iter()
                    .filter(|tile| {
                        if tile.name.starts_with("Main") || tile.name.starts_with("Realm of Light") {
                            true
                        } else {
                            show_non_main.get()
                        }
                    })
                    .map(|tile| view! { <MapTile tile grayscale /> })
                    .collect_view()
            }}
        </div>
    }
}

/// A component that displays the default map tiles fetched from the server.
#[component]
pub fn WithDefaultMapTiles(
    #[prop(default = false.into(), into)] grayscale: Signal<bool>,
) -> impl IntoView {
    let DefaultMapTiles(tiles) = expect_context();

    view! { <MapTiles tiles={tiles} grayscale /> }
}

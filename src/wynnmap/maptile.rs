use std::time::Duration;

use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::maptile::MapTile;

use crate::{datasource, settings::use_toggle};

#[component]
pub fn MapTile(tile: Signal<MapTile>) -> impl IntoView {
    view! {
        <img
            src=tile.get().url
            class="wynnmap-tile"
            style:width=move || format!("{}px", tile.read().location.width() + 1)
            style:height=move || format!("{}px", tile.read().location.height() + 1)
            style:transform=move || {
                format!(
                    "translate3D({}px, {}px, 0)",
                    tile.read().location.left_side(),
                    tile.read().location.top_side(),
                )
            }
        />
    }
}

#[component]
pub fn MapTiles(#[prop(into)] tiles: Signal<Vec<MapTile>>) -> impl IntoView {
    let show_non_main = use_toggle("show_non_main_maps", false);

    view! {
        <div class="wynnmap-tiles">
            {move || {
                tiles.get()
                    .into_iter()
                    .filter(|tile| {
                        if tile.name.contains("Main") || tile.name.contains("Realm of Light") {
                            true
                        } else {
                            show_non_main.get()
                        }
                    })
                    .map(|tile| view! { <MapTile tile=tile.into() /> })
                    .collect_view()
            }}
        </div>
    }
}

/// A component that displays the default map tiles fetched from the server.
#[component]
pub fn DefaultMapTiles() -> impl IntoView {
    let tiles = RwSignal::new(Vec::new());

    let load_tiles = move |tiles: RwSignal<_>| async move {
        if let Ok(data) = datasource::load_map_tiles().await {
            tiles.set(data);
        }
    };

    spawn_local(load_tiles(tiles));

    // Update the map tiles every hour to ensure they stay up to date
    let map_tile_updater = set_interval_with_handle(
        move || {
            spawn_local(load_tiles(tiles));
        },
        Duration::from_hours(1),
    )
    .ok();

    on_cleanup(move || {
        if let Some(i) = map_tile_updater {
            i.clear();
        }
    });

    view! { <MapTiles tiles={tiles} /> }
}

use std::time::Duration;

use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::maptile::MapTile;

use crate::{
    datasource,
    dialog::{Dialogs, info::info_dialog},
    settings::use_toggle,
};

#[component]
pub fn MapTile(tile: Signal<MapTile>) -> impl IntoView {
    view! {
        <img
            src=tile.get().url
            class="wynnmap-tile"
            style:width=move || format!("{}px", tile.read().location.width())
            style:height=move || format!("{}px", tile.read().location.height())
            style:top=move || format!("{}px", tile.read().location.top_side())
            style:left=move || format!("{}px", tile.read().location.left_side())
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
                        if tile.name.starts_with("Main") || tile.name.starts_with("Realm of Light") {
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
    let dialogs = use_context::<Dialogs>().expect("Dialogs context not found");
    let tiles = RwSignal::new(Vec::new());

    let load_tiles = move |tiles: RwSignal<_>| async move {
        match datasource::load_map_tiles().await {
            Ok(data) => tiles.set(data),
            Err(err) => {
                if !dialogs.contains("err_maptiles") {
                    dialogs.add("err_maptiles", move || {
                        info_dialog(
                            String::from("Failed to load map tiles"),
                            view! {
                                <p>"An error occured while loading api data"</p>
                                <pre class="p-2 bg-neutral-800 rounded my-1">{format!("{err:?}")}</pre>
                            },
                        )
                    });
                }
            }
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

use std::time::Duration;

use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::maptile::MapTile;

use crate::{
    datasource,
    dialog::{Dialogs, info::info_dialog},
    wynnmap::util::get_viewport_middle,
};

#[component]
pub fn MapContextProvider(children: Children) -> impl IntoView {
    provide_map_position();
    provide_relmousepos();
    provide_default_map_tiles();

    children()
}

#[derive(Clone)]
pub struct MapPosition {
    pub position: RwSignal<[f64; 2]>,
    pub zoom: RwSignal<f64>,
}

fn provide_map_position() {
    let screen_middle = get_viewport_middle();
    // use the midpoint to position the map so that it is centered
    let position = RwSignal::new([100.0 + screen_middle[0], 1200.0 + screen_middle[1]]);

    let zoom = RwSignal::new(0.5);

    provide_context(MapPosition { position, zoom });
}

/// Mouse position on the map atlas
#[derive(Clone)]
pub struct RelMousePos(pub RwSignal<Option<[i32; 2]>>);

fn provide_relmousepos() {
    provide_context(RelMousePos(RwSignal::new(None)));
}

#[derive(Clone)]
pub struct DefaultMapTiles(pub RwSignal<Vec<MapTile>>);

fn provide_default_map_tiles() {
    let dialogs = expect_context::<Dialogs>();
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

    provide_context(DefaultMapTiles(tiles));
}

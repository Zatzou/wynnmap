use leptos::prelude::*;
use wynnmap_types::maptile::MapTile;

use crate::{components::loader::loader, datasource, settings::use_toggle};

#[derive(Clone)]
pub struct DefaultMapTilesCtx(pub RwSignal<Vec<MapTile>>);

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
pub fn MapTiles(tiles: Signal<Vec<MapTile>>) -> impl IntoView {
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
    let DefaultMapTilesCtx(tiles) = use_context().expect("Default maptiles context not found");

    view! { <MapTiles tiles={tiles.into()} /> }
    // let tiles = LocalResource::new(async move || datasource::load_map_tiles().await);

    // view! {
    //     {move || match tiles.get() {
    //         None => view! {"loading"}.into_any(),
    //         Some(Err(e)) => view! {"Error"}.into_any(),
    //         Some(Ok(tiles)) => .into_any()
    //     }}
    // }
}

/// Provide the default maptiles
#[component]
pub fn ProvideDefaultMapTiles(children: ChildrenFn) -> impl IntoView {
    let tiles = LocalResource::new(async move || datasource::load_map_tiles().await);

    move || {
        loader(tiles, |tiles| {
            let tiles = RwSignal::new(tiles);

            provide_context(DefaultMapTilesCtx(tiles));

            view! {{children()}}.into_any()
        })
    }
}

use std::{collections::BTreeMap, time::Duration};

use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::gather::GatherSpots;

use crate::{
    components::{sidebar::Sidebar, sidecard::SideCard},
    datasource,
    modes::gather::noderender::NodeRenderer,
    wynnmap::{WynnMap, maptile::DefaultMapTiles},
};

mod clustering;
mod noderender;

#[component]
pub fn GatherMap() -> impl IntoView {
    let nodes = RwSignal::new(GatherSpots::default());
    let data = RwSignal::new(BTreeMap::new());

    let load_data = move |nodes: RwSignal<_>| async move {
        match (
            datasource::get_gather_nodes().await,
            datasource::get_mat_data().await,
        ) {
            (Ok(n), Ok(d)) => {
                nodes.set(n);
                data.set(d);
            }
            _ => {
                panic!()
            }
        }
    };

    spawn_local(load_data(nodes));

    let mouse_pos = RwSignal::new(None);
    let hovered = RwSignal::new(Vec::new());

    view! {
        <WynnMap>
            <DefaultMapTiles grayscale=true />

            <NodeRenderer nodes data mouse_pos hovered />
        </WynnMap>

        <SideCard hover=true>
            <div>
                <span>"X: "{move || mouse_pos.get().map(|p| p[0])}" / Y: "{move || mouse_pos.get().map(|p| p[1])}</span>
            </div>
            {move || hovered.get().into_iter().map(|node| view!{
                <div>
                    <span>{node.res.name}</span>
                </div>
            }).collect::<Vec<_>>()}
        </SideCard>

        <Sidebar>
        </Sidebar>
    }
}

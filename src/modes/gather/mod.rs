use std::{collections::BTreeMap, time::Duration};

use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::gather::GatherSpots;

use crate::{
    components::{checkbox::Checkbox, sidebar::Sidebar, sidecard::SideCard}, datasource, modes::gather::noderender::NodeRenderer, settings::use_toggle, wynnmap::{WynnMap, maptile::DefaultMapTiles}
};

mod clustering;
mod noderender;

#[component]
pub fn GatherMap() -> impl IntoView {
    let nodes = RwSignal::new(GatherSpots::default());
    let data = RwSignal::new(BTreeMap::new());
    
    let nodes_all = use_toggle("nodes_all", true);
    let nodes_crop = use_toggle("nodes_crop", true);
    let nodes_fish = use_toggle("nodes_fish", true);
    let nodes_ore = use_toggle("nodes_ore", true);
    let nodes_wood = use_toggle("nodes_wood", true);

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

    // Update the territory data every 10 minutes to ensure the map stays up to date
    let data_updater = set_interval_with_handle(
        move || {
            spawn_local(load_data(nodes));
        },
        Duration::from_hours(1),
    )
    .ok();

    on_cleanup(move || {
        if let Some(i) = data_updater {
            i.clear();
        }
    });

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
            <div class="flex-1 flex flex-col gap-2 p-2 text-lg">
                <div>
                    <Checkbox id="nodes_all" checked={nodes_all}>"Show all nodes"</Checkbox>
                    <div class="flex flex-col gap-1 ml-6" class:opacity-50=nodes_all>
                        <Checkbox id="nodes_crop" checked={nodes_crop} disabled=nodes_all>"Show crops"</Checkbox>
                        <Checkbox id="nodes_fish" checked={nodes_fish} disabled=nodes_all>"Show fish"</Checkbox>
                        <Checkbox id="nodes_ore"  checked={nodes_ore}  disabled=nodes_all>"Show ore"</Checkbox>
                        <Checkbox id="nodes_wood" checked={nodes_wood} disabled=nodes_all>"Show wood"</Checkbox>
                    </div>
                </div>
            </div>
        </Sidebar>
    }
}

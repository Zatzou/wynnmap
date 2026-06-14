use std::{collections::BTreeMap, sync::Arc, time::Duration};

use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::gather::{GatherSpots, MatData, Profession};

use crate::{
    components::{checkbox::Checkbox, sidebar::Sidebar, sidecard::SideCard}, datasource, modes::gather::noderender::NodeRenderer, settings::use_toggle, wynnmap::{WynnMap, maptile::DefaultMapTiles}
};

mod clustering;
mod noderender;

#[component]
pub fn GatherMap() -> impl IntoView {
    let nodes = RwSignal::new(GatherSpots::default());
    let data = RwSignal::new(BTreeMap::new());
    
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
    let a = move || get_namelist(data.get());
    let b = move || (RwSignal::new(a().0),RwSignal::new(a().1),RwSignal::new(a().2),RwSignal::new(a().3));
    let sig_crop = move || b().0;
    let sig_fish = move || b().1;
    let sig_ore = move || b().2;
    let sig_wood = move || b().3;

    let crop_sigs_arr = move || sigs_arr_gen(sig_crop);
    let fish_sigs_arr = move || sigs_arr_gen(sig_fish);
    let ore_sigs_arr = move || sigs_arr_gen(sig_ore);
    let wood_sigs_arr = move || sigs_arr_gen(sig_wood);
    

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
                <Checkbox id="nodes_crop" checked={nodes_crop}>"Crops"</Checkbox>
                <div class="flex flex-col gap-1 ml-6">
                    <For 
                        each=move || sig_crop().get()
                        key=|corp| corp.clone()
                        children=move |corp| {
                            view! {
                                <Checkbox id=corp.to_string() checked={crop_sigs_arr()[&corp]}>
                                    {corp}
                                </Checkbox>
                            }
                        }
                    />
                </div>
                <Checkbox id="nodes_fish" checked={nodes_fish}>"Fish"</Checkbox>
                <div class="flex flex-col gap-1 ml-6">
                    <For 
                        each=move || sig_fish().get()
                        key=|corp| corp.clone()
                        children=move |corp| {
                            view! {
                                <Checkbox id=corp.to_string() checked={fish_sigs_arr()[&corp]}>
                                    {corp}
                                </Checkbox>
                            }
                        }
                    />
                </div>
                <Checkbox id="nodes_ore" checked={nodes_ore}>"Ore"</Checkbox>
                <div class="flex flex-col gap-1 ml-6">
                    <For 
                        each=move || sig_ore().get()
                        key=|corp| corp.clone()
                        children=move |corp| {
                            view! {
                                <Checkbox id=corp.to_string() checked={ore_sigs_arr()[&corp]}>
                                    {corp}
                                </Checkbox>
                            }
                        }
                    />
                </div>
                <Checkbox id="nodes_wood" checked={nodes_wood}>"Wood"</Checkbox>
                <div class="flex flex-col gap-1 ml-6">
                    <For 
                        each=move || sig_wood().get()
                        key=|corp| corp.clone()
                        children=move |corp| {
                            view! {
                                <Checkbox id=corp.to_string() checked={wood_sigs_arr()[&corp]}>
                                    {corp}
                                </Checkbox>
                            }
                        }
                    />
                </div>
            </div>
        </Sidebar>
    }
}

fn get_namelist(data: BTreeMap<Arc<str>, MatData>) -> (Vec<Arc<str>>,Vec<Arc<str>>,Vec<Arc<str>>,Vec<Arc<str>>) {
    let (mut corp, mut fsh, mut roe, mut ood) = (Vec::new(),Vec::new(),Vec::new(),Vec::new());
    for (name,mat) in data {
        match mat.prof {
            Profession::Mining => roe.push(name),
            Profession::Woodcutting => ood.push(name),
            Profession::Fishing => fsh.push(name),
            Profession::Farming => corp.push(name),
        }
    }
    (corp,fsh,roe,ood)
}
fn sigs_arr_gen(fnsig: impl Fn() -> RwSignal<Vec<Arc<str>>>) -> BTreeMap<Arc<str>,RwSignal<bool>> {
    let mut map: BTreeMap<Arc<str>,RwSignal<bool>> = BTreeMap::new();
    for i in fnsig().get() {
        map.insert(i,RwSignal::new(true));
    }
    map
}
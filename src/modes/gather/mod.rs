use std::collections::BTreeMap;

use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::gather::{GatherSpots, Material};

use crate::{
    components::{checkbox::Checkbox, sidebar::Sidebar, sidecard::SideCard},
    datasource,
    modes::gather::noderender::NodeRenderer,
    settings::use_toggle,
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

    let toggles = RwSignal::new(Vec::new());

    Effect::new(move |_| {
        toggles.update(|toggles| {
            toggles.clear();

            for (n, mat) in nodes.read().resources.iter().enumerate() {
                toggles.push(MatToggle {
                    mat: mat.clone(),
                    toggle: use_toggle(format!("gather-mat-{}", mat.name), true),
                    count: nodes
                        .read()
                        .spots
                        .iter()
                        .filter(|s| s.resource == n)
                        .count(),
                })
            }
        });
    });

    let hidelist = Memo::new(move |_| {
        let mut list = Vec::new();

        for toggle in toggles.read().iter() {
            if !toggle.toggle.get() {
                list.push(toggle.mat.name.clone())
            }
        }

        list
    });

    let show_all = move |_| {
        for t in toggles.read().iter() {
            t.toggle.set(true);
        }
    };

    let show_none = move |_| {
        for t in toggles.read().iter() {
            t.toggle.set(false);
        }
    };

    let search_str = RwSignal::new(String::new());

    view! {
        <WynnMap>
            <DefaultMapTiles grayscale=true />

            <NodeRenderer nodes data mouse_pos hovered hidden={hidelist} />
        </WynnMap>

        <SideCard hover=true>
            <div>
                <span>"X: "{move || mouse_pos.get().map(|p| p[0])}" / Y: "{move || mouse_pos.get().map(|p| p[1])}</span>
            </div>
            {move || hovered.get().into_iter().map(|node| view!{
                <div>
                    <span>{titlecase(node.res.name)}</span>
                </div>
            }).collect::<Vec<_>>()}
        </SideCard>

        <Sidebar>
            <div class="matsearch">
                <div class="buttons">
                    <button on:click=show_all>"Show all"</button>
                    <button on:click=show_none>"Show none"</button>
                </div>
                <input type="text" placeholder="Search materials" bind:value=search_str/>
            </div>
            <div class="mattoggles">
                <div/>
                <span>"Material"</span>
                <span>"Level"</span>
                <span>"Count"</span>
                <For
                    each=move || toggles.get().into_iter().filter(move |t| t.mat.name.contains(&search_str.read().to_ascii_uppercase()))
                    key=move |val| val.mat.clone()
                    children=move |val| view! {
                        <ToggleComponent toggle=val/>
                    }
                />
            </div>
        </Sidebar>
    }
}

#[derive(Clone)]
struct MatToggle {
    mat: Material,
    toggle: RwSignal<bool>,
    count: usize,
}

#[component]
fn ToggleComponent(toggle: MatToggle) -> impl IntoView {
    view! {
        <Checkbox id=toggle.mat.name.as_ref() checked=toggle.toggle>
            <span>{titlecase(toggle.mat.name)}</span>
            <span>Lv. {toggle.mat.level}</span>
            <span>{toggle.count}</span>
        </Checkbox>
    }
}

/// Format a string in Titlecase
fn titlecase(name: impl AsRef<str>) -> String {
    let mut c = name.as_ref().chars();
    match c.next() {
        None => String::new(),
        Some(f) => f
            .to_uppercase()
            .chain(c.flat_map(|c| c.to_lowercase()))
            .collect(),
    }
}

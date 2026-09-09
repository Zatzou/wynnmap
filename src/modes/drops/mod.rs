use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::{
    color::Color,
    drops::{DropArea, Item, ItemDrops},
};

use crate::{
    components::{sidebar::Sidebar, sidecard::SideCard},
    datasource,
    modes::drops::{search::DropSearch, spotrender::SpotRenderer},
    wynnmap::{WynnMap, context::RelMousePos, maptile::WithDefaultMapTiles},
};

mod search;
mod spotrender;

#[component]
pub fn DropsMap() -> impl IntoView {
    let itemdrops = RwSignal::new(ItemDrops::default());

    let load_data = move || async move {
        match datasource::load_json("/api/v3/items/drops").await {
            Ok(data) => {
                itemdrops.set(data);
            }
            _ => {
                panic!()
            }
        }
    };

    spawn_local(load_data());

    let search_str = RwSignal::new(String::new());
    let selected_item = RwSignal::new(None::<Item>);

    let RelMousePos(mouse_rel) = expect_context();
    let hovered_spots = Memo::new(move |_| {
        let mut spots = BTreeMap::new();

        if let Some(item) = &*selected_item.read()
            && let Some(rel) = mouse_rel.get()
        {
            for (name, areas) in &item.dropped_by {
                let matched = areas
                    .iter()
                    .filter(|spot| spot.within(rel, 5))
                    .cloned()
                    .collect::<BTreeSet<_>>();

                if !matched.is_empty() {
                    spots.insert(name.clone(), matched);
                }
            }
        }

        spots
    });

    let selected_spots = RwSignal::new(BTreeMap::new());

    let onclick = Callback::new(move |_| {
        selected_spots.set(hovered_spots.get());
    });

    view! {
        <WynnMap onclick>
            <WithDefaultMapTiles grayscale=true />
            <SpotRenderer selected_item/>
        </WynnMap>

        <Show when=move || selected_spots.read().is_empty()>
            <SideCard hover=true>
                <For
                    each=move || hovered_spots.get()
                    key=move |spot| spot.clone()
                    let(spot)
                >
                    <DropInfoCard spot/>
                </For>
            </SideCard>
        </Show>

        <Sidebar>
            {move || if let Some(selected) = selected_item.get() {
                view! { <ItemSelected selected back=move || selected_item.set(None)/> }.into_any()
            } else {
                view! { <DropSearch itemdrops selected_item search_str/> }.into_any()
            }}
        </Sidebar>

        <Show when=move || !selected_spots.read().is_empty()>
            <SideCard on_close=move |_| selected_spots.set(BTreeMap::new())>
                <For
                    each=move || selected_spots.get()
                    key=move |spot| spot.clone()
                    let(spot)
                >
                    <DropInfoCard spot/>
                </For>
            </SideCard>
        </Show>
    }
}

#[component]
fn ItemSelected(selected: Item, back: impl Fn() + 'static) -> impl IntoView {
    view! {
        <div class="dropsiteminfo">
            <div class="backbtn" on:click=move |_| back()>
                <icons::ChevronLeft/>
                <h2>"Back"</h2>
            </div>

            <div class="info">
                <h2>{selected.name}</h2>
                <h3>{selected.kind}</h3>
            </div>

            <div class="dropsources">
                {selected.dropped_by.into_iter().map(|(name, locations)| {
                    let col = Color::from_hash(name.as_bytes()).to_rgb_values();

                    view! {
                        <div class="source" style:--col=col>
                            <div/>
                            <div>
                                <span>{name}</span>
                                <span>{locations.len()}</span>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn DropInfoCard(
    #[prop(name = "spot")] (name, spots): (Arc<str>, BTreeSet<DropArea>),
) -> impl IntoView {
    let col = Color::from_hash(name.as_bytes()).to_rgb_values();

    view! {
        <div class="dropinfocard" style:--col=col>
            <div/>
            <div>
                <h2>{name}</h2>
                {spots.into_iter().map(|area| {
                    let [x, y, z] = area.location;

                    let copystring = move |_| {
                        let clipboard = window().navigator().clipboard();

                        let _promise = clipboard.write_text(&format!("/comp {x} {y} {z}"));
                    };

                    view! {
                        <p><icons::Clipboard on:click=copystring size=16/>" "{x}" "{y}" "{z}" radius: "{area.radius}</p>
                    }}).collect::<Vec<_>>()
                }
            </div>
        </div>
    }
}

use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::{
    color::Color,
    drops::{Item, ItemDrops},
};

use crate::{
    components::{sidebar::Sidebar, sidecard::SideCard},
    datasource,
    modes::drops::spotrender::SpotRenderer,
    wynnmap::{WynnMap, maptile::WithDefaultMapTiles},
};

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

    view! {
        <WynnMap>
            <WithDefaultMapTiles grayscale=true />
            <SpotRenderer selected_item/>
        </WynnMap>

        <SideCard hover=true>
        </SideCard>

        <Sidebar>
            {move || if let Some(selected) = selected_item.get() {
                view! { <ItemSelected selected back=move || selected_item.set(None)/> }.into_any()
            } else {
                view! { <DropSearch itemdrops selected_item search_str/> }.into_any()
            }}
        </Sidebar>
    }
}

#[component]
fn DropSearch(
    itemdrops: RwSignal<ItemDrops>,
    selected_item: RwSignal<Option<Item>>,
    search_str: RwSignal<String>,
) -> impl IntoView {
    let results = Memo::new(move |_| {
        let search = search_str.get().to_ascii_lowercase();

        if !search.is_empty() {
            itemdrops
                .read()
                .iter()
                .filter(|item| !item.dropped_by.is_empty())
                .filter(|item| item.name.to_ascii_lowercase().contains(&search))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    });

    view! {
        <div class="dropsearch">
            <input type="text" placeholder="Search items" bind:value=search_str/>
        </div>
        <div class="dropsearchres">
            <For
                each=move || results.get()
                key=move |item| item.name.clone()
                children=move |item| {
                    view! {
                        <div on:click=move |_| selected_item.set(Some(item.clone()))>
                            {item.name.clone()}
                        </div>
                    }
                }
            />
        </div>
        <Show when=move || results.read().is_empty()>
            <div class="dropsearch-hint">
                <p>"Start typing to search"</p>
                <p>"Only items with known drop locations are included"</p>
            </div>
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

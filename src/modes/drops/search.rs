use leptos::prelude::*;
use wynnmap_types::drops::{Item, ItemDrops};

#[component]
pub fn DropSearch(
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

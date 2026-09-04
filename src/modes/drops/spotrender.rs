use std::collections::BTreeMap;

use leptos::prelude::*;
use wynnmap_types::drops::Item;

#[component]
pub fn SpotRenderer(selected_item: RwSignal<Option<Item>>) -> impl IntoView {
    let spots = move || {
        if let Some(item) = selected_item.get() {
            item.dropped_by
        } else {
            BTreeMap::new()
        }
    };

    view! {
        <svg style="position: absolute; overflow: visible">
            <For
                each=spots
                key=move |(name, locations)| (name.clone(), locations.clone())
                let((name, locations))
            >
                {locations.into_iter().map(|loc| {
                    let [x, _, y] = loc.location;
                    let r = loc.radius;

                    view! {
                        <circle cx=x cy=y r=r fill="red" stroke="black" />
                    }
                }).collect::<Vec<_>>()}
            </For>
        </svg>
    }
}

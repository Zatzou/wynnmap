use std::collections::BTreeMap;

use leptos::prelude::*;
use wynnmap_types::{color::Color, drops::Item};

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
                children=move |(name, locations)| {
                    let col = Color::from_hash(name.as_bytes());
                    let fill = col.to_rgb_with_alpha(0.4);
                    let stroke = col.to_hex();

                    locations.into_iter().map(|loc| {
                        let [x, _, y] = loc.location;
                        let r = loc.radius.max(5);

                        view! {
                            <circle cx=x cy=y r=r fill=fill.clone() stroke=stroke.clone() />
                        }
                    }).collect::<Vec<_>>()
                }
            />
        </svg>
    }
}

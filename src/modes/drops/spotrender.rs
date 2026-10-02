use leptos::prelude::*;
use wynnmap_types::{color::Color, drops::Item};

#[component]
pub fn SpotRenderer(selected_item: RwSignal<Option<Item>>) -> impl IntoView {
    let spots = move || {
        let mut spots = Vec::new();

        for (name, locations) in selected_item.get().unwrap_or_default().dropped_by {
            let col = Color::from_hash(name.as_bytes()).saturate(0.75);

            for loc in locations {
                spots.push((col, loc.location, loc.radius));
            }
        }

        spots.sort_by_key(|(_, _, r)| -r);

        spots
    };

    view! {
        <svg style="position: absolute; overflow: visible">
            <For
                each=spots
                key=move |(col, loc, r)| (*col, *loc, *r)
                children=move |(col, loc, r)| {
                    let fill = col.to_rgb_with_alpha(0.4);
                    let stroke = col.to_hex();

                    let [x, _, y] = loc;
                    let r = r.max(5);

                    view! {
                        <circle cx=x cy=y r=r fill=fill.clone() stroke=stroke.clone() stroke-width=3 />
                    }
                }
            />
        </svg>
    }
}

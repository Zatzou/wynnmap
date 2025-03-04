use std::{collections::HashMap, sync::Arc};

use leptos::prelude::*;
use wynnmap_types::Territory;

#[component]
pub fn Gleaderboard(
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
    class: &'static str,
) -> impl IntoView {
    let guild_leaderboard = move || {
        let mut leadb = HashMap::new();

        for (_, v) in terrs.read().iter() {
            let guild = v.guild.clone();
            let terr = leadb.entry(guild).or_insert(0);
            *terr += 1;
        }

        let mut leadb: Vec<_> = leadb.into_iter().collect();

        leadb.sort_by(|a, b| b.1.cmp(&a.1));

        leadb
    };

    view! {
        <table class={class} class:table-auto=true>
            <tbody>
                <For
                    each=move || guild_leaderboard().into_iter()
                    key=|(k, v)| (k.clone(), v.clone())
                    children=move |(k, v)| {
                        let col = k.get_color();
                        let col = format!("{}, {}, {}", col.0, col.1, col.2);
                        let name = k.name.clone();
                        let link = move || format!("https://wynncraft.com/stats/guild/{}", name);
                        view! {
                            <tr class="even:bg-neutral-800" style={format!("background-color: rgba({}, 0.3)", col)}>
                                <td><a href={link()} target="_blank" class="block pl-2 font-mono">"["{k.prefix}"]"</a></td>
                                <td><a href={link()} target="_blank" class="block">{k.name}</a></td>
                                <td><a href={link()} target="_blank" class="block text-right pr-2">{v}</a></td>
                            </tr>
                        }
                    }
                />
            </tbody>
        </table>
    }
}

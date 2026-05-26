use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use leptos::prelude::*;
use wynnmap_types::terr::TerrState;

#[component]
pub fn Gleaderboard(#[prop(into)] state: Signal<BTreeMap<Arc<str>, TerrState>>) -> impl IntoView {
    let guild_leaderboard = move || {
        let mut guilds = HashMap::new();

        for (_, v) in state.read().iter() {
            let guild = v.guild.clone();
            let terr = guilds.entry(guild).or_insert(0);
            *terr += 1;
        }

        let mut leaderboard: Vec<_> = guilds.into_iter().collect();

        // alphabetically sort the guilds
        leaderboard.sort_by(|a, b| a.0.name.cmp(&b.0.name));
        // sort by the number of territories while keeping the alphabetical order for any ties
        leaderboard.sort_by_key(|b| std::cmp::Reverse(b.1));

        leaderboard
    };

    view! {
        <div class="gleaderboard">
            <For
                each=move || guild_leaderboard().into_iter()
                key=|(k, v)| (k.clone(), *v)
                children=move |(k, v)| {
                    let col = k.get_color();
                    let col = format!("{}, {}, {}", col.0, col.1, col.2);
                    let name = k.name.clone();
                    let link = move || format!("https://wynncraft.com/stats/guild/{}", name.clone());

                    view! {
                        <a class="glrow" style:--col=col href=link() target="_blank">
                            <div/>
                            <span>"["{k.prefix}"]"</span>
                            <span>{k.name}</span>
                            <span>{v}</span>
                        </a>
                    }
                }
            />
        </div>
    }
}

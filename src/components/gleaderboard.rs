use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use jiff::Timestamp;
use leptos::{prelude::*, task::spawn_local};
use wynnmap_types::{
    leaderboard::{GuildLeaderboardEntry, SeasonLeaderboard, SeasonList},
    terr::TerrState,
};

use crate::datasource;

#[component]
pub fn Gleaderboard(
    #[prop(into)] state: Signal<BTreeMap<Arc<str>, TerrState>>,
    #[prop(into, default = RwSignal::new(true))] show_guild_leaderboard: RwSignal<bool>,
    #[prop(into, default = false)] enable_seasons: bool,
) -> impl IntoView {
    view! {
        <div class="gleaderboard">
            <hr/>
            {move ||
                if enable_seasons {
                    gleaderboard_with_seasons(state, show_guild_leaderboard)
                } else {
                    gleaderboard_no_seasons(state, show_guild_leaderboard)
                }
            }
        </div>
    }
}

fn gleaderboard_with_seasons(
    state: Signal<BTreeMap<Arc<str>, TerrState>>,
    show_guild_leaderboard: RwSignal<bool>,
) -> AnyView {
    let seasons = RwSignal::new(SeasonList::default());
    let selected_season = RwSignal::new(None::<Arc<str>>);

    spawn_local(load_seasons(seasons));

    let season_scores = LocalResource::new(move || {
        let selected_season = selected_season.get();

        async {
            if let Some(s) = selected_season {
                let lb =
                    datasource::load_json::<SeasonLeaderboard>(format!("/api/v3/lb/season/{s}"))
                        .await;

                if let Ok(lb) = lb {
                    return Some(lb);
                }
            }

            None
        }
    });

    // Update the seasons every hour
    let season_data_updater = set_interval_with_handle(
        move || {
            spawn_local(load_seasons(seasons));

            if let Some(Some(sl)) = season_scores.get()
                && (sl.season.end + Duration::from_hours(10)) > Timestamp::now()
            {
                season_scores.refetch();
            }
        },
        Duration::from_hours(1),
    )
    .ok();

    on_cleanup(move || {
        if let Some(i) = season_data_updater {
            i.clear();
        }
    });

    let scores = move || {
        if let Some(Some(s)) = season_scores.get() {
            s.leaderboard
        } else {
            terr_count_score(state)
        }
    };

    view! {
        <div class="title" on:click={move |_| show_guild_leaderboard.update(|s| *s = !*s)}>
            <h2>"Guild leaderboard"</h2>

            <select
                class="select"
                on:change:target=move |ev| {
                    let val = ev.target().value();

                    if val == "territory" {
                        selected_season.set(None);
                    } else {
                        selected_season.set(Some(val.into()));
                    }
                }
                prop:value=move || {
                    if let Some(s) = selected_season.get() {
                        s
                    } else {
                        "territory".into()
                    }
                }
                on:click=move |ev| {
                    ev.stop_propagation();
                }
            >
                <option value="territory">"Territories"</option>
                <option value="" disabled="disabled">"────────"</option>

                {move || seasons.get().seasons.into_iter().map(|s| {
                    let value = s.name.clone();

                    view! {
                        <option value=value>"Season "{s.name}</option>
                    }
                }).collect::<Vec<_>>()}
            </select>

            <Show when=move || !show_guild_leaderboard.get()><icons::ChevronUp size=24/></Show>
            <Show when=move || show_guild_leaderboard.get()><icons::ChevronDown size=24/></Show>
        </div>
        <div class="content" class:hidden={move || !show_guild_leaderboard.get()}>
            <hr/>
            <GleaderboardContent scores/>
        </div>
    }
    .into_any()
}

fn gleaderboard_no_seasons(
    state: Signal<BTreeMap<Arc<str>, TerrState>>,
    show_guild_leaderboard: RwSignal<bool>,
) -> AnyView {
    let scores = move || terr_count_score(state);

    view! {
        <div class="title" on:click={move |_| show_guild_leaderboard.update(|s| *s = !*s)}>
            <h2>"Guild leaderboard"</h2>
            <Show when=move || !show_guild_leaderboard.get()><icons::ChevronUp size=24/></Show>
            <Show when=move || show_guild_leaderboard.get()><icons::ChevronDown size=24/></Show>
        </div>
        <div class="content" class:hidden={move || !show_guild_leaderboard.get()}>
            <hr/>
            <GleaderboardContent scores/>
        </div>
    }
    .into_any()
}

async fn load_seasons(seasons: RwSignal<SeasonList>) {
    if let Ok(data) = datasource::load_json::<SeasonList>("/api/v3/lb/seasons").await {
        seasons.set(data);
    }
}

fn terr_count_score(state: Signal<BTreeMap<Arc<str>, TerrState>>) -> Vec<GuildLeaderboardEntry> {
    let mut guilds = HashMap::new();

    for v in state.read().values() {
        let guild = v.guild.clone();
        let terr = guilds.entry(guild).or_insert(0);
        *terr += 1;
    }

    let mut leaderboard: Vec<_> = guilds
        .into_iter()
        .map(|(guild, score)| GuildLeaderboardEntry { guild, score })
        .collect();

    // alphabetically sort the guilds
    leaderboard.sort_by(|a, b| a.guild.name.cmp(&b.guild.name));
    // sort by the number of territories while keeping the alphabetical order for any ties
    leaderboard.sort_by_key(|b| std::cmp::Reverse(b.score));

    leaderboard
}

#[component]
fn GleaderboardContent(#[prop(into)] scores: Signal<Vec<GuildLeaderboardEntry>>) -> impl IntoView {
    view! {
        <div class="content-inner">
            <For
                each=move || scores.get().into_iter()
                key=|v| v.clone()
                children=move |score| {
                    let col = score.guild.get_color().to_rgb_values();
                    let name = score.guild.name.clone();
                    let link = move || format!("https://wynncraft.com/stats/guild/{}", name.clone());

                    view! {
                        <a class="glrow" style:--col=col href=link() target="_blank">
                            <div/>
                            <span>"["{score.guild.prefix}"]"</span>
                            <span>{score.guild.name}</span>
                            <span>{score.score}</span>
                        </a>
                    }
                }
            />
        </div>
    }
}

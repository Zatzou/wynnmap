use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    sync::Arc,
};

use leptos::prelude::*;
use wynnmap_types::terr::{TerrOwner, Territory};

use crate::{
    components::{
        checkbox::Checkbox,
        gleaderboard::Gleaderboard,
        incrementor::Incrementor,
        loader::loader,
        sidebar::Sidebar,
        sidecard::{
            SideCard, SideCardHover,
            terr::{GuildInfo, TerrInfo},
        },
    },
    datasource,
    settings::use_toggle,
    wynnmap::{WynnMap, conns::Connections, maptile::DefaultMapTiles, terrs::TerrView},
};

#[component]
pub fn WarMap() -> impl IntoView {
    let data: LocalResource<Result<_, String>> = LocalResource::new(async move || {
        let terrs = datasource::get_terrs().await?;
        let owners = datasource::get_owners().await?;

        Ok((terrs, owners))
    });

    move || {
        loader(data, |(terrs, owners)| {
            warmap_inner(terrs, owners).into_any()
        })
    }
}

fn warmap_inner(
    terrs: HashMap<Arc<str>, Territory>,
    owners: HashMap<Arc<str>, TerrOwner>,
) -> impl IntoView {
    let show_terrs = use_toggle("terrs", true);
    let show_conns = use_toggle("conns", true);
    let show_res = use_toggle("resico", true);
    let show_timers = use_toggle("timers", true);
    let show_guild_leaderboard = use_toggle("gleaderboard", true);

    let terrs = RwSignal::new(terrs);
    let owners = RwSignal::new(owners);

    datasource::ws_terr_changes(owners).unwrap();

    let hovered = RwSignal::new(None);
    let selected = RwSignal::new(None);

    view! {
        <WynnMap>
            <DefaultMapTiles />

            // conns
            <Show when={move || show_conns.get()}>
                <Connections terrs={terrs} />
            </Show>

            // territories
            <Show when={move || show_terrs.get()}>
                <TerrView terrs={terrs} owners={owners} hovered=hovered selected=selected />
            </Show>
        </WynnMap>

        // hover box
        {move || if let Some(hovered) = hovered.get() {
            if selected.get().is_some() {
                return None;
            }

            Some(view! {
                <SideCardHover>
                    <TerrStats
                        name={hovered}
                        terrs={terrs}
                        owners={owners}
                    />
                </SideCardHover>
            })
        } else {None}}

        <Sidebar>
            // checkboxes
            <div class="flex-1 flex flex-col gap-2 p-2 text-lg">
                <div>
                    <Checkbox id="terrs" checked={show_terrs}>"Territories"</Checkbox>
                    <div class="flex flex-col gap-1 ml-6" class:hidden={move || !show_terrs.get()}>
                        <Checkbox id="resico" checked={show_res}>"Resource icons"</Checkbox>
                        <Checkbox id="timers" checked={show_timers}>"Timers"</Checkbox>
                    </div>
                </div>
                <Checkbox id="conns" checked={show_conns}>"Connections"</Checkbox>
            </div>

            // guild leaderboard
            <div class="flex flex-col min-h-0">
                <hr class="border-neutral-600" />
                <div class="flex justify-between items-center text-xl p-2 py-1" on:click={move |_| show_guild_leaderboard.set(!show_guild_leaderboard.get())}>
                    <h2>"Guild leaderboard"</h2>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6 cursor-pointer" >
                        <path stroke-linecap="round" stroke-linejoin="round" d="m4.5 15.75 7.5-7.5 7.5 7.5" class:hidden={move || show_guild_leaderboard.get()} />
                        <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" class:hidden={move || !show_guild_leaderboard.get()} />
                    </svg>
                </div>
                <hr class="border-neutral-600" class:hidden={move || !show_guild_leaderboard.get()} />
                <div class="overflow-y-auto shrink min-h-0">
                    <Gleaderboard owners={owners} class="w-full" class:hidden={move || !show_guild_leaderboard.get()} />
                </div>
            </div>
        </Sidebar>

        // selected terr info
        {move || selected.get().map(|sel| {
            let sel2 = sel.clone();

            Some(view! {
                <SideCard closefn={move || selected.set(None)}>
                    <TerrStats name={sel} terrs={terrs} owners={owners} />

                    <TerrCalc name={sel2} terrs={terrs} owners={owners} />
                </SideCard>
            })
        })}
    }
}

#[component]
fn TerrStats(
    #[prop(into)] name: Signal<Arc<str>>,
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
    #[prop(into)] owners: Signal<HashMap<Arc<str>, TerrOwner>>,
) -> impl IntoView {
    let owner = move || owners.read().get(&name.get()).cloned().unwrap_or_default();

    view! {
        <TerrInfo name={name} terrs={terrs} />

        <hr class="border-neutral-600" />

        <GuildInfo owner={Signal::derive(owner)} />
    }
}

#[component]
fn TerrCalc(
    #[prop(into)] name: Signal<Arc<str>>,
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
    #[prop(into)] owners: Signal<HashMap<Arc<str>, TerrOwner>>,
) -> impl IntoView {
    let guild = Memo::new(move |_| {
        owners
            .read()
            .get(&name.get())
            .map_or_else(Arc::default, |t| t.guild.prefix.clone())
    });

    let conn_names = Memo::new(move |_| {
        terrs
            .read()
            .get(&name.get())
            .map_or_else(HashSet::new, |e| e.connections.clone())
    });
    let ext_names =
        Memo::new(move |_| wynnmap_types::terr::find_externals(&name.read(), &terrs.read()));

    let hq = RwSignal::new(false);

    let damage = RwSignal::new(11);
    let attacks = RwSignal::new(11);
    let health = RwSignal::new(11);
    let defense = RwSignal::new(11);
    let aura = RwSignal::new(0);
    let volley = RwSignal::new(0);

    let conns = RwSignal::new(
        conn_names
            .read_untracked()
            .iter()
            .filter(|n| {
                owners
                    .read_untracked()
                    .get(*n)
                    .is_some_and(|t| t.guild.prefix == *guild.read_untracked())
            })
            .count() as i32,
    );
    let max_conns = Memo::new(move |_| conn_names.read().len() as i32);
    let externs = RwSignal::new(
        ext_names
            .read_untracked()
            .iter()
            .filter(|n| {
                owners
                    .read_untracked()
                    .get(*n)
                    .is_some_and(|t| t.guild.prefix == *guild.read_untracked())
            })
            .count() as i32,
    );
    let max_externs = Memo::new(move |_| ext_names.read().len() as i32);

    // calculate a stat based on the current values
    let calc_stat = move |val: f64| {
        if hq.get() {
            val * (0.25f64.mul_add(f64::from(externs.get()), 1.5))
                * (0.3f64.mul_add(f64::from(conns.get()), 1.0))
        } else {
            val * (0.3f64.mul_add(f64::from(conns.get()), 1.0))
        }
    };

    let def_num = Memo::new(move |_| {
        let x =
            damage.get() + attacks.get() + health.get() + defense.get() + aura.get() + volley.get();

        let mut x = x as i32;

        if aura.get() == 0 {
            x -= 5;
        }
        if volley.get() == 0 {
            x -= 3;
        }

        x
    });

    let def_name = move || {
        let def_num = def_num.get();
        if def_num >= 41 {
            "Very High"
        } else if def_num >= 23 {
            "High"
        } else if def_num >= 11 {
            "Medium"
        } else if def_num >= -2 {
            "Low"
        } else {
            "Very Low"
        }
    };

    let def_col = move || {
        let def_num = def_num.get();
        if def_num >= 41 {
            "oklch(0.637 0.237 25.331)"
        } else if def_num >= 23 {
            "oklch(0.705 0.213 47.604)"
        } else if def_num >= 11 {
            "oklch(0.795 0.184 86.047)"
        } else if def_num >= -2 {
            "oklch(0.768 0.233 130.85)"
        } else {
            "oklch(0.723 0.219 149.579)"
        }
    };

    view! {
        <hr class="border-neutral-600" />

        <div class="p-2">
            <h1 class="text-xl">"Tower"</h1>
            <div class="p-2">
                <Checkbox id="hq" checked={hq}>"HQ"</Checkbox>

                <div class="flex flex-col gap-2">
                    <div class="flex justify-between">
                        <h2>"Damage: "{move || fmt_num(calc_stat(DAMAGES[damage.get()].start))}" - "{move || fmt_num(calc_stat(DAMAGES[damage.get()].end))}</h2>
                        <Incrementor value={damage} max=11 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Attacks per second: "{move || ATTACK_RATES[attacks.get()]}</h2>
                        <Incrementor value={attacks} max=11 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Health: "{move || fmt_num(calc_stat(HEALTHS[health.get()]))}</h2>
                        <Incrementor value={health} max=11 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Defense: "{move || DEFENSES[defense.get()] * 100.0}"%"</h2>
                        <Incrementor value={defense} max=11 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Aura: "{move || AURA_TIMES[aura.get()]}</h2>
                        <Incrementor value={aura} max=3 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Volley: "{move || VOLLEY_TIMES[volley.get()]}</h2>
                        <Incrementor value={volley} max=3 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Connections: "{conns}"/"{max_conns}</h2>
                        <Incrementor value={conns} max={max_conns} />
                    </div>
                    <div class="flex justify-between" class:hidden={move || !hq.get()}>
                        <h2>"Externals: "{externs}/{max_externs}</h2>
                        <Incrementor value={externs} max={max_externs} />
                    </div>
                </div>
            </div>
        </div>

        <hr class="border-neutral-600" />

        <div class="p-4">
            <h2>"Avg DPS: "{move || {
                let dmg_low = calc_stat(DAMAGES[*damage.read()].start);
                let dmg_high = calc_stat(DAMAGES[*damage.read()].end);
                let dmg_avg = f64::midpoint(dmg_low, dmg_high);

                let att_rate = ATTACK_RATES[*attacks.read()];

                fmt_num(dmg_avg * att_rate)
            }}</h2>
            <h2>"EHP: "{move || {
                let health = calc_stat(HEALTHS[*health.read()]);
                let def = DEFENSES[*defense.read()];

                fmt_num(health / (1.0 - def))
            }}</h2>
            <h2>"Defense: "<span style:color=move || def_col()>{move || def_name()}</span></h2>
        </div>
    }
}

fn fmt_num(n: f64) -> String {
    let s = format!("{n:.2}");
    let mut out = String::new();

    let (start, end) = s.split_once('.').unwrap();

    for (i, c) in start.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.insert(0, ' ');
        }
        out.insert(0, c);
    }

    out.push('.');
    out.push_str(end);

    if out.ends_with(".00") {
        out.truncate(out.len() - 3);
    } else if out.ends_with(".0") {
        out.truncate(out.len() - 2);
    }

    out
}

const DAMAGES: [Range<f64>; 12] = [
    1000.0..1500.0,
    1400.0..2100.0,
    1800.0..2700.0,
    2200.0..3300.0,
    2600.0..3900.0,
    3000.0..4500.0,
    3400.0..5100.0,
    3800.0..5700.0,
    4200.0..6300.0,
    4600.0..6900.0,
    5000.0..7500.0,
    5400.0..8100.0,
];

const ATTACK_RATES: [f64; 12] = [0.5, 0.75, 1.0, 1.25, 1.6, 2.0, 2.5, 3.0, 3.6, 3.8, 4.2, 4.7];

const HEALTHS: [f64; 12] = [
    300_000.0,
    450_000.0,
    600_000.0,
    750_000.0,
    960_000.0,
    1_200_000.0,
    1_500_000.0,
    1_860_000.0,
    2_220_000.0,
    2_580_000.0,
    2_940_000.0,
    3_300_000.0,
];

const DEFENSES: [f64; 12] = [
    0.1, 0.4, 0.55, 0.625, 0.7, 0.75, 0.79, 0.82, 0.84, 0.86, 0.88, 0.9,
];

const AURA_TIMES: [&str; 4] = ["N/A", "24s", "18s", "12s"];
const VOLLEY_TIMES: [&str; 4] = ["N/A", "20s", "15s", "10s"];

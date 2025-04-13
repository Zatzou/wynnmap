use std::{collections::HashMap, ops::Range, sync::Arc, time::Duration};

use leptos::prelude::*;
use wynnmap_types::{ExTerrInfo, Territory};

use crate::{
    components::{
        checkbox::Checkbox, gleaderboard::Gleaderboard, incrementor::Incrementor, sidebar::Sidebar,
    },
    datasource,
    settings::use_toggle,
    wynnmap::{WynnMap, conns::Connections, maptile::DefaultMapTiles, terrs::TerrView},
};

#[component]
pub fn WarMap() -> impl IntoView {
    let show_terrs = use_toggle("terrs", true);
    let show_conns = use_toggle("conns", true);
    let show_res = use_toggle("resico", true);
    let show_timers = use_toggle("timers", true);
    let show_guild_leaderboard = use_toggle("gleaderboard", true);

    let extradata =
        LocalResource::new(async move || datasource::get_extra_terr_info().await.unwrap());

    let extradata = move || extradata.get().map_or_else(HashMap::new, |t| t.take());

    let terrs = RwSignal::new(HashMap::new());

    datasource::ws_terr_changes(terrs).unwrap();

    let hovered = RwSignal::new(None);
    let selected = RwSignal::new(None);

    view! {
        <WynnMap>
            <DefaultMapTiles />

            // conns
            <Show when={move || show_conns.get()}>
                <Connections terrs={terrs} extradata={Signal::derive(extradata)} />
            </Show>

            // territories
            <Show when={move || show_terrs.get()}>
                <TerrView terrs={terrs} extradata={Signal::derive(extradata)} hovered=hovered selected=selected />
            </Show>
        </WynnMap>

        // hover box
        {move || if let Some(hovered) = hovered.get() {
            if selected.get().is_some() {
                return None;
            }

            Some(view! {
                <div class="fixed top-4 right-4 bg-neutral-900 text-white rounded-md w-sm terrinfo-hoverbox pointer-events-none">
                    <TerrStats name={hovered} terrs={terrs} extradata={Signal::derive(extradata)} />
                </div>
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
                <div class="overflow-scroll shrink min-h-0">
                    <Gleaderboard terrs={terrs} class="w-full" class:hidden={move || !show_guild_leaderboard.get()} />
                </div>
            </div>
        </Sidebar>

        // selected terr info
        {move || selected.get().map(|sel| {
            let sel2 = sel.clone();

            Some(view! {
                <div class="fixed top-0 right-0 bg-neutral-900 text-white w-full max-w-full md:max-w-sm md:top-4 md:right-4 md:rounded max-h-dvh overflow-x-hidden overflow-y-auto">
                    // close button
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 cursor-pointer absolute top-2 right-2" on:click={move |_| selected.set(None)}>
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                    </svg>

                    <TerrStats name={sel} terrs={terrs} extradata={Signal::derive(extradata)} />

                    <TerrCalc name={sel2} terrs={terrs} extradata={Signal::derive(extradata)} />
                </div>
            })
        })}
    }
}

#[component]
fn TerrStats(
    #[prop(into)] name: Signal<Arc<str>>,
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
    extradata: Signal<HashMap<Arc<str>, ExTerrInfo>>,
) -> impl IntoView {
    let exdata = Memo::new(move |_| extradata.get().get(&name.get()).unwrap().clone());
    let t = Memo::new(move |_| terrs.get().get(&name.get()).unwrap().clone());

    let now = chrono::Utc::now();
    let time = now
        .signed_duration_since(t.read_untracked().acquired)
        .num_seconds();

    let (time, set_time) = signal(time);

    let i = set_interval_with_handle(
        move || {
            let now = chrono::Utc::now();

            let time = now.signed_duration_since(t.read().acquired).num_seconds();

            set_time.set(time);
        },
        Duration::from_millis(1000),
    )
    .ok();

    on_cleanup(move || {
        if let Some(i) = i {
            i.clear();
        }
    });

    let timestr = move || {
        let time = time.get();

        let days = time / 86400;
        let hours = (time % 86400) / 3600;
        let minutes = (time % 3600) / 60;
        let seconds = time % 60;

        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    };

    let treas_col = move || {
        let time = time.get();

        // times based on treasury
        if time < 3600 {
            "oklch(0.723 0.219 149.579)"
        } else if time < (3600 * 24) {
            "oklch(0.768 0.233 130.85)"
        } else if time < (3600 * 24 * 5) {
            "oklch(0.795 0.184 86.047)"
        } else if time < (3600 * 24 * 12) {
            "oklch(0.705 0.213 47.604)"
        } else {
            "oklch(0.637 0.237 25.331)"
        }
    };

    let treas_text = move || {
        let time = time.get();

        // times based on treasury
        if time < 3600 {
            "Very Low"
        } else if time < (3600 * 24) {
            "Low"
        } else if time < (3600 * 24 * 5) {
            "Medium"
        } else if time < (3600 * 24 * 12) {
            "High"
        } else {
            "Very High"
        }
    };

    view! {
        <div class="p-2">
            <h1 class="text-2xl">{name}</h1>

            <div class="p-2">
                <Show when={move || exdata.read().resources.emeralds > 0}>
                    <div class="flex gap-1 items-center">
                        <div class="icon-emerald"></div>
                        <p>"+"{exdata.read().resources.emeralds}" emeralds per hour"</p>
                    </div>
                </Show>
                <Show when={move || exdata.read().resources.ore > 0}>
                    <div class="flex gap-1 items-center">
                        <div class="icon-ores"></div>
                        <p>"+"{exdata.read().resources.ore}" ore per hour"</p>
                    </div>
                </Show>
                <Show when={move || exdata.read().resources.wood > 0}>
                    <div class="flex gap-1 items-center">
                        <div class="icon-wood"></div>
                        <p>"+"{exdata.read().resources.wood}" wood per hour"</p>
                    </div>
                </Show>
                <Show when={move || exdata.read().resources.fish > 0}>
                    <div class="flex gap-1 items-center">
                        <div class="icon-fish"></div>
                        <p>"+"{exdata.read().resources.fish}" fish per hour"</p>
                    </div>
                </Show>
                <Show when={move || exdata.read().resources.crops > 0}>
                    <div class="flex gap-1 items-center">
                        <div class="icon-crops"></div>
                        <p>"+"{exdata.read().resources.crops}" crops per hour"</p>
                    </div>
                </Show>
            </div>
        </div>

        <hr class="border-neutral-600" />

        <div class="p-2">
            <h1 class="text-xl">
                {move || t.get().guild.name}" "
                <span class="font-mono">"["{move || t.get().guild.prefix}"]"</span>
            </h1>

            <div class="p-2">
                <h2>"Time held: "{move || timestr()}</h2>
                <h2>"Treasury: "<span style:color=move || treas_col()>{move || treas_text()}</span></h2>
            </div>
        </div>
    }
}

#[component]
fn TerrCalc(
    #[prop(into)] name: Signal<Arc<str>>,
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
    extradata: Signal<HashMap<Arc<str>, ExTerrInfo>>,
) -> impl IntoView {
    let guild = Memo::new(move |_| {
        terrs
            .read()
            .get(&name.get())
            .map_or_else(Arc::default, |t| t.guild.prefix.clone())
    });
    let conn_names = Memo::new(move |_| {
        extradata
            .read()
            .get(&name.get())
            .map_or_else(Vec::new, |e| e.conns.clone())
    });
    let ext_names =
        Memo::new(move |_| wynnmap_types::util::find_externals(name.get(), extradata.get()));

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
                terrs
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
                terrs
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
        let mut x =
            damage.get() + attacks.get() + health.get() + defense.get() + aura.get() + volley.get();
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
                        <h2>"Damage: "{move || fmt_num(calc_stat(DAMAGES[damage.get() as usize].start))}" - "{move || fmt_num(calc_stat(DAMAGES[damage.get() as usize].end))}</h2>
                        <Incrementor value={damage} max=11 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Attacks per second: "{move || ATTACK_RATES[attacks.get() as usize]}</h2>
                        <Incrementor value={attacks} max=11 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Health: "{move || fmt_num(calc_stat(HEALTHS[health.get() as usize]))}</h2>
                        <Incrementor value={health} max=11 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Defense: "{move || DEFENSES[defense.get() as usize] * 100.0}"%"</h2>
                        <Incrementor value={defense} max=11 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Aura: "{move || AURA_TIMES[aura.get() as usize]}</h2>
                        <Incrementor value={aura} max=3 />
                    </div>
                    <div class="flex justify-between">
                        <h2>"Volley: "{move || VOLLEY_TIMES[volley.get() as usize]}</h2>
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
                let dmg_low = calc_stat(DAMAGES[*damage.read() as usize].start);
                let dmg_high = calc_stat(DAMAGES[*damage.read() as usize].end);
                let dmg_avg = (dmg_low + dmg_high) / 2.0;

                let att_rate = ATTACK_RATES[*attacks.read() as usize];

                fmt_num(dmg_avg * att_rate)
            }}</h2>
            <h2>"EHP: "{move || {
                let health = calc_stat(HEALTHS[*health.read() as usize]);
                let def = DEFENSES[*defense.read() as usize];

                fmt_num(health / (1.0 - def))
            }}</h2>
            <h2>"Defense: "<span style:color=move || def_col()>{move || def_name()}</span></h2>
        </div>
    }
}

fn fmt_num(n: f64) -> String {
    let s = format!("{:.2}", n);
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

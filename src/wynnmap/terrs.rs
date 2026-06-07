use std::{collections::BTreeMap, sync::Arc, time::Duration};

use leptos::prelude::*;
use wynnmap_types::{
    resources::BaseResGen,
    terr::{TerrState, Territory},
    tier::WynnTier,
};

use crate::{
    sectimer::SecondTimer,
    settings::use_toggle,
    util::{as_px, fmt_time_short},
    wynnmap::RelMousePos,
};

#[component]
pub fn TerrView(
    #[prop(into)] terrs: Signal<BTreeMap<Arc<str>, Territory>>,
    #[prop(into)] state: Signal<BTreeMap<Arc<str>, TerrState>>,
    #[prop(optional)] hovered: RwSignal<Option<Arc<str>>>,
    #[prop(optional)] hide_timers: bool,
) -> impl IntoView {
    let pos = expect_context::<RelMousePos>();

    Effect::new(move || {
        if let Some(pos) = *pos.0.read() {
            let t = terrs
                .read()
                .iter()
                .find(|(_, t)| t.location.contains(pos))
                .map(|(n, _)| n.clone());

            hovered.set(t);
        } else {
            hovered.set(None);
        }
    });

    view! {
        <div class="wynnmap-items">
            <For
                each=move || terrs.get().into_iter()
                key=move |(k, _)| k.clone()
                children=move |(name, terr)| {
                    let state = Memo::new({
                        let name = name.clone();
                        move |_| state.read().get(&name).cloned().unwrap_or_default()
                    });

                    view! {
                        <Territory terr state hide_timers/>
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn Territory(
    #[prop(into)] terr: Signal<Territory>,
    #[prop(into)] state: Signal<TerrState>,
    #[prop(optional)] hide_timers: bool,
) -> impl IntoView {
    let col_rgb = move || {
        let col = state.read().guild.get_color();
        format!("{} {} {}", col.0, col.1, col.2)
    };

    // toggles for showing territory parts
    let show_gtag = use_toggle("terrs_show_guildtag", true);
    let show_res = use_toggle("resico", true);
    let show_timers = use_toggle("timers", true);
    let use_transparency = use_toggle("use_transparency", true);

    let location = move || terr.read().location;
    let namesize = Memo::new(move |_| (location().width() / 3).min(40));

    view! {
        <div class="guildterr" class:hq={move || state.read().hq}
            class:guildterr-notrans=move || !use_transparency.get()
            style:width=move || as_px(location().width())
            style:height=move || as_px(location().height())
            style:top=move || as_px(location().top_side())
            style:left=move || as_px(location().left_side())
            style:--guild-col=move || col_rgb()
        >
            // attack timer border
            {move || state.read().acquired.map(|acquired| view! {
                <Show when={move || !hide_timers}>
                    <AttackBorder acquired/>
                </Show>
            })}

            // guild hq crown
            <Show when=move || state.read().hq>
                <div class="spriteicon icon-crown" />
            </Show>

            // guild tag
            <Show when={move || show_gtag.get()}>
                <h1
                    class="guildtag"
                    style:--tsize=move || as_px(namesize.read())
                >
                    {state.read().guild.prefix.clone()}
                </h1>
            </Show>

            // resource icons
            <Show when={move || show_res.get()}>
                <ResIcons terr={Signal::derive(move || terr.get().generates)} />
            </Show>

            // timer
            <Show when={move || show_timers.get() && !hide_timers}>
                {move || state.read().acquired.map(|acquired| view! {
                    <TerrTimer acquired/>
                })}
            </Show>
        </div>
    }
}

#[component]
fn ResIcons(terr: Signal<BaseResGen>) -> impl IntoView {
    view! {
        <div class="resicons wynnmap-hide-zoomedout" >
            {move || {
                let t = terr.read();

                [
                    (t.has_emerald(), "emeralds"),

                    (t.has_crop(), "crops"),
                    (t.has_double_crop(), "crops"),

                    (t.has_fish(), "fish"),
                    (t.has_double_fish(), "fish"),

                    (t.has_ore(), "ore"),
                    (t.has_double_ore(), "ore"),

                    (t.has_wood(), "wood"),
                    (t.has_double_wood(), "wood")
                ].into_iter()
                    .filter(|(b, _)| *b)
                    .map(|(_, n)| view! { <div class={move || format!("spriteicon icon-{n}")} /> })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}

#[component]
fn TerrTimer(#[prop(into)] acquired: Signal<chrono::DateTime<chrono::Utc>>) -> impl IntoView {
    let SecondTimer(now) = expect_context::<SecondTimer>();

    let time = Memo::new(move |_| now.read().signed_duration_since(acquired.read()));

    let timestr = move || fmt_time_short(time.get());

    let color = move || WynnTier::from_time_held(time.get()).color();

    view! {
        <div class="terrtimer">
            <h4 style:--bg-col={color}>{timestr}</h4>
        </div>
    }
}

/// The component rendering the attack timer border for territories which are on attack cooldown.
#[component]
fn AttackBorder(#[prop(into)] acquired: Signal<chrono::DateTime<chrono::Utc>>) -> impl IntoView {
    let now = chrono::Utc::now();
    let time = now
        .signed_duration_since(acquired.read_untracked())
        .num_milliseconds();

    let (time, set_time) = signal(time);

    move || {
        if time.get() < 599_000 {
            let h = set_timeout_with_handle(
                move || {
                    set_time.set(599_000);
                },
                Duration::from_millis((599_000 - time.get()).max(1000) as u64),
            )
            .ok();

            on_cleanup(move || {
                if let Some(i) = h {
                    i.clear();
                }
            });

            Some(view! {
                <div class="attacktmr" style:animation={move || format!("600s linear {}ms attackdelay", -time.get())} />
            }.into_any())
        } else if time.get() < 600_000 {
            let h = set_timeout_with_handle(
                move || {
                    set_time.set(600_000);
                },
                Duration::from_secs(1),
            )
            .ok();

            on_cleanup(move || {
                if let Some(i) = h {
                    i.clear();
                }
            });

            Some(view! {
                <div class="attacktmr" style:animation={move || String::from("0.2s linear 5 flash")} />
            }.into_any())
        } else {
            None
        }
    }
}

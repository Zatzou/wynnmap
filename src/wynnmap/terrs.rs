use std::{collections::BTreeMap, sync::Arc};

use jiff::{SignedDuration, Timestamp};
use leptos::prelude::*;
use wynnmap_types::{
    Region,
    resources::BaseResGen,
    terr::{TerrState, Territory},
    tier::WynnTier,
};

use crate::{
    sectimer::SecondTimer,
    settings::use_toggle,
    util::{as_px, fmt_time_short},
    wynnmap::context::RelMousePos,
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
                        move |_| state.read().get(&name).cloned().unwrap_or_default()
                    });

                    view! {
                        <TerritoryBox terr state/>
                    }
                }
            />
        </div>
        <Show when={move || !hide_timers}>
            <AttackBorders terrs state/>
        </Show>
        <div class="wynnmap-items">
            <For
                each=move || terrs.get().into_iter()
                key=move |(k, _)| k.clone()
                children=move |(name, terr)| {
                    let state = Memo::new({
                        move |_| state.read().get(&name).cloned().unwrap_or_default()
                    });

                    view! {
                        <TerritoryInfo terr state hide_timers/>
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn TerritoryBox(
    #[prop(into)] terr: Signal<Territory>,
    #[prop(into)] state: Signal<TerrState>,
) -> impl IntoView {
    let col_rgb = move || state.read().guild.get_color().to_rgb_values();

    let use_transparency = use_toggle("use_transparency", true);

    let location = move || terr.read().location;

    view! {
        <div
            class="guildterr-box"
            class:hq={move || state.read().hq}
            class:guildterr-notrans=move || !use_transparency.get()
            style:width=move || as_px(location().width())
            style:height=move || as_px(location().height())
            style:top=move || as_px(location().top_side())
            style:left=move || as_px(location().left_side())
            style:--guild-col=move || col_rgb()
        />
    }
}

#[component]
pub fn TerritoryInfo(
    #[prop(into)] terr: Signal<Territory>,
    #[prop(into)] state: Signal<TerrState>,
    #[prop(optional)] hide_timers: bool,
) -> impl IntoView {
    let top = move || {
        f64::from(terr.read().location.top_side())
            + (f64::from(terr.read().location.height()) / 2.0)
    };
    let left = move || {
        f64::from(terr.read().location.left_side())
            + (f64::from(terr.read().location.width()) / 2.0)
    };

    let show_gtag = use_toggle("terrs_show_guildtag", true);
    let show_res = use_toggle("resico", true);
    let show_timers = use_toggle("timers", true);

    let location = move || terr.read().location;
    let namesize = Memo::new(move |_| (location().width() / 3).min(40));

    view! {
        <div class="guildterr-info"
            style:top=move || as_px(top())
            style:left=move || as_px(left())
        >
            // guild hq crown
            <Show when=move || state.read().hq>
                <div class="spriteicon icon-crown" />
            </Show>

            // guild tag
            <Show when={move || show_gtag.get()}>
                <span
                    class="guildtag"
                    style:--tsize=move || as_px(namesize.read())
                >
                    {state.read().guild.prefix.clone()}
                </span>
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
fn TerrTimer(#[prop(into)] acquired: Signal<Timestamp>) -> impl IntoView {
    let SecondTimer(now) = expect_context::<SecondTimer>();

    let time = Memo::new(move |_| now.read().duration_since(*acquired.read()));

    let timestr = move || fmt_time_short(time.get());

    let color = move || WynnTier::from_time_held(time.get()).color();

    view! {
        <div class="terrtimer">
            <span style:--bg-col={color}>{timestr}</span>
        </div>
    }
}

#[component]
fn AttackBorders(
    #[prop(into)] terrs: Signal<BTreeMap<Arc<str>, Territory>>,
    #[prop(into)] state: Signal<BTreeMap<Arc<str>, TerrState>>,
) -> impl IntoView {
    let active = RwSignal::new(BTreeMap::new());

    // update list
    Effect::new(move || {
        let now = Timestamp::now();

        active.update(|active| {
            // remove old ones
            active.retain(|_, acq: &mut Timestamp| {
                acq.duration_until(now) <= SignedDuration::from_secs(601)
            });

            for (name, s) in state.read().iter() {
                if let Some(acq) = s.acquired
                    && acq.duration_until(now) <= SignedDuration::from_secs(601)
                    && let Some(terr) = terrs.read().get(name)
                {
                    active.insert(terr.location, acq);
                }
            }
        });
    });

    view! {
        <div class="wynnmap-items">
            <For
                each=move || active.get().into_iter()
                key=move |d| *d
                children=move |(reg, acq)| {

                    view! {
                        <AttackBorder reg acq/>
                    }
                }
            />
        </div>
    }
}

#[component]
fn AttackBorder(reg: Region, acq: Timestamp) -> impl IntoView {
    let now = Timestamp::now();
    let time = now.duration_since(acq).as_millis() as i64;

    view! {
        <div
            class="attackborder"
            style:width=as_px(reg.width() + 2)
            style:height=as_px(reg.height() + 2)
            style:top=as_px(reg.top_side() - 1)
            style:left=as_px(reg.left_side() - 1)

            style:animation-delay=format!("{}ms", -time)
        />
    }
}

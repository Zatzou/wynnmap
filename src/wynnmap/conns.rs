use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    sync::Arc,
};

use leptos::prelude::*;
use wynnmap_types::terr::Territory;

#[component]
pub fn Connections(#[prop(into)] terrs: Signal<BTreeMap<Arc<str>, Territory>>) -> impl IntoView {
    let conn_path = move || create_route_paths(&terrs.read());
    let bounds = Memo::new(move |_| bounds(&terrs.read()));

    let orig_x = move || bounds.read().0;
    let orig_y = move || bounds.read().1;
    let width = move || bounds.read().2;
    let height = move || bounds.read().3;

    let viewbox = move || format!("{} {} {} {}", orig_x(), orig_y(), width(), height());

    let style = move || {
        format!(
            "position:fixed;contain:strict;width:{}px;height:{}px;top:{}px;left:{}px",
            width(),
            height(),
            orig_y(),
            orig_x()
        )
    };

    view! {
        <svg style={style} viewBox={viewbox} >
            <path
                id="connpath"
                d={move || conn_path()}
                style="fill:none;"
                stroke-linecap="round"
            />
            <g inner_html=
            {
                "
                <use
                    href=\"#connpath\"
                    style=\"stroke:black;stroke-width:4;\"
                />
                <use
                    href=\"#connpath\"
                    style=\"stroke:white;stroke-width:2;\"
                />
                "
            }/>
        </svg>
    }
}

fn create_route_paths(terrs: &BTreeMap<Arc<str>, Territory>) -> String {
    let mut terr_conns: BTreeSet<((i32, i32), (i32, i32))> = BTreeSet::new();
    for (name, terr) in terrs {
        for conn in &terr.connections {
            if let Some(other_terr) = terrs.get(conn) {
                if name < conn {
                    terr_conns.insert((
                        terr.location.get_midpoint(),
                        other_terr.location.get_midpoint(),
                    ));
                } else {
                    terr_conns.insert((
                        other_terr.location.get_midpoint(),
                        terr.location.get_midpoint(),
                    ));
                }
            }
        }
    }

    let mut pathing = String::new();
    for (start, end) in terr_conns {
        write!(
            pathing,
            "M{} {}L{} {}",
            start.0,
            start.1, // x and y of starting point
            end.0,
            end.1 // x and y of ending point
        )
        .expect("Write should not fail");
    }

    pathing
}

fn bounds(terrs: &BTreeMap<Arc<str>, Territory>) -> (i32, i32, i32, i32) {
    let mut max_x = 0;
    let mut min_x = 0;
    let mut max_y = 0;
    let mut min_y = 0;

    for t in terrs.values() {
        let l = t.location;

        max_x = max_x.max(l.right_side());
        min_x = min_x.min(l.left_side());
        max_y = max_y.max(l.top_side());
        min_y = min_y.min(l.bottom_side());
    }

    (min_x, min_y, max_x - min_x, max_y - min_y)
}

use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use leptos::prelude::*;
use wynnmap_types::{ExTerrInfo, Territory};

#[component]
pub fn Connections(
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
    extradata: impl Fn() -> HashMap<Arc<str>, ExTerrInfo> + Send + Sync + 'static,
) -> impl IntoView {
    let conn_path = move || create_route_paths(&*terrs.read(), extradata());

    view! {
        <svg style="position: absolute;overflow: visible;contain: layout;" >
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

pub fn create_route_paths(
    terrs: &HashMap<Arc<str>, Territory>,
    extradata: HashMap<Arc<str>, ExTerrInfo>,
) -> String {
    let mut terr_conns: BTreeSet<((i32, i32), (i32, i32))> = BTreeSet::new();
    for (orig, v) in extradata {
        for conn in v.conns {
            if orig < conn {
                terr_conns.insert((
                    terrs
                        .get(&orig)
                        .map(|v| v.location.get_midpoint())
                        .unwrap_or((0, 0)),
                    terrs
                        .get(&conn)
                        .map(|v| v.location.get_midpoint())
                        .unwrap_or((0, 0)),
                ));
            } else {
                terr_conns.insert((
                    terrs
                        .get(&conn)
                        .map(|v| v.location.get_midpoint())
                        .unwrap_or((0, 0)),
                    terrs
                        .get(&orig)
                        .map(|v| v.location.get_midpoint())
                        .unwrap_or((0, 0)),
                ));
            }
        }
    }

    let mut pathing = String::new();
    for (start, end) in terr_conns {
        pathing.push_str(&format!(
            "M{} {} L{} {} ",
            start.0,
            start.1, // x and y of starting point
            end.0,
            end.1 // x and y of ending point
        ));
    }

    pathing
}

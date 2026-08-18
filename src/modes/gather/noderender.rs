use std::{collections::BTreeMap, sync::Arc};

use leptos::prelude::*;
use leptos_use::{UseWindowSizeReturn, use_window_size};
use wynnmap_types::gather::{GatherSpots, MatData, Material};

use crate::{
    modes::gather::clustering::cluster_all,
    wynnmap::context::{MapPosition, RelMousePos},
};

#[derive(Clone, PartialEq)]
pub struct GatherNode {
    pub pos: [i32; 2],
    pub radius: f64,

    pub count: usize,
    pub res: Material,
}

impl GatherNode {
    #[inline]
    const fn rad(&self, min_r: f64) -> f64 {
        self.radius.max(min_r)
    }

    fn contains(&self, point: [i32; 2], min_r: f64) -> bool {
        let dist_x = self.pos[0].abs_diff(point[0]);
        let dist_z = self.pos[1].abs_diff(point[1]);

        let dist = f64::from(dist_x.pow(2) + dist_z.pow(2));

        dist <= self.rad(min_r).powi(2)
    }

    fn within_area(&self, start: [f64; 2], end: [f64; 2], margin: f64) -> bool {
        let [x, y] = self.pos.map(f64::from);
        let r = self.radius;

        x + r >= start[0] - margin
            && x - r <= end[0] + margin
            && y + r >= start[1] - margin
            && y - r <= end[1]
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Setting {
    zoom: f64,
    max_d: f64,
    min_r: f64,
    stroke_w: f64,
}

impl Setting {
    const fn new(zoom: f64, max_d: f64, min_r: f64, stroke_w: f64) -> Self {
        Self {
            zoom,
            max_d,
            min_r,
            stroke_w,
        }
    }
}

const SETTINGS: [Setting; 4] = [
    Setting::new(1.0, 20.0, 10.0, 5.0),
    Setting::new(2.0, 10.0, 6.0, 2.5),
    Setting::new(3.0, 0.0, 3.5, 2.0),
    Setting::new(65.0, 0.0, 2.0, 1.0),
];

#[component]
pub fn NodeRenderer(
    nodes: RwSignal<GatherSpots>,
    data: RwSignal<BTreeMap<Arc<str>, MatData>>,
    hovered: RwSignal<Vec<GatherNode>>,
    #[prop(into, optional)] hidden: Signal<Vec<Arc<str>>>,
) -> impl IntoView {
    let RelMousePos(mouse_rel) = expect_context();
    let MapPosition {
        zoom,
        position: map_pos,
    } = expect_context();
    let UseWindowSizeReturn { width, height } = use_window_size();

    let style = Memo::new(move |_| {
        let ids = hidden
            .read()
            .iter()
            .map(|n| format!(".mat-{n}"))
            .collect::<Vec<_>>()
            .join(",");

        format!("{ids}{{display: none;}}")
    });

    let current_setting = Memo::new(move |_| {
        let zoom = zoom.get();

        for set in SETTINGS {
            if zoom < set.zoom {
                return set;
            }
        }

        unreachable!()
    });

    let clusters = Memo::new(move |_| cluster_all(&nodes.read(), current_setting.read().max_d));

    let culled = move || {
        let zoom = zoom.get();

        let start = map_pos.get().map(|p| -p / zoom);
        let end = [
            start[0] + width.get() / zoom,
            start[1] + height.get() / zoom,
        ];

        clusters
            .get()
            .into_iter()
            .filter(|node| node.within_area(start, end, 10.0 / zoom))
            .collect::<Vec<_>>()
    };

    let paths = move || build_paths(&culled(), current_setting.read().min_r);

    Effect::new(move || {
        let hov = if let Some(pos) = mouse_rel.get() {
            clusters
                .read()
                .iter()
                .filter(|n| n.contains(pos, current_setting.read().min_r))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        hovered.set(hov);
    });

    view! {
        <svg style="position: absolute; overflow: visible">
            <style>{style}</style>
            {move || paths().into_iter().map(|(mat_name, path)| {
                let matdata = data.read().get(&mat_name).cloned().unwrap_or_default();
                view!{
                    <path d=path fill=matdata.color.clone() stroke=matdata.prof.color() stroke-width=move || current_setting.read().stroke_w class=format!("mat-{}", mat_name) />
                }
            }).collect::<Vec<_>>()}
        </svg>
    }
}

fn build_paths(clusters: &[GatherNode], min_r: f64) -> Vec<(Arc<str>, String)> {
    let mut by_mat: BTreeMap<Arc<str>, String> = BTreeMap::new();
    let mut counts: BTreeMap<Arc<str>, usize> = BTreeMap::new();

    for c in clusters {
        let [cx, cy] = c.pos.map(f64::from);
        let r = c.rad(min_r);

        by_mat
            .entry(c.res.name.clone())
            .or_default()
            .push_str(&format!(
                "M{} {}a{} {} 0 1 0 {} 0a{} {} 0 1 0 {} 0z",
                cx - r,
                cy,
                r,
                r,
                r * 2.0,
                r,
                r,
                r * -2.0
            ));

        *counts.entry(c.res.name.clone()).or_default() += c.count;
    }

    let mut counts: Vec<_> = counts.into_iter().collect();
    counts.sort_by_key(|(_, c)| *c);
    counts.reverse();

    let mut out = Vec::new();

    for (name, _) in counts {
        out.push(by_mat.remove_entry(&name).unwrap());
    }

    out
}

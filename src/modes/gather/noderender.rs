use std::{collections::BTreeMap, fmt::Write, sync::Arc};

use leptos::prelude::*;
use leptos_use::{UseWindowSizeReturn, use_window_size};
use wynnmap_types::gather::{GatherSpots, MatData, Material};

use crate::{
    modes::gather::clustering::cluster_all,
    util::zip_map,
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

    #[inline]
    fn within_area(&self, start: [f64; 2], end: [f64; 2]) -> bool {
        let [x, y] = self.pos.map(f64::from);
        let r = self.radius;

        x.algebraic_add(r) >= start[0]
            && x.algebraic_sub(r) <= end[0]
            && y.algebraic_add(r) >= start[1]
            && y.algebraic_sub(r) <= end[1]
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

    let cull_pos = move || {
        let zoom = zoom.get();

        let start = map_pos
            .get()
            .map(|p| p.algebraic_mul(-1.0).algebraic_div(zoom));
        let end = zip_map(start, [width.get(), height.get()], |s, x| {
            s.algebraic_add(x.algebraic_div(zoom))
        });

        (start, end)
    };

    let paths = move || build_paths(&clusters.read(), current_setting.read().min_r, cull_pos());

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
                    <path
                        d=path
                        fill=matdata.color.clone()
                        stroke=matdata.prof.color()
                        stroke-width=move || current_setting.read().stroke_w
                        class=format!("mat-{}", mat_name)
                    />
                }
            }).collect::<Vec<_>>()}
        </svg>
    }
}

fn build_paths(
    clusters: &[GatherNode],
    min_r: f64,
    (cull_start, cull_end): ([f64; 2], [f64; 2]),
) -> Vec<(Arc<str>, String)> {
    let mut by_mat: BTreeMap<&Arc<str>, (String, usize)> = BTreeMap::new();

    for c in clusters
        .iter()
        .filter(|node| node.within_area(cull_start, cull_end))
    {
        let [cx, cy] = c.pos;
        let r = c.rad(min_r);

        let (pathstr, count) = by_mat.entry(&c.res.name).or_default();

        let _ = write!(
            pathstr,
            "M{} {}a{} {} 0 1 0 {} 0a{} {} 0 1 0 {} 0z",
            f64::from(cx).algebraic_sub(r),
            cy,
            r,
            r,
            r.algebraic_mul(2.0),
            r,
            r,
            r.algebraic_mul(-2.0)
        );

        *count += c.count;
    }

    let mut out: Vec<_> = by_mat.into_iter().collect();
    out.sort_by_key(|(_, c)| c.1);
    out.reverse();

    out.into_iter().map(|(k, (v, _))| (k.clone(), v)).collect()
}

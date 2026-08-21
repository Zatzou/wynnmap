use leptos::prelude::*;

macro_rules! icon {
    ($name:ident { $($svg:tt)+ }) => {

        #[component]
        pub fn $name(
            #[prop(default = 24.into(), into)] size: Signal<usize>,
            #[prop(default = "currentColor".into(), into)] color: Signal<String>,
            #[prop(default = "none".into(), into)] fill: Signal<String>,
            #[prop(default = 2.into(), into)] stroke_width: Signal<usize>,
        ) -> impl IntoView {
            view! {
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width=size
                    height=size
                    viewBox="0 0 24 24"
                    fill=fill
                    stroke=color
                    stroke-width=stroke_width
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    $($svg)+
                </svg>
            }
        }
    };
}

icon!(X { <path d="M18 6 6 18" /> <path d="m6 6 12 12" /> });

icon!(Menu { <path d="M4 5h16" /> <path d="M4 12h16" /> <path d="M4 19h16" /> });

icon!(Settings {
    <path
        d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915" />
    <circle cx="12" cy="12" r="3" />
});

icon!(ChevronUp { <path d="m18 15-6-6-6 6" /> });

icon!(ChevronDown { <path d="m6 9 6 6 6-6" /> });

icon!(Swords {
    <polyline points="14.5 17.5 3 6 3 3 6 3 17.5 14.5" />
    <line x1="13" x2="19" y1="19" y2="13" />
    <line x1="16" x2="20" y1="16" y2="20" />
    <line x1="19" x2="21" y1="21" y2="19" />
    <polyline points="14.5 6.5 18 3 21 3 21 6 17.5 9.5" />
    <line x1="5" x2="9" y1="14" y2="18" />
    <line x1="7" x2="4" y1="17" y2="20" />
    <line x1="3" x2="5" y1="19" y2="21" />
});

icon!(LandPlot {
    <path d="m12 8 6-3-6-3v10" />
    <path
        d="m8 11.99-5.5 3.14a1 1 0 0 0 0 1.74l8.5 4.86a2 2 0 0 0 2 0l8.5-4.86a1 1 0 0 0 0-1.74L16 12" />
    <path d="m6.49 12.85 11.02 6.3" />
    <path d="M17.51 12.85 6.5 19.15" />
});

icon!(Axe {
    <path d="m14 12-8.381 8.38a1 1 0 0 1-3.001-3L11 9" />
    <path
        d="M15 15.5a.5.5 0 0 0 .5.5A6.5 6.5 0 0 0 22 9.5a.5.5 0 0 0-.5-.5h-1.672a2 2 0 0 1-1.414-.586l-5.062-5.062a1.205 1.205 0 0 0-1.704 0L9.352 5.648a1.205 1.205 0 0 0 0 1.704l5.062 5.062A2 2 0 0 1 15 13.828z" />
});

icon!(Clipboard {
    <rect width="8" height="4" x="8" y="2" rx="1" ry="1" />
    <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
});

icon!(SquarePen {
    <path d="M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
    <path
        d="M18.375 2.625a1 1 0 0 1 3 3l-9.013 9.014a2 2 0 0 1-.853.505l-2.873.84a.5.5 0 0 1-.62-.62l.84-2.873a2 2 0 0 1 .506-.852z" />
});

icon!(Trash {
    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
    <path d="M3 6h18" />
    <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
});

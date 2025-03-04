use leptos::prelude::*;
use wynnmap_types::Guild;

#[component]
pub fn Gleaderboard(
    leaderboard: impl Fn() -> Vec<(Guild, i32)> + Send + Sync + 'static,
    class: &'static str,
) -> impl IntoView {
    view! {
        <table class={class} class:table-auto=true>
            <tbody>
                <For
                    each=move || leaderboard().into_iter()
                    key=|(k, v)| (k.clone(), v.clone())
                    children=move |(k, v)| {
                        let col = k.get_color();
                        let col = format!("{}, {}, {}", col.0, col.1, col.2);
                        let name = k.name.clone();
                        let link = move || format!("https://wynncraft.com/stats/guild/{}", name);
                        view! {
                            <tr class="even:bg-neutral-800" style={format!("background-color: rgba({}, 0.3)", col)}>
                                <td><a href={link()} target="_blank" class="block pl-2 font-mono">"["{k.prefix}"]"</a></td>
                                <td><a href={link()} target="_blank" class="block">{k.name}</a></td>
                                <td><a href={link()} target="_blank" class="block text-right pr-2">{v}</a></td>
                            </tr>
                        }
                    }
                />
            </tbody>
        </table>
    }
}

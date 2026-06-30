use grind_core::failure::Failure;
use grind_core::refusal::Refusal;
use grind_core::weigh_in::WeighIn;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::tauri;

fn today() -> String {
    let now = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        now.get_full_year(),
        now.get_month() + 1,
        now.get_date()
    )
}

fn row_key(weighed: &WeighIn) -> String {
    format!("{}:{}", weighed.date, weighed.kg)
}

fn message_of(failure: &Failure) -> String {
    match failure {
        Failure::Refused(Refusal::NotADate) => "Not a real date".into(),
        Failure::Refused(Refusal::NotAMass) => "A body mass above zero, please".into(),
        Failure::Refused(Refusal::NotAWaist) => "A waist above zero, please".into(),
        Failure::Refused(Refusal::NotAHeight) => "A height above zero, please".into(),
        Failure::Broken(reason) => format!("Something broke: {reason}"),
    }
}

#[component]
pub fn App() -> impl IntoView {
    let date = RwSignal::new(today());
    let kg = RwSignal::new(String::new());
    let weigh_ins = RwSignal::new(Vec::<WeighIn>::new());
    let problem = RwSignal::new(None::<String>);

    spawn_local(async move {
        match tauri::recent_weigh_ins().await {
            Ok(recent) => weigh_ins.set(recent),
            Err(failure) => problem.set(Some(message_of(&failure))),
        }
    });

    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let typed = kg.get_untracked().trim().replace(',', ".");
        let Ok(mass) = typed.parse::<f64>() else {
            problem.set(Some(message_of(&Failure::Refused(Refusal::NotAMass))));
            return;
        };
        let on = date.get_untracked();
        spawn_local(async move {
            match tauri::log_weigh_in(&on, mass).await {
                Ok(recent) => {
                    weigh_ins.set(recent);
                    kg.set(String::new());
                    problem.set(None);
                }
                Err(failure) => problem.set(Some(message_of(&failure))),
            }
        });
    };

    let last_mass =
        move || weigh_ins.with(|recent| recent.first().map(|latest| latest.kg.to_string()));

    view! {
        <main class="wrap">
            <header class="top">
                <h1>"GRINDSTONE"</h1>
                <span class="tag">"sharpen this rusty you!"</span>
            </header>
            <section class="card">
                <h2>"Weigh-in"</h2>
                <form on:submit=save>
                    <div class="row">
                        <label class="field grow2">
                            "Date"
                            <input
                                type="date"
                                prop:value=move || date.get()
                                on:input=move |event| date.set(event_target_value(&event))
                            />
                        </label>
                        <label class="field">
                            "Body mass (kg)"
                            <input
                                inputmode="decimal"
                                autocomplete="off"
                                placeholder=last_mass
                                prop:value=move || kg.get()
                                on:input=move |event| kg.set(event_target_value(&event))
                            />
                        </label>
                    </div>
                    <button class="cta" type="submit">"Save"</button>
                    {move || problem.get().map(|message| view! { <p class="bad">{message}</p> })}
                </form>
            </section>
            <section class="card">
                <h2>"Last weigh-ins"</h2>
                <Show
                    when=move || weigh_ins.with(|recent| !recent.is_empty())
                    fallback=|| view! { <p class="empty">"No weigh-in yet"</p> }
                >
                    <table>
                        <thead>
                            <tr>
                                <th>"Date"</th>
                                <th>"kg"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || weigh_ins.get()
                                key=row_key
                                children=|weighed| {
                                    view! {
                                        <tr>
                                            <td>{weighed.date}</td>
                                            <td>{format!("{:.1}", weighed.kg)}</td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </Show>
            </section>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_weighed_again_gets_a_new_row_so_the_list_redraws_it() {
        let first = WeighIn {
            date: "2026-10-03".into(),
            kg: 72.4,
        };
        let again = WeighIn {
            date: "2026-10-03".into(),
            kg: 72.1,
        };

        assert_ne!(row_key(&first), row_key(&again));
    }
}

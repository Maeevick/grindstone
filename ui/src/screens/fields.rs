use leptos::prelude::*;

use crate::choice::{Choice, matching};

#[component]
pub fn FigureField(
    label: &'static str,
    #[prop(into)] name: String,
    value: RwSignal<String>,
    #[prop(into, optional)] placeholder: MaybeProp<String>,
    #[prop(into, optional)] required: MaybeProp<bool>,
    #[prop(into, optional)] disabled: MaybeProp<bool>,
) -> impl IntoView {
    view! {
        <label class="field">
            <span>
                {label}
                {move || {
                    required
                        .get()
                        .unwrap_or(false)
                        .then(|| view! { <span class="required">" *"</span> })
                }}
            </span>
            <input
                name=name
                inputmode="decimal"
                autocomplete="off"
                placeholder=move || placeholder.get()
                prop:disabled=move || disabled.get().unwrap_or(false)
                prop:value=move || value.get()
                on:input=move |event| value.set(event_target_value(&event))
            />
        </label>
    }
}

#[component]
pub fn DateField(
    label: &'static str,
    name: &'static str,
    value: RwSignal<String>,
) -> impl IntoView {
    view! {
        <label class="field">
            {label}
            <span class="date">
                <span class="shown" class:unset=move || value.get().is_empty()>
                    {move || {
                        let day = value.get();
                        if day.is_empty() { "yyyy-mm-dd".to_string() } else { day }
                    }}
                </span>
                <input
                    name=name
                    type="date"
                    prop:value=move || value.get()
                    on:input=move |event| value.set(event_target_value(&event))
                />
            </span>
        </label>
    }
}

#[component]
pub fn Chooser(
    label: &'static str,
    name: &'static str,
    #[prop(into)] options: Signal<Vec<Choice>>,
    typed: RwSignal<String>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let active = RwSignal::new(None::<usize>);
    let field = format!("{name}-chooser");
    let list = format!("{name}-choices");
    let option_id = move |at: usize| format!("{name}-option-{at}");
    let offered = Signal::derive(move || {
        let offered = options.get();
        let names: Vec<String> = offered.iter().map(|choice| choice.name.clone()).collect();
        matching(&names, &typed.get())
            .into_iter()
            .map(|option| {
                let shown = offered
                    .iter()
                    .find(|choice| choice.name == option)
                    .map_or_else(|| option.clone(), Choice::shown);
                (option, shown)
            })
            .collect::<Vec<(String, String)>>()
    });
    let pick = move |option: String| {
        typed.set(option);
        open.set(false);
        active.set(None);
    };
    let keyed = move |event: leptos::ev::KeyboardEvent| {
        let count = offered.with_untracked(Vec::len);
        match event.key().as_str() {
            "ArrowDown" => {
                event.prevent_default();
                open.set(true);
                if count > 0 {
                    active.update(|at| *at = Some(at.map_or(0, |at| (at + 1).min(count - 1))));
                }
            }
            "ArrowUp" => {
                event.prevent_default();
                active.update(|at| *at = at.map(|at| at.saturating_sub(1)));
            }
            "Enter" => {
                let chosen = active
                    .get_untracked()
                    .filter(|_| open.get_untracked())
                    .and_then(|at| offered.with_untracked(|offered| offered.get(at).cloned()));
                if let Some((option, _)) = chosen {
                    event.prevent_default();
                    pick(option);
                }
            }
            "Escape" => {
                open.set(false);
                active.set(None);
            }
            _ => {}
        }
    };
    view! {
        <div class="field grow2 chooser">
            <label for=field.clone()>{label}</label>
            <input
                id=field
                name=name
                role="combobox"
                aria-autocomplete="list"
                aria-controls=list.clone()
                aria-expanded=move || open.get().to_string()
                aria-activedescendant=move || active.get().filter(|_| open.get()).map(option_id)
                autocomplete="off"
                placeholder="Select me"
                prop:value=move || typed.get()
                on:input=move |event| {
                    typed.set(event_target_value(&event));
                    open.set(true);
                    active.set(None);
                }
                on:keydown=keyed
                on:focus=move |_| open.set(true)
                on:blur=move |_| {
                    open.set(false);
                    active.set(None);
                }
            />
            <Show when=move || open.get()>
                <ul class="choices" role="listbox" id=list.clone()>
                    {move || {
                        offered
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(at, (option, shown))| {
                                view! {
                                    <li
                                        id=option_id(at)
                                        role="option"
                                        aria-selected=move || (active.get() == Some(at)).to_string()
                                        on:mousedown=move |event| {
                                            event.prevent_default();
                                            pick(option.clone());
                                        }
                                    >
                                        {shown}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
        </div>
    }
}

/// What a form will record, under its fields.
pub fn preview_line(line: impl Fn() -> Option<String> + Send + Sync + 'static) -> impl IntoView {
    move || line().map(|line| view! { <p class="hint preview">{line}</p> })
}

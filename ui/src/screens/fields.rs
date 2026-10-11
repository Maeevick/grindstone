use leptos::prelude::*;

use crate::choice::matching;

#[component]
pub fn FigureField(
    label: &'static str,
    name: &'static str,
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
    #[prop(into)] options: Signal<Vec<String>>,
    typed: RwSignal<String>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    view! {
        <div class="field grow2 chooser">
            <span>{label}</span>
            <input
                name=name
                autocomplete="off"
                placeholder="Select me"
                prop:value=move || typed.get()
                on:input=move |event| {
                    typed.set(event_target_value(&event));
                    open.set(true);
                }
                on:focus=move |_| open.set(true)
                on:blur=move |_| open.set(false)
            />
            <Show when=move || open.get()>
                <ul class="choices" role="listbox">
                    {move || {
                        matching(&options.get(), &typed.get())
                            .into_iter()
                            .map(|option| {
                                let picked = option.clone();
                                view! {
                                    <li
                                        role="option"
                                        on:mousedown=move |event| {
                                            event.prevent_default();
                                            typed.set(picked.clone());
                                            open.set(false);
                                        }
                                    >
                                        {option}
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

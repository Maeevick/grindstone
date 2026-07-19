use leptos::prelude::*;

#[component]
pub fn FigureField(
    label: &'static str,
    name: &'static str,
    value: RwSignal<String>,
) -> impl IntoView {
    view! {
        <label class="field">
            {label}
            <input
                name=name
                inputmode="decimal"
                autocomplete="off"
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
            <input
                name=name
                type="date"
                prop:value=move || value.get()
                on:input=move |event| value.set(event_target_value(&event))
            />
        </label>
    }
}

use grind_core::gender::Gender;
use leptos::prelude::*;

#[component]
pub fn ProfileScreen() -> impl IntoView {
    let gender = RwSignal::new(Gender::Male);
    let chosen = move |event: leptos::ev::Event| {
        gender.set(if event_target_value(&event) == "Female" {
            Gender::Female
        } else {
            Gender::Male
        });
    };
    view! {
        <section class="card">
            <h2>"Profile"</h2>
            <form>
                <div class="row">
                    <label class="field">
                        "Gender" <select name="gender" on:change=chosen>
                            <option value="Male">"Male"</option>
                            <option value="Female">"Female"</option>
                        </select>
                    </label>
                    <Show when=move || gender.get() == Gender::Female>
                        <label class="field">
                            "Hip (cm)" <input name="hip" inputmode="decimal" autocomplete="off" />
                        </label>
                    </Show>
                </div>
            </form>
        </section>
    }
}

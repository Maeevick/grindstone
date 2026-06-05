use grind_core::failure::Failure;
use grind_core::weigh_in::WeighIn;
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(command: &str, arguments: JsValue) -> Result<JsValue, JsValue>;
}

fn broken(error: impl ToString) -> Failure {
    Failure::Broken(error.to_string())
}

async fn call<Output: DeserializeOwned>(
    command: &str,
    arguments: impl Serialize,
) -> Result<Output, Failure> {
    let arguments = serde_wasm_bindgen::to_value(&arguments).map_err(broken)?;
    match invoke(command, arguments).await {
        Ok(answer) => serde_wasm_bindgen::from_value(answer).map_err(broken),
        Err(refusal) => Err(serde_wasm_bindgen::from_value(refusal).unwrap_or_else(broken)),
    }
}

#[derive(Serialize)]
struct Nothing {}

#[derive(Serialize)]
struct Weighing<'a> {
    date: &'a str,
    kg: f64,
}

pub async fn recent_weigh_ins() -> Result<Vec<WeighIn>, Failure> {
    call("recent_weigh_ins", Nothing {}).await
}

pub async fn log_weigh_in(date: &str, kg: f64) -> Result<Vec<WeighIn>, Failure> {
    call("log_weigh_in", Weighing { date, kg }).await
}

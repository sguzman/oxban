use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = quackbanInvoke)]
    fn quackban_invoke(command: &str, args: JsValue) -> js_sys::Promise;
}

pub async fn invoke<R: DeserializeOwned, A: Serialize>(
    command: &str,
    args: &A,
) -> Result<R, String> {
    let js_args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    tracing::debug!(command, "invoking tauri command");

    let promise = quackban_invoke(command, js_args);
    let value = JsFuture::from(promise)
        .await
        .map_err(|error| format!("js invoke error: {error:?}"))?;

    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn invoke_args<R: DeserializeOwned, A: Serialize>(
    command: &str,
    args: &A,
) -> Result<R, String> {
    #[derive(Serialize)]
    struct WrappedArgs<'a, T> {
        args: &'a T,
    }

    invoke(command, &WrappedArgs { args }).await
}

pub async fn invoke_no_args<R: DeserializeOwned>(command: &str) -> Result<R, String> {
    #[derive(Serialize)]
    struct Empty {}

    invoke(command, &Empty {}).await
}

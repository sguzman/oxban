mod app;
mod components;
mod routes;
mod state;
mod tauri_bridge;

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    tracing::info!("starting Oxban UI");
    yew::Renderer::<app::App>::new().render();
}

use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::board::BoardPage;
use crate::routes::Route;

#[function_component(App)]
pub fn app() -> Html {
    html! {
      <BrowserRouter>
        <Switch<Route> render={switch} />
      </BrowserRouter>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <BoardPage /> },
        Route::Board { .. } => html! { <BoardPage /> },
        Route::NotFound => html! { <div>{ "404" }</div> },
    }
}

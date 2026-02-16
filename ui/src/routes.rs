use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/board/:id")]
    Board { id: String },

    #[not_found]
    #[at("/404")]
    NotFound,
}

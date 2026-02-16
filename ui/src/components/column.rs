use oxban_core::{Card, Column};
use uuid::Uuid;
use yew::prelude::*;

use crate::components::card::CardView;

#[derive(Properties, PartialEq)]
pub struct ColumnProps {
    pub column: Column,
    pub cards: Vec<Card>,
    pub on_create_card: Callback<(Uuid, String)>,
    pub on_open_card: Callback<Uuid>,
    pub on_delete_card: Callback<Uuid>,
    pub on_move_card_to_column_end: Callback<(Uuid, Uuid)>,
    pub on_drag_start: Callback<Uuid>,
    pub on_drag_end: Callback<()>,
    pub on_drag_over_column: Callback<Uuid>,
    pub drop_hint: bool,
}

#[function_component(ColumnView)]
pub fn column_view(props: &ColumnProps) -> Html {
    let column_id = props.column.id;
    let new_title = use_state(String::new);

    let on_title_input = {
        let new_title = new_title.clone();
        Callback::from(move |event: InputEvent| {
            let value = event
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            new_title.set(value);
        })
    };

    let on_add = {
        let new_title = new_title.clone();
        let on_create_card = props.on_create_card.clone();
        Callback::from(move |_| {
            let title = new_title.trim().to_string();
            if !title.is_empty() {
                on_create_card.emit((column_id, title));
                new_title.set(String::new());
            }
        })
    };

    let on_drag_over = {
        let on_drag_over_column = props.on_drag_over_column.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            on_drag_over_column.emit(column_id);
        })
    };

    let on_drop = {
        let on_move_card_to_column_end = props.on_move_card_to_column_end.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            if let Some(data_transfer) = event.data_transfer() {
                if let Ok(raw_card_id) = data_transfer.get_data("text/plain") {
                    if let Ok(card_id) = Uuid::parse_str(&raw_card_id) {
                        on_move_card_to_column_end.emit((card_id, column_id));
                    }
                }
            }
        })
    };

    let class_name = if props.drop_hint {
        "column drop-hint"
    } else {
        "column"
    };

    html! {
      <div class={class_name} ondragover={on_drag_over} ondrop={on_drop}>
        <div class="column-header">
          <p class="column-title">{ props.column.name.clone() }</p>
          <span class="pill">{ props.cards.len() }</span>
        </div>

        <div class="column-body">
          <div class="row">
            <input placeholder="New card title" value={(*new_title).clone()} oninput={on_title_input} />
            <button class="btn primary" onclick={on_add}>{ "Add" }</button>
          </div>

          {
            for props.cards.iter().map(|card| {
                html! {
                  <CardView
                    card={card.clone()}
                    on_open={props.on_open_card.clone()}
                    on_delete={props.on_delete_card.clone()}
                    on_drag_start={props.on_drag_start.clone()}
                    on_drag_end={props.on_drag_end.clone()}
                  />
                }
            })
          }
        </div>
      </div>
    }
}

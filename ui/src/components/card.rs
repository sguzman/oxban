use oxban_core::Card;
use uuid::Uuid;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CardProps {
    pub card: Card,
    pub on_open: Callback<Uuid>,
    pub on_delete: Callback<Uuid>,
    pub on_drag_start: Callback<Uuid>,
    pub on_drag_end: Callback<()>,
}

#[function_component(CardView)]
pub fn card_view(props: &CardProps) -> Html {
    let card_id = props.card.id;

    let on_open = {
        let on_open = props.on_open.clone();
        Callback::from(move |_| on_open.emit(card_id))
    };

    let on_delete = {
        let on_delete = props.on_delete.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            on_delete.emit(card_id);
        })
    };

    let on_drag_start = {
        let on_drag_start = props.on_drag_start.clone();
        Callback::from(move |event: DragEvent| {
            if let Some(data_transfer) = event.data_transfer() {
                let _ = data_transfer.set_data("text/plain", &card_id.to_string());
            }
            on_drag_start.emit(card_id);
        })
    };

    let on_drag_end = {
        let on_drag_end = props.on_drag_end.clone();
        Callback::from(move |_| on_drag_end.emit(()))
    };

    let due_pill = props.card.due_date.map(|due| {
        html! {
            <span class="pill">{ format!("Due {}", due.date_naive()) }</span>
        }
    });

    html! {
      <div class="card" draggable="true" onclick={on_open} ondragstart={on_drag_start} ondragend={on_drag_end}>
        <div class="row" style="justify-content: space-between; align-items: flex-start;">
          <h3 class="card-title">{ props.card.title.clone() }</h3>
          <button class="btn danger" onclick={on_delete} title="Delete card">{ "x" }</button>
        </div>

        <div class="card-meta">
          {
            if props.card.priority > 0 {
                html! { <span class="pill">{ format!("P{}", props.card.priority) }</span> }
            } else {
                html! {}
            }
          }

          { due_pill.unwrap_or_else(|| html! {}) }

          {
            for props
                .card
                .tags
                .iter()
                .take(4)
                .map(|tag| html! { <span class="pill">{ tag }</span> })
          }
        </div>
      </div>
    }
}

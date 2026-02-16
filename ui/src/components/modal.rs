use oxban_core::Card;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ModalProps {
    pub card: Card,
    pub on_close: Callback<()>,
    pub on_save: Callback<Card>,
}

#[function_component(CardModal)]
pub fn card_modal(props: &ModalProps) -> Html {
    let working = use_state(|| props.card.clone());

    // Keep modal state in sync when a different card is opened.
    {
        let working = working.clone();
        let card = props.card.clone();
        use_effect_with(props.card.id, move |_| {
            working.set(card);
            || ()
        });
    }

    let on_close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    let on_title_input = {
        let working = working.clone();
        Callback::from(move |event: InputEvent| {
            let mut card = (*working).clone();
            card.title = event
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            working.set(card);
        })
    };

    let on_description_input = {
        let working = working.clone();
        Callback::from(move |event: InputEvent| {
            let mut card = (*working).clone();
            card.description = event
                .target_unchecked_into::<web_sys::HtmlTextAreaElement>()
                .value();
            working.set(card);
        })
    };

    let on_tags_input = {
        let working = working.clone();
        Callback::from(move |event: InputEvent| {
            let mut card = (*working).clone();
            let raw = event
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            card.tags = raw
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect();
            working.set(card);
        })
    };

    let on_priority_input = {
        let working = working.clone();
        Callback::from(move |event: InputEvent| {
            let mut card = (*working).clone();
            let raw = event
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            card.priority = raw.parse().unwrap_or(0);
            working.set(card);
        })
    };

    let on_save = {
        let on_save = props.on_save.clone();
        let working = working.clone();
        Callback::from(move |_| on_save.emit((*working).clone()))
    };

    let working_card = (*working).clone();
    let tags_csv = working_card.tags.join(", ");

    html! {
      <div class="modal-backdrop" onclick={on_close.clone()}>
        <div class="modal" onclick={Callback::from(|event: MouseEvent| event.stop_propagation())}>
          <div class="modal-header">
            <h2>{ "Card details" }</h2>
            <button class="btn" onclick={on_close.clone()}>{ "Close" }</button>
          </div>

          <div class="modal-body">
            <label class="col">
              <span class="pill">{ "Title" }</span>
              <input value={working_card.title} oninput={on_title_input} />
            </label>

            <label class="col">
              <span class="pill">{ "Description" }</span>
              <textarea value={working_card.description} oninput={on_description_input} />
            </label>

            <div class="row">
              <label class="col" style="flex: 1;">
                <span class="pill">{ "Tags (comma separated)" }</span>
                <input value={tags_csv} oninput={on_tags_input} />
              </label>

              <label class="col" style="width: 140px;">
                <span class="pill">{ "Priority" }</span>
                <input value={working_card.priority.to_string()} oninput={on_priority_input} />
              </label>
            </div>

            <div class="row" style="justify-content: flex-end;">
              <button class="btn primary" onclick={on_save}>{ "Save" }</button>
            </div>
          </div>
        </div>
      </div>
    }
}

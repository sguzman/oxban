use oxban_core::{
    BoardState, BoardSummary, Card, CreateBoardArgs, CreateCardArgs, CreateColumnArgs,
    DeleteCardArgs, GetBoardArgs, MoveCardArgs, UpdateCardArgs,
};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::column::ColumnView;
use crate::components::modal::CardModal;
use crate::routes::Route;
use crate::state::{
    Modal, UiState, card_matches_search, cards_for_column, columns_sorted, parse_uuid,
};
use crate::tauri_bridge::{invoke_args, invoke_no_args};

#[function_component(BoardPage)]
pub fn board_page() -> Html {
    let navigator = use_navigator();
    let route = use_route::<Route>();

    let boards = use_state(Vec::<BoardSummary>::new);
    let board_state = use_state(|| None::<BoardState>);
    let active_board_id = use_state(|| None::<Uuid>);
    let error = use_state(|| None::<String>);
    let ui = use_state(UiState::default);
    let new_column_title = use_state(String::new);
    let is_light_theme = use_state(|| false);

    {
        let is_light_theme = is_light_theme.clone();
        use_effect_with(*is_light_theme, move |is_light| {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(root) = document.document_element() {
                        if *is_light {
                            let _ = root.set_attribute("data-theme", "light");
                        } else {
                            let _ = root.remove_attribute("data-theme");
                        }
                    }
                }
            }
            || ()
        });
    }

    {
        let boards = boards.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                tracing::info!("loading board list");
                match invoke_no_args::<Vec<BoardSummary>>("list_boards").await {
                    Ok(found_boards) => boards.set(found_boards),
                    Err(message) => {
                        tracing::error!(%message, "list_boards failed");
                        error.set(Some(message));
                    }
                }
            });
            || ()
        });
    }

    {
        let boards = boards.clone();
        let error = error.clone();
        use_effect_with((*boards).len(), move |board_count| {
            if *board_count == 0 {
                let boards = boards.clone();
                let error = error.clone();
                spawn_local(async move {
                    tracing::info!("no boards found; creating default board");
                    let args = CreateBoardArgs {
                        name: "My Board".to_string(),
                    };

                    match invoke_args::<Uuid, _>("create_board", &args).await {
                        Ok(_) => match invoke_no_args::<Vec<BoardSummary>>("list_boards").await {
                            Ok(found_boards) => boards.set(found_boards),
                            Err(message) => error.set(Some(message)),
                        },
                        Err(message) => error.set(Some(message)),
                    }
                });
            }

            || ()
        });
    }

    {
        let boards_snapshot = (*boards).clone();
        let route_snapshot = route.clone();
        let active_board_id = active_board_id.clone();

        use_effect_with((boards_snapshot, route_snapshot), move |(boards, route)| {
            if !boards.is_empty() {
                let route_board_id = match route {
                    Some(Route::Board { id }) => parse_uuid(id),
                    _ => None,
                };

                let selected = route_board_id
                    .filter(|candidate| boards.iter().any(|board| board.id == *candidate))
                    .or_else(|| boards.first().map(|board| board.id));

                if selected != *active_board_id {
                    active_board_id.set(selected);
                }
            }

            || ()
        });
    }

    {
        let active_board_id_value = *active_board_id;
        let board_state = board_state.clone();
        let error = error.clone();

        use_effect_with(active_board_id_value, move |selected| {
            if let Some(board_id) = *selected {
                let board_state = board_state.clone();
                let error = error.clone();
                spawn_local(async move {
                    tracing::info!(%board_id, "loading board snapshot");
                    let args = GetBoardArgs { board_id };
                    match invoke_args::<BoardState, _>("get_board", &args).await {
                        Ok(snapshot) => board_state.set(Some(snapshot)),
                        Err(message) => {
                            tracing::error!(%board_id, %message, "get_board failed");
                            error.set(Some(message));
                        }
                    }
                });
            }

            || ()
        });
    }

    let refresh_boards = {
        let boards = boards.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let boards = boards.clone();
            let error = error.clone();
            spawn_local(async move {
                match invoke_no_args::<Vec<BoardSummary>>("list_boards").await {
                    Ok(found_boards) => boards.set(found_boards),
                    Err(message) => error.set(Some(message)),
                }
            });
        })
    };

    let reload_active_board = {
        let active_board_id = active_board_id.clone();
        let board_state = board_state.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let board_state = board_state.clone();
            let error = error.clone();
            if let Some(board_id) = *active_board_id {
                spawn_local(async move {
                    let args = GetBoardArgs { board_id };
                    match invoke_args::<BoardState, _>("get_board", &args).await {
                        Ok(snapshot) => board_state.set(Some(snapshot)),
                        Err(message) => error.set(Some(message)),
                    }
                });
            }
        })
    };

    let on_select_board = {
        let active_board_id = active_board_id.clone();
        let navigator = navigator.clone();
        Callback::from(move |board_id: Uuid| {
            tracing::info!(%board_id, "selected board from sidebar");
            active_board_id.set(Some(board_id));
            if let Some(nav) = navigator.clone() {
                nav.push(&Route::Board {
                    id: board_id.to_string(),
                });
            }
        })
    };

    let on_create_board = {
        let active_board_id = active_board_id.clone();
        let navigator = navigator.clone();
        let refresh_boards = refresh_boards.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let active_board_id = active_board_id.clone();
            let navigator = navigator.clone();
            let refresh_boards = refresh_boards.clone();
            let error = error.clone();
            spawn_local(async move {
                let args = CreateBoardArgs {
                    name: format!("Board {}", chrono::Utc::now().format("%H%M%S")),
                };
                match invoke_args::<Uuid, _>("create_board", &args).await {
                    Ok(board_id) => {
                        tracing::info!(%board_id, "created board");
                        active_board_id.set(Some(board_id));
                        refresh_boards.emit(());
                        if let Some(nav) = navigator {
                            nav.push(&Route::Board {
                                id: board_id.to_string(),
                            });
                        }
                    }
                    Err(message) => error.set(Some(message)),
                }
            });
        })
    };

    let on_create_column = {
        let active_board_id = active_board_id.clone();
        let new_column_title = new_column_title.clone();
        let reload_active_board = reload_active_board.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let Some(board_id) = *active_board_id else {
                return;
            };

            let title = new_column_title.trim().to_string();
            if title.is_empty() {
                return;
            }

            let new_column_title = new_column_title.clone();
            let reload_active_board = reload_active_board.clone();
            let error = error.clone();

            spawn_local(async move {
                let args = CreateColumnArgs {
                    board_id,
                    name: title,
                };
                match invoke_args::<Uuid, _>("create_column", &args).await {
                    Ok(column_id) => {
                        tracing::info!(%column_id, "created column");
                        new_column_title.set(String::new());
                        reload_active_board.emit(());
                    }
                    Err(message) => error.set(Some(message)),
                }
            });
        })
    };

    let on_new_column_input = {
        let new_column_title = new_column_title.clone();
        Callback::from(move |event: InputEvent| {
            let value = event
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            new_column_title.set(value);
        })
    };

    let on_search = {
        let ui = ui.clone();
        Callback::from(move |event: InputEvent| {
            let mut next = (*ui).clone();
            next.search = event
                .target_unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            ui.set(next);
        })
    };

    let on_toggle_theme = {
        let is_light_theme = is_light_theme.clone();
        Callback::from(move |_| {
            let next = !*is_light_theme;
            tracing::info!(is_light_theme = next, "theme toggled");
            is_light_theme.set(next);
        })
    };

    let on_open_card = {
        let ui = ui.clone();
        Callback::from(move |card_id: Uuid| {
            tracing::debug!(%card_id, "opening card modal");
            let mut next = (*ui).clone();
            next.modal = Modal::CardDetail { card_id };
            ui.set(next);
        })
    };

    let on_close_modal = {
        let ui = ui.clone();
        Callback::from(move |_| {
            let mut next = (*ui).clone();
            next.modal = Modal::None;
            ui.set(next);
        })
    };

    let on_save_card = {
        let reload_active_board = reload_active_board.clone();
        let ui = ui.clone();
        let error = error.clone();
        Callback::from(move |card: Card| {
            let reload_active_board = reload_active_board.clone();
            let ui = ui.clone();
            let error = error.clone();
            spawn_local(async move {
                let args = UpdateCardArgs {
                    card_id: card.id,
                    title: Some(card.title),
                    description: Some(card.description),
                    tags: Some(card.tags),
                    due_date: Some(card.due_date),
                    priority: Some(card.priority),
                };

                match invoke_args::<(), _>("update_card", &args).await {
                    Ok(()) => {
                        tracing::info!(card_id = %args.card_id, "saved card updates");
                        reload_active_board.emit(());
                        let mut next = (*ui).clone();
                        next.modal = Modal::None;
                        ui.set(next);
                    }
                    Err(message) => error.set(Some(message)),
                }
            });
        })
    };

    let on_create_card = {
        let active_board_id = active_board_id.clone();
        let reload_active_board = reload_active_board.clone();
        let error = error.clone();
        Callback::from(move |(column_id, title): (Uuid, String)| {
            let Some(board_id) = *active_board_id else {
                return;
            };

            let reload_active_board = reload_active_board.clone();
            let error = error.clone();

            spawn_local(async move {
                let args = CreateCardArgs {
                    board_id,
                    column_id,
                    title,
                };

                match invoke_args::<Uuid, _>("create_card", &args).await {
                    Ok(card_id) => {
                        tracing::info!(%card_id, %column_id, "created card");
                        reload_active_board.emit(());
                    }
                    Err(message) => error.set(Some(message)),
                }
            });
        })
    };

    let on_delete_card = {
        let reload_active_board = reload_active_board.clone();
        let error = error.clone();
        Callback::from(move |card_id: Uuid| {
            let reload_active_board = reload_active_board.clone();
            let error = error.clone();
            spawn_local(async move {
                let args = DeleteCardArgs { card_id };
                match invoke_args::<(), _>("delete_card", &args).await {
                    Ok(()) => {
                        tracing::info!(%card_id, "deleted card");
                        reload_active_board.emit(());
                    }
                    Err(message) => error.set(Some(message)),
                }
            });
        })
    };

    let on_drag_start = {
        let ui = ui.clone();
        Callback::from(move |card_id: Uuid| {
            tracing::debug!(%card_id, "drag start");
            let mut next = (*ui).clone();
            next.dragging_card = Some(card_id);
            ui.set(next);
        })
    };

    let on_drag_end = {
        let ui = ui.clone();
        Callback::from(move |_| {
            tracing::debug!("drag end");
            let mut next = (*ui).clone();
            next.dragging_card = None;
            next.drag_over_column = None;
            ui.set(next);
        })
    };

    let on_drag_over_column = {
        let ui = ui.clone();
        Callback::from(move |column_id: Uuid| {
            let mut next = (*ui).clone();
            next.drag_over_column = Some(column_id);
            ui.set(next);
        })
    };

    let on_move_card_to_column_end = {
        let reload_active_board = reload_active_board.clone();
        let error = error.clone();
        Callback::from(move |(card_id, to_column_id): (Uuid, Uuid)| {
            let reload_active_board = reload_active_board.clone();
            let error = error.clone();
            spawn_local(async move {
                let args = MoveCardArgs {
                    card_id,
                    to_column_id,
                    before_card_id: None,
                    after_card_id: None,
                };

                match invoke_args::<(), _>("move_card", &args).await {
                    Ok(()) => {
                        tracing::info!(%card_id, %to_column_id, "moved card");
                        reload_active_board.emit(());
                    }
                    Err(message) => error.set(Some(message)),
                }
            });
        })
    };

    let selected_board_id = *active_board_id;

    let content = if let Some(snapshot) = (*board_state).clone() {
        let columns = columns_sorted(&snapshot.columns);

        html! {
          <>
            <div class="topbar">
              <div class="col" style="gap: 4px;">
                <h2 class="board-title">{ snapshot.board.name.clone() }</h2>
                <span class="pill">{ format!("{} columns | {} cards", snapshot.columns.len(), snapshot.cards.len()) }</span>
              </div>

              <div class="row" style="flex-wrap: wrap;">
                <input placeholder="Search cards" value={ui.search.clone()} oninput={on_search} />
                <input placeholder="New column" value={(*new_column_title).clone()} oninput={on_new_column_input} />
                <button class="btn" onclick={on_toggle_theme}>
                  { if *is_light_theme { "Night mode" } else { "Day mode" } }
                </button>
                <button class="btn" onclick={on_create_column}>{ "Add column" }</button>
                <button class="btn primary" onclick={on_create_board.clone()}>{ "New board" }</button>
              </div>
            </div>

            <div class="kanban">
              {
                for columns.into_iter().map(|column| {
                    let cards = cards_for_column(&snapshot.cards, column.id)
                        .into_iter()
                        .filter(|card| card_matches_search(card, &ui.search))
                        .collect::<Vec<Card>>();

                    let drop_hint = ui.drag_over_column == Some(column.id);

                    html! {
                      <ColumnView
                        column={column}
                        cards={cards}
                        on_create_card={on_create_card.clone()}
                        on_open_card={on_open_card.clone()}
                        on_delete_card={on_delete_card.clone()}
                        on_move_card_to_column_end={on_move_card_to_column_end.clone()}
                        on_drag_start={on_drag_start.clone()}
                        on_drag_end={on_drag_end.clone()}
                        on_drag_over_column={on_drag_over_column.clone()}
                        drop_hint={drop_hint}
                      />
                    }
                })
              }
            </div>
          </>
        }
    } else {
        html! {
          <div class="col">
            <h2>{ "Loading board" }</h2>
            <span class="pill">{ "If this persists, check backend logs." }</span>
          </div>
        }
    };

    let modal = match &ui.modal {
        Modal::None => html! {},
        Modal::CardDetail { card_id } => {
            let card = (*board_state)
                .clone()
                .and_then(|snapshot| snapshot.cards.into_iter().find(|card| card.id == *card_id));

            if let Some(card) = card {
                html! {
                  <CardModal card={card} on_close={on_close_modal} on_save={on_save_card} />
                }
            } else {
                html! {}
            }
        }
    };

    html! {
      <div class="shell">
        <aside class="sidebar">
          <h1>{ "Oxban" }</h1>

          <div class="row" style="margin-bottom: 10px;">
            <button class="btn primary" onclick={on_create_board}>{ "New board" }</button>
          </div>

          {
            if let Some(message) = (*error).clone() {
                html! { <div class="error">{ format!("Error: {message}") }</div> }
            } else {
                html! {}
            }
          }

          <div class="col" style="margin-top: 10px;">
            {
              for (*boards).iter().map(|board| {
                let board_id = board.id;
                let on_click = {
                    let on_select_board = on_select_board.clone();
                    Callback::from(move |_| on_select_board.emit(board_id))
                };

                let class_name = if Some(board.id) == selected_board_id {
                    "board-item active"
                } else {
                    "board-item"
                };

                html! {
                  <div class={class_name} onclick={on_click}>
                    <div style="font-weight: 650;">{ board.name.clone() }</div>
                    <div style="font-size: 12px; color: var(--muted);">
                      { format!("Updated {}", board.updated_at.date_naive()) }
                    </div>
                  </div>
                }
              })
            }
          </div>
        </aside>

        <main class="main">
          { content }
          { modal }
        </main>
      </div>
    }
}

use axum::Form;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::ErrorResponse;
use axum::{extract::State, response::Result};
use maud::{DOCTYPE, Markup, html};
use serde::Deserialize;
use tracing::error;

use crate::state::AppState;
use crate::todos::{self, Todo};

pub async fn home(State(AppState { pool }): State<AppState>) -> Result<Markup> {
    let all_todos = match todos::get_all_todos(&pool).await {
        Ok(t) => t,
        Err(e) => return Err(internal_server_error(e)),
    };

    Ok(html! {
        (DOCTYPE)
        head {
            title { "Mash Todos" }
            meta name="viewport" content="width=device-width, initial-scale=1" {}
            link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>🥔</text></svg>" {}
            link rel="stylesheet" href="/public/css/bulma_1.0.4/bulma.min.css" {}
            link rel="stylesheet" href="/public/css/app.css" {}
            script src="/public/js/htmx_2.0.4/htmx.min.js" type="text/javascript" {}
        }
        body {
            section .section {
                div .container {
                    h1 .title { "Mash Todos" }
                    br;

                    div .is-size-4 {
                        ul #todo-list {
                            // display todos
                            @for todo in &all_todos {
                                (render_todo(todo))
                            }
                        }
                        form .pt-4
                            hx-post="/api/v1/todos"
                            hx-target="#todo-list"
                            hx-swap="beforeend"
                            hx-on::after-request="if(event.detail.successful) this.reset()"
                        {
                            input .input .is-medium
                                type="text"
                                id="description"
                                name="description"
                                placeholder="What do you need to do?"
                                title="Add a new item to your todo list"
                                required;
                            input type="submit" tabindex="-1" hidden;
                        }
                    }
                }
            }

            footer .footer {
                div .content.has-text-centered {
                    p {
                        "Made with ☕, 🦀, and ❤️ by "
                        a href="github.com/SteveXCIV" { "@stevexciv" }
                    }
                }
            }
        }
    })
}

#[derive(Deserialize)]
pub struct AddTodoForm {
    pub description: String,
}

pub async fn add_todo(
    State(AppState { pool }): State<AppState>,
    Form(add_todo): Form<AddTodoForm>,
) -> Result<Markup> {
    let new_todo =
        match todos::add_todo(&pool, add_todo.description.to_string()).await {
            Ok(t) => t,
            Err(e) => return Err(internal_server_error(e)),
        };

    Ok(render_todo(&new_todo))
}

pub async fn toggle_todo(
    State(AppState { pool }): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Markup> {
    match todos::toggle_todo(&pool, id).await {
        Ok(t) => Ok(render_todo(&t)),
        Err(e) => Err(internal_server_error(e)),
    }
}

fn get_todo_id(todo: &Todo) -> String {
    format!("todo-{}", todo.id)
}

fn render_todo(todo: &Todo) -> Markup {
    let id = get_todo_id(todo);
    html! {
        li #(&id) {
            label .checkbox {
                input .big-checkbox .mr-4
                    hx-put={"/api/v1/todos/" (todo.id) "/toggle"}
                    hx-target={"#" (&id)}
                    hx-swap="outerHTML"
                    type="checkbox"
                    checked[todo.is_completed()];
                @if todo.is_completed() {
                    s { (todo.description) }
                } @else {
                    (todo.description)
                }
            }
        }
    }
}

fn internal_server_error<E>(error: E) -> ErrorResponse
where
    E: std::fmt::Debug,
{
    error!("internal error: {:?}", error);
    (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong").into()
}

use axum::Form;
use axum::http::StatusCode;
use axum::{extract::State, response::Result};
use maud::{DOCTYPE, Markup, html};
use serde::Deserialize;
use tracing::error;

use crate::state::AppState;
use crate::todos;

pub async fn home(State(AppState { pool }): State<AppState>) -> Result<Markup> {
    let all_todos = match todos::get_all_todos(&pool).await {
        Ok(t) => t,
        Err(e) => {
            error!("failed to fetch todos: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "something went wrong",
            )
                .into());
        }
    };

    Ok(html! {
        (DOCTYPE)
        head {
            title { "Mash Todos" }
            script src="/public/js/htmx_2.0.4/htmx.min.js" type="text/javascript" {}
        }
        body {
            h1 { "Mash Todos" }
            br;

            ul #todo-list {
                // display todos
                @for todo in &all_todos {
                    li #{ "todo-" (todo.id) } { (todo.description) }
                }
            }
            form
                hx-post="/api/v1/todos"
                hx-target="#todo-list"
                hx-swap="beforeend"
                hx-on::after-request="if(event.detail.successful) this.reset()"
            {
                input
                    type="text"
                    id="description"
                    name="description"
                    placeholder="What do you need to do?"
                    required;
                input type="submit" tabindex="-1" hidden;
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
            Err(e) => {
                error!("failed to add new todo: {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong",
                )
                    .into());
            }
        };

    Ok(html! {
        li #{ "todo-" (new_todo.id) } { (&new_todo.description) }
    })
}

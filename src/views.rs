use crate::todos::Todo;
use maud::{DOCTYPE, Markup, html};

pub fn render_home(all_todos: Vec<Todo>) -> Markup {
    html! {
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
                    p .is-size-7 .has-text-grey {
                        "This project is open source under either the MIT or Apache-2.0 licenses."
                        " Find it "
                        a href="https://github.com/SteveXCIV/mash_todo" { "on GitHub"}
                        "."
                    }
                }
            }
        }
    }
}

pub fn render_todo(todo: &Todo) -> Markup {
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

fn get_todo_id(todo: &Todo) -> String {
    format!("todo-{}", todo.id)
}

use maud::{DOCTYPE, Markup, html};

pub async fn home() -> Markup {
    html! {
        (DOCTYPE)
        head {
            title { "Home" }
            script src="/public/js/htmx_2.0.4/htmx.min.js" type="text/javascript" {}
        }
        body {
            p { "Hello world!" }
            br;
            button hx-post="/api/v1/thebutton" hx-trigger="click" hx-swap="outerHTML" {
                "Click me!"
            }
        }
    }
}

pub async fn clicked_the_button() -> Markup {
    html! {
        p { "You clicked the button!" }
    }
}

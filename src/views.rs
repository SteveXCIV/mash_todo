use maud::{DOCTYPE, Markup, html};

pub async fn home() -> Markup {
    html! {
        (DOCTYPE)
        head {
            title { "Home" }
        }
        body {
            p { "Hello world!" }
        }
    }
}

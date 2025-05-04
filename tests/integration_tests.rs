use anyhow::{Result, anyhow};
use axum::{
    Router,
    body::Body,
    http::{Request, header, request},
    response::Response,
};
use http_body_util::BodyExt;
use mash_todo::{
    db::create_pool, handlers::AddTodoForm, routes::create_router,
    state::AppState, todos::TodoSqliteDao,
};
use scraper::{Html, Selector};
use serde::Serialize;
use tower::ServiceExt;

async fn create_router_for_test() -> Router {
    let pool = create_pool("sqlite::memory:").await.unwrap();
    let dao = TodoSqliteDao::new(pool);
    let app_state = AppState::new(dao);
    create_router(app_state)
}

trait RequestBuilderExt {
    type Output;

    fn form<T>(self, form: T) -> Result<Self::Output>
    where
        T: Serialize;
}

impl RequestBuilderExt for request::Builder {
    type Output = Request<Body>;

    fn form<T>(self, form: T) -> Result<Self::Output>
    where
        T: Serialize,
    {
        let body = serde_urlencoded::to_string(&form)?;
        Ok(self
            .header(
                header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .body(Body::from(body))?)
    }
}

trait ResponseExt {
    async fn html(self) -> Result<Html>;
}

impl ResponseExt for Response {
    async fn html(self) -> Result<Html> {
        Ok(Html::parse_fragment(
            String::from_utf8(
                self.into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes()
                    .to_vec(),
            )?
            .as_ref(),
        ))
    }
}

#[tokio::test]
pub async fn test_add_todo() -> Result<()> {
    let router = create_router_for_test().await;

    let response = router
        .oneshot(Request::post("/api/v1/todos").form(AddTodoForm {
            description: "Buy potatoes".to_string(),
        })?)
        .await?;

    assert_eq!(response.status(), 200);
    let actual_html = response.html().await?;
    let label = {
        let s = Selector::parse("label").map_err(|e| anyhow!("{:?}", e))?;
        actual_html.select(&s).next().unwrap()
    };
    assert_eq!(label.text().collect::<Vec<_>>(), vec!["Buy potatoes"]);
    let input = {
        let s = Selector::parse("input[type=checkbox]")
            .map_err(|e| anyhow!("{:?}", e))?;
        actual_html.select(&s).next().unwrap()
    };
    assert_eq!(input.value().attr("hx-put"), Some("/api/v1/todos/1/toggle"));
    assert_eq!(input.value().attr("hx-target"), Some("#todo-1"));
    assert_eq!(input.value().attr("hx-swap"), Some("outerHTML"));

    Ok(())
}

#[tokio::test]
pub async fn test_toggle_todo() -> Result<()> {
    let mut router = create_router_for_test().await;

    // First, add a todo
    let response_add = router
        .as_service()
        .oneshot(Request::post("/api/v1/todos").form(AddTodoForm {
            description: "Buy potatoes".to_string(),
        })?)
        .await?;
    assert_eq!(response_add.status(), 200);
    let added_todo_html = response_add.html().await?;
    let checkbox = {
        let s = Selector::parse("input[type=checkbox]")
            .map_err(|e| anyhow!("{:?}", e))?;
        added_todo_html.select(&s).next().unwrap()
    };
    assert_eq!(
        checkbox.value().attr("hx-put"),
        Some("/api/v1/todos/1/toggle")
    );
    assert_eq!(checkbox.value().attr("hx-target"), Some("#todo-1"));
    assert_eq!(checkbox.value().attr("hx-swap"), Some("outerHTML"));

    // Now, toggle the todo
    let response_toggle = router
        .as_service()
        .oneshot(Request::put("/api/v1/todos/1/toggle").form(AddTodoForm {
            description: "Buy potatoes".to_string(),
        })?)
        .await?;

    assert_eq!(response_toggle.status(), 200);
    let toggled_todo_html = response_toggle.html().await?;
    let checkbox_toggled = {
        let s = Selector::parse("input[type=checkbox]")
            .map_err(|e| anyhow!("{:?}", e))?;
        toggled_todo_html.select(&s).next().unwrap()
    };
    assert_eq!(
        checkbox_toggled.value().attr("hx-put"),
        Some("/api/v1/todos/1/toggle")
    );
    assert_eq!(checkbox_toggled.value().attr("hx-target"), Some("#todo-1"));
    assert_eq!(checkbox_toggled.value().attr("hx-swap"), Some("outerHTML"));
    assert!(checkbox_toggled.attr("checked").is_some());

    Ok(())
}

#[tokio::test]
pub async fn test_add_two_todos() -> Result<()> {
    let mut router = create_router_for_test().await;

    // First, add the first todo
    let response_first_todo = router
        .as_service()
        .oneshot(Request::post("/api/v1/todos").form(AddTodoForm {
            description: "Buy potatoes".to_string(),
        })?)
        .await?;

    assert_eq!(response_first_todo.status(), 200);
    let first_todo_html = response_first_todo.html().await?;
    let label_first = {
        let s = Selector::parse("label").map_err(|e| anyhow!("{:?}", e))?;
        first_todo_html.select(&s).next().unwrap()
    };
    assert_eq!(label_first.text().collect::<Vec<_>>(), vec!["Buy potatoes"]);

    // Now, add the second todo
    let response_second_todo = router
        .as_service()
        .oneshot(Request::post("/api/v1/todos").form(AddTodoForm {
            description: "Clean dishes".to_string(),
        })?)
        .await?;

    assert_eq!(response_second_todo.status(), 200);
    let second_todo_html = response_second_todo.html().await?;
    let label_second = {
        let s = Selector::parse("label").map_err(|e| anyhow!("{:?}", e))?;
        second_todo_html.select(&s).next().unwrap()
    };
    assert_eq!(
        label_second.text().collect::<Vec<_>>(),
        vec!["Clean dishes"]
    );

    // Verify both todos are in the list
    let response_list = router
        .as_service()
        .oneshot(Request::get("/").body(Body::empty())?)
        .await?;
    assert_eq!(response_list.status(), 200);
    let home_html = response_list.html().await?;
    let labels = {
        let s = Selector::parse("ul#todo-list > li > label")
            .map_err(|e| anyhow!("{:?}", e))?;
        home_html.select(&s).collect::<Vec<_>>()
    };

    let mut found_first_todo = false;
    let mut found_second_todo = false;

    for label in labels {
        if label.text().collect::<Vec<_>>()[0] == "Buy potatoes" {
            found_first_todo = true;
        } else if label.text().collect::<Vec<_>>()[0] == "Clean dishes" {
            found_second_todo = true;
        }
    }

    assert!(found_first_todo, "First todo was not found in the list");
    assert!(found_second_todo, "Second todo was not found in the list");

    Ok(())
}

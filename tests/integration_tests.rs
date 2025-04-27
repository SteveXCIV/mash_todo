use anyhow::{Result, anyhow};
use axum::{
    Router,
    body::Body,
    http::{Request, header, request},
    response::Response,
};
use http_body_util::BodyExt;
use mash_todo::{
    db::create_pool, routes::create_router, state::AppState, views::AddTodoForm,
};
use scraper::{Html, Selector};
use serde::Serialize;
use tower::ServiceExt;

async fn create_router_for_test() -> Router {
    let pool = create_pool("sqlite::memory:").await.unwrap();
    let app_state = AppState { pool };
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
async fn test_add_todo() -> Result<()> {
    let router = create_router_for_test().await;

    let response = router
        .oneshot(Request::post("/api/v1/todos").form(AddTodoForm {
            description: "Buy potatoes".to_string(),
        })?)
        .await?;

    assert_eq!(response.status(), 200);
    let actual_html = response.html().await?;
    let list_extractor =
        Selector::parse("ul#todo-list > li").map_err(|e| anyhow!("{:?}", e))?;
    let list_items: Vec<_> = actual_html.select(&list_extractor).collect();
    assert_eq!(
        list_items.len(),
        1,
        "There should be one todo item in the list"
    );
    let label = {
        let s = Selector::parse("label").map_err(|e| anyhow!("{:?}", e))?;
        list_items[0].select(&s).next().unwrap()
    };
    assert_eq!(label.text().collect::<Vec<_>>(), vec!["Buy potatoes"]);
    let input = {
        let s = Selector::parse("input[type=checkbox]")
            .map_err(|e| anyhow!("{:?}", e))?;
        list_items[0].select(&s).next().unwrap()
    };
    assert_eq!(input.value().attr("hx-put"), Some("/api/v1/todos/1/toggle"));
    assert_eq!(input.value().attr("hx-target"), Some("#todo-1"));
    assert_eq!(input.value().attr("hx-swap"), Some("outerHTML"));

    Ok(())
}

use axum::{
    Router,
    body::Body,
    http::{Error as HTTPError, Request, header, request},
    response::Response,
};
use http_body_util::BodyExt;
use mash_todo::{
    db::create_pool, routes::create_router, state::AppState, views::AddTodoForm,
};
use scraper::Html;
use serde::Serialize;
use tower::ServiceExt;

async fn create_router_for_test() -> Router {
    let pool = create_pool("sqlite::memory:").await.unwrap();
    let app_state = AppState { pool };
    create_router(app_state)
}

trait RequestBuilderExt {
    type Output;
    type Error;

    fn form<T>(self, form: T) -> Result<Self::Output, Self::Error>
    where
        T: Serialize;
}

impl RequestBuilderExt for request::Builder {
    type Output = Request<Body>;
    type Error = HTTPError;

    fn form<T>(self, form: T) -> Result<Self::Output, Self::Error>
    where
        T: Serialize,
    {
        let body = serde_urlencoded::to_string(&form)
            .expect("failed to serialize form");
        self.header(
            header::CONTENT_TYPE,
            mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
        )
        .body(Body::from(body))
    }
}

trait ResponseExt {
    async fn html(self) -> Html;
}

impl ResponseExt for Response {
    async fn html(self) -> Html {
        Html::parse_fragment(
            String::from_utf8(
                self.into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes()
                    .to_vec(),
            )
            .unwrap()
            .as_ref(),
        )
    }
}

#[tokio::test]
async fn test_add_todo() {
    let router = create_router_for_test().await;

    let response = router
        .oneshot(
            Request::post("/api/v1/todos")
                .form(AddTodoForm {
                    description: "Buy potatoes".to_string(),
                })
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    // TODO: actually do assertions on the HTML
    let actual_html = response.html().await;
    println!("{actual_html:?}")
}

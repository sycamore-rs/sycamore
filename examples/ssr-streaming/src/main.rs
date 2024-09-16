mod app;

use axum::body::Body;
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tokio::runtime::Handle;
use tokio::task::LocalSet;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(root));

    // run our app with hyper, listening globally on port 8080
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn root() -> impl IntoResponse {
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::task::spawn_blocking(|| {
        let handle = Handle::current();
        handle.block_on(async move {
            let local = LocalSet::new();
            local.spawn_local(async move {
                tx.send(sycamore::render_to_string_stream(app::App))
                    .ok()
                    .unwrap();
            });
            local.await;
        })
    });
    let stream = rx.await.unwrap();

    let body = Body::from_stream(stream.map(Ok::<_, std::convert::Infallible>));

    ([(header::CONTENT_TYPE, "text/html")], body)
}

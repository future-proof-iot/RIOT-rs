use picoserve::response::IntoResponse;

pub async fn index() -> impl IntoResponse {
    picoserve::response::File::html(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/static/index.html",
    )))
}

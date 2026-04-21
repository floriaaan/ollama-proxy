use std::{io::Cursor, path::PathBuf, sync::Arc, time::Instant};

use rocket::{
    data::{Data, ToByteUnit},
    delete, get, head,
    http::{Header, Status},
    options, patch, post, put,
    request::{FromRequest, Outcome, Request},
    response::{Responder, Response},
    routes, Build, Rocket, State,
};

use crate::{
    application::use_cases::ForwardProxyRequestUseCase,
    domain::{ProxyRequest, ProxyResponse},
    interfaces::cli::LogLevel,
};

pub struct AppState {
    pub use_case: Arc<ForwardProxyRequestUseCase>,
    pub log_level: LogLevel,
}

struct RawResponse(Response<'static>);

impl<'r> Responder<'r, 'static> for RawResponse {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        Ok(self.0)
    }
}

#[derive(Clone)]
struct IncomingRequestMeta {
    client_ip: Option<String>,
    method: String,
    uri: String,
    query: Option<String>,
    headers: Vec<(String, String)>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IncomingRequestMeta {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(Self {
            client_ip: request.client_ip().map(|value| value.to_string()),
            method: request.method().as_str().to_string(),
            uri: request.uri().to_string(),
            query: request
                .uri()
                .query()
                .map(|value| value.as_str().to_string()),
            headers: request
                .headers()
                .iter()
                .map(|header| (header.name().to_string(), header.value().to_string()))
                .collect(),
        })
    }
}

pub fn build_rocket(
    port: u16,
    use_case: Arc<ForwardProxyRequestUseCase>,
    log_level: LogLevel,
) -> Rocket<Build> {
    let figment = rocket::Config::figment()
        .merge(("address", "127.0.0.1"))
        .merge(("port", port))
        .merge(("log_level", rocket::config::LogLevel::Off));

    rocket::custom(figment)
        .manage(AppState {
            use_case,
            log_level,
        })
        .mount(
            "/",
            routes![
                get_root,
                get_path,
                post_root,
                post_path,
                put_root,
                put_path,
                patch_root,
                patch_path,
                delete_root,
                delete_path,
                options_root,
                options_path,
                head_root,
                head_path
            ],
        )
}

#[get("/", data = "<data>")]
async fn get_root(
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(None, meta, data, state).await
}

#[get("/<path..>", data = "<data>")]
async fn get_path(
    path: PathBuf,
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(Some(path), meta, data, state).await
}

#[post("/", data = "<data>")]
async fn post_root(
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(None, meta, data, state).await
}

#[post("/<path..>", data = "<data>")]
async fn post_path(
    path: PathBuf,
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(Some(path), meta, data, state).await
}

#[put("/", data = "<data>")]
async fn put_root(
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(None, meta, data, state).await
}

#[put("/<path..>", data = "<data>")]
async fn put_path(
    path: PathBuf,
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(Some(path), meta, data, state).await
}

#[patch("/", data = "<data>")]
async fn patch_root(
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(None, meta, data, state).await
}

#[patch("/<path..>", data = "<data>")]
async fn patch_path(
    path: PathBuf,
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(Some(path), meta, data, state).await
}

#[delete("/", data = "<data>")]
async fn delete_root(
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(None, meta, data, state).await
}

#[delete("/<path..>", data = "<data>")]
async fn delete_path(
    path: PathBuf,
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(Some(path), meta, data, state).await
}

#[options("/", data = "<data>")]
async fn options_root(
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(None, meta, data, state).await
}

#[options("/<path..>", data = "<data>")]
async fn options_path(
    path: PathBuf,
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(Some(path), meta, data, state).await
}

#[head("/", data = "<data>")]
async fn head_root(
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(None, meta, data, state).await
}

#[head("/<path..>", data = "<data>")]
async fn head_path(
    path: PathBuf,
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    proxy(Some(path), meta, data, state).await
}

async fn proxy(
    path: Option<PathBuf>,
    meta: IncomingRequestMeta,
    data: Data<'_>,
    state: &State<AppState>,
) -> RawResponse {
    let start = Instant::now();

    let body = match data.open(64.mebibytes()).into_bytes().await {
        Ok(bytes) if bytes.is_complete() => bytes.into_inner(),
        _ => {
            let status = Status::PayloadTooLarge;
            log_request(
                &meta,
                status.code,
                start.elapsed().as_millis(),
                state.log_level,
                None,
                0,
            );
            return RawResponse(Response::build().status(status).finalize());
        }
    };

    let mut path_and_query = match path {
        Some(path) => format!("/{}", path.display()),
        None => "/".to_string(),
    };

    if let Some(query) = &meta.query {
        path_and_query.push('?');
        path_and_query.push_str(query);
    }

    let proxy_request = ProxyRequest {
        method: meta.method.clone(),
        path_and_query: path_and_query.clone(),
        headers: meta.headers.clone(),
        body,
    };

    let response = match state.use_case.execute(proxy_request).await {
        Ok(response) => response,
        Err(_) => ProxyResponse {
            status: Status::BadGateway.code,
            headers: Vec::new(),
            body: b"upstream error".to_vec(),
        },
    };

    let latency_ms = start.elapsed().as_millis();
    let status = response.status;
    let body_size = response.body.len();
    log_request(
        &meta,
        status,
        latency_ms,
        state.log_level,
        Some(&path_and_query),
        body_size,
    );

    RawResponse(response_to_rocket(response))
}

fn response_to_rocket(proxy_response: ProxyResponse) -> Response<'static> {
    let mut builder = Response::build();

    if let Some(status) = Status::from_code(proxy_response.status) {
        builder.status(status);
    } else {
        builder.status(Status::BadGateway);
    }

    for (name, value) in proxy_response.headers {
        if name.eq_ignore_ascii_case("connection")
            || name.eq_ignore_ascii_case("transfer-encoding")
            || name.eq_ignore_ascii_case("content-length")
        {
            continue;
        }
        builder.header(Header::new(name, value));
    }

    builder.sized_body(proxy_response.body.len(), Cursor::new(proxy_response.body));
    builder.finalize()
}

fn log_request(
    meta: &IncomingRequestMeta,
    status: u16,
    latency_ms: u128,
    log_level: LogLevel,
    upstream: Option<&str>,
    body_size: usize,
) {
    let ip = meta.client_ip.clone().unwrap_or_else(|| "-".to_string());
    let method = &meta.method;
    let path = &meta.uri;

    if log_level == LogLevel::Debug {
        println!(
            "{ip} \"{method} {path}\" {status} {latency_ms}ms bytes={body_size} upstream={}",
            upstream.unwrap_or("-")
        );
    } else {
        println!("{ip} \"{method} {path}\" {status} {latency_ms}ms");
    }
}

use domain::{Limiter, Stoppable};
use sliding_window_log::SlidingWindowLog;
use std::{net::SocketAddr, sync::Arc};
use token_bucket::TokenBucket;
use tokio::sync::Mutex;
use warp::{http::StatusCode, reply, Filter};
use window_counter::WindowLimiter;

pub mod domain;
pub mod sliding_window_log;
pub mod token_bucket;
pub mod window_counter;

async fn show_unlimited_message() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::html("Unlimited! Let's Go!"))
}

async fn show_limited_message_with_limiter(
    remote_ip: Option<SocketAddr>,
    window_limiter: SingletonLimiter,
) -> Result<impl warp::Reply, warp::Rejection> {
    let ip = match remote_ip {
        None => {
            return Ok(reply::with_status(
                "Unable to get the ip address",
                StatusCode::IM_A_TEAPOT,
            ))
        }
        Some(ip) => ip.to_string(),
    };

    return match window_limiter.lock().await.use_token(ip) {
        Ok(_) => Ok(reply::with_status("", StatusCode::OK)),
        _ => Ok(reply::with_status(
            "Too Many Request",
            StatusCode::TOO_MANY_REQUESTS,
        )),
    };
}

fn with_limiter(
    limiter: SingletonLimiter,
) -> impl Filter<Extract = (SingletonLimiter,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || limiter.clone())
}

type SingletonLimiter = Arc<Mutex<dyn Limiter + Send>>;

#[tokio::main]
async fn main() {
    // tokio::task::spawn(add_tokens());

    let window_limiter: SingletonLimiter = Arc::new(Mutex::new(WindowLimiter::new()));
    let sliging_window_log: SingletonLimiter = Arc::new(Mutex::new(SlidingWindowLog::new()));
    let mut plain_token_bucket = TokenBucket::new();
    plain_token_bucket.start();
    let token_limiter: SingletonLimiter = Arc::new(Mutex::new(plain_token_bucket));

    let unlimited = warp::get()
        .and(warp::path("unlimited"))
        .and(warp::path::end())
        .and_then(show_unlimited_message);

    let limited_with_token_bucket = warp::get()
        .and(warp::path("limited-by-token-bucket"))
        .and(warp::path::end())
        .and(warp::addr::remote())
        .and(with_limiter(token_limiter))
        .and_then(show_limited_message_with_limiter);

    let limited_window_counter = warp::get()
        .and(warp::path("limited-window-counter"))
        .and(warp::path::end())
        .and(warp::addr::remote())
        .and(with_limiter(window_limiter))
        .and_then(show_limited_message_with_limiter);

    let limited_sliding_window_log = warp::get()
        .and(warp::path("limited-sliding-window-log"))
        .and(warp::path::end())
        .and(warp::addr::remote())
        .and(with_limiter(sliging_window_log))
        .and_then(show_limited_message_with_limiter);

    let routes = unlimited
        .or(limited_window_counter)
        .or(limited_sliding_window_log)
        .or(limited_with_token_bucket);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

use domain::Limiter;
use sliding_window_log::SlidingWindowLog;
use std::{net::SocketAddr, sync::Arc};
use token_bucket::{add_tokens, use_token};
use tokio::sync::Mutex;
use warp::{http::StatusCode, reply, Filter};
use window_counter::WindowLimiter;

pub mod domain;
pub mod sliding_window_log;
pub mod token_bucket;
pub mod window_counter;

async fn show_limited_message_with_token_bucket(
    remote_ip: Option<SocketAddr>,
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

    return match use_token(ip) {
        Ok(_) => Ok(reply::with_status("", StatusCode::OK)),
        _ => Ok(reply::with_status(
            "Too Many Request",
            StatusCode::TOO_MANY_REQUESTS,
        )),
    };
}
async fn show_unlimited_message() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::html("Unlimited! Let's Go!"))
}

async fn show_limited_message_with_limiter(
    remote_ip: Option<SocketAddr>,
    window_limiter: SingletonWindowLimiter,
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

// TODO: Use generics here!!!!
async fn show_limited_message_with_sliding_limiter(
    remote_ip: Option<SocketAddr>,
    window_limiter: SingletonSlidingWindowLog,
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

// TODO: Use generics here!!!!
fn with_window_limiter(
    limiter: SingletonWindowLimiter,
) -> impl Filter<Extract = (SingletonWindowLimiter,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || limiter.clone())
}

fn with_sliding_window_log(
    limiter: SingletonSlidingWindowLog,
) -> impl Filter<Extract = (SingletonSlidingWindowLog,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || limiter.clone())
}

type SingletonWindowLimiter = Arc<Mutex<WindowLimiter>>;
type SingletonSlidingWindowLog = Arc<Mutex<SlidingWindowLog>>;

#[tokio::main]
async fn main() {
    tokio::task::spawn(add_tokens());

    let window_limiter: SingletonWindowLimiter = Arc::new(Mutex::new(WindowLimiter::new()));
    let sliging_window_log: SingletonSlidingWindowLog =
        Arc::new(Mutex::new(SlidingWindowLog::new()));

    let unlimited = warp::get()
        .and(warp::path("unlimited"))
        .and(warp::path::end())
        .and_then(show_unlimited_message);

    let limited_with_token_bucket = warp::get()
        .and(warp::path("limited"))
        .and(warp::path::end())
        .and(warp::addr::remote())
        .and_then(show_limited_message_with_token_bucket);

    let limited_window_counter = warp::get()
        .and(warp::path("limited-window-counter"))
        .and(warp::path::end())
        .and(warp::addr::remote())
        .and(with_window_limiter(window_limiter))
        .and_then(show_limited_message_with_limiter);

    let limited_sliding_window_log = warp::get()
        .and(warp::path("limited-sliding-window-log"))
        .and(warp::path::end())
        .and(warp::addr::remote())
        .and(with_sliding_window_log(sliging_window_log))
        .and_then(show_limited_message_with_sliding_limiter);

    let routes = unlimited
        .or(limited_window_counter)
        .or(limited_sliding_window_log)
        .or(limited_with_token_bucket);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

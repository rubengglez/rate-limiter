use std::net::SocketAddr;
use token_bucket::{add_tokens, use_token};
use warp::{http::StatusCode, reply, Filter};

pub mod token_bucket;

async fn show_limited_message(
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

#[tokio::main]
async fn main() {
    tokio::task::spawn(add_tokens());

    let unlimited = warp::get()
        .and(warp::path("unlimited"))
        .and(warp::path::end())
        .and_then(show_unlimited_message);
    let limited = warp::get()
        .and(warp::path("limited"))
        .and(warp::path::end())
        .and(warp::addr::remote())
        .and_then(show_limited_message);

    let routes = unlimited.or(limited);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

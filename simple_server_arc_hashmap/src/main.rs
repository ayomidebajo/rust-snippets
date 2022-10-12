use warp::{http, Filter};

mod store;

#[tokio::main]
async fn main() {
    let hello_path = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(hello_path).run(([127, 0, 0, 1], 8000)).await;

    //  let routes = warp::any().map(|| "Hello, World!");

    // warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

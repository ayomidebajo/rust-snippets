use store::{add_grocery_list_item, get_grocery_list, Item, Store};
use warp::Filter;

mod store;

fn json_body() -> impl Filter<Extract = (Item,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let add_items = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("groceries"))
        .and(warp::path::end())
        .and(json_body())
        .and(store_filter.clone())
        .and_then(add_grocery_list_item);

    let get_items = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("groceries"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_grocery_list);

    let routes = add_items.or(get_items);

    // let hello_path = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;

    //  let routes = warp::any().map(|| "Hello, World!");

    // warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

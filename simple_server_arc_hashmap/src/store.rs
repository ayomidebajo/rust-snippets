use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::http;

pub type Items = HashMap<String, i32>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Item {
    name: String,
    quantity: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Id {
    name: String,
}

#[derive(Clone)]
pub struct Store {
    grocery_list: Arc<Mutex<Items>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            grocery_list: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub async fn add_grocery_list_item(
    item: Item,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut var_store = store.grocery_list.lock().expect("Data is poisoned");

    var_store.insert(item.name, item.quantity);

    Ok(warp::reply::with_status(
        "Added to store",
        http::StatusCode::CREATED,
    ))
}

pub async fn get_grocery_list(store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let mut result = HashMap::new();

    let r = store.grocery_list.lock().expect("Poisoned!!!");

    for (key, value) in r.iter() {
        result.insert(key, value);
    }

    Ok(warp::reply::json(&result))
}

pub async fn delete_grocery_list_item(
    id: Id,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut r = store.grocery_list.lock().expect("Poisoned!!!");

    r.remove(&id.name);

    Ok(warp::reply::with_status(
        "Removed item from list",
        http::StatusCode::OK,
    ))
}

pub async fn update_grocery_list_item(
    item: Item,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut var_store = store.grocery_list.lock().expect("Data is poisoned");

    var_store.insert(item.name, item.quantity);

    Ok(warp::reply::with_status(
        "Updated item in store",
        http::StatusCode::CREATED,
    ))
}

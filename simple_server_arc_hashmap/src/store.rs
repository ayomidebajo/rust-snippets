use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Items = HashMap<String, i32>;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Item {
    name: String,
    quantity: i32,
}

#[derive(Clone)]
struct Store {
    grocery_list: Arc<Mutex<Items>>,
}

impl Store {
    fn new() -> Self {
        Store {
            grocery_list: Arc::new(Mutex::new(Hash::new())),
        }
    }
}

async fn add_grocery_list_item(
    store: Store,
    item: Item,
) -> Result<impl warp::Reply, warp::Rejection> {
    let var_store = store.grocery_list.lock().expect("Data is poisoned");

    var_store.insert(item.name, item.quantity);

    Ok(warp::reply::with_status(
        "Added to store",
        https::StatusCode::CREATED,
    ))
}

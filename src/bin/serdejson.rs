use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    #[serde(flatten)]
    inner: HashMap<usize, Vec<usize>>,
}

fn main() {
    let mut db = Database {
        inner: HashMap::new(),
    };
    db.inner.insert(1, vec![1, 2, 3]);
    db.inner.insert(2, vec![2, 3, 4]);
    let v = serde_json::to_string(&db).unwrap();

    let db2 = serde_json::from_str::<Database>(&v).unwrap();
    println!("{:?}", db2.inner.get(&1));
}

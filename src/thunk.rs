use crate::idgen::IdGen;
use crate::messages::*;
use crate::node::Node;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum LazyValue<T> {
    #[default]
    UnLoaded,
    Loading,
    LoadFailed,
    Loaded(T),
}

impl<T> LazyValue<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            LazyValue::Loaded(v) => Some(v),
            _ => None,
        }
    }

    pub fn value_mut(&mut self) -> Option<&mut T> {
        match self {
            LazyValue::Loaded(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_or_insert_with<F>(&mut self, f: F) -> Option<&T>
    where
        F: FnOnce() -> LazyValue<T>,
    {
        if let LazyValue::UnLoaded = self {
            *self = f();
        }
        self.value()
    }
}

/// A Thunk is essentially a lazy loading mechanism for storing
/// and retrieving values.
#[derive(Debug, Clone, Default)]
pub struct Thunk<T> {
    // shared references to Node
    pub node: Rc<RefCell<Node>>,
    pub id: RefCell<LazyValue<String>>,
    pub value: RefCell<LazyValue<T>>,
    pub dirty: RefCell<bool>,
}

impl<T: Serialize> Serialize for Thunk<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.id.borrow().serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Thunk<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Thunk {
            node: Default::default(), // should be set by the caller
            id: RefCell::new(LazyValue::Loaded(String::deserialize(deserializer)?)),
            value: RefCell::new(LazyValue::UnLoaded),
            dirty: RefCell::new(false),
        })
    }
}

impl<T: Clone + Default + Serialize + for<'de> Deserialize<'de>> Thunk<T> {
    const SVC: &'static str = "lin-kv";

    pub fn new(node: Rc<RefCell<Node>>) -> Self {
        Self {
            node,
            id: RefCell::new(LazyValue::UnLoaded),
            value: RefCell::new(LazyValue::UnLoaded),
            dirty: RefCell::new(false),
        }
    }

    pub fn id(&self) -> Option<String> {
        let mut id_ref = self.id.borrow_mut();
        if matches!(*id_ref, LazyValue::UnLoaded) {
            *id_ref = LazyValue::Loaded(IdGen::new(&self.node.borrow()).next_id());
        }
        id_ref.value().cloned()
    }

    pub fn value(&self) -> Option<T> {
        let id = self.id();
        if id.is_none() {
            return None;
        }
        let node = self.node.clone();
        self.value.borrow_mut().get_or_insert_with(|| {
            let res = node.borrow_mut().sync_rpc(
                Self::SVC,
                MessageExtra::KvRead(KvReadExtra { key: id.into() }),
            );
            match res {
                MessageExtra::KvReadOk(p) => {
                    LazyValue::Loaded(serde_json::from_value(p.value).unwrap())
                }
                MessageExtra::Error(_) => LazyValue::LoadFailed,
                _ => {
                    eprintln!("wrong message type, {res:?}");
                    LazyValue::LoadFailed
                }
            }
        }).cloned()
    }

    pub fn set_value(&mut self, value: T) {
        if !*self.dirty.borrow() {
            self.id = RefCell::new(LazyValue::UnLoaded);
            let _ = self.id();
        }
        *self.value.borrow_mut() = LazyValue::Loaded(value);
        *self.dirty.borrow_mut() = true;
    }

    pub fn dirty(&self) -> bool {
        *self.dirty.borrow()
    }

    pub fn merge(&mut self, merge_fn: impl FnOnce(&mut T) -> bool) -> bool {
        let mut value = self.value.borrow_mut();
        if let Some(v) = value.value_mut() {
            if merge_fn(v) {
                *self.dirty.borrow_mut() = true;
                // update id
                self.id = RefCell::new(LazyValue::UnLoaded);
                self.id();
            }
            true
        } else {
            false
        }
    }

    pub fn save(&self) -> Result<(), MessageExtra> {
        if self.dirty() {
            let id = self.id();
            let value = self.to_json();
            let res = self.node.borrow_mut().sync_rpc(
                Self::SVC,
                MessageExtra::KvWrite(KvWriteExtra {
                    key: id.into(),
                    value,
                }),
            );
            if matches!(res, MessageExtra::KvWriteOk) {
                *self.dirty.borrow_mut() = false;
                Ok(())
            } else {
                Err(res)
            }
        } else {
            Ok(())
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self.value()).unwrap()
    }

    pub fn from_json(json: serde_json::Value) -> T {
        serde_json::from_value(json).unwrap()
    }
}

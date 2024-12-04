use serde::{Deserialize, Serialize};

use crate::idgen::IdGen;
use crate::messages::*;
use crate::node::Node;
use std::cell::RefCell;
use std::rc::Rc;

/// A Thunk is essentially a lazy loading mechanism for storing
/// and retrieving values.
#[derive(Debug, Clone, Default)]
pub struct Thunk<T> {
    // shared references to Node
    pub node: Rc<RefCell<Node>>,
    pub id: Option<String>,
    pub value: Option<T>,
    pub dirty: bool,
}

impl<T: Serialize> Serialize for Thunk<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.id.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Thunk<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Thunk {
            node: Default::default(), // should be set by the caller
            id: Some(String::deserialize(deserializer)?),
            value: None,
            dirty: false,
        })
    }
}

// high-ranked trait bounds
impl<T: Clone + Default + Serialize +  for<'de> Deserialize<'de>> Thunk<T> {
    const SVC: &'static str = "lin-kv";

    pub fn id(&mut self) -> &str {
        if self.id.is_none() {
            self.id = Some(IdGen::new(&self.node.borrow()).next_id());
        }
        self.id.as_ref().unwrap()
    }

    pub fn value(&mut self) -> &mut T {
        if self.value.is_none() {
            let id = self.id().to_string();
            let res = self.node.borrow_mut().sync_rpc(
                Self::SVC,
                MessageExtra::KvRead(KvReadExtra { key: id.into() }),
            );
            self.value = match res {
                MessageExtra::KvReadOk(p) => {
                    Some(Thunk::from_json(p.value))
                }
                MessageExtra::Error(_) => Some(T::default()),
                _ => panic!("wrong message type"),
            };
        }
        self.value.as_mut().unwrap()
    }

    pub fn value_ref(&self) -> &T {
        self.value.as_ref().unwrap()
    }

    pub fn save(&mut self) -> Result<(), MessageExtra> {
        if self.dirty {
            let id = self.id(); // mutable borrow 1
                                // must clone the string to avoid the borrow checker complaining
                                // about more than one mutable borrow
            let id_clone = id.to_string();

            // after clone, the mutable borrow 1 is dropped, so we can
            // use the immutable borrow 2

            let value = self.to_json();
            let res = self.node.borrow_mut().sync_rpc(
                // mutable borrow 2
                Self::SVC,
                MessageExtra::KvWrite(KvWriteExtra {
                    key: id_clone.into(),
                    value,
                }),
            );
            if matches!(res, MessageExtra::KvWriteOk) {
                self.dirty = false;
                Ok(())
            } else {
                Err(res)
            }
        } else {
            Ok(())
        }
    }

    pub fn to_json(&mut self) -> serde_json::Value {
        serde_json::to_value(self.value()).unwrap()
    }

    pub fn from_json(json: serde_json::Value) -> T {
        serde_json::from_value(json).unwrap()
    }
}

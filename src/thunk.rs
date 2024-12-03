use crate::idgen::IdGen;
use crate::messages::*;
use crate::node::Node;
use std::cell::RefCell;
use std::rc::Rc;

/// A Thunk is essentially a lazy loading mechanism for storing
/// and retrieving values.
#[derive(Debug, Clone)]
pub struct Thunk {
    // shared references to Node
    pub node: Rc<RefCell<Node>>,
    pub id: Option<String>,
    pub value: Option<Vec<usize>>,
    pub dirty: bool,
}

impl Thunk {
    const SVC: &'static str = "lin-kv";

    pub fn id(&mut self) -> &str {
        if self.id.is_none() {
            self.id = Some(IdGen::new(&self.node.borrow()).next_id());
        }
        self.id.as_ref().unwrap()
    }

    pub fn value(&mut self) -> Vec<usize> {
        if self.value.is_none() {
            let id = self.id().to_string();
            let res = self.node.borrow_mut().sync_rpc(
                Self::SVC,
                MessageExtra::KvRead(KvReadExtra { key: id.into() }),
            );
            self.value = Some(match res {
                MessageExtra::KvReadOk(v) => serde_json::from_value(v.value).unwrap(),
                MessageExtra::Error(_) => vec![],
                _ => panic!("wrong message type"),
            });
        }
        self.value.as_ref().unwrap().clone()
    }

    pub fn value_ref(&self) -> &Vec<usize> {
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

            let res = self.node.borrow_mut().sync_rpc(
                // mutable borrow 2
                Self::SVC,
                MessageExtra::KvWrite(KvWriteExtra {
                    key: id_clone.into(),
                    value: serde_json::to_value(self.value.clone().unwrap()).unwrap(),
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
}

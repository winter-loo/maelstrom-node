use crate::node::Node;

static mut NEXT_ID: usize = 0;

pub struct IdGen<'a> {
    node: &'a Node,
}

impl<'a> IdGen<'a> {
    pub fn new(node: &'a Node) -> IdGen<'a> {
        IdGen { node }
    }

    pub fn next_id(&mut self) -> String {
        unsafe { NEXT_ID += 1; }
        format!("{}-{}", self.node.id, unsafe { NEXT_ID })
    }
}
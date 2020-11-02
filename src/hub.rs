use std::{cell::RefCell, rc::Rc};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Hub {
    state: Rc<RefCell<HubInner>>,
}

impl Hub {
    pub fn new(id: String) -> Self {
        Hub {
            state: Rc::new(RefCell::new(HubInner {
                id,
                message_count: 0,
            })),
        }
    }

    pub fn id(&self) -> String {
        self.state.borrow().id.clone()
    }

    pub fn message_count(&self) -> u64 {
        self.state.borrow().message_count
    }

    pub fn inc_message_count(&self) {
        self.state.borrow_mut().message_count = self.message_count() + 1;
    }

    pub fn print(&self) {
        println!(
            "Hub: {}, Message Count: {}",
            &self.state.borrow().id,
            self.message_count()
        );
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct HubInner {
    id: String,
    message_count: u64,
}

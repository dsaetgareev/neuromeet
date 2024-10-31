use yewdux::store::{Store, Reducer};
use std::rc::Rc;


#[derive(Clone, PartialEq, Store)]
pub struct AppStore {
    pub name: String,
    pub id: String,
}


impl Default for AppStore {
    fn default() -> Self {
        Self { 
            name: Default::default(),
            id: Default::default(),
        }
    }
}


pub enum AppMsg {
    SetName(String),
    SetId(String),
}

impl Reducer<AppStore> for AppMsg {
    fn apply(self, mut store: Rc<AppStore>) -> Rc<AppStore> {
        let state = Rc::make_mut(&mut store);
        // let dispatch = Dispatch::<AppStore>::global().get();
        match self {
            AppMsg::SetName(name) => {
                state.name = name;
            }
            AppMsg::SetId(id) => {
                state.id = id;
            } 
        }
        store
    } 
}
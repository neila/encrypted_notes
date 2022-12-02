use crate::devices_store::DevicesStore;
use crate::notes_store::NotesStore;
use std::cell::RefCell;

thread_local!(
    pub static DEVICES_STORE: RefCell<DevicesStore> = RefCell::default();
    pub static NOTES_STORE: RefCell<NotesStore> = RefCell::default();
);

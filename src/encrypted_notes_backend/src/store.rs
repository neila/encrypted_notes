use crate::notes_store::NotesStore;
use std::cell::RefCell;

thread_local!(
    pub static NOTES_STORE: RefCell<NotesStore> = RefCell::default();
);

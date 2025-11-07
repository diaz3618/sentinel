use std::cell::RefCell;

thread_local! {
    static RESERVE: RefCell<Option<Vec<u8>>> = RefCell::new(None);
}

pub fn hold(megabytes: u64) {
    let bytes = (megabytes as usize) * 1024 * 1024;
    RESERVE.with(|r| {
        let mut slot = r.borrow_mut();
        *slot = Some(vec![0u8; bytes]);
    });
}

pub fn release() {
    RESERVE.with(|r| {
        let mut slot = r.borrow_mut();
        *slot = None;
    });
}

pub fn is_held() -> bool {
    RESERVE.with(|r| r.borrow().is_some())
}


use std::cell::RefCell;

pub struct Context {
    error: RefCell<Option<String>>
}

impl Context {

    pub fn new() -> Self {
        Context {
            error: RefCell::new(Some(String::new()))
        }
    }

    pub fn error(&self, error: &str) {
        let mut cur_error = self.error.borrow_mut();
        let mut cur_error = cur_error.get_or_insert_with(String::new);

        if !cur_error.is_empty() {
            cur_error.push('\n');
        }
        *cur_error += &error;
    }

    pub fn check(&self) -> Result<(), String> {
        self.error.borrow_mut()
            .take().into_iter()
            .filter(|s| !s.is_empty())
            .next()
            .map_or(Ok(()), Err)
    }

}

impl Drop for Context {

    fn drop(&mut self) {
        if self.error.borrow().is_some() {
            panic!("Failed to check for errors in context");
        }
    }

}
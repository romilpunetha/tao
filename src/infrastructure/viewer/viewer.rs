#[derive(Clone)]
pub struct ViewerContext {
    pub user_id: i64,
}

impl ViewerContext {
    pub fn new(user_id: i64) -> Self {
        ViewerContext { user_id }
    }
}

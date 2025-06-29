use crate::error::{AppError, AppResult};
use crate::infrastructure::tao::Tao;
use once_cell::sync::OnceCell;
use std::sync::Arc;

static TAO_INSTANCE: OnceCell<Arc<Tao>> = OnceCell::new();

pub fn set_global_tao(tao: Arc<Tao>) -> AppResult<()> {
    TAO_INSTANCE
        .set(tao)
        .map_err(|_| AppError::Internal("Global TAO instance already set".to_string()))
}

pub fn get_global_tao() -> AppResult<&'static Arc<Tao>> {
    TAO_INSTANCE
        .get()
        .ok_or_else(|| AppError::Internal("Global TAO instance not initialized".to_string()))
}

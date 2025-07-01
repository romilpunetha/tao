use std::sync::Arc;
use crate::infrastructure::tao_core::tao_core::TaoOperations;

pub trait HasTao: Send + Sync {
    fn get_tao(&self) -> Option<Arc<dyn TaoOperations>>;
    fn set_tao(&mut self, tao: Arc<dyn TaoOperations>);
}

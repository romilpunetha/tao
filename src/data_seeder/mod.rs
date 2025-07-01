use crate::{
    error::AppResult,
    infrastructure::tao_core::tao::Tao,
};
use std::sync::Arc;

pub async fn seed_data_into_tao(_tao: Arc<Tao>) -> AppResult<()> {
    // TODO: Re-enable after entities are regenerated
    println!("⚠️  Data seeding temporarily disabled during entity regeneration");
    Ok(())
}

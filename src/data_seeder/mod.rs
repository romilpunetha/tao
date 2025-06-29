use crate::{
    error::AppResult,
    infrastructure::{
        tao::Tao,
        tao_core::{create_tao_association, current_time_millis, TaoOperations},
    },
    // domains::user::EntUser,
    // domains::post::EntPost,
    // domains::comment::EntComment,
};
use serde_json::json;
use std::sync::Arc;

pub async fn seed_data_into_tao(_tao: Arc<Tao>) -> AppResult<()> {
    // TODO: Re-enable after entities are regenerated
    println!("⚠️  Data seeding temporarily disabled during entity regeneration");
    Ok(())
}

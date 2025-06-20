// EntPost - Meta's Entity Framework database methods

use super::{Entity, TaoEntity};
use crate::models::{EntityType, EntPost};

impl Entity for EntPost {
    fn entity_type() -> EntityType {
        EntityType::EntPost
    }
}

impl TaoEntity for EntPost {}
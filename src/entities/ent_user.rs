// EntUser - Meta's Entity Framework database methods

use super::{Entity, TaoEntity};
use crate::models::{EntityType, EntUser};

impl Entity for EntUser {
    fn entity_type() -> EntityType {
        EntityType::EntUser
    }
}

impl TaoEntity for EntUser {}
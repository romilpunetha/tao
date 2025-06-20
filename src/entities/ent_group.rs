// EntGroup - Meta's Entity Framework database methods

use super::{Entity, TaoEntity};
use crate::models::{EntityType, EntGroup};

impl Entity for EntGroup {
    fn entity_type() -> EntityType {
        EntityType::EntGroup
    }
}

impl TaoEntity for EntGroup {}
// EntPage - Meta's Entity Framework database methods

use super::{Entity, TaoEntity};
use crate::models::{EntityType, EntPage};

impl Entity for EntPage {
    fn entity_type() -> EntityType {
        EntityType::EntPage
    }
}

impl TaoEntity for EntPage {}
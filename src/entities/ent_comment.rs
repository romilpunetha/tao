// EntComment - Meta's Entity Framework database methods

use super::{Entity, TaoEntity};
use crate::models::{EntityType, EntComment};

impl Entity for EntComment {
    fn entity_type() -> EntityType {
        EntityType::EntComment
    }
}

impl TaoEntity for EntComment {}
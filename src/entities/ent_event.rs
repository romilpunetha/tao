// EntEvent - Meta's Entity Framework database methods

use super::{Entity, TaoEntity};
use crate::models::{EntityType, EntEvent};

impl Entity for EntEvent {
    fn entity_type() -> EntityType {
        EntityType::EntEvent
    }
}

impl TaoEntity for EntEvent {}
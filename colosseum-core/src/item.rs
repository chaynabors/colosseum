// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

use crate::bodywear::BodywearIdentifier;
use crate::consumable::ConsumableIdentifier;
use crate::footwear::FootwearIdentifier;
use crate::handwear::HandwearIdentifier;
use crate::headwear::HeadwearIdentifier;
use crate::legwear::LegwearIdentifier;
use crate::weapon::WeaponIdentifier;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Item {
    Bodywear(BodywearIdentifier),
    Consumable(ConsumableIdentifier),
    Footwear(FootwearIdentifier),
    Handwear(HandwearIdentifier),
    Headwear(HeadwearIdentifier),
    Legwear(LegwearIdentifier),
    Weapon(WeaponIdentifier),
}

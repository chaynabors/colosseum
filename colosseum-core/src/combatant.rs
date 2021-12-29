// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

use crate::aspect::Aspect;
use crate::attribute::Attribute;
use crate::bodywear::Bodywear;
use crate::bodywear::BodywearIdentifier;
use crate::dot::DOT;
use crate::footwear::Footwear;
use crate::footwear::FootwearIdentifier;
use crate::gender::Gender;
use crate::handwear::Handwear;
use crate::handwear::HandwearIdentifier;
use crate::headwear::Headwear;
use crate::headwear::HeadwearIdentifier;
use crate::legwear::Legwear;
use crate::legwear::LegwearIdentifier;
use crate::modifier::Modifier;
use crate::modifier::ModifierExpression;
use crate::skill::SkillIdentifier;
use crate::weapon::WeaponIdentifier;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Combatant {
    pub name: String,
    pub gender: Gender,
    pub skills: Vec<SkillIdentifier>,

    pub agility: f64,
    pub dexterity: f64,
    pub intelligence: f64,
    pub mind: f64,
    pub strength: f64,
    pub vigor: f64,
    pub vitality: f64,

    pub bodywear: Option<BodywearIdentifier>,
    pub footwear: Option<FootwearIdentifier>,
    pub handwear: Option<HandwearIdentifier>,
    pub headwear: Option<HeadwearIdentifier>,
    pub legwear: Option<LegwearIdentifier>,
    pub weapon: Option<WeaponIdentifier>,

    pub hp: f64,
    pub fatigue: f64,
    pub dots: Vec<DOT>,

    pub agility_modifiers: Vec<Modifier>,
    pub dexterity_modifiers: Vec<Modifier>,
    pub intelligence_modifiers: Vec<Modifier>,
    pub mind_modifiers: Vec<Modifier>,
    pub strength_modifiers: Vec<Modifier>,
    pub vigor_modifiers: Vec<Modifier>,
    pub vitality_modifiers: Vec<Modifier>,
}

impl Combatant {
    pub fn hp_max(&self) -> f64 {
        self.attribute(Attribute::Vigor)
    }

    pub fn hp_max_initial(vigor: f64) -> f64 {
        vigor
    }

    pub fn alive(&self) -> bool {
        self.hp > 0. && self.hp_max() > 0.
    }

    pub fn dead(&self) -> bool {
        !self.alive()
    }

    pub fn ready(&self) -> bool {
        self.alive() && self.fatigue <= 0.
    }

    pub fn attribute_raw(&self, attribute: Attribute) -> f64 {
        use Attribute::*;
        match attribute {
            Agility => self.agility,
            Dexterity => self.dexterity,
            Intelligence => self.intelligence,
            Mind => self.mind,
            Strength => self.strength,
            Vigor => self.vigor,
            Vitality => self.vitality,
        }
    }

    pub fn attribute(&self, attribute: Attribute) -> f64 {
        use Attribute::*;
        let attribute_modifiers = match attribute {
            Agility => &self.agility_modifiers,
            Dexterity => &self.dexterity_modifiers,
            Intelligence => &self.intelligence_modifiers,
            Mind => &self.mind_modifiers,
            Strength => &self.strength_modifiers,
            Vigor => &self.vigor_modifiers,
            Vitality => &self.vitality_modifiers,
        };

        let mut add = 0.;
        let mut subtract = 0.;
        let mut multiply = 1.;
        for modifier in attribute_modifiers {
            match modifier.expression {
                ModifierExpression::Add(value) => match value.signum() {
                    sign if sign == 1.0 => add += value,
                    sign if sign == -1.0 => subtract += value,
                    _ => (),
                },
                ModifierExpression::Multiply(value) => multiply *= value,
            }
        }

        let mut value = self.attribute_raw(attribute);
        value += add;
        value -= value.min(subtract);
        value *= multiply;

        value
    }

    pub fn raw_damage(&self, aspect: Aspect) -> f64 {
        match aspect {
            Aspect::Fire => self.attribute(Attribute::Intelligence) * self.attribute(Attribute::Mind) * 0.5,
            Aspect::Frost => todo!(),
            Aspect::Lightning => todo!(),
            Aspect::Physical => self.attribute(Attribute::Strength),
        }
    }

    pub fn defense(&self, aspect: Aspect) -> f64 {
        let mut value = 0.;
        if let Some(identifier) = self.bodywear {
            value += <&Bodywear>::from(identifier).defense(aspect);
        }
        if let Some(identifier) = self.footwear {
            value += <&Footwear>::from(identifier).defense(aspect);
        }
        if let Some(identifier) = self.handwear {
            value += <&Handwear>::from(identifier).defense(aspect);
        }
        if let Some(identifier) = self.headwear {
            value += <&Headwear>::from(identifier).defense(aspect);
        }
        if let Some(identifier) = self.legwear {
            value += <&Legwear>::from(identifier).defense(aspect);
        }
        value
    }

    pub fn absorbtion(&self, _aspect: Aspect) -> f64 {
        0. // TODO: determine where to get absorbtion values from
    }
}

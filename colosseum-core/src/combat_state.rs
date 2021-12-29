// Copyright 2021 Chay Nabors.

use serde::Deserialize;
use serde::Serialize;

use crate::aspect::Aspect;
use crate::attribute::Attribute;
use crate::combat_event::CombatEvent;
use crate::combatant::Combatant;
use crate::consumable::Consumable;
use crate::dot::DOT;
use crate::effect::SubEffect;
use crate::lifetime::Lifetime;
use crate::party::Party;
use crate::skill::Skill;
use crate::target::Target;
use crate::weapon::Weapon;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CombatState {
    pub parties: Vec<Party>,
}

impl CombatState {
    pub fn process_event(&mut self, combat_event: &CombatEvent) {
        use CombatEvent::*;
        match combat_event {
            AttackEvent { source, targets } => {
                for target in targets {
                    let (source, target) = self.get_combatant_handles(*source, *target);
                    let weapon = match source {
                        None => target.weapon,
                        Some(source) => source.weapon,
                    };

                    match weapon {
                        None => {
                            handle_sub_effect(source, target, SubEffect::Damage { aspect: Aspect::Physical, multiplier: 1. })
                        },
                        Some(weapon) => {
                            for sub_effect in &<&Weapon>::from(weapon).effect.sub_effects {
                                handle_sub_effect(source, target, *sub_effect);
                            }
                        },
                    }
                }
            },
            ConsumableEvent { source, targets, consumable } => {
                let consumable = <&Consumable>::from(*consumable);
                for target in targets {
                    let (source, target) = self.get_combatant_handles(*source, *target);
                    for sub_effect in &consumable.effect.sub_effects {
                        handle_sub_effect(source, target, *sub_effect);
                    }
                }
            },
            SkillEvent { source, targets, skill } => {
                for target in targets {
                    let (source, target) = self.get_combatant_handles(*source, *target);
                    let skill = <&Skill>::from(*skill);
                    for sub_effect in &skill.effect.sub_effects {
                        handle_sub_effect(source, target, *sub_effect);
                    }
                }
            },
            SkipEvent => (),
        }

        for party in &mut self.parties {
            for combatant in &mut party.members {
                for i in 0..combatant.dots.len() {
                    process_damage(combatant, combatant.dots[i].aspect, combatant.dots[i].damage_value);
                    match combatant.dots[i].lifetime {
                        Lifetime::Active { ref mut duration } => {
                            if *duration > 0 {
                                *duration -= 1
                            }
                        },
                        Lifetime::Constant => (),
                    }
                }

                macro_rules! update_modifiers {
                    ($modifiers:ident) => {
                        for modifier in &mut combatant.$modifiers {
                            match modifier.lifetime {
                                Lifetime::Active { ref mut duration } => {
                                    if *duration > 0 {
                                        *duration -= 1
                                    }
                                },
                                Lifetime::Constant => (),
                            }
                        }
                    };
                }

                update_modifiers!(agility_modifiers);
                update_modifiers!(dexterity_modifiers);
                update_modifiers!(intelligence_modifiers);
                update_modifiers!(mind_modifiers);
                update_modifiers!(strength_modifiers);
                update_modifiers!(vigor_modifiers);
                update_modifiers!(vitality_modifiers);
            }
        }
    }

    fn get_combatant_handles(&mut self, source: Target, target: Target) -> (Option<&Combatant>, &mut Combatant) {
        if source.party_index > target.party_index {
            let (target_container, source_container) = self.parties.split_at_mut(source.party_index);
            let source = Some(&source_container[0].members[source.member_index]);
            let target = &mut target_container[target.party_index].members[target.member_index];
            (source, target)
        } else if source.party_index < target.party_index {
            let (source_container, target_container) = self.parties.split_at_mut(target.party_index);
            let source = Some(&source_container[source.party_index].members[source.member_index]);
            let target = &mut target_container[0].members[target.member_index];
            (source, target)
        } else {
            if source.member_index > target.member_index {
                let (target_container, source_container) =
                    self.parties[source.party_index].members.split_at_mut(source.member_index);
                let source = Some(&source_container[0]);
                let target = &mut target_container[target.member_index];
                (source, target)
            } else if source.member_index < target.member_index {
                let (source_container, target_container) =
                    self.parties[source.party_index].members.split_at_mut(target.member_index);
                let source = Some(&source_container[source.member_index]);
                let target = &mut target_container[0];
                (source, target)
            } else {
                let target = &mut self.parties[source.party_index].members[source.member_index];
                (None, target)
            }
        }
    }

    pub fn get_target_list(&self) -> Vec<Target> {
        let mut target_list = vec![];

        for party_index in 0..self.parties.len() {
            for member_index in 0..self.parties[party_index].members.len() {
                target_list.push(Target { party_index, member_index });
            }
        }

        target_list
    }

    pub fn next_combatant(&mut self) -> Target {
        loop {
            let mut readied = vec![];

            for party_index in 0..self.parties.len() {
                for member_index in 0..self.parties[party_index].members.len() {
                    if self.parties[party_index].members[member_index].ready() {
                        readied.push(Target { party_index, member_index })
                    }
                }
            }

            if let Some(ready) = readied.first() {
                self.parties[ready.party_index].members[ready.member_index].fatigue = f64::MAX;
                return *ready;
            }

            let mut fatigue_agility_ratio = f64::MAX;
            for party in &self.parties {
                for member in &party.members {
                    if member.alive() {
                        fatigue_agility_ratio =
                            fatigue_agility_ratio.min(member.fatigue / member.attribute(Attribute::Agility));
                    }
                }
            }

            for party in &mut self.parties {
                for member in &mut party.members {
                    if member.alive() {
                        let ready_amount = member.attribute(Attribute::Agility) * fatigue_agility_ratio;
                        member.fatigue -= member.fatigue.min(ready_amount);
                    }
                }
            }
        }
    }
}

fn handle_sub_effect(source: Option<&Combatant>, target: &mut Combatant, sub_effect: SubEffect) {
    match sub_effect {
        SubEffect::Damage { aspect, multiplier } => {
            let damage_value = calculate_damage_value(source, target, aspect, multiplier);
            process_damage(target, aspect, damage_value);
        },
        SubEffect::DOT { aspect, multiplier, lifetime } => {
            let damage_value = calculate_damage_value(source, target, aspect, multiplier);

            let dot = DOT { aspect, damage_value, lifetime };

            target.dots.push(dot);
        },
        SubEffect::Modifier { modifier, attribute } => {
            use Attribute::*;
            match attribute {
                Agility => target.agility_modifiers.push(modifier),
                Dexterity => target.dexterity_modifiers.push(modifier),
                Intelligence => target.intelligence_modifiers.push(modifier),
                Mind => target.mind_modifiers.push(modifier),
                Strength => target.strength_modifiers.push(modifier),
                Vigor => target.vigor_modifiers.push(modifier),
                Vitality => target.vitality_modifiers.push(modifier),
            }
        },
    }
}

fn calculate_damage_value(source: Option<&Combatant>, target: &Combatant, aspect: Aspect, multiplier: f64) -> f64 {
    let raw_damage = match source {
        Some(source) => source.raw_damage(aspect),
        None => target.raw_damage(aspect),
    };

    raw_damage * multiplier
}

fn process_damage(target: &mut Combatant, aspect: Aspect, damage: f64) {
    let defense = target.defense(aspect);
    let absorbtion = target.absorbtion(aspect);

    if damage > defense {
        target.hp = 0.0_f64.max(target.hp - (damage - defense) * (1. - absorbtion));
    }
}

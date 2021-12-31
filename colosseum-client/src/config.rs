// Copyright 2021 Chay Nabors.

use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;

use colosseum_core::bodywear::BodywearIdentifier;
use colosseum_core::combatant::Combatant;
use colosseum_core::footwear::FootwearIdentifier;
use colosseum_core::gender::Gender;
use colosseum_core::handwear::HandwearIdentifier;
use colosseum_core::legwear::LegwearIdentifier;
use colosseum_core::party::Party;
use colosseum_core::skill::SkillIdentifier;
use colosseum_core::weapon::WeaponIdentifier;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub server_address: SocketAddr,
    pub resolution: [u32; 2],
    pub test_party: Party,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 20000)),
            resolution: [1280, 720],
            test_party: Party {
                members: vec![Combatant {
                    name: "Angelo".into(),
                    gender: Gender::Male,
                    skills: vec![SkillIdentifier::Sweep],

                    agility: 10.,
                    dexterity: 13.,
                    intelligence: 6.,
                    mind: 8.,
                    strength: 5.,
                    vigor: 20.,
                    vitality: 12.,

                    bodywear: Some(BodywearIdentifier::BreakersLongsleeve),
                    footwear: Some(FootwearIdentifier::BreakersSneakers),
                    handwear: Some(HandwearIdentifier::BreakersWraps),
                    headwear: None,
                    legwear: Some(LegwearIdentifier::BreakersHaremPants),
                    weapon: Some(WeaponIdentifier::PipeIron),

                    hp: 20.,
                    fatigue: f64::MAX,
                    dots: vec![],

                    agility_modifiers: vec![],
                    dexterity_modifiers: vec![],
                    intelligence_modifiers: vec![],
                    mind_modifiers: vec![],
                    strength_modifiers: vec![],
                    vigor_modifiers: vec![],
                    vitality_modifiers: vec![],
                }],
                inventory: vec![],
            },
        }
    }
}

mod test {
    #[test]
    fn create_default_config() {
        use std::fs;
        use std::path::Path;

        use super::Config;

        let path = Path::new("client.json");

        if !path.exists() {
            let config = Config::default();
            fs::write(path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
        }
    }
}

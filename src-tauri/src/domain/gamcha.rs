use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GamchaRarity {
    Common,
    Rare,
    Epic,
    Legendary,
    Special,
}

impl GamchaRarity {
    pub const fn count(self) -> u16 {
        match self {
            Self::Common => 72,
            Self::Rare => 48,
            Self::Epic => 24,
            Self::Legendary => 9,
            Self::Special => 3,
        }
    }

    const fn prefix(self) -> &'static str {
        match self {
            Self::Common => "common",
            Self::Rare => "rare",
            Self::Epic => "epic",
            Self::Legendary => "legendary",
            Self::Special => "special",
        }
    }

    pub fn costume_id(self, zero_based_index: u16) -> String {
        format!(
            "{}_{:03}",
            self.prefix(),
            zero_based_index % self.count() + 1
        )
    }
}

pub const fn rarity_for_roll(roll: u16) -> GamchaRarity {
    match roll % 10_000 {
        0..=5_999 => GamchaRarity::Common,
        6_000..=8_499 => GamchaRarity::Rare,
        8_500..=9_499 => GamchaRarity::Epic,
        9_500..=9_899 => GamchaRarity::Legendary,
        _ => GamchaRarity::Special,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rarity_boundaries_match_the_published_rates() {
        assert_eq!(rarity_for_roll(0), GamchaRarity::Common);
        assert_eq!(rarity_for_roll(5_999), GamchaRarity::Common);
        assert_eq!(rarity_for_roll(6_000), GamchaRarity::Rare);
        assert_eq!(rarity_for_roll(8_500), GamchaRarity::Epic);
        assert_eq!(rarity_for_roll(9_500), GamchaRarity::Legendary);
        assert_eq!(rarity_for_roll(9_900), GamchaRarity::Special);
        assert_eq!(rarity_for_roll(9_999), GamchaRarity::Special);
    }

    #[test]
    fn costume_ids_use_pack_manifest_names() {
        assert_eq!(GamchaRarity::Common.costume_id(0), "common_001");
        assert_eq!(GamchaRarity::Common.costume_id(71), "common_072");
        assert_eq!(GamchaRarity::Rare.costume_id(48), "rare_001");
        assert_eq!(GamchaRarity::Special.costume_id(2), "special_003");
    }

    #[test]
    fn configured_inventory_matches_every_drawable_manifest_entry() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../../pack/manifest.json")).unwrap();
        let manifest_ids = manifest["costumes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|costume| costume["rarity"] != "default")
            .map(|costume| costume["id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        let generated_ids = [
            GamchaRarity::Common,
            GamchaRarity::Rare,
            GamchaRarity::Epic,
            GamchaRarity::Legendary,
            GamchaRarity::Special,
        ]
        .into_iter()
        .flat_map(|rarity| (0..rarity.count()).map(move |index| rarity.costume_id(index)))
        .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(manifest_ids.len(), 156);
        assert_eq!(generated_ids.len(), 156);
        assert!(generated_ids
            .iter()
            .all(|id| manifest_ids.contains(id.as_str())));
    }
}

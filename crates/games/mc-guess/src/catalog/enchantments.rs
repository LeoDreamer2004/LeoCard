use std::collections::BTreeMap;
use std::sync::OnceLock;

pub(super) struct EnchantmentItems {
    members: BTreeMap<String, Vec<String>>,
    descriptions: BTreeMap<Vec<String>, Vec<String>>,
}

impl EnchantmentItems {
    pub fn bundled() -> &'static Self {
        static ITEMS: OnceLock<EnchantmentItems> = OnceLock::new();
        ITEMS.get_or_init(|| {
            let data = include_str!("../../../../../assets/data/minecraft/enchantment-items.csv");
            let members: BTreeMap<String, Vec<String>> = csv::Reader::from_reader(data.as_bytes())
                .records()
                .map(|row| {
                    let row = row.expect("bundled enchantment items are valid CSV");
                    let items = row[1].split('、').map(str::to_owned).collect();
                    (row[0].to_owned(), items)
                })
                .collect();

            let data = include_str!("../../../../../assets/data/minecraft/enchantments.csv");
            let mut reader = csv::Reader::from_reader(data.as_bytes());
            let column = reader
                .headers()
                .expect("bundled enchantments have a header")
                .iter()
                .position(|label| label == "适用物品")
                .expect("bundled enchantments describe applicable items");
            let descriptions = reader
                .records()
                .map(|row| {
                    let row = row.expect("bundled enchantment is valid CSV");
                    let items = members
                        .get(&row[1])
                        .expect("every enchantment has an item membership list")
                        .clone();
                    let labels = row[column].split(['、', '；']).map(str::to_owned).collect();
                    (items, labels)
                })
                .collect();
            Self {
                members,
                descriptions,
            }
        })
    }

    pub fn expand(&self, enchantment: &str) -> Vec<String> {
        self.members
            .get(enchantment)
            .expect("bundled enchantment has an item membership list")
            .clone()
    }

    pub fn describe(&self, values: &[String]) -> Vec<String> {
        self.descriptions
            .get(values)
            .cloned()
            .unwrap_or_else(|| values.to_vec())
    }
}

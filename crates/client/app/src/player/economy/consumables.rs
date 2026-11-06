use super::PropId;
use super::storage::EconomyArchive;

impl EconomyArchive {
    pub(super) fn buy_consumable(&mut self, prop: PropId) -> Result<(), String> {
        if !prop.is_consumable() {
            return Err("这个道具不是消耗品".to_owned());
        }
        let count = self
            .consumables
            .get(&prop)
            .copied()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("道具数量已达上限")?;
        self.ledger.spend(prop.definition().price)?;
        self.consumables.insert(prop, count);
        Ok(())
    }

    pub(super) fn consume_prop(&mut self, prop: PropId) -> Result<(), String> {
        if !prop.is_consumable() {
            return Err("这个道具不是消耗品".to_owned());
        }
        let count = self.consumables.get_mut(&prop).ok_or("道具数量不足")?;
        *count = count.checked_sub(1).ok_or("道具数量不足")?;
        if *count == 0 {
            self.consumables.remove(&prop);
        }
        Ok(())
    }
}

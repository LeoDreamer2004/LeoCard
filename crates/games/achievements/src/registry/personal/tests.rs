use crate::{AchievementBook, AchievementRegistry, AchievementTrigger, PersonalEvent};
use std::sync::LazyLock;

static REGISTRY: LazyLock<AchievementRegistry> = LazyLock::new(AchievementRegistry::default);

fn apply(book: &mut AchievementBook, event: PersonalEvent) -> Vec<&'static str> {
    book.trigger_with_registry(&AchievementTrigger::Personal(event), None, 1, &REGISTRY)
        .unlocked
        .iter()
        .map(|definition| definition.id)
        .collect()
}

fn achieved(book: &AchievementBook, id: &str) -> bool {
    book.achieved(REGISTRY.by_id(id).unwrap())
}

#[test]
fn purchases_count_transactions_and_check_the_single_item_price() {
    let mut book = AchievementBook::default();
    for count in 1..=1000 {
        apply(&mut book, PersonalEvent::ItemPurchased { price: 199 });
        assert!(achieved(&book, "leocard:personal/first_purchase"));
        assert!(!achieved(&book, "leocard:personal/expensive_purchase"));
        assert_eq!(
            achieved(&book, "leocard:personal/hundred_purchases"),
            count >= 100
        );
        assert_eq!(
            achieved(&book, "leocard:personal/thousand_purchases"),
            count >= 1000
        );
    }
    assert_eq!(
        apply(&mut book, PersonalEvent::ItemPurchased { price: 200 }),
        ["leocard:personal/expensive_purchase"]
    );
    assert!(apply(&mut book, PersonalEvent::ItemPurchased { price: 300 }).is_empty());
    assert!(!achieved(&book, "leocard:personal/ten_thousand_coins"));
}

#[test]
fn wealth_uses_the_observed_balance_instead_of_accumulating_reports() {
    let mut book = AchievementBook::default();
    for _ in 0..12 {
        assert!(apply(&mut book, PersonalEvent::CoinBalance { coins: 9999 }).is_empty());
    }
    assert_eq!(
        apply(&mut book, PersonalEvent::CoinBalance { coins: 10_000 }),
        ["leocard:personal/ten_thousand_coins"]
    );
    assert!(apply(&mut book, PersonalEvent::CoinBalance { coins: 99_999 }).is_empty());
    assert_eq!(
        apply(&mut book, PersonalEvent::CoinBalance { coins: 100_000 }),
        ["leocard:personal/hundred_thousand_coins"]
    );
    assert!(apply(&mut book, PersonalEvent::CoinBalance { coins: 0 }).is_empty());
    assert!(achieved(&book, "leocard:personal/ten_thousand_coins"));
    assert!(achieved(&book, "leocard:personal/hundred_thousand_coins"));
    assert!(!achieved(&book, "leocard:personal/first_purchase"));

    let mut loaded_balance = AchievementBook::default();
    assert_eq!(
        apply(
            &mut loaded_balance,
            PersonalEvent::CoinBalance { coins: 100_000 }
        ),
        [
            "leocard:personal/ten_thousand_coins",
            "leocard:personal/hundred_thousand_coins"
        ]
    );
}

#[test]
fn sakura_requires_its_success_event_and_only_unlocks_once() {
    let mut book = AchievementBook::default();
    apply(&mut book, PersonalEvent::ClientUpdated);
    assert!(!achieved(&book, "leocard:personal/sakura_tunnel"));
    assert_eq!(
        apply(&mut book, PersonalEvent::SakuraTunnelStarted),
        ["leocard:personal/sakura_tunnel"]
    );
    assert!(apply(&mut book, PersonalEvent::SakuraTunnelStarted).is_empty());
    assert!(!achieved(&book, "leocard:personal/first_purchase"));
}

use super::support::*;
use leocard_achievements::{AchievementCategory, AchievementTier, achievements_in};
use leocard_protocol::ChatEmoji;

#[test]
fn personal_registry_has_all_eighteen_awards_and_local_actions_are_independent() {
    let entries = achievements_in(AchievementCategory::Personal).collect::<Vec<_>>();
    assert_eq!(entries.len(), 18);
    for (tier, expected) in [
        (AchievementTier::Bronze, 9),
        (AchievementTier::Silver, 5),
        (AchievementTier::Gold, 4),
    ] {
        assert_eq!(
            entries.iter().filter(|entry| entry.tier == tier).count(),
            expected
        );
    }
    for (id, event) in [
        ("avatar", PersonalEvent::AvatarSaved),
        ("background", PersonalEvent::TableBackgroundChanged),
        ("update", PersonalEvent::ClientUpdated),
    ] {
        let mut book = AchievementBook::default();
        let trigger = AchievementTrigger::Personal(event);
        let result = book.trigger(&trigger, None, 20);
        assert_eq!(result.unlocked.len(), 1);
        assert_eq!(result.unlocked[0].id, definition(id).id);
        assert!(book.trigger(&trigger, None, 30).unlocked.is_empty());
        assert_eq!(book.earned_at(definition(id)), Some(20));
    }
}

#[test]
fn accepted_social_actions_count_only_the_local_sender_or_recipient() {
    use PlayerInteractionKind::{Egg, Flower, Shoe, Wine};
    for (kind, flowers, eggs) in [(Flower, 1, 0), (Wine, 10, 0), (Egg, 0, 1), (Shoe, 0, 10)] {
        let received = social(kind, 1, 0);
        assert_eq!(amount("ten_thousand_flowers", &received), flowers);
        assert_eq!(amount("first_egg", &received), eggs);
        assert_eq!(amount("wine", &received), u64::from(kind == Wine));
        assert_eq!(amount("flowers_sent", &received), 0);
        let sent = social(kind, 0, 1);
        assert_eq!(amount("flowers_sent", &sent), flowers);
        assert_eq!(amount("ten_thousand_flowers", &sent), 0);
        assert_eq!(amount("first_egg", &sent), 0);
        assert_eq!(amount("flowers_sent", &social(kind, 1, 2)), 0);
    }
}

#[test]
fn chat_counts_unicode_text_and_quick_voice_separately_for_the_local_author() {
    let text = chat(0, ChatContent::Text("中文🙂a".into()));
    assert_eq!(amount("text_chat", &text), 4);
    assert_eq!(amount("first_voice", &text), 0);
    assert_eq!(
        amount("first_voice", &chat(0, ChatContent::QuickVoice(0))),
        1
    );
    assert_eq!(
        amount("first_voice", &chat(1, ChatContent::QuickVoice(0))),
        0
    );
    assert_eq!(
        amount("text_chat", &chat(1, ChatContent::Text("字".into()))),
        0
    );
    assert_eq!(
        amount("text_chat", &chat(0, ChatContent::Emoji(ChatEmoji::Laugh))),
        0
    );
}

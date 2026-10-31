use super::super::StoredPlayerProfile;
use super::archive::decode_archive;
use leocard_texas_holdem::TexasProfileStats;

pub(super) const V5_PROFILE_HEADER: &[u8] = b"LeoCard/profile/v5\0";

pub(super) fn decode_v5(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    decode_archive::<TexasProfileStats>(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::profile::migration::decode_player_profile;
    use crate::player::profile::{PROFILE_HEADER, StoredRatingProfile};
    use leocard_blackjack::BlackjackProfileStats;
    use leocard_doudizhu::DoudizhuProfileStats;
    use leocard_mahjong::MahjongProfileStats;
    use leocard_mc_guess::McGuessProfileStats;
    use leocard_protocol::{MatchId, PlayerInteractionStats};
    use leocard_qigui523::QiGui523ProfileStats;
    use leocard_shengji::ShengjiProfileStats;
    use leocard_uno::UnoProfileStats;

    #[test]
    fn v5_archive_preserves_all_existing_bytes_and_persists_new_statistics() {
        let key = [37u8; 32];
        let rating = StoredRatingProfile {
            reference_points: -123,
            completed_games: 47,
            applied_matches: vec![MatchId([9; 16])],
            qigui523_stats: Some(QiGui523ProfileStats {
                total_score: 840,
                ..Default::default()
            }),
        };
        let statistics = (
            Some(TexasProfileStats {
                placement_counts: [1, 2, 3, 4, 5, 6, 7, 8],
                entered_hands: 13,
                ..Default::default()
            }),
            Some(ShengjiProfileStats {
                dealer_games: 7,
                ..Default::default()
            }),
            Some(UnoProfileStats {
                uno_calls: 11,
                ..Default::default()
            }),
            Some(PlayerInteractionStats {
                flowers_received: 99,
                eggs_received: 21,
            }),
            Some(MahjongProfileStats {
                wins: 8,
                ..Default::default()
            }),
            Some(McGuessProfileStats {
                total_score: 456,
                ..Default::default()
            }),
            Some(BlackjackProfileStats {
                completed_games: 12,
                ..Default::default()
            }),
        );
        // Postcard structs and tuples serialize consecutive fields without framing.
        let payload = postcard::to_allocvec(&(key, rating, statistics)).unwrap();
        let mut old = V5_PROFILE_HEADER.to_vec();
        old.extend(&payload);
        let mut migrated = decode_player_profile(&old).unwrap();
        assert_eq!(migrated.secret_key, key);
        assert!(migrated.games.doudizhu_stats.is_none());
        let current = postcard::to_allocvec(&migrated).unwrap();
        assert_eq!(&current[..payload.len()], payload);
        assert_eq!(&current[payload.len()..], &[0]);
        let expected = DoudizhuProfileStats {
            completed_games: 2,
            total_match_score: -17,
            bid_actions: 5,
            total_bid_score: 6,
            bomb_plays: 3,
            ..Default::default()
        };
        migrated.games.doudizhu_stats = Some(expected.clone());
        let bytes = migrated.encode().unwrap();
        assert!(bytes.starts_with(PROFILE_HEADER));
        assert_eq!(
            decode_player_profile(&bytes).unwrap().games.doudizhu_stats,
            Some(expected)
        );
        assert!(decode_player_profile(&old[..old.len() - 1]).is_err());
    }
}

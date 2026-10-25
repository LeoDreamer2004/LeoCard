use crate::{
    GameEvent, MahjongEvent, MinecraftGuessEvent, QiGui523Event, ShengjiDeclarationView,
    ShengjiEvent, TexasHoldemEvent, UnoEvent,
};
use leocard_achievements::{GameFacts, ParticipantId};
use leocard_mahjong::achievements::{MahjongAchievementEvent, MahjongHandFacts, MahjongWinFacts};
use leocard_minecraft_guess::achievements::GuessAchievementEvent;
use leocard_qigui523::achievements::QiGuiAchievementEvent;
use leocard_shengji::achievements::{
    ShengjiAchievementEvent, ShengjiDeclarationFacts, ShengjiPlayFacts,
};
use leocard_texas_holdem::achievements::TexasAchievementEvent;
use leocard_uno::achievements::UnoAchievementEvent;

impl From<&GameEvent> for GameFacts {
    fn from(event: &GameEvent) -> Self {
        match event {
            GameEvent::Mahjong(event) => match event {
                MahjongEvent::HandFinished { result } => MahjongAchievementEvent::HandFinished {
                    result: MahjongHandFacts {
                        winners: result
                            .winners
                            .iter()
                            .map(|win| MahjongWinFacts {
                                player: ParticipantId(win.player.0),
                                from: win.from.map(|player| ParticipantId(player.0)),
                                concealed_hand: win.concealed_hand,
                                score: win.score.clone(),
                                wait_kind_count: win.wait_kind_count,
                            })
                            .collect(),
                        match_length: result.match_length,
                        match_progress: result.match_progress,
                        deltas: result.deltas,
                        match_scores: result.match_scores,
                        match_complete: result.match_complete,
                    },
                }
                .into(),
                MahjongEvent::FlowerReplaced { player } => {
                    MahjongAchievementEvent::FlowerReplaced {
                        player: ParticipantId(player.0),
                    }
                    .into()
                }
                MahjongEvent::FalseWin { player, .. } => MahjongAchievementEvent::FalseWin {
                    player: ParticipantId(player.0),
                }
                .into(),
                MahjongEvent::WinUnavailable {
                    player,
                    points_without_flowers,
                } => MahjongAchievementEvent::WinUnavailable {
                    player: ParticipantId(player.0),
                    points_without_flowers: *points_without_flowers,
                }
                .into(),
                _ => GameFacts::default(),
            },
            GameEvent::Shengji(event) => shengji_facts(event),
            GameEvent::TexasHoldem(TexasHoldemEvent::HandAnalyzed { player, statistics }) => {
                TexasAchievementEvent::HandAnalyzed {
                    player: ParticipantId(player.0),
                    statistics: statistics.clone(),
                }
                .into()
            }
            GameEvent::TexasHoldem(TexasHoldemEvent::ActionApplied {
                player, statistics, ..
            }) => TexasAchievementEvent::ActionApplied {
                player: ParticipantId(player.0),
                statistics: *statistics,
            }
            .into(),
            GameEvent::TexasHoldem(TexasHoldemEvent::SquidAwarded { player, statistics }) => {
                TexasAchievementEvent::SquidAwarded {
                    player: ParticipantId(player.0),
                    statistics: *statistics,
                }
                .into()
            }
            GameEvent::QiGui523(QiGui523Event::ActionAnalyzed { player, statistics }) => {
                QiGuiAchievementEvent::ActionAnalyzed {
                    player: ParticipantId(player.0),
                    statistics: statistics.clone(),
                }
                .into()
            }
            GameEvent::Uno(UnoEvent::ActionAnalyzed { player, statistics }) => {
                UnoAchievementEvent::ActionAnalyzed {
                    player: ParticipantId(player.0),
                    statistics: statistics.clone(),
                }
                .into()
            }
            GameEvent::MinecraftGuess(MinecraftGuessEvent::ActionAnalyzed {
                player,
                statistics,
            }) => GuessAchievementEvent::ActionAnalyzed {
                player: ParticipantId(player.0),
                statistics: statistics.clone(),
            }
            .into(),
            GameEvent::MinecraftGuess(MinecraftGuessEvent::HandAnalyzed { player, statistics }) => {
                GuessAchievementEvent::HandAnalyzed {
                    player: ParticipantId(player.0),
                    statistics: *statistics,
                }
                .into()
            }
            _ => GameFacts::default(),
        }
    }
}

fn shengji_facts(event: &ShengjiEvent) -> GameFacts {
    match event {
        ShengjiEvent::DeclarationChanged { declaration } => {
            ShengjiAchievementEvent::DeclarationChanged {
                declaration: declaration_facts(declaration),
            }
            .into()
        }
        ShengjiEvent::BottomCopied { declaration } => ShengjiAchievementEvent::BottomCopied {
            declaration: declaration_facts(declaration),
        }
        .into(),
        ShengjiEvent::CardsPlayed { play, is_lead } => ShengjiAchievementEvent::CardsPlayed {
            play: ShengjiPlayFacts {
                player: ParticipantId(play.player.0),
                play: play.play.clone(),
            },
            is_lead: *is_lead,
        }
        .into(),
        ShengjiEvent::HandAnalyzed {
            player,
            statistics,
            match_statistics,
        } => ShengjiAchievementEvent::HandAnalyzed {
            player: ParticipantId(player.0),
            statistics: statistics.clone(),
            match_statistics: *match_statistics,
        }
        .into(),
        ShengjiEvent::RedealRequired { reason } => {
            ShengjiAchievementEvent::RedealRequired { reason: *reason }.into()
        }
        _ => GameFacts::default(),
    }
}

fn declaration_facts(declaration: &ShengjiDeclarationView) -> ShengjiDeclarationFacts {
    ShengjiDeclarationFacts {
        player: ParticipantId(declaration.player.0),
        trump: declaration.trump,
        kind: declaration.kind,
    }
}

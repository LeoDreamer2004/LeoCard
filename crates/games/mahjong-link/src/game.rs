use crate::{
    BOARD_ROWS, MahjongLinkAction, MahjongLinkBoard, MahjongLinkError, MahjongLinkOutcome,
    MahjongLinkRuleSet, PushSide,
};
use leocard_mahjong_common::MahjongTileKind;

#[derive(Clone, Debug)]
pub struct MahjongLinkGameState {
    board: MahjongLinkBoard,
    scores: Vec<u16>,
    current_player: u8,
    turn: u32,
    push_used: bool,
    consecutive_passes: u8,
    complete: bool,
}

impl MahjongLinkGameState {
    pub fn new(player_count: u8, deck: Vec<MahjongTileKind>) -> Result<Self, MahjongLinkError> {
        if !(MahjongLinkRuleSet::MIN_PLAYERS..=MahjongLinkRuleSet::MAX_PLAYERS)
            .contains(&player_count)
        {
            return Err(MahjongLinkError::InvalidPlayerCount);
        }
        Ok(Self {
            board: MahjongLinkBoard::new(deck)?,
            scores: vec![0; usize::from(player_count)],
            current_player: 0,
            turn: 0,
            push_used: false,
            consecutive_passes: 0,
            complete: false,
        })
    }

    pub fn player_count(&self) -> u8 {
        self.scores.len() as u8
    }

    pub fn board(&self) -> &MahjongLinkBoard {
        &self.board
    }

    pub fn scores(&self) -> &[u16] {
        &self.scores
    }

    pub fn current_player(&self) -> Option<u8> {
        (!self.complete).then_some(self.current_player)
    }

    pub fn turn(&self) -> u32 {
        self.turn
    }

    pub fn consecutive_passes(&self) -> u8 {
        self.consecutive_passes
    }

    pub fn push_used(&self) -> bool {
        self.push_used
    }

    pub fn complete(&self) -> bool {
        self.complete
    }

    pub fn act(
        &mut self,
        player: u8,
        action: MahjongLinkAction,
    ) -> Result<MahjongLinkOutcome, MahjongLinkError> {
        if player >= self.player_count() {
            return Err(MahjongLinkError::InvalidPlayer);
        }
        if self.complete {
            return Err(MahjongLinkError::MatchComplete);
        }
        if player != self.current_player {
            return Err(MahjongLinkError::NotPlayersTurn);
        }
        let (points, path, movements) = match action {
            MahjongLinkAction::Match { first, second } => {
                let (points, path) = self.board.remove_pair(first, second)?;
                (points, path, Vec::new())
            }
            MahjongLinkAction::Push {
                row,
                from,
                distance,
            } => {
                if self.push_used {
                    return Err(MahjongLinkError::PushAlreadyUsed);
                }
                let movements = self.board.push(row, from, distance)?;
                self.push_used = true;
                (0, Vec::new(), movements)
            }
            MahjongLinkAction::Pass => (0, Vec::new(), Vec::new()),
        };
        self.scores[usize::from(player)] += points;
        self.consecutive_passes = if action == MahjongLinkAction::Pass {
            self.consecutive_passes + 1
        } else {
            0
        };
        self.complete =
            self.board.remaining() == 0 || self.consecutive_passes == self.player_count();
        if !matches!(action, MahjongLinkAction::Push { .. }) {
            self.turn += 1;
            self.current_player = (self.current_player + 1) % self.player_count();
            self.push_used = false;
        }
        Ok(MahjongLinkOutcome {
            action,
            points,
            path,
            movements,
            complete: self.complete,
        })
    }

    /// Bots and disconnected players observe the same public board as a human.
    pub fn automatic_action(&self) -> MahjongLinkAction {
        if let Some((first, second)) = self.board.best_pair() {
            return MahjongLinkAction::Match { first, second };
        }
        if self.push_used {
            return MahjongLinkAction::Pass;
        }
        for row in 0..BOARD_ROWS as u8 {
            for from in [PushSide::Left, PushSide::Right] {
                for distance in 1..=self.board.maximum_push(row, from) {
                    let mut board = self.board.clone();
                    if board.push(row, from, distance).is_ok() && board.best_pair().is_some() {
                        return MahjongLinkAction::Push {
                            row,
                            from,
                            distance,
                        };
                    }
                }
            }
        }
        MahjongLinkAction::Pass
    }
}

use leocard_mahjong_link::MahjongLinkAction;
use leocard_protocol::{
    BlackjackCommand, DoudizhuCommand, GameCommand, GuandanCommand, MahjongCommand,
    MahjongLinkCommand, McGuessCommand, QiGui523Command, ShengjiCommand, TexasCommand, UnoCommand,
};

pub(super) fn is_decision(command: &GameCommand) -> bool {
    match command {
        GameCommand::QiGui523(command) => matches!(
            command,
            QiGui523Command::PlayCards { .. } | QiGui523Command::Pass
        ),
        GameCommand::Texas(command) => matches!(
            command,
            TexasCommand::Act { .. } | TexasCommand::ChooseRunout { .. }
        ),
        GameCommand::Shengji(command) => !matches!(
            command,
            ShengjiCommand::SetAutoPlay { .. } | ShengjiCommand::UpdateRules { .. }
        ),
        GameCommand::Uno(command) => !matches!(
            command,
            UnoCommand::SetAutoPlay { .. }
                | UnoCommand::UpdateRules { .. }
                | UnoCommand::JumpIn { .. }
                | UnoCommand::CallUno
                | UnoCommand::ReportUno { .. }
        ),
        GameCommand::Mahjong(command) => !matches!(
            command,
            MahjongCommand::SetAutoPlay { .. }
                | MahjongCommand::UpdateRules { .. }
                | MahjongCommand::SetDeveloperHand { .. }
        ),
        GameCommand::McGuess(command) => {
            matches!(command, McGuessCommand::Guess { .. } | McGuessCommand::Skip)
        }
        GameCommand::Blackjack(command) => matches!(command, BlackjackCommand::Act { .. }),
        GameCommand::Doudizhu(command) => matches!(command, DoudizhuCommand::Act(_)),
        GameCommand::Guandan(command) => matches!(command, GuandanCommand::Act(_)),
        GameCommand::MahjongLink(command) => matches!(
            command,
            MahjongLinkCommand::Act {
                action: MahjongLinkAction::Match { .. } | MahjongLinkAction::Pass
            }
        ),
    }
}

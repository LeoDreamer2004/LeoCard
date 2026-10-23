use super::schema::{Archive, Rules};
use crate::player::preferences::{
    GlobalPreferences, MahjongPreferences, SavedPreferences, TexasHoldemPreferences,
};
use leocard_minecraft_guess::{
    GraphRules, GuessMode, GuessOptions, MinecraftGuessRuleSet, RaceRules, TurnBasedRules,
};
use serde::Deserialize;

/// v0.9.1 had the five-game archive followed by the first published Minecraft settings.
#[derive(Deserialize)]
pub(super) struct Preferences {
    base: Archive<GlobalPreferences, MahjongPreferences, TexasHoldemPreferences>,
    minecraft_guess: Rules<MinecraftRules>,
}

#[derive(Deserialize)]
struct MinecraftRules {
    mode: GuessMode,
    turn_based: TurnBasedRules,
    race: RaceRules,
    graph: GraphSettings,
}

#[derive(Deserialize)]
struct GraphSettings {
    options: GuessOptions,
    nodes: u8,
    temperature: u8,
    guess_seconds: u16,
    show_character_count: bool,
    show_directions: bool,
    include_tags: bool,
}

impl From<Preferences> for SavedPreferences {
    fn from(value: Preferences) -> Self {
        let mut saved: Self = value.base.into();
        let rules = value.minecraft_guess.host_rules;
        let graph = rules.graph;
        saved.games.minecraft_guess.host_rules = MinecraftGuessRuleSet {
            mode: rules.mode,
            turn_based: rules.turn_based,
            race: rules.race,
            graph: GraphRules {
                options: graph.options,
                round_robin: true,
                nodes: graph.nodes,
                temperature: graph.temperature,
                guess_seconds: graph.guess_seconds,
                show_character_count: graph.show_character_count,
                show_directions: graph.show_directions,
                include_tags: graph.include_tags,
            },
        };
        saved
    }
}

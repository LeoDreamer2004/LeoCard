//! 德州扑克事件驱动音效。
//!
//! 音效只由服务器事件入队，与可重复重建的 UI 实体解耦，避免一次动作因界面刷新
//! 被反复播放。密集筹码动作按“动作”聚合成少量错开的采样，不逐枚筹码发声。

use super::*;
use leocard_texas_holdem::TexasHoldemAction;

#[derive(Default)]
pub struct TexasSoundAssets {
    chip_lay: Vec<Handle<AudioSource>>,
    chip_handle: Vec<Handle<AudioSource>>,
    chip_collide: Vec<Handle<AudioSource>>,
    chip_stack: Vec<Handle<AudioSource>>,
    check: Vec<Handle<AudioSource>>,
    fold: Vec<Handle<AudioSource>>,
    card_place: Vec<Handle<AudioSource>>,
    showdown_fan: Vec<Handle<AudioSource>>,
    all_in: Vec<Handle<AudioSource>>,
    pot_divide: Vec<Handle<AudioSource>>,
    turn: Vec<Handle<AudioSource>>,
    confirm: Vec<Handle<AudioSource>>,
}

impl TexasSoundAssets {
    pub fn load(asset_server: &AssetServer) -> Self {
        let casino = "vendor/kenney/casino-audio/Audio";
        let interface = "vendor/kenney/interface-sounds/Audio";
        Self {
            chip_lay: numbered_sounds(asset_server, casino, "chip-lay", 3),
            chip_handle: numbered_sounds(asset_server, casino, "chips-handle", 6),
            chip_collide: numbered_sounds(asset_server, casino, "chips-collide", 4),
            chip_stack: numbered_sounds(asset_server, casino, "chips-stack", 6),
            check: ["tap-a.ogg", "tap-b.ogg"]
                .map(|name| asset_server.load(format!("vendor/kenney/ui/Sounds/{name}")))
                .to_vec(),
            fold: numbered_sounds(asset_server, casino, "card-shove", 4),
            card_place: numbered_sounds(asset_server, casino, "card-place", 4),
            showdown_fan: numbered_sounds(asset_server, casino, "card-fan", 2),
            all_in: vec![asset_server.load(format!("{interface}/bong_001.ogg"))],
            pot_divide: ["switch_003.ogg", "switch_004.ogg"]
                .map(|name| asset_server.load(format!("{interface}/{name}")))
                .to_vec(),
            turn: ["pluck_001.ogg", "pluck_002.ogg"]
                .map(|name| asset_server.load(format!("{interface}/{name}")))
                .to_vec(),
            confirm: (1..=4)
                .map(|index| asset_server.load(format!("{interface}/confirmation_00{index}.ogg")))
                .collect(),
        }
    }

    fn variants(&self, kind: TexasSoundKind) -> &[Handle<AudioSource>] {
        match kind {
            TexasSoundKind::ChipLay => &self.chip_lay,
            TexasSoundKind::ChipHandle => &self.chip_handle,
            TexasSoundKind::ChipCollide => &self.chip_collide,
            TexasSoundKind::ChipStack => &self.chip_stack,
            TexasSoundKind::Check => &self.check,
            TexasSoundKind::Fold => &self.fold,
            TexasSoundKind::CardPlace => &self.card_place,
            TexasSoundKind::ShowdownFan => &self.showdown_fan,
            TexasSoundKind::AllIn => &self.all_in,
            TexasSoundKind::PotDivide => &self.pot_divide,
            TexasSoundKind::Turn => &self.turn,
            TexasSoundKind::Confirm => &self.confirm,
        }
    }
}

fn numbered_sounds(
    asset_server: &AssetServer,
    directory: &str,
    prefix: &str,
    count: usize,
) -> Vec<Handle<AudioSource>> {
    (1..=count)
        .map(|index| asset_server.load(format!("{directory}/{prefix}-{index}.ogg")))
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TexasSoundKind {
    ChipLay,
    ChipHandle,
    ChipCollide,
    ChipStack,
    Check,
    Fold,
    CardPlace,
    ShowdownFan,
    AllIn,
    PotDivide,
    Turn,
    Confirm,
}

#[derive(Clone, Copy, Debug)]
pub struct TexasAudioCue {
    pub kind: TexasSoundKind,
    pub remaining: f32,
    pub volume: f32,
    pub seed: u64,
}

impl TexasAudioCue {
    fn new(kind: TexasSoundKind, remaining: f32, volume: f32, seed: u64) -> Self {
        Self {
            kind,
            remaining,
            volume,
            seed,
        }
    }
}

pub fn texas_action_sound_plan(
    action: TexasHoldemAction,
    amount: u32,
    seed: u64,
) -> Vec<TexasAudioCue> {
    match action {
        TexasHoldemAction::PostBlind | TexasHoldemAction::Call => vec![TexasAudioCue::new(
            TexasSoundKind::ChipLay,
            0.10,
            0.52,
            seed,
        )],
        TexasHoldemAction::Check => {
            vec![TexasAudioCue::new(TexasSoundKind::Check, 0.0, 0.34, seed)]
        }
        TexasHoldemAction::Fold => vec![TexasAudioCue::new(TexasSoundKind::Fold, 0.03, 0.50, seed)],
        TexasHoldemAction::RaiseTo(_) => vec![
            TexasAudioCue::new(TexasSoundKind::ChipHandle, 0.0, 0.42, seed),
            TexasAudioCue::new(TexasSoundKind::ChipLay, 0.15, 0.54, seed + 1),
            TexasAudioCue::new(TexasSoundKind::ChipCollide, 0.28, 0.40, seed + 2),
        ],
        TexasHoldemAction::AllIn => {
            let density = if amount >= 20 { 3 } else { 2 };
            let mut cues = vec![
                TexasAudioCue::new(TexasSoundKind::AllIn, 0.0, 0.46, seed),
                TexasAudioCue::new(TexasSoundKind::ChipHandle, 0.04, 0.48, seed + 1),
            ];
            for index in 0..density {
                cues.push(TexasAudioCue::new(
                    TexasSoundKind::ChipCollide,
                    0.20 + index as f32 * 0.085,
                    0.42 - index as f32 * 0.035,
                    seed + 2 + index as u64,
                ));
            }
            cues
        }
    }
}

pub fn texas_street_sound_plan(card_count: usize, seed: u64) -> Vec<TexasAudioCue> {
    let mut cues = vec![
        TexasAudioCue::new(TexasSoundKind::ChipHandle, 0.02, 0.34, seed),
        TexasAudioCue::new(TexasSoundKind::ChipCollide, 0.38, 0.46, seed + 1),
    ];
    cues.extend((0..card_count).map(|index| {
        TexasAudioCue::new(
            TexasSoundKind::CardPlace,
            0.12 + index as f32 * 0.08,
            0.43,
            seed + 2 + index as u64,
        )
    }));
    cues
}

pub fn texas_hand_finish_sound_plan(showdown: bool, seed: u64) -> Vec<TexasAudioCue> {
    let mut cues = vec![
        TexasAudioCue::new(TexasSoundKind::ChipHandle, 0.02, 0.34, seed),
        TexasAudioCue::new(TexasSoundKind::ChipStack, 0.62, 0.54, seed + 1),
    ];
    if showdown {
        cues.push(TexasAudioCue::new(
            TexasSoundKind::ShowdownFan,
            0.15,
            0.46,
            seed + 2,
        ));
        cues.extend((0..5).map(|index| {
            TexasAudioCue::new(
                TexasSoundKind::CardPlace,
                0.48 + index as f32 * 0.085,
                0.30,
                seed + 3 + index as u64,
            )
        }));
        cues.push(TexasAudioCue::new(
            TexasSoundKind::Confirm,
            1.02,
            0.38,
            seed + 8,
        ));
    }
    cues
}

pub fn texas_side_pot_sound_plan(seed: u64) -> Vec<TexasAudioCue> {
    vec![
        TexasAudioCue::new(TexasSoundKind::PotDivide, 0.18, 0.30, seed),
        TexasAudioCue::new(TexasSoundKind::ChipHandle, 0.25, 0.32, seed + 1),
        TexasAudioCue::new(TexasSoundKind::ChipStack, 0.62, 0.44, seed + 2),
    ]
}

pub fn queue_texas_turn_sound(cues: &mut Vec<TexasAudioCue>, seed: u64) {
    cues.push(TexasAudioCue::new(TexasSoundKind::Turn, 0.04, 0.28, seed));
}

pub fn play_texas_audio_cues(
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut state: ResMut<TexasChipTableState>,
    mut commands: Commands,
) {
    let mut waiting = Vec::with_capacity(state.audio_cues.len());
    let mut ready = Vec::new();
    for mut cue in std::mem::take(&mut state.audio_cues) {
        cue.remaining -= time.delta_secs();
        if cue.remaining <= 0.0 {
            ready.push(cue);
        } else {
            waiting.push(cue);
        }
    }
    state.audio_cues = waiting;

    for cue in ready {
        let variants = assets.games.texas_sounds.variants(cue.kind);
        if variants.is_empty() {
            continue;
        }
        let sound = variants[cue.seed as usize % variants.len()].clone();
        commands.spawn((
            AudioPlayer::new(sound),
            PlaybackSettings {
                volume: Volume::Linear(cue.volume),
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_have_distinct_and_bounded_sound_plans() {
        let check = texas_action_sound_plan(TexasHoldemAction::Check, 0, 1);
        let fold = texas_action_sound_plan(TexasHoldemAction::Fold, 0, 1);
        let raise = texas_action_sound_plan(TexasHoldemAction::RaiseTo(10), 10, 1);
        let all_in = texas_action_sound_plan(TexasHoldemAction::AllIn, 20, 1);
        assert_eq!(
            check.iter().map(|cue| cue.kind).collect::<Vec<_>>(),
            [TexasSoundKind::Check]
        );
        assert_eq!(
            fold.iter().map(|cue| cue.kind).collect::<Vec<_>>(),
            [TexasSoundKind::Fold]
        );
        assert_eq!(
            raise.iter().map(|cue| cue.kind).collect::<Vec<_>>(),
            [
                TexasSoundKind::ChipHandle,
                TexasSoundKind::ChipLay,
                TexasSoundKind::ChipCollide,
            ]
        );
        assert_eq!(all_in.len(), 5);
        assert!(all_in.iter().all(|cue| (0.0..=1.0).contains(&cue.volume)));
    }

    #[test]
    fn street_side_pot_and_showdown_plans_match_their_visual_beats() {
        let street = texas_street_sound_plan(3, 4);
        assert_eq!(
            street
                .iter()
                .filter(|cue| cue.kind == TexasSoundKind::CardPlace)
                .count(),
            3
        );
        let side_pot = texas_side_pot_sound_plan(5);
        assert_eq!(side_pot[0].kind, TexasSoundKind::PotDivide);
        assert!(side_pot[0].remaining < side_pot[2].remaining);

        let showdown = texas_hand_finish_sound_plan(true, 6);
        assert!(
            showdown
                .iter()
                .any(|cue| cue.kind == TexasSoundKind::ShowdownFan)
        );
        assert_eq!(
            showdown
                .iter()
                .filter(|cue| cue.kind == TexasSoundKind::CardPlace)
                .count(),
            5
        );
        assert!(
            showdown
                .iter()
                .any(|cue| cue.kind == TexasSoundKind::Confirm)
        );
    }
}

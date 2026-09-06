use super::*;
use std::path::PathBuf;

#[test]
fn completed_update_dialog_offers_restart_and_later_actions() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let root = commands.spawn(Node::default()).id();
        let mut updater = UpdateManager::default();
        updater.state = UpdateState::Ready {
            version: semver::Version::new(1, 2, 3),
            staged: PathBuf::from("leocard.update"),
        };
        updater.dialog_open = true;
        render_update_dialog(&mut commands, root, &updater, &assets);
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let actions = app
        .world_mut()
        .query::<&UiAction>()
        .iter(app.world())
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        actions
            .iter()
            .any(|action| matches!(action, UiAction::RestartToUpdate))
    );
    assert!(
        actions
            .iter()
            .any(|action| matches!(action, UiAction::HideUpdateDialog))
    );
    assert!(
        actions
            .iter()
            .all(|action| !matches!(action, UiAction::OpenGitHubRepository))
    );
    let github_buttons = app
        .world_mut()
        .query_filtered::<&UiAction, (With<Button>, With<GitHubRepositoryButton>)>()
        .iter(app.world())
        .count();
    assert_eq!(github_buttons, 0);
}

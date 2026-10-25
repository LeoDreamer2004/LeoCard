use super::*;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use leocard_protocol::{
    AVATAR_DIMENSION, ClientCommand, PlayerViolation, ReconnectToken, RejectReason, ServerEvent,
};
use leocard_qigui523::build_deck;
use std::io::Cursor;

fn avatar_png(shade: u8) -> Vec<u8> {
    let image = RgbaImage::from_pixel(
        AVATAR_DIMENSION,
        AVATAR_DIMENSION,
        Rgba([shade, 80, 160, 255]),
    );
    let mut png = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image)
        .write_to(&mut png, ImageFormat::Png)
        .unwrap();
    png.into_inner()
}

#[test]
fn changing_an_avatar_midgame_broadcasts_the_new_image_and_preserves_it_for_later_joiners() {
    let mut session = QiGui523Session::new(ROOM, 52300, rules(), build_deck(1)).unwrap();
    join_three(&mut session);
    let original = avatar_png(40);
    let uploaded = send_next(
        &mut session,
        HOST,
        ClientCommand::SetAvatar { png: original },
    );
    assert!(rejection(&uploaded).is_none());
    let old_id = session.players[0].avatar.unwrap();
    for connection in [HOST, SECOND, THIRD] {
        assert!(uploaded.iter().any(|delivery| {
            delivery.recipient == connection
                && matches!(&delivery.message.event, ServerEvent::LobbySnapshot(lobby)
                    if lobby.players[0].avatar == Some(old_id))
        }));
        send_next(
            &mut session,
            connection,
            ClientCommand::SetReady { ready: true },
        );
    }
    send_next(&mut session, HOST, ClientCommand::StartGame);
    let current_player = session.game().unwrap().trick().unwrap().current_player();
    let replacement = avatar_png(200);
    let changed = send_next(
        &mut session,
        HOST,
        ClientCommand::SetAvatar {
            png: replacement.clone(),
        },
    );
    assert!(rejection(&changed).is_none());
    let new_id = session.players[0].avatar.unwrap();
    assert_ne!(old_id, new_id);
    for connection in [HOST, SECOND, THIRD] {
        let image_index = changed
            .iter()
            .position(|delivery| {
                delivery.recipient == connection
                    && matches!(&delivery.message.event, ServerEvent::AvatarData { id, png }
                        if *id == new_id && *png == replacement)
            })
            .expect("every player receives the replacement image");
        let snapshot_index = changed
            .iter()
            .position(|delivery| {
                delivery.recipient == connection
                    && qigui523_snapshot(&delivery.message.event)
                        .is_some_and(|snapshot| snapshot.players[0].avatar == Some(new_id))
            })
            .expect("every player's snapshot refers to the replacement");
        assert!(image_index < snapshot_index);
    }
    assert_eq!(
        session.game().unwrap().trick().unwrap().current_player(),
        current_player
    );

    let repeated = send_next(
        &mut session,
        HOST,
        ClientCommand::SetAvatar {
            png: replacement.clone(),
        },
    );
    assert!(rejection(&repeated).is_none());
    assert_eq!(session.players[0].avatar, Some(new_id));
    assert!(
        repeated
            .iter()
            .all(|delivery| { !matches!(delivery.message.event, ServerEvent::AvatarData { .. }) })
    );
    let invalid = send_next(
        &mut session,
        HOST,
        ClientCommand::SetAvatar { png: vec![0] },
    );
    assert_eq!(
        rejection(&invalid),
        Some(&RejectReason::Player(PlayerViolation::InvalidAvatar))
    );
    assert_eq!(session.players[0].avatar, Some(new_id));

    let observer = ConnectionId(99);
    let joined = session.handle(
        observer,
        message(1, join_command("观战者", ReconnectToken(99))),
    );
    assert!(rejection(&joined).is_none());
    assert!(joined.iter().any(|delivery| {
        delivery.recipient == observer
            && matches!(&delivery.message.event, ServerEvent::AvatarData { id, png }
                if *id == new_id && *png == replacement)
    }));
    assert!(joined.iter().all(|delivery| {
        !matches!(&delivery.message.event, ServerEvent::AvatarData { id, .. } if *id == old_id)
    }));
}

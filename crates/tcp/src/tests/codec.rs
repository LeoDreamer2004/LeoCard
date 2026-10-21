use super::prelude::*;
use crate::codec::FrameReader;
use leocard_protocol::encode_frame;
use std::future::{Future, poll_fn};
use std::task::Poll;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn framed_codec_works_when_bytes_arrive_in_fragments() {
    let (mut left, mut right) = tokio::io::duplex(64);
    let message = ClientMessage::new(RoomId(1), RequestId(1), ClientCommand::RequestSnapshot);
    let frame = encode_frame(&message).unwrap();
    let writer = tokio::spawn(async move {
        for chunk in frame.chunks(2) {
            left.write_all(chunk).await.unwrap();
        }
    });
    let decoded: ClientMessage = read_message(&mut right).await.unwrap();
    writer.await.unwrap();
    assert_eq!(decoded, message);
}

#[tokio::test]
async fn cancelled_receives_preserve_partial_headers_and_bodies() {
    let (mut sender, receiver) = tokio::io::duplex(4096);
    let mut reader = FrameReader::new(receiver);
    let message = ClientMessage::new(RoomId(1), RequestId(9), ClientCommand::RequestSnapshot);
    let frame = encode_frame(&message).unwrap();
    for range in [0..2, 2..5] {
        sender.write_all(&frame[range]).await.unwrap();
        let mut pending = Box::pin(reader.receive::<ClientMessage>());
        poll_fn(|cx| {
            assert!(pending.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        // The select branch that owns this receive can now be cancelled safely.
    }
    sender.write_all(&frame[5..]).await.unwrap();
    sender.write_all(&frame).await.unwrap();
    assert_eq!(reader.receive::<ClientMessage>().await.unwrap(), message);
    assert_eq!(reader.receive::<ClientMessage>().await.unwrap(), message);
}

#[tokio::test]
async fn partial_frame_eof_is_reported_without_decoding_truncated_data() {
    let (mut sender, receiver) = tokio::io::duplex(32);
    sender.write_all(&[0, 0, 0, 10, 1]).await.unwrap();
    drop(sender);
    let mut reader = FrameReader::new(receiver);
    assert!(matches!(
        reader.receive::<ClientMessage>().await,
        Err(TcpError::ConnectionClosed)
    ));
}

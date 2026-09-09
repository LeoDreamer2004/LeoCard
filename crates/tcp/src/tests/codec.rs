use super::prelude::*;

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

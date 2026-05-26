use super::message_utils::build_telegram_ack_reaction_request;

#[test]
fn reaction_request_uses_telegram_emoji_shape() {
    let request = build_telegram_ack_reaction_request("1", 2, "👌");

    assert_eq!(request["reaction"][0]["type"], "emoji");
    assert_eq!(request["reaction"][0]["emoji"], "👌");
}

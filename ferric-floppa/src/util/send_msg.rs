use std::borrow::Cow;

use serenity::{
    builder::{CreateMessage, ParseValue},
    http::Http,
    model::prelude::{AttachmentType, ChannelId, MessageReference},
};

use crate::consts::FlopResult;

pub async fn send_text(
    http: impl AsRef<Http>,
    channel: &ChannelId,
    text: impl ToString,
    reference: Option<impl Into<MessageReference>>,
) -> FlopResult<()> {
    let text = text.to_string();
    if text.len() <= 2000 {
        send_flop_message(
            http,
            channel,
            |m| {
                m.content(text);
            },
            reference,
        )
        .await?;
    } else {
        send_flop_message(
            http,
            channel,
            |m| {
                m.add_file(AttachmentType::Bytes {
                    data: Cow::from(text.as_bytes()),
                    filename: "result.txt".to_string(),
                });
            },
            reference,
        )
        .await?;
    }

    Ok(())
}

pub async fn send_flop_message<'a, F>(
    http: impl AsRef<Http>,
    channel: &ChannelId,
    f: F,
    reference: Option<impl Into<MessageReference>>,
) -> FlopResult<()>
where
    for<'b> F: FnOnce(&'b mut CreateMessage<'a>),
{
    channel
        .send_message(http, |m| {
            f(m);
            m.allowed_mentions(|am| am.parse(ParseValue::Users));
            if let Some(reference) = reference {
                m.reference_message(reference);
            }
            m
        })
        .await?;
    Ok(())
}

#[macro_export]
macro_rules! send_reply_text {
    ($str:expr, $ctx:ident, $msg:ident) => {
        $crate::handle_error!(
            send_msg::send_text(&$ctx.http, &$msg.channel_id, $str, Some(&$msg)).await,
            "Error sending message",
            $msg
        )
    };
}

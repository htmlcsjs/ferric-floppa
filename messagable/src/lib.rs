use std::fmt::Display;

use serenity::{builder::CreateMessage, http::Http, model::prelude::*, Error};

#[async_trait::async_trait]
pub trait Messagable {
    //! Allows for the construction of messages from a value, without having to write
    //! all the boilerplate for message construction.
    //!
    //! This is automatically implmented for any type that implments [`ToString`],
    //! sending a message that has the value as its content.
    //!
    //! It is techincally using the [`async_trait::async_trait`] macro however
    //! this is not needed in impls due to it being only used on default methods
    //!
    //! ## Examples
    //!
    //! ```
    //! use messagable::Messagable;
    //! use serenity::builder::CreateMessage;
    //!
    //! struct MyStruct {
    //!     a: i32,
    //!     b: i32,
    //! }
    //!
    //! impl Messagable for MyStruct {
    //!     fn modify_message<'a, 'b>(&self, builder: &'a mut CreateMessage<'b>) -> &'a mut CreateMessage<'b> {
    //!         builder.content(self.a * self.b)
    //!     }
    //! }
    //! ```

    /// Takes a [`CreateMessage`] and adds/changes content to it, finally returning it.
    fn modify_message<'a, 'b>(
        &self,
        builder: &'a mut CreateMessage<'b>,
    ) -> &'a mut CreateMessage<'b>;

    /// Chains together this with another [`Messagable`].
    ///
    /// The order of execution is `self` then `other`.
    ///
    /// ## Examples
    /// ```
    /// use messagable::Messagable;
    /// use serenity::builder::CreateMessage;
    ///
    /// struct CharReact(char);
    ///
    /// impl Messagable for CharReact {
    ///     fn modify_message<'a, 'b>(&self, builder: &'a mut CreateMessage<'b>) -> &'a mut CreateMessage<'b> {
    ///         builder.reactions([self.0])
    ///     }
    /// }
    ///
    /// fn test() {
    ///     "a".chain(ChatReact('🍎')); // "a" is set as message body *then* '🍎' is added as an reaction
    /// }
    /// ```
    fn chain<T>(self, other: T) -> Chain<Self, T>
    where
        Self: Sized,
        T: Messagable,
    {
        Chain {
            first: self,
            second: other,
        }
    }

    /// Replies to a message with this as the message data
    ///
    /// Returns a result with the message or an error
    async fn reply(&self, msg: &Message, http: &Http) -> Result<Message, Error> {
        msg.channel_id
            .send_message(http, |b| {
                b.reference_message(msg);
                self.modify_message(b)
            })
            .await
    }

    /// Sends a message in a channel with this as the message data
    ///
    /// Returns a result with the message or an error
    async fn send(&self, channel: ChannelId, http: &Http) -> Result<Message, Error> {
        channel.send_message(http, |b| self.modify_message(b)).await
    }
}

impl<T> Messagable for T
where
    T: Display,
{
    fn modify_message<'a, 'b>(
        &self,
        builder: &'a mut CreateMessage<'b>,
    ) -> &'a mut CreateMessage<'b> {
        builder.content(self)
    }
}

/// Chains together two things that are [`Messagable`].
///
/// Helper trait for [`Messagable::chain`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chain<T, U>
where
    T: Messagable,
    U: Messagable,
{
    first: T,
    second: U,
}

impl<T, U> Messagable for Chain<T, U>
where
    T: Messagable,
    U: Messagable,
{
    fn modify_message<'a, 'b>(
        &self,
        builder: &'a mut CreateMessage<'b>,
    ) -> &'a mut CreateMessage<'b> {
        let builder = self.first.modify_message(builder);
        builder.reactions(['🍎']);
        self.second.modify_message(builder)
    }
}

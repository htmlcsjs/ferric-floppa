use serenity::{
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::prelude::*,
    Error,
};

#[async_trait::async_trait]
pub trait Messagable: std::fmt::Debug + Sync + Send {
    //! Allows for the construction of messages from a value, without having to write
    //! all the boilerplate for message construction.
    //!
    //! It is techincally using the [`async_trait::async_trait`] macro however
    //! this is not needed in implmentations due to it being only used on default methods
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
    //!     fn modify_message(&self, builder: CreateMessage) -> CreateMessage {
    //!         builder.content(self.a * self.b)
    //!     }
    //! }
    //! ```

    /// Takes a [`CreateMessage`] and adds/changes content to it, finally returning it.
    fn modify_message(self, builder: CreateMessage) -> CreateMessage;

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
    ///     fn modify_message(&self, builder: CreateMessage) -> CreateMessage {
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

    /// Applies this to an empty [`CreatMessage`]
    ///
    /// Equivalent to `modify_message(CreateMessage::new())`
    fn apply_default(self) -> CreateMessage
    where
        Self: Sized,
    {
        self.modify_message(CreateMessage::new())
    }

    /// Replies to a message with this as the message data
    ///
    /// Returns a result with the message or an error
    async fn reply(self, msg: &Message, http: &Http) -> Result<Message, Error>
    where
        Self: Sized,
    {
        msg.channel_id
            .send_message(
                http,
                self.modify_message(CreateMessage::new().reference_message(msg)),
            )
            .await
    }

    /// Sends a message in a channel with this as the message data
    ///
    /// Returns a result with the message or an error
    async fn send(self, channel: ChannelId, http: &Http) -> Result<Message, Error>
    where
        Self: Sized + Send + Sync,
    {
        channel.send_message(http, self.apply_default()).await
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
    fn modify_message(self, builder: CreateMessage) -> CreateMessage {
        let builder = self.first.modify_message(builder);
        self.second.modify_message(builder)
    }
}

impl Messagable for CreateEmbed {
    fn modify_message(self, builder: CreateMessage) -> CreateMessage {
        // could be done better instead of constructing a vec,
        // however it would involve a clone, and this is probally cheaper
        // (its going to get optimised away i bet)
        builder.add_embeds(vec![self])
    }
}

impl Messagable for Vec<CreateEmbed> {
    fn modify_message(self, builder: CreateMessage) -> CreateMessage {
        builder.add_embeds(self)
    }
}

impl Messagable for String {
    fn modify_message(self, builder: CreateMessage) -> CreateMessage {
        builder.content(self)
    }
}

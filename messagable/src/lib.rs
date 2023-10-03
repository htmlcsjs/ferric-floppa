use std::fmt::Display;

use serenity::builder::CreateMessage;

/// Allows for the construction of messages from a value, without having to write
/// all the boilerplate for message construction.
///
/// This is automatically implmented for any type that implments [`ToString`],
/// sending a message that has the value as its content.
///
/// ## Examples
///
/// ```
/// use messagable::Messagable;
/// use serenity::builder::CreateMessage;
///
/// struct MyStruct {
///     a: i32,
///     b: i32,
/// }
///
/// impl Messagable for MyStruct {
///     fn modify_message<'a, 'b>(&self, builder: &'a mut CreateMessage<'b>) -> &'a mut CreateMessage<'b> {
///         builder.content(self.a * self.b)
///     }
/// }
/// ```
pub trait Messagable {
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
    /// impl Messagable for MyStruct {
    ///     fn modify_message<'a, 'b>(&self, builder: &'a mut CreateMessage<'b>) -> &'a mut CreateMessage<'b> {
    ///         builder.reactions([self.0])
    ///     }
    /// }
    ///
    /// fn test() {
    /// "a".chain(ChatReact('🍎')); // "a" is set as message body *then* '🍎' is added as an reaction
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

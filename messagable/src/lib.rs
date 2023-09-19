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
///     fn create<'a, 'b>(&self, builder: &'a mut CreateMessage<'b>) -> &'a mut CreateMessage<'b> {
///         builder.content(self.a * self.b)
///     }
/// }
/// ```
pub trait Messagable {
    /// Takes a [`CreateMessage`] and adds/changes content to it, finally returning it.
    fn create<'a, 'b>(&self, builder: &'a mut CreateMessage<'b>) -> &'a mut CreateMessage<'b>;
}

impl<T> Messagable for T
where
    T: Display,
{
    fn create<'a, 'b>(&self, builder: &'a mut CreateMessage<'b>) -> &'a mut CreateMessage<'b> {
        builder.content(self)
    }
}

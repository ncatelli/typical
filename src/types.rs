/// EntityId represents an an identifier to an Abstract Entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityId<T: Copy + Eq + Into<usize>>(T);

impl<T> From<EntityId<T>> for usize
where
    T: Copy + Eq + Into<usize>,
{
    fn from(src: EntityId<T>) -> Self {
        src.0.into()
    }
}

impl<T> EntityId<T>
where
    T: Copy + Eq + Into<usize>,
{
    pub fn new(id: T) -> Self {
        Self(id)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TypeError {
    Converge,
}

impl std::fmt::Debug for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Converge => write!(f, "unable to converge types."),
        }
    }
}

/// Implementation requirements for an abstract type entity.
pub trait AbstractEntity
where
    Self: Clone + Eq,
{
    /// Represents the top-level, least constrained Abstract entity. In many
    /// implementations this could be an `Any` type.
    fn unconstrained_type() -> Self;
    /// The arity, if applicable, of the type,
    fn arity(&self) -> Option<usize>;
    /// Attempts to converge two types to their more constrained commonality.
    fn converge(&self, other: &Self) -> Result<Self, TypeError>;
}

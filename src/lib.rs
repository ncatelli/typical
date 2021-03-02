//! This crate functions as a test/toy implementation of an algebraic
//! type-checker based on the work by Robert Grosse.

mod graph;

pub type EntityId = usize;

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

pub trait AbstractTypes<V, U> {
    type Error;

    fn meet(lhs: &V, rhs: &U) -> Result<Vec<(Value, Use)>, Self::Error>;
}

#[derive(Copy, Clone, Debug)]
pub struct Value(usize);
#[derive(Copy, Clone, Debug)]
pub struct Use(usize);

#[derive(Debug, Clone)]
enum TypeNode<V, U> {
    Var,
    Value(V),
    Use(U),
}

#[derive(Debug, Clone)]
pub struct TypeChecker<V, U, AT>
where
    AT: AbstractTypes<V, U>,
{
    r: graph::Graph<EntityId>,
    types: Vec<TypeNode<V, U>>,
    abstract_type_mapper: AT,
}

impl<V, U, AT> TypeChecker<V, U, AT>
where
    AT: AbstractTypes<V, U>,
{
    pub fn new(abstract_type_mapper: AT) -> Self {
        Self {
            r: Default::default(),
            types: Vec::new(),
            abstract_type_mapper,
        }
    }

    pub fn new_val(&mut self, val_type: V) -> Value {
        let i = self.r.add_node_mut();
        assert!(i == self.types.len());
        self.types.push(TypeNode::Value(val_type));
        Value(i)
    }

    pub fn new_use(&mut self, constraint: U) -> Use {
        let i = self.r.add_node_mut();
        assert!(i == self.types.len());
        self.types.push(TypeNode::Use(constraint));
        Use(i)
    }

    pub fn var(&mut self) -> (Value, Use) {
        let i = self.r.add_node_mut();
        assert!(i == self.types.len());
        self.types.push(TypeNode::Var);
        (Value(i), Use(i))
    }

    pub fn flow(&mut self, lhs: Value, rhs: Use) -> Result<(), AT::Error> {
        let mut pending_edges = vec![(lhs, rhs)];
        let mut type_pairs_to_check = Vec::new();
        while let Some((lhs, rhs)) = pending_edges.pop() {
            type_pairs_to_check.extend(self.r.add_edge_mut(lhs.0, rhs.0));

            // Check if adding that edge resulted in any new type pairs needing to be checked
            while let Some((lhs, rhs)) = type_pairs_to_check.pop() {
                if let TypeNode::Value(lhs_head) = &self.types[lhs] {
                    if let TypeNode::Use(rhs_head) = &self.types[rhs] {
                        let new_edges = AT::meet(lhs_head, rhs_head)?;
                        pending_edges.extend(new_edges.into_iter());
                    }
                }
            }
        }
        assert!(pending_edges.is_empty() && type_pairs_to_check.is_empty());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum AbstractTypeValue {
        VBool,
        VInteger,
        VFloat,
        VString,
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum AbstractTypeUse {
        UBool,
        UInteger,
        UFloat,
        UString,
    }

    #[derive(Debug)]
    pub struct LiteralTypeSystem;

    impl AbstractTypes<AbstractTypeValue, AbstractTypeUse> for LiteralTypeSystem {
        type Error = TypeError;

        fn meet(
            lhs: &AbstractTypeValue,
            rhs: &AbstractTypeUse,
        ) -> Result<Vec<(Value, Use)>, Self::Error> {
            let out = vec![];
            match (lhs, rhs) {
                (&AbstractTypeValue::VBool, &AbstractTypeUse::UBool) => Ok(out),
                (&AbstractTypeValue::VInteger, &AbstractTypeUse::UInteger) => Ok(out),
                (&AbstractTypeValue::VFloat, &AbstractTypeUse::UFloat) => Ok(out),
                (&AbstractTypeValue::VString, &AbstractTypeUse::UString) => Ok(out),
                _ => Err(TypeError::Converge),
            }
        }
    }

    #[test]
    fn type_match() {
        let mut t = TypeChecker::new(LiteralTypeSystem);
        let vid = t.new_val(AbstractTypeValue::VBool);
        let uid = t.new_use(AbstractTypeUse::UBool);
        assert!(t.flow(vid, uid).is_ok());
    }

    #[test]
    fn type_mismatch() {
        let mut t = TypeChecker::new(LiteralTypeSystem);
        let vid = t.new_val(AbstractTypeValue::VBool);
        let uid = t.new_use(AbstractTypeUse::UFloat);
        assert!(t.flow(vid, uid).is_err());
    }
}

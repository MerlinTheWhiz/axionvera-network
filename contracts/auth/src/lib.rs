#![no_std]

/// Shared failure reasons emitted by reusable access policies.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PolicyViolation {
    Unauthorized,
    AddressMismatch,
    PredicateFailed,
}

/// Contract-agnostic interface for enforcing access rules against a context.
pub trait AccessPolicy<Context> {
    fn enforce(&self, context: &Context) -> Result<(), PolicyViolation>;

    fn and<P>(self, other: P) -> AllOf<Self, P>
    where
        Self: Sized,
        P: AccessPolicy<Context>,
    {
        AllOf {
            left: self,
            right: other,
        }
    }

    fn or<P>(self, other: P) -> AnyOf<Self, P>
    where
        Self: Sized,
        P: AccessPolicy<Context>,
    {
        AnyOf {
            left: self,
            right: other,
        }
    }
}

/// A composed policy that requires all nested policies to succeed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AllOf<A, B> {
    left: A,
    right: B,
}

impl<Context, A, B> AccessPolicy<Context> for AllOf<A, B>
where
    A: AccessPolicy<Context>,
    B: AccessPolicy<Context>,
{
    fn enforce(&self, context: &Context) -> Result<(), PolicyViolation> {
        self.left.enforce(context)?;
        self.right.enforce(context)
    }
}

/// A composed policy that succeeds when any nested policy succeeds.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AnyOf<A, B> {
    left: A,
    right: B,
}

impl<Context, A, B> AccessPolicy<Context> for AnyOf<A, B>
where
    A: AccessPolicy<Context>,
    B: AccessPolicy<Context>,
{
    fn enforce(&self, context: &Context) -> Result<(), PolicyViolation> {
        match self.left.enforce(context) {
            Ok(()) => Ok(()),
            Err(left_error) => match self.right.enforce(context) {
                Ok(()) => Ok(()),
                Err(_) => Err(left_error),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AccessPolicy, PolicyViolation};

    #[derive(Copy, Clone)]
    struct AllowIfEven;

    impl AccessPolicy<u32> for AllowIfEven {
        fn enforce(&self, value: &u32) -> Result<(), PolicyViolation> {
            if value % 2 == 0 {
                Ok(())
            } else {
                Err(PolicyViolation::PredicateFailed)
            }
        }
    }

    #[derive(Copy, Clone)]
    struct AllowIfLargeEnough;

    impl AccessPolicy<u32> for AllowIfLargeEnough {
        fn enforce(&self, value: &u32) -> Result<(), PolicyViolation> {
            if *value >= 10 {
                Ok(())
            } else {
                Err(PolicyViolation::Unauthorized)
            }
        }
    }

    #[test]
    fn and_policy_requires_all_children() {
        let policy = AllowIfEven.and(AllowIfLargeEnough);

        assert_eq!(policy.enforce(&12), Ok(()));
        assert_eq!(policy.enforce(&8), Err(PolicyViolation::Unauthorized));
        assert_eq!(policy.enforce(&11), Err(PolicyViolation::PredicateFailed));
    }

    #[test]
    fn or_policy_accepts_any_child() {
        let policy = AllowIfEven.or(AllowIfLargeEnough);

        assert_eq!(policy.enforce(&12), Ok(()));
        assert_eq!(policy.enforce(&9), Err(PolicyViolation::PredicateFailed));
        assert_eq!(policy.enforce(&8), Ok(()));
        assert_eq!(policy.enforce(&11), Ok(()));
    }
}

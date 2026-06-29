#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LifecycleState {
    Deployed,
    Initialized,
    Active,
    Maintenance,
    Deprecated,
    Retired,
}

pub struct ContractLifecycle {
    pub current_state: LifecycleState,
    pub admin: String,
}

impl ContractLifecycle {
    pub fn new(admin: String) -> Self {
        Self {
            current_state: LifecycleState::Deployed,
            admin,
        }
    }

    pub fn transition_to(&mut self, sender: &str, next_state: LifecycleState) -> Result<(), &'static str> {
        if sender != self.admin {
            return Err("Unauthorized: Only administrative controls work");
        }

        let is_valid = match (self.current_state, next_state) {
            (LifecycleState::Deployed, LifecycleState::Initialized) => true,
            (LifecycleState::Initialized, LifecycleState::Active) => true,
            (LifecycleState::Active, LifecycleState::Maintenance) => true,
            (LifecycleState::Maintenance, LifecycleState::Active) => true,
            (LifecycleState::Active, LifecycleState::Deprecated) => true,
            (LifecycleState::Deprecated, LifecycleState::Retired) => true,
            _ => false,
        };

        if is_valid {
            self.current_state = next_state;
            println!("Event emitted: Lifecycle transitioned to {:?}", next_state);
            Ok(())
        } else {
            Err("Invalid state transition rejected")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_lifecycle_flow() {
        let mut lifecycle = ContractLifecycle::new("admin_user".to_string());
        assert_eq!(lifecycle.current_state, LifecycleState::Deployed);

        assert!(lifecycle.transition_to("admin_user", LifecycleState::Initialized).is_ok());
        assert!(lifecycle.transition_to("admin_user", LifecycleState::Active).is_ok());
    }

    #[test]
    fn test_invalid_transition_rejected() {
        let mut lifecycle = ContractLifecycle::new("admin_user".to_string());
        assert!(lifecycle.transition_to("admin_user", LifecycleState::Active).is_err());
    }

    #[test]
    fn test_unauthorized_admin_control() {
        let mut lifecycle = ContractLifecycle::new("admin_user".to_string());
        assert!(lifecycle.transition_to("attacker", LifecycleState::Initialized).is_err());
    }
}

use soroban_sdk::{Address, Env, Symbol};
/// Provides events for access control operations
pub struct AccessControlEvents {}

impl AccessControlEvents {
    /// Emitted when a role is granted to an account
    ///
    /// - topics - `["role_granted", role: Symbol, account: Address]`
    /// - data - `sender: Address`
    pub fn role_granted(e: &Env, role: Symbol, account: Address) {
        let topics = (Symbol::new(e, "role_granted"), role);
        e.events().publish(topics, account);
    }

    /// Emitted when a role is revoked from an account
    ///
    /// - topics - `["role_revoked", role: Symbol, account: Address]`
    /// - data - `sender: Address`
    pub fn role_revoked(e: &Env, role: Symbol, account: Address) {
        let topics = (Symbol::new(e, "role_revoked"), role);
        e.events().publish(topics, account);
    }

    /// Emitted when a role's admin role is changed
    ///
    /// - topics - `["role_admin_changed", role: Symbol, new_admin_role: Symbol]`
    /// - data - `sender: Address`
    pub fn role_admin_changed(e: &Env, role: Symbol, new_admin_role: Symbol) {
        let topics = (Symbol::new(e, "role_admin_changed"), role);
        e.events().publish(topics, new_admin_role);
    }
}

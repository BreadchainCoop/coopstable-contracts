use soroban_sdk::{
    contracttype, 
    Symbol, 
    Map,
    Address,
    Env
};

#[derive(Clone)]
#[contracttype]
pub struct RoleData {
    pub admin_role: Symbol,
    pub members: Map<Address, bool>,
}

impl RoleData {
    pub fn new(e: &Env, admin_role: Symbol) -> Self {
        Self {
            admin_role,
            members: Map::new(e),
        }
    }

    pub fn has_member(&self, account: &Address) -> bool {
        self.members.get(account.clone()).unwrap_or(false)
    }

    pub fn add_member(&mut self, account: &Address) {
        self.members.set(account.clone(), true);
    }

    pub fn remove_member(&mut self, account: &Address) {
        self.members.remove(account.clone());
    }
}

/// A storage structure for all roles in the contract
#[derive(Clone)]
#[contracttype]
pub struct RolesMap {
    pub roles: Map<Symbol, RoleData>,
}

impl RolesMap {
    pub fn new(e: &Env) -> Self {
        Self {
            roles: Map::new(e),
        }
    }

    pub fn get_role_data(&self, role: Symbol) -> Option<RoleData> {
        self.roles.get(role)
    }

    pub fn set_role_data(&mut self, role: Symbol, data: RoleData) {
        self.roles.set(role, data);
    }

    pub fn has_role(&self, role: Symbol, account: &Address) -> bool {
        match self.roles.get(role) {
            Some(role_data) => role_data.has_member(account),
            None => false,
        }
    }
}
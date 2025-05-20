use soroban_sdk::{
    vec, 
    Address, 
    Env, 
    Symbol, 
    Vec, 
    panic_with_error
};

use crate::{
    constants::{
        DEFAULT_ADMIN_ROLE, DEFAULT_INSTANCE_BUMP_AMOUNT, DEFAULT_INSTANCE_LIFETIME_THRESHOLD,
        ROLES_KEY,
    },
    contract_types::{RoleData, RolesMap},
    events::AccessControlEvents,
    error::AccessControlError
};

pub struct AccessControl {
    lifetime_threshold: u32,
    bump_amount: u32,
}

impl AccessControl {
    pub fn new() -> Self {
        Self {
            lifetime_threshold: DEFAULT_INSTANCE_LIFETIME_THRESHOLD,
            bump_amount: DEFAULT_INSTANCE_BUMP_AMOUNT,
        }
    }

    pub fn with_config(lifetime_threshold: u32, bump_amount: u32) -> Self {
        Self {
            lifetime_threshold,
            bump_amount,
        }
    }

    pub fn initialize(&self, e: &Env, admin: &Address) {
        let mut roles_map = self.read_roles_map(e);

        let mut admin_role_data = RoleData::new(e, DEFAULT_ADMIN_ROLE); // sets admin_role here
        admin_role_data.add_member(admin);
        roles_map.set_role_data(DEFAULT_ADMIN_ROLE, admin_role_data);

        self.write_roles_map(e, &roles_map);
    }

    fn read_roles_map(&self, e: &Env) -> RolesMap {
        e.storage()
            .instance()
            .extend_ttl(self.lifetime_threshold, self.bump_amount);

        match e.storage().instance().get(&ROLES_KEY) {
            Some(map) => map,
            None => RolesMap::new(e),
        }
    }

    fn write_roles_map(&self, e: &Env, roles_map: &RolesMap) {
        e.storage()
            .instance()
            .extend_ttl(self.lifetime_threshold, self.bump_amount);

        e.storage().instance().set(&ROLES_KEY, roles_map);
    }

    pub fn has_role(&self, e: &Env, role: Symbol, account: &Address) -> bool {
        let roles_map = self.read_roles_map(e);
        roles_map.has_role(role, account)
    }

    pub fn get_role_admin(&self, e: &Env, role: Symbol) -> Symbol {
        let roles_map = self.read_roles_map(e);

        match roles_map.get_role_data(role) {
            Some(role_data) => role_data.admin_role,
            None => DEFAULT_ADMIN_ROLE,
        }
    }

    pub fn set_role_admin(&self, e: &Env, role: Symbol, admin_role: Symbol) {
        let mut roles_map = self.read_roles_map(e);

        let role_data = match roles_map.get_role_data(role.clone()) {
            Some(mut data) => {
                data.admin_role = admin_role.clone();
                data
            }
            None => RoleData::new(e, admin_role.clone()),
        };

        roles_map.set_role_data(role.clone(), role_data);
        self.write_roles_map(e, &roles_map);

        AccessControlEvents::role_admin_changed(e, role, admin_role);
    }

    pub fn grant_role(&self, e: &Env, sender: Address, role: Symbol, account: &Address) {
        sender.require_auth();

        let admin_role = self.get_role_admin(e, role.clone());
        if !self.has_role(e, admin_role, &sender) {
            panic_with_error!(e, AccessControlError::OnlyRoleAdmin);
        }

        self._grant_role(e, role, account);
    }

    pub fn _grant_role(&self, e: &Env, role: Symbol, account: &Address) {
        let mut roles_map = self.read_roles_map(e);

        let role_data = match roles_map.get_role_data(role.clone()) {
            Some(mut data) => {
                if !data.has_member(account) {
                    data.add_member(account);
                    data
                } else {
                    return;
                }
            }
            None => {
                let mut data = RoleData::new(e, DEFAULT_ADMIN_ROLE);
                data.add_member(account);
                data
            }
        };

        roles_map.set_role_data(role.clone(), role_data);
        self.write_roles_map(e, &roles_map);

        AccessControlEvents::role_granted(e, role, account.clone());
    }

    pub fn revoke_role(&self, e: &Env, sender: Address, role: Symbol, account: &Address) {
        sender.require_auth();

        let admin_role = self.get_role_admin(e, role.clone());
        if !self.has_role(e, admin_role, &sender) {
            panic_with_error!(e, AccessControlError::OnlyRoleAdmin);
        }

        self._revoke_role(e, role, account);
    }

    pub fn _revoke_role(&self, e: &Env, role: Symbol, account: &Address) {
        let mut roles_map = self.read_roles_map(e);

        if let Some(mut role_data) = roles_map.get_role_data(role.clone()) {
            if role_data.has_member(account) {
                role_data.remove_member(account);
                roles_map.set_role_data(role.clone(), role_data);
                self.write_roles_map(e, &roles_map);

                AccessControlEvents::role_revoked(e, role, account.clone());
            }
        }
    }

    pub fn renounce_role(&self, e: &Env, role: Symbol, account: &Address) {
        account.require_auth();

        self._revoke_role(e, role, account);
    }

    pub fn check_role(&self, e: &Env, role: Symbol, account: &Address) {
        if !self.has_role(e, role, account) {
            panic_with_error!(e, AccessControlError::UnAuhtorizedRole);
        }
    }

    pub fn only_default_admin(&self, e: &Env, account: &Address) {
        self.only_role(e, account, DEFAULT_ADMIN_ROLE);
    }

    pub fn only_role(&self, e: &Env, account: &Address, role: Symbol) {
        account.require_auth();
        self.check_role(e, role, account);
    }

    pub fn get_role_members(&self, e: &Env, role: Symbol) -> Vec<Address> {
        let roles_map = self.read_roles_map(e);
        let members = vec![e];

        if let Some(role_data) = roles_map.get_role_data(role) {
            let keys = role_data.members.keys();
            let mut result = vec![e];

            for key in keys.iter() {
                if role_data.has_member(&key) {
                    result.push_back(key);
                }
            }

            return result;
        }

        members
    }
}

pub fn default_access_control(e: &Env) -> AccessControl {
    let access_control = AccessControl::new();
    e.storage().instance().extend_ttl(
        access_control.lifetime_threshold,
        access_control.bump_amount,
    );
    access_control
}

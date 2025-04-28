use soroban_sdk::contractimpl;

#[contractimpl]
pub trait LendingPoolAdapter {
    fn deposit_to_pool(&mut self, amount: i128);
    fn withdraw_from_pool(&mut self, amount: i128);
    fn get_yield(&self) -> i128;
    fn get_balance();
}
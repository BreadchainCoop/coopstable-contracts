use soroban_sdk::{contractclient, Address, Env, Symbol, Vec, Val};

/// ### LendingAdapter
///
/// Common interface for all lending protocol adapters in the Coopstable ecosystem.
/// Implementations of this trait enable integration with various lending protocols.
#[contractclient(name = "LendingAdapterClient")]
pub trait LendingAdapter {
    /// (Yield Controller only) Deposit assets into the lending protocol on behalf of a user
    ///
    /// Returns the actual amount deposited
    ///
    /// ### Arguments
    /// * `user` - The address of the user depositing assets
    /// * `asset` - The address of the asset to deposit
    /// * `amount` - The amount of the asset to deposit
    ///
    /// ### Panics
    /// If the caller is not the yield controller
    fn deposit(env: &Env, user: Address, asset: Address, amount: i128) -> i128;

    /// Get the authorization arguments for a deposit operation
    ///
    /// Returns the contract address, function name, and arguments for authorization
    ///
    /// ### Arguments
    /// * `user` - The address of the user depositing assets
    /// * `asset` - The address of the asset to deposit
    /// * `amount` - The amount of the asset to deposit
    fn deposit_auth(env: &Env, user: Address, asset: Address, amount: i128) -> Option<(Address, Symbol, Vec<Val>)>;
    
    /// (Yield Controller only) Withdraw assets from the lending protocol on behalf of a user
    ///
    /// Returns the actual amount withdrawn
    ///
    /// ### Arguments
    /// * `user` - The address of the user withdrawing assets
    /// * `asset` - The address of the asset to withdraw
    /// * `amount` - The amount of the asset to withdraw
    ///
    /// ### Panics
    /// If the caller is not the yield controller
    fn withdraw(env: &Env, user: Address, asset: Address, amount: i128) -> i128;

    /// Get the authorization arguments for a withdraw operation
    ///
    /// Returns the contract address, function name, and arguments for authorization
    ///
    /// ### Arguments
    /// * `user` - The address of the user withdrawing assets
    /// * `asset` - The address of the asset to withdraw
    /// * `amount` - The amount of the asset to withdraw
    fn withdraw_auth(env: &Env, user: Address, asset: Address, amount: i128) -> Option<(Address, Symbol, Vec<Val>)>;

    /// Fetch the accumulated yield for a specific asset
    ///
    /// ### Arguments
    /// * `asset` - The address of the asset to check yield for
    fn get_yield(env: &Env, asset: Address) -> i128;

    /// (Yield Controller only) Claim accumulated yield for a specific asset
    ///
    /// Returns the actual amount of yield claimed
    ///
    /// ### Arguments
    /// * `asset` - The address of the asset to claim yield for
    /// * `amount` - The amount of yield to claim
    ///
    /// ### Panics
    /// If the caller is not the yield controller
    fn claim_yield(env: &Env, asset: Address, amount: i128) -> i128;

    /// Get the authorization arguments for claiming yield
    ///
    /// Returns the contract address, function name, and arguments for authorization
    ///
    /// ### Arguments
    /// * `asset` - The address of the asset to claim yield for
    /// * `amount` - The amount of yield to claim
    fn claim_yield_auth(env: &Env, asset: Address, amount: i128) -> Option<(Address, Symbol, Vec<Val>)>;

    /// (Yield Controller only) Claim emissions rewards and send to a recipient
    ///
    /// Returns the amount of emissions claimed
    ///
    /// ### Arguments
    /// * `to` - The recipient address for the emissions
    /// * `asset` - The address of the asset to claim emissions for
    ///
    /// ### Panics
    /// If the caller is not the yield controller
    fn claim_emissions(e: &Env, to: Address, asset: Address) -> i128;

    /// Get the authorization arguments for claiming emissions
    ///
    /// Returns the contract address, function name, and arguments for authorization
    ///
    /// ### Arguments
    /// * `to` - The recipient address for the emissions
    /// * `asset` - The address of the asset to claim emissions for
    fn claim_emissions_auth(e: &Env, to: Address, asset: Address) -> Option<(Address, Symbol, Vec<Val>)>;

    /// Fetch the current APY for a specific asset in the lending protocol
    ///
    /// ### Arguments
    /// * `asset` - The address of the asset to check APY for
    fn get_apy(env: &Env, asset: Address) -> u32;
        
    /// Fetch the accumulated emissions rewards for a specific asset
    ///
    /// ### Arguments
    /// * `asset` - The address of the asset to check emissions for
    fn get_emissions(e: &Env, asset: Address) -> i128;

    /// Fetch the total amount deposited for a specific asset
    ///
    /// ### Arguments
    /// * `asset` - The address of the asset to check deposits for
    fn get_total_deposited(e: &Env, asset: Address) -> i128;
    
    /// Fetch the balance of a user for a specific asset in the lending protocol
    ///
    /// ### Arguments
    /// * `user` - The address of the user to check balance for
    /// * `asset` - The address of the asset to check balance for
    fn get_balance(e: &Env, user: Address, asset: Address) -> i128;
    
    /// Fetch the address of the protocol's native token (e.g., BLND for Blend)
    fn protocol_token(e: &Env) -> Address;

    /// (Yield Controller only) Update the principal amount for an epoch
    ///
    /// ### Arguments
    /// * `asset` - The address of the asset
    /// * `epoch` - The epoch number
    /// * `principal` - The principal amount for the epoch
    ///
    /// ### Panics
    /// If the caller is not the yield controller
    fn update_epoch_principal(env: &Env, asset: Address, epoch: u64, principal: i128);

    /// Initialize the lending adapter contract
    ///
    /// ### Arguments
    /// * `yield_controller` - The address of the yield controller contract
    /// * `lending_pool_id` - The address of the lending pool contract
    /// * `protocol_token_id` - The address of the protocol's native token
    fn __constructor(e: Env, yield_controller: Address, lending_pool_id: Address, protocol_token_id: Address); 
}

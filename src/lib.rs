
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

mod storage;
pub use storage::*;

#[elrond_wasm_derive::contract(BUSDCoinImpl)]
pub trait BUSDCoin {

    // CONSTRUCTOR

    /// constructor function
    /// is called immediately after the contract is created
    fn init(&self) {
        // owner will be deploy caller
        let owner = self.get_caller();
        self.storage_store_bytes32(&OWNER_KEY.into(), &owner.as_fixed_bytes());
        
        // owner is also the initial supply controller
        self.storage_store_bytes32(&SUPPLY_CONTROLLER_KEY.into(), &owner.as_fixed_bytes());
    
        // the contract starts paused
        self.storage_store_i64(&PAUSED_KEY.into(), 1);
    }

    #[view]
    fn name() -> Vec<u8> {
        NAME.to_vec()
    }

    #[view]
    fn symbol() -> Vec<u8> {
        SYMBOL.to_vec()
    }

    #[view]
    fn decimals() -> usize {
        DECIMALS
    }

    // ERC20 LOGIC

    /// Total number of tokens in existence.
    #[view]
    fn totalSupply(&self) -> BigUint {
        let total_supply = self.storage_load_big_uint(&TOTAL_SUPPLY_KEY.into());
        total_supply
    }

    #[private]
    fn _save_total_supply(&self, new_total_supply: &BigUint) {
        self.storage_store_big_uint(&TOTAL_SUPPLY_KEY.into(), new_total_supply);
    }

    #[private]
    fn _perform_transfer(&self, sender: Address, recipient: Address, amount: BigUint) -> Result<(), &str> {
        if self.isPaused() {
            return Err("paused");
        }

        if self.isFrozen(&sender) || self.isFrozen(&recipient) {
            return Err("address frozen");
        }
        
        // load sender balance
        let sender_balance_key = self._balance_key(&sender);
        let mut sender_balance = self.storage_load_big_uint(&sender_balance_key);
    
        // check if enough funds
        if &amount > &sender_balance {
            return Err("insufficient funds");
        }
    
        // decrease & save sender balance
        sender_balance -= &amount;
        self.storage_store_big_uint(&sender_balance_key, &sender_balance);
    
        // load, increase & save receiver balance
        let rec_balance_key = self._balance_key(&recipient);
        let mut rec_balance = self.storage_load_big_uint(&rec_balance_key);
        rec_balance += &amount;
        self.storage_store_big_uint(&rec_balance_key, &rec_balance);
    
        // log operation
        self.transfer_event(&sender, &recipient, &amount);

        Ok(())
    }

    /// Transfer token to a specified address from sender.
    /// 
    /// Arguments:
    /// 
    /// * `to` The address to transfer to.
    /// 
    fn transfer(&self, to: Address, amount: BigUint) -> Result<(), &str> {
        // sender is the caller
        let sender = self.get_caller();

        self._perform_transfer(sender, to, amount)
    }

    /// Gets the balance of the specified address.
    /// 
    /// Arguments:
    /// 
    /// * `address` The address to query the the balance of
    /// 
    #[view]
    fn balanceOf(&self, address: Address) -> BigUint {
        // load balance
        let balance_key = self._balance_key(&address);
        let balance = self.storage_load_big_uint(&balance_key);

        // return balance as big int
        balance
    }

    // ERC20 FUNCTIONALITY
 
    /// Use allowance to transfer funds between two accounts.
    /// 
    /// Arguments:
    /// 
    /// * `sender` The address to transfer from.
    /// * `recipient` The address to transfer to.
    /// * `amount` the amount of tokens to be transferred.
    /// 
    fn transferFrom(&self, sender: Address, recipient: Address, amount: BigUint) -> Result<(), &str> {
        if self.isPaused() {
            return Err("paused");
        }
        
        // get caller
        let caller = self.get_caller();

        // load allowance
        let allowance_key = self._allowance_key(&sender, &caller);
        let mut allowance = self.storage_load_big_uint(&allowance_key);

        // amount should not exceed allowance
        if &amount > &allowance {
            return Err("allowance exceeded");
        }

        // update allowance
        allowance -= &amount;
        self.storage_store_big_uint(&allowance_key, &allowance);

        self._perform_transfer(sender, recipient, amount)
    }

    /// Approve the given address to spend the specified amount of tokens on behalf of the sender.
    /// It overwrites any previously existing allowance from sender to beneficiary.
    /// 
    /// Arguments:
    /// 
    /// * `spender` The address that will spend the funds.
    /// * `amount` The amount of tokens to be spent.
    /// 
    fn approve(&self, spender: Address, amount: BigUint) -> Result<(), &str> {
        if self.isPaused() {
            return Err("paused");
        }

        // sender is the caller
        let caller = self.get_caller();

        if self.isFrozen(&caller) || self.isFrozen(&spender) {
            return Err("address frozen");
        }

        // store allowance
        let allowance_key = self._allowance_key(&caller, &spender);
        self.storage_store_big_uint(&allowance_key, &amount);
      
        // log operation
        self.approve_event(&caller, &spender, &amount);
        Ok(())
    }

    /// Function to check the amount of tokens that an owner allowed to a spender.
    /// 
    /// Arguments:
    /// 
    /// * `owner` The address that owns the funds.
    /// * `spender` The address that will spend the funds.
    /// 
    #[view]
    fn allowance(&self, owner: Address, spender: Address) -> BigUint {
        // get allowance
        let allowance_key = self._allowance_key(&owner, &spender);
        let res = self.storage_load_big_uint(&allowance_key);

        // return allowance as big int
        res
    }

    // OWNER FUNCTIONALITY

    /// Yields the current contract owner.
    #[view]
    fn getContractOwner(&self) -> Address {
        self.storage_load_bytes32(&OWNER_KEY.into()).into()
    }

    /// Yields the currently proposed new owner, if any.
    #[view]
    fn getProposedOwner(&self) -> Option<Address> {
        if self.storage_load_len(&PROPOSED_OWNER_KEY.into()) == 0 {
            None
        } else {
            let proposed_owner_bytes = self.storage_load_bytes32(
                &PROPOSED_OWNER_KEY.into());
            Some(proposed_owner_bytes.into())
        }
    }

    /// Allows the current owner to begin transferring control of the contract to a proposedOwner
    /// 
    /// Arguments:
    /// 
    /// * `proposed_owner` The address to transfer ownership to.
    /// 
    fn proposeOwner(&self, proposed_owner: Address) -> Result<(), &str> {
        let caller = self.get_caller();
        if caller != self.getContractOwner() {
            return Err("only owner can propose another owner");
        }
        if caller == proposed_owner {
            return Err("current owner cannot propose itself");
        }
        if let Some(previous_proposed_owner) = self.getProposedOwner() {
            if proposed_owner == previous_proposed_owner {
                return Err("caller already is proposed owner"); 
            }
        }
        self.storage_store_bytes32(
            &PROPOSED_OWNER_KEY.into(), 
            &proposed_owner.as_fixed_bytes());

        self.ownership_transfer_proposed_event(&caller, &proposed_owner, ());
        Ok(())
    }

    /// Allows the current owner or proposed owner to cancel transferring control of the contract to the proposed owner.
    fn disregardProposedOwner() -> Result<(), &str> {
        match self.getProposedOwner() {
            None => Err("can only disregard a proposed owner that was previously set"),
            Some(proposed_owner) => {
                let caller = self.get_caller();
                if caller != self.getContractOwner() && caller != proposed_owner {
                    return Err("only proposedOwner or owner can disregard proposed owner"); 
                }
                self.storage_store(
                    &PROPOSED_OWNER_KEY.into(),
                    &[]);

                self.ownership_transfer_disregarded_event(&proposed_owner, ());
                Ok(())
            }
        }
    }

    /// Allows the proposed owner to complete transferring control of the contract to herself..
    fn claimOwnership() -> Result<(), &str> {
        match self.getProposedOwner() {
            None => Err("no owner proposed"),
            Some(proposed_owner) => {
                let caller = self.get_caller();
                if caller != proposed_owner {
                    return Err("only proposed owner can claim ownership")
                }
                
                let old_owner = self.getContractOwner();

                // save new owner
                self.storage_store_bytes32(
                    &OWNER_KEY.into(),
                    &proposed_owner.as_fixed_bytes());
                // clear proposed owner
                self.storage_store(
                    &PROPOSED_OWNER_KEY.into(),
                    &[]);

                self.ownership_transferred_event(&old_owner, &proposed_owner, ());
                Ok(())  
            }
        }
    }

    /// Reclaim all BUSD at the contract address.
    /// This sends all the BUSD tokens that the address of the contract itself is holding to the owner.
    /// Note: this is not affected by freeze constraints.
    fn reclaimBUSD() -> Result<(), &str> {
        let caller = self.get_caller();
        if caller != self.getContractOwner() {
            return Err("only owner can reclaim"); 
        }

        // load contract own balance
        let contract_address = self.get_own_address();
        let contract_balance_key = self._balance_key(&contract_address);
        let contract_balance = self.storage_load_big_uint(&contract_balance_key);

        // clear contract own balance
        self.storage_store(&contract_balance_key, &[]);

        // increment owner balance
        let owner_balance_key = self._balance_key(&caller);
        let mut owner_balance = self.storage_load_big_uint(&owner_balance_key);
        owner_balance += &contract_balance;
        self.storage_store_big_uint(&owner_balance_key, &owner_balance);
    
        // log operation
        self.transfer_event(&contract_address, &caller, &contract_balance);

        Ok(())
    }

    // PAUSABILITY FUNCTIONALITY

    #[view]
    fn isPaused(&self) -> bool {
        self.storage_load_len(&PAUSED_KEY.into()) > 0
    }

    /// Called by the owner to pause, triggers stopped state
    fn pause(&self) -> Result<(), &str> {
        if self.isPaused() {
            return Err("already paused")
        }
        self.storage_store_i64(&PAUSED_KEY.into(), 1);

        self.pause_event(());
        Ok(())
    }

    /// Called by the owner to unpause, returns to normal state
    fn unpause(&self) -> Result<(), &str> {
        if !self.isPaused() {
            return Err("already unpaused")
        }
        self.storage_store_i64(&PAUSED_KEY.into(), 0);

        self.unpause_event(());
        Ok(())
    }

    // ASSET PROTECTION FUNCTIONALITY

    /// Yields the currently proposed new owner, if any.
    #[view]
    fn getAssetProtectionRole(&self) -> Option<Address> {
        if self.storage_load_len(&ASSET_PROTECTION_ROLE_KEY.into()) == 0 {
            None
        } else {
            Some(self.storage_load_bytes32(&ASSET_PROTECTION_ROLE_KEY.into()).into())
        }
    }

    #[private]
    fn _caller_is_asset_protection_role(&self) -> bool {
        if let Some(asset_prot_role) = self.getAssetProtectionRole() {
            if self.get_caller() == asset_prot_role {
                return true;
            }
        }
        false
    }

    /// Sets a new asset Protection role address.
    /// 
    /// Arguments:
    /// 
    /// * `new_asset_prot_role` The new address allowed to freeze/unfreeze addresses and seize their tokens.
    /// 
    fn setAssetProtectionRole(&self, new_asset_prot_role: &Address) -> Result<(), &str> {
        let caller = self.get_caller();
        if caller != self.getContractOwner() && 
           !self._caller_is_asset_protection_role() {
            return Err("only asset protection role or owner can change asset protection role")
        }

        // needed for logging
        let old_asset_protection_role = self
            .getAssetProtectionRole()
            .unwrap_or_else(|| Address::from([0u8; 32]));

        // change asset protection role
        self.storage_store_bytes32(
            &ASSET_PROTECTION_ROLE_KEY.into(),
            &new_asset_prot_role.as_fixed_bytes());

        // log event
        self.asset_protection_role_set_event(
            &old_asset_protection_role,
            new_asset_prot_role,
            ()
        );

        Ok(())
    }

    /// Freezes an address balance, preventing any transfers involving it.
    /// 
    /// Arguments:
    /// 
    /// * `address` The address to freeze.
    /// 
    fn freeze(&self, address: &Address) -> Result<(), &str> {
        if !self._caller_is_asset_protection_role() {
            return Err("only asset protection role can freeze");
        }
        let frozen_key = self._frozen_key(&address);
        if self.storage_load_len(&frozen_key) > 0 {
            return Err("address already frozen");
        }
        self.storage_store_i64(&frozen_key, 1);

        self.address_frozen_event(&address, ());
        Ok(())
    }

    /// Unfreezes an address balance, allowing transfers involving it.
    /// 
    /// Arguments:
    /// 
    /// * `address` The address to unfreeze.
    /// 
    fn unfreeze(&self, address: &Address) -> Result<(), &str> {
        if !self._caller_is_asset_protection_role() {
            return Err("only asset protection role can unfreeze");
        }
        let frozen_key = self._frozen_key(&address);
        if self.storage_load_len(&frozen_key) == 0 {
            return Err("address already unfrozen");
        }
        self.storage_store_i64(&frozen_key, 0);

        self.address_frozen_event(&address, ());
        Ok(())
    }

    /// Wipes the balance of a frozen address, burning the tokens
    /// and setting the approval to zero.
    /// 
    /// Arguments:
    /// 
    /// * `address` The address to wipe.
    /// 
    fn wipeFrozenAddress(&self, address: &Address) -> Result<(), &str> {
        if !self._caller_is_asset_protection_role() {
            return Err("only asset protection role can wipe");
        }
        if !self.isFrozen(&address) {
            return Err("address is not frozen");
        }

        // erase balance
        let balance_key = self._balance_key(&address);
        let wiped_balance = self.storage_load_big_uint(&balance_key);
        self.storage_store_i64(&balance_key, 0); // clear balance
        
        // decrease total supply
        let mut total_supply = self.totalSupply();
        total_supply -= &wiped_balance;
        self._save_total_supply(&total_supply);

        // log operation
        self.frozen_address_wiped_event(&address, ());
        self.supply_decreased_event(&address, &wiped_balance);
        self.transfer_event(&address,  &[0u8; 32].into(), &wiped_balance);

        Ok(())
    }

    /// Gets whether the address is currently frozen.
    /// 
    /// Arguments:
    /// 
    /// * `address` The address to check if frozen.
    /// 
    #[view]
    fn isFrozen(&self, address: &Address) -> bool {
        let frozen_key = self._frozen_key(&address);
        self.storage_load_len(&frozen_key) > 0
    }

    // SUPPLY CONTROL FUNCTIONALITY

    /// Yields the currently proposed new owner, if any.
    #[view]
    fn getSupplyController(&self) -> Address {
        self.storage_load_bytes32(&SUPPLY_CONTROLLER_KEY.into()).into()
    }

    #[private]
    fn _caller_is_supply_controller(&self) -> bool {
        return self.get_caller() == self.getSupplyController()
    }

    /// Sets a new supply controller address.
    /// 
    /// Arguments:
    /// 
    /// * `new_supply_controller` The address allowed to burn/mint tokens to control supply.
    /// 
    fn setSupplyController(&self, new_supply_controller: &Address) -> Result<(), &str> {
        let caller = self.get_caller();
        if caller != self.getContractOwner() && 
           !self._caller_is_supply_controller() {
            return Err("only supply controller or owner can change supply controller")
        }

        // needed for logging
        let old_supply_controller = self.getSupplyController();

        // change supply controller
        self.storage_store_bytes32(
            &SUPPLY_CONTROLLER_KEY.into(),
            &new_supply_controller.as_fixed_bytes());

        // log event
        self.supply_controller_set_event(
            &old_supply_controller,
            new_supply_controller,
            ()
        );

        Ok(())
    }

    /// Increases the total supply by minting the specified number of tokens to the supply controller account.
    /// 
    /// Arguments:
    /// 
    /// * `amount` The number of tokens to add.
    /// 
    fn increaseSupply(&self, amount: BigUint) -> Result<(), &str> {
        if !self._caller_is_supply_controller() {
            return Err("only supply controller can increase supply");
        }
        let supply_controller = self.get_caller();

        // increase supply controller balance
        let balance_key = self._balance_key(&supply_controller);
        let mut supply_contr_balance = self.storage_load_big_uint(&balance_key);
        supply_contr_balance += &amount;
        self.storage_store_big_uint(&balance_key, &supply_contr_balance);

        // increase total supply
        let mut total_supply = self.totalSupply();
        total_supply += &amount;
        self._save_total_supply(&total_supply);

        // log operation
        self.supply_increased_event(&supply_controller, &amount);
        self.transfer_event(&[0u8; 32].into(), &supply_controller, &amount);

        Ok(())
    }

    /// Decreases the total supply by burning the specified number of tokens from the supply controller account.
    /// 
    /// Arguments:
    /// 
    /// * `amount` The number of tokens to remove.
    /// 
    fn decreaseSupply(&self, amount: BigUint) -> Result<(), &str> {
        if !self._caller_is_supply_controller() {
            return Err("only supply controller can decrease supply");
        }
        let supply_controller = self.get_caller();

        // get supply controller balance
        let balance_key = self._balance_key(&supply_controller);
        let mut supply_contr_balance = self.storage_load_big_uint(&balance_key);

        // check
        if amount > supply_contr_balance {
            return Err("not enough supply to decrease")
        }

        // decrease supply controller balance
        supply_contr_balance -= &amount;
        self.storage_store_big_uint(&balance_key, &supply_contr_balance);

        // decrease total supply
        let mut total_supply = self.totalSupply();
        total_supply -= &amount;
        self._save_total_supply(&total_supply);

        // log operation
        self.supply_decreased_event(&supply_controller, &amount);
        self.transfer_event(&supply_controller, &[0u8; 32].into(), &amount);

        Ok(())
    }

    // STORAGE KEYS

    /// generates the balance key that maps balances with their owners
    #[private]
    fn _balance_key(&self, address: &Address) -> StorageKey {
        // this compresses the raw key down to 32 bytes
        self.keccak256(&balance_key_raw(address)).into()
    }

    /// generates the allowance key that maps allowances with the respective sender-receiver pairs
    #[private]
    fn _allowance_key(&self, from: &Address, to: &Address) -> StorageKey {
        // this compresses the raw key down to 32 bytes
        self.keccak256(&allowance_key_raw(from, to)).into()
    }

    /// generates the frozen key in the map that indicates whether an account is frozen 
    #[private]
    fn _frozen_key(&self, address: &Address) -> StorageKey {
        // this compresses the raw key down to 32 bytes
        self.keccak256(&frozen_key_raw(address)).into()
    }


    // ERC20 BASIC EVENTS
    
    #[event("0x0000000000000000000000000000000000000000000000000000000000000001")]
    fn transfer_event(&self,
        sender: &Address,
        recipient: &Address,
        amount: &BigUint);

    // ERC20 EVENTS

    #[event("0x0000000000000000000000000000000000000000000000000000000000000002")]
    fn approve_event(&self,
        sender: &Address,
        recipient: &Address,
        amount: &BigUint);

    // OWNABLE EVENTS

    #[event("0x0000000000000000000000000000000000000000000000000000000000000003")]
    fn ownership_transfer_proposed_event(&self, 
        current_owner: &Address,
        proposed_owner: &Address,
        _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000004")]
    fn ownership_transfer_disregarded_event(&self, 
        old_proposed_owner: &Address,
        _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000005")]
    fn ownership_transferred_event(&self, 
        old_owner: &Address,
        new_owner: &Address,
        _data: ());
    
    // PAUSABLE EVENTS

    #[event("0x0000000000000000000000000000000000000000000000000000000000000006")]
    fn pause_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000007")]
    fn unpause_event(&self, _data: ());

    // ASSET PROTECTION EVENTS

    #[event("0x0000000000000000000000000000000000000000000000000000000000000008")]
    fn address_frozen_event(&self, address: &Address, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000009")]
    fn address_unfrozen_event(&self, address: &Address, _data: ());

    #[event("0x000000000000000000000000000000000000000000000000000000000000000a")]
    fn frozen_address_wiped_event(&self, address: &Address, _data: ());

    #[event("0x000000000000000000000000000000000000000000000000000000000000000b")]
    fn asset_protection_role_set_event(&self, 
        old_asset_protection_role: &Address,
        new_asset_protection_role: &Address,
        _data: ());

    // SUPPLY CONTROL EVENTS

    #[event("0x000000000000000000000000000000000000000000000000000000000000000c")]
    fn supply_increased_event(&self, to: &Address, amount: &BigUint);

    #[event("0x000000000000000000000000000000000000000000000000000000000000000d")]
    fn supply_decreased_event(&self, from: &Address, amount: &BigUint);

    #[event("0x000000000000000000000000000000000000000000000000000000000000000e")]
    fn supply_controller_set_event(&self, 
        old_supply_controller: &Address,
        new_supply_controller: &Address,
        _data: ());
    
    // DELEGATED TRANSFER EVENTS

    // TODO
}


#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

static TOTAL_SUPPLY_KEY: [u8; 32] = [0u8; 32];

#[elrond_wasm_derive::contract(SimpleCoinImpl)]
pub trait SimpleCoin {
    /// constructor function
    /// is called immediately after the contract is created
    /// will set the fixed global token supply and give all the supply to the creator
    fn init(&self, total_supply: &BigUint) {
        let sender = self.get_caller();

        // save total supply
        self.storage_store_big_uint(&TOTAL_SUPPLY_KEY.into(), &total_supply);

        // sender balance <- total supply
        let balance_key = self._balance_key(&sender);
        self.storage_store_big_uint(&balance_key, &total_supply);
    }

    /// getter function: retrieves total token supply
    fn totalSupply(&self) -> BigUint {
        let total_supply = self.storage_load_big_uint(&TOTAL_SUPPLY_KEY.into());
        total_supply
    }

    /// generates the balance key that maps balances with their owners
    #[private]
    fn _balance_key(&self, address: &Address) -> StorageKey {
        let mut raw_key: Vec<u8> = Vec::with_capacity(33);
        raw_key.push(1u8); // "1" is for balance keys
        raw_key.extend_from_slice(address.as_fixed_bytes()); // append the entire address
        let key = self.keccak256(&raw_key); // this compresses the key down to 32 bytes
        key.into()
    }

    /// generates the allowance key that maps allowances with the respective sender-receiver pairs
    #[private]
    fn _allowance_key(&self, from: &Address, to: &Address) -> StorageKey {
        let mut raw_key: Vec<u8> = Vec::with_capacity(65);
        raw_key.push(2u8); // "2" is for balance keys
        raw_key.extend_from_slice(from.as_fixed_bytes()); // append the entire "from" address
        raw_key.extend_from_slice(to.as_fixed_bytes()); // append the entire "to" address
        let key = self.keccak256(&raw_key); // this compresses the key down to 32 bytes
        key.into()
    }

    /// getter function: retrieves balance for an account
    fn balanceOf(&self, subject: Address) -> BigUint {
        // load balance
        let balance_key = self._balance_key(&subject);
        let balance = self.storage_load_big_uint(&balance_key);

        // return balance as big int
        balance
    }

    /// getter function: retrieves allowance granted from one account to another
    fn allowance(&self, sender: Address, recipient: Address) -> BigUint {
        // get allowance
        let allowance_key = self._allowance_key(&sender, &recipient);
        let res = self.storage_load_big_uint(&allowance_key);

        // return allowance as big int
        res
    }

    #[private]
    fn _perform_transfer(&self, sender: Address, recipient: Address, amount: BigUint) -> Result<(), &str> {
        // load sender balance
        let sender_balance_key = self._balance_key(&sender);
        let mut sender_balance = self.storage_load_big_uint(&sender_balance_key);
    
        // check if enough funds
        if &amount > &sender_balance {
            return Err("insufficient funds");
        }
    
        // update sender balance
        sender_balance -= &amount;
        self.storage_store_big_uint(&sender_balance_key, &sender_balance);
    
        // load & update receiver balance
        let rec_balance_key = self._balance_key(&recipient);
        let mut rec_balance = self.storage_load_big_uint(&rec_balance_key);
        rec_balance += &amount;
        self.storage_store_big_uint(&rec_balance_key, &rec_balance);
    
        // log operation
        self.transfer_event(&sender, &recipient, &amount);

        Ok(())
    }

    /// transfers tokens from sender to another account
    fn transferToken(&self, recipient: Address, amount: BigUint) -> Result<(), &str> {
        // sender is the caller
        let sender = self.get_caller();

        self._perform_transfer(sender, recipient, amount)
    }

    /// sender allows beneficiary to use given amount of tokens from sender's balance
    /// it will completely overwrite any previously existing allowance from sender to beneficiary
    fn approve(&self, recipient: Address, amount: BigUint) -> Result<(), &str> {
        // sender is the caller
        let sender = self.get_caller();
      
        // store allowance
        let allowance_key = self._allowance_key(&sender, &recipient);
        self.storage_store_big_uint(&allowance_key, &amount);
      
        // log operation
        self.approve_event(&sender, &recipient, &amount);
        Ok(())
    }
 
    /// caller uses allowance to transfer funds between 2 other accounts
    fn transferFrom(&self, sender: Address, recipient: Address, amount: BigUint) -> Result<(), &str> {
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

    #[event("0x7134692b230b9e1ffa39098904722134159652b09c5bc41d88d6698779d228ff")]
    fn approve_event(&self, sender: &Address, recipient: &Address, amount: &BigUint);

    #[event("0xf099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9")]
    fn transfer_event(&self, sender: &Address, recipient: &Address, amount: &BigUint);
}

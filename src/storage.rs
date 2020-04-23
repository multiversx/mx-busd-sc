use elrond_wasm::*;

// ERC20 BASIC DATA
pub static TOTAL_SUPPLY_KEY:              [u8; 32] = [0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static BALANCE_KEY_PREFIX:                u8       =  0x01;
pub const NAME:                           &[u8]    = b"Binance USD";
pub const SYMBOL:                         &[u8]    = b"BUSD";
pub const DECIMALS:                       usize    = 18;

// ERC20 DATA
static ALLOWANCE_KEY_PREFIX:              u8       =  0x02;

// OWNER DATA
pub static OWNER_KEY:                     [u8; 32] = [0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
pub static PROPOSED_OWNER_KEY:            [u8; 32] = [0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];

// PAUSABILITY DATA
pub static PAUSED_KEY:                    [u8; 32] = [0x05, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];

// ASSET PROTECTION DATA
pub static ASSET_PROTECTION_ROLE_KEY:     [u8; 32] = [0x06, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static FROZEN_KEY_PREFIX:                 u8       =  0x07;

// SUPPLY CONTROL DATA
pub static SUPPLY_CONTROLLER_KEY:         [u8; 32] = [0x08, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];

// DELEGATED TRANSFER DATA
pub static BETA_DELEGATE_WHITELISTER_KEY: [u8; 32] = [0x09, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static BETA_DELEGATE_WHITELIST_PREFIX:    u8       =  0x0a;
static NEXT_SEQS_PREFIX:                  u8       =  0x0b;


// RAW STORAGE KEYS

pub fn balance_key_raw(address: &Address) -> Vec<u8> {
    let mut raw_key: Vec<u8> = Vec::with_capacity(33);
    raw_key.push(BALANCE_KEY_PREFIX);
    raw_key.extend_from_slice(address.as_fixed_bytes()); // append the entire address
    raw_key
}

pub fn allowance_key_raw(from: &Address, to: &Address) -> Vec<u8> {
    let mut raw_key: Vec<u8> = Vec::with_capacity(65);
    raw_key.push(ALLOWANCE_KEY_PREFIX);
    raw_key.extend_from_slice(from.as_fixed_bytes()); // append the entire "from" address
    raw_key.extend_from_slice(to.as_fixed_bytes()); // append the entire "to" address
    raw_key
}

pub fn frozen_key_raw(address: &Address) -> Vec<u8> {
    let mut raw_key: Vec<u8> = Vec::with_capacity(33);
    raw_key.push(FROZEN_KEY_PREFIX);
    raw_key.extend_from_slice(address.as_fixed_bytes()); // append the entire address
    raw_key
}

pub fn beta_delegate_whitelist_key_raw(address: &Address) -> Vec<u8> {
    let mut raw_key: Vec<u8> = Vec::with_capacity(33);
    raw_key.push(BETA_DELEGATE_WHITELIST_PREFIX);
    raw_key.extend_from_slice(address.as_fixed_bytes()); // append the entire address
    raw_key
}

pub fn next_seqs_key_raw(address: &Address) -> Vec<u8> {
    let mut raw_key: Vec<u8> = Vec::with_capacity(33);
    raw_key.push(NEXT_SEQS_PREFIX);
    raw_key.extend_from_slice(address.as_fixed_bytes()); // append the entire address
    raw_key
}

#![feature(prelude_import)]
#![no_std]
#![feature(alloc)]

#[macro_use]
extern crate sgx_tstd as std;

extern crate libcontract_common;
#[macro_use]
extern crate libcontract_trusted;

#[macro_use]
extern crate ethtoken_api;

extern crate protobuf;

extern crate sha3;
extern crate bigint;
extern crate hexutil;
extern crate sputnikvm;
extern crate alloc;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

use std::collections::HashMap;

use ethtoken_api::{InitStateRequest, InitStateResponse, EthState, AccountState, CreateTokenRequest,
                  CreateTokenResponse, TransferTokenRequest, TransferTokenResponse,
                  GetBalanceRequest, GetBalanceResponse};

use libcontract_common::ContractError;

use sha3::{Digest, Keccak256};
use bigint::{Gas, Address, U256, M256, H256, Sign};
use hexutil::{read_hex, to_hex};
use sputnikvm::{HeaderParams, SeqTransactionVM, ValidTransaction, VM,
                AccountCommitment, AccountChange, RequireError, TransactionAction,
                MainnetEIP160Patch, Storage};

use core::str::FromStr;
use std::rc::Rc;

create_enclave_api!();

// Contract API methods

fn create_token(
    state: &EthState,
    request: &CreateTokenRequest
) -> Result<(EthState, CreateTokenResponse), ContractError> {

    println!("create_token creator={}", request.get_creator_address());

    let creator_addr = Address::from_str(request.get_creator_address()).unwrap();

    // EVM bytecode for ERC20 token contract (from https://ethereum.org/token) with the following parameters:
    //
    // decimals: 0
    // initialSupply: <filled from request>
    // tokenName: "Test"
    // tokenSymbol: "TST"
    //
    let mut bytecode: Vec<u8> = read_hex("0x60606040526000600260006101000a81548160ff021916908360ff16021790555034156200002c57600080fd5b604051620012263803806200122683398101604052808051906020019091908051820191906020018051820191905050600260009054906101000a900460ff1660ff16600a0a8302600381905550600354600460003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508160009080519060200190620000d8929190620000fb565b508060019080519060200190620000f1929190620000fb565b50505050620001aa565b828054600181600116156101000203166002900490600052602060002090601f016020900481019282601f106200013e57805160ff19168380011785556200016f565b828001600101855582156200016f579182015b828111156200016e57825182559160200191906001019062000151565b5b5090506200017e919062000182565b5090565b620001a791905b80821115620001a357600081600090555060010162000189565b5090565b90565b61106c80620001ba6000396000f3006060604052600436106100ba576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff16806306fdde03146100bf578063095ea7b31461014d57806318160ddd146101a757806323b872dd146101d0578063313ce5671461024957806342966c681461027857806370a08231146102b357806379cc67901461030057806395d89b411461035a578063a9059cbb146103e8578063cae9ca511461042a578063dd62ed3e146104c7575b600080fd5b34156100ca57600080fd5b6100d2610533565b6040518080602001828103825283818151815260200191508051906020019080838360005b838110156101125780820151818401526020810190506100f7565b50505050905090810190601f16801561013f5780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b341561015857600080fd5b61018d600480803573ffffffffffffffffffffffffffffffffffffffff169060200190919080359060200190919050506105d1565b604051808215151515815260200191505060405180910390f35b34156101b257600080fd5b6101ba61065e565b6040518082815260200191505060405180910390f35b34156101db57600080fd5b61022f600480803573ffffffffffffffffffffffffffffffffffffffff1690602001909190803573ffffffffffffffffffffffffffffffffffffffff16906020019091908035906020019091905050610664565b604051808215151515815260200191505060405180910390f35b341561025457600080fd5b61025c610791565b604051808260ff1660ff16815260200191505060405180910390f35b341561028357600080fd5b61029960048080359060200190919050506107a4565b604051808215151515815260200191505060405180910390f35b34156102be57600080fd5b6102ea600480803573ffffffffffffffffffffffffffffffffffffffff169060200190919050506108a8565b6040518082815260200191505060405180910390f35b341561030b57600080fd5b610340600480803573ffffffffffffffffffffffffffffffffffffffff169060200190919080359060200190919050506108c0565b604051808215151515815260200191505060405180910390f35b341561036557600080fd5b61036d610ada565b6040518080602001828103825283818151815260200191508051906020019080838360005b838110156103ad578082015181840152602081019050610392565b50505050905090810190601f1680156103da5780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b34156103f357600080fd5b610428600480803573ffffffffffffffffffffffffffffffffffffffff16906020019091908035906020019091905050610b78565b005b341561043557600080fd5b6104ad600480803573ffffffffffffffffffffffffffffffffffffffff1690602001909190803590602001909190803590602001908201803590602001908080601f01602080910402602001604051908101604052809392919081815260200183838082843782019150505050505091905050610b87565b604051808215151515815260200191505060405180910390f35b34156104d257600080fd5b61051d600480803573ffffffffffffffffffffffffffffffffffffffff1690602001909190803573ffffffffffffffffffffffffffffffffffffffff16906020019091905050610d05565b6040518082815260200191505060405180910390f35b60008054600181600116156101000203166002900480601f0160208091040260200160405190810160405280929190818152602001828054600181600116156101000203166002900480156105c95780601f1061059e576101008083540402835291602001916105c9565b820191906000526020600020905b8154815290600101906020018083116105ac57829003601f168201915b505050505081565b600081600560003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055506001905092915050565b60035481565b6000600560008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205482111515156106f157600080fd5b81600560008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008282540392505081905550610786848484610d2a565b600190509392505050565b600260009054906101000a900460ff1681565b600081600460003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054101515156107f457600080fd5b81600460003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008282540392505081905550816003600082825403925050819055503373ffffffffffffffffffffffffffffffffffffffff167fcc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5836040518082815260200191505060405180910390a260019050919050565b60046020528060005260406000206000915090505481565b600081600460008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020541015151561091057600080fd5b600560008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054821115151561099b57600080fd5b81600460008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000828254039250508190555081600560008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008282540392505081905550816003600082825403925050819055508273ffffffffffffffffffffffffffffffffffffffff167fcc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5836040518082815260200191505060405180910390a26001905092915050565b60018054600181600116156101000203166002900480601f016020809104026020016040519081016040528092919081815260200182805460018160011615610100020316600290048015610b705780601f10610b4557610100808354040283529160200191610b70565b820191906000526020600020905b815481529060010190602001808311610b5357829003601f168201915b505050505081565b610b83338383610d2a565b5050565b600080849050610b9785856105d1565b15610cfc578073ffffffffffffffffffffffffffffffffffffffff16638f4ffcb1338630876040518563ffffffff167c0100000000000000000000000000000000000000000000000000000000028152600401808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020018481526020018373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200180602001828103825283818151815260200191508051906020019080838360005b83811015610c91578082015181840152602081019050610c76565b50505050905090810190601f168015610cbe5780820380516001836020036101000a031916815260200191505b5095505050505050600060405180830381600087803b1515610cdf57600080fd5b6102c65a03f11515610cf057600080fd5b50505060019150610cfd565b5b509392505050565b6005602052816000526040600020602052806000526040600020600091509150505481565b6000808373ffffffffffffffffffffffffffffffffffffffff1614151515610d5157600080fd5b81600460008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205410151515610d9f57600080fd5b600460008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205482600460008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205401111515610e2d57600080fd5b600460008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054600460008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205401905081600460008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000828254039250508190555081600460008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040518082815260200191505060405180910390a380600460008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054600460008773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020540114151561103a57fe5b505050505600a165627a7a72305820dc6fef21893d6cdf9152b43f2a9936b1c504347c605819e960cb9f343d2166db0029").unwrap();
    bytecode.extend_from_slice(&H256::from(request.get_initial_supply()));
    bytecode.extend_from_slice(&read_hex("0x000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000004546573740000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000035453540000000000000000000000000000000000000000000000000000000000").unwrap());

    let transactions = [
        ValidTransaction {
            caller: Some(creator_addr),
            action: TransactionAction::Create,
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::zero(),
            input: Rc::new(bytecode),
            nonce: U256::zero(),
        }
    ];

    let (new_state, _) = fire_transactions_and_update_state(&transactions, state);

    // Compute address of new token contract. In practice, a web3 client handling a "create" action
    // returns a transaction hash, and the caller needs to wait until the next block is mined to
    // retrieve the contract's address. For simplicity, we manually compute the address and return
    // it immediately. The address is a function of the caller and nonce (see https://ethereum.stackexchange.com/questions/760/how-is-the-address-of-an-ethereum-contract-computed)
    //
    let token_contract_addr = {
        let mut vec = read_hex("0xd694").unwrap().to_vec();
        vec.extend_from_slice(&creator_addr);
        vec.extend_from_slice(&[0x80]);
        to_hex(&Keccak256::digest(&vec)[12..])
    };

    let mut response = CreateTokenResponse::new();
    response.set_contract_address(token_contract_addr.clone());

    Ok((new_state, response))
}

fn transfer_tokens(
    state: &EthState,
    request: &TransferTokenRequest
) -> Result<(EthState, TransferTokenResponse), ContractError> {

    println!("transfer_tokens amount={}, from={}, to={}", request.amount, request.from_address, request.to_address);

    let to_addr = Address::from_str(request.get_to_address()).unwrap();

    // Construct the EVM payload for this transaction.
    //
    // To call the contract's "transfer" method, we take the first 4 bytes from the Keccak256 hash
    // of the the function's signature, then append the parameters values (destination and amount),
    // encoded and padded according to the Ethereum ABI spec.
    //
    // For more information, see https://github.com/ethereum/wiki/wiki/Ethereum-Contract-ABI.
    //
    let mut payload = Keccak256::digest("transfer(address,uint256)".as_bytes()).as_slice()[..4].to_vec();
    payload.extend_from_slice(&H256::from(to_addr));
    payload.extend_from_slice(&H256::from(request.get_amount()));

    let caller = Address::from_str(request.get_from_address()).unwrap();
    let contract_addr = Address::from_str(request.get_contract_address()).unwrap();

    let transactions = [
        ValidTransaction {
            caller: Some(caller),
            action: TransactionAction::Call(contract_addr),
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::zero(),
            input: Rc::new(payload),
            nonce: U256::zero(),
        }
    ];

    let (new_state, _) = fire_transactions_and_update_state(&transactions, state);
    let response = TransferTokenResponse::new();

    Ok((new_state, response))
}

fn get_balance(
    state: &EthState,
    request: &GetBalanceRequest
) -> Result<GetBalanceResponse, ContractError> {

    println!("get_balance addr={}", request.get_address());

    let address = Address::from_str(request.get_address()).unwrap();
    let contract_addr = Address::from_str(request.get_contract_address()).unwrap();

    // Construct the EVM payload for this transaction. See comment in transfer_tokens() for explanation.
    let mut payload = Keccak256::digest("balanceOf(address)".as_bytes()).as_slice()[..4].to_vec();
    payload.extend_from_slice(&H256::from(address));

    let transactions = [
        ValidTransaction {
            caller: Some(Address::default()),
            action: TransactionAction::Call(contract_addr),
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::zero(),
            input: Rc::new(payload),
            nonce: U256::zero(),
        }
    ];

    let (_, result) = fire_transactions_and_update_state(&transactions, state);

    let mut response = GetBalanceResponse::new();
    let result_as_u64 = U256::from(result.as_slice()).as_u64();
    response.set_balance(result_as_u64);

    Ok(response)
}

fn init_genesis_state(_request: &InitStateRequest) -> Result<(EthState, InitStateResponse), ContractError>  {
    let state = EthState::new();
    let response = InitStateResponse::new();
    Ok((state, response))
}

// Internal methods. These methods handle the EVM and provide a bridge between Ethereum state
// and Ekiden state.

fn handle_fire(vm: &mut SeqTransactionVM<MainnetEIP160Patch>, state: &EthState) {
    loop {
        match vm.fire() {
            Ok(()) => break,
            Err(RequireError::Account(address)) => {
                let addr_str = address.hex();
                let commit = match state.accounts.get(&addr_str) {
                    Some(b) => {
                        let result = AccountCommitment::Full {
                            nonce: U256::from_dec_str(b.get_nonce()).unwrap(),
                            address: address,
                            balance: U256::from_dec_str(b.get_balance()).unwrap(),
                            code: Rc::new(read_hex(b.get_code()).unwrap())
                        };
                        result
                    },
                    None => {
                        AccountCommitment::Nonexist(address)
                    }
                };
                vm.commit_account(commit).unwrap();
            },
            Err(RequireError::AccountStorage(address, index)) => {
                let addr_str = address.hex();
                let index_str = format!("{}", index);

                let value = match state.accounts.get(&addr_str).unwrap().storage.get(&index_str) {
                    Some(b) => M256(U256::from_dec_str(b).unwrap()),
                    None => M256::zero()
                };

                vm.commit_account(AccountCommitment::Storage {
                    address: address,
                    index: index,
                    value: value,
                }).unwrap();
            },
            Err(RequireError::AccountCode(address)) => {
                vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
            },
            Err(RequireError::Blockhash(number)) => {
                vm.commit_blockhash(number, H256::default()).unwrap();
            },
        }
    }
}

fn create_account_state(
    nonce: U256,
    address: Address,
    balance: U256,
    storage: &Storage,
    code: &Rc<Vec<u8>>
) -> (String, AccountState) {
    let mut storage_map: HashMap<String, String> = HashMap::new();
    let vm_storage_as_map: alloc::BTreeMap<U256, M256> = storage.clone().into();
    for (key, val) in vm_storage_as_map.iter() {
        let val_as_u256: U256 = val.clone().into();
        storage_map.insert(format!("{}", key), format!("{}", val_as_u256));
    }

    let address_str = address.hex();
    let mut account_state = AccountState::new();

    account_state.set_nonce(format!("{}", nonce));
    account_state.set_address(address_str.clone());
    account_state.set_balance(format!("{}", balance));
    account_state.set_storage(storage_map);
    account_state.set_code(to_hex(code));

    (address_str, account_state)
}

fn update_account_balance(
    address_str: &String,
    amount: U256,
    sign: Sign,
    state: &EthState
) -> AccountState {
    match state.accounts.get(address_str) {
        Some(b) => { // Found account. Update balance.
            let mut updated_account = b.clone();
            let prev_balance: U256 = U256::from_str(b.get_balance()).unwrap();
            let new_balance = match sign {
                Sign::Plus => prev_balance + amount,
                Sign::Minus => prev_balance - amount,
                _ => panic!()
            };
            updated_account.set_balance(format!("{}", new_balance));
            updated_account
        },
        None => { // Account doesn't exist; create it.
            assert_eq!(sign, Sign::Plus, "Can't decrease balance of nonexistent account");
            let mut account_state = AccountState::new();
            account_state.set_nonce("0".to_string());
            account_state.set_address(address_str.clone());
            account_state.set_balance(format!("{}", amount));
            account_state
        }
    }
}

fn update_state_from_vm(vm: &SeqTransactionVM<MainnetEIP160Patch>, _state: &EthState) -> EthState {
    let mut state = _state.clone();

    for account in vm.accounts() {
        match account {
            &AccountChange::Create { nonce, address, balance, ref storage, ref code } => {
                let (addr_str, account_state) = create_account_state(nonce, address, balance, storage, code);
                state.accounts.insert(addr_str, account_state);
            }
            &AccountChange::Full { nonce, address, balance, ref changing_storage, ref code } => {
                let (addr_str, mut account_state) = create_account_state(nonce, address, balance, changing_storage, code);
                let prev_storage = &_state.accounts.get(&addr_str).unwrap().storage;

                // This type of change registers a *diff* of the storage, so place previous values
                // in the new map.
                for (key, value) in prev_storage.iter() {
                    if !account_state.storage.contains_key(key) {
                        account_state.mut_storage().insert(key.clone(), value.clone());
                    }
                }

                state.mut_accounts().insert(addr_str, account_state);
            }
            &AccountChange::IncreaseBalance(address, amount) => {
                let address_str = address.hex();
                let new_account = update_account_balance(&address_str, amount, Sign::Plus, &state);
                state.accounts.insert(address_str, new_account);
            }
            &AccountChange::DecreaseBalance(address, amount) => {
                let address_str = address.hex();
                let new_account = update_account_balance(&address_str, amount, Sign::Minus, &state);
                state.accounts.insert(address_str, new_account);
            }
            &AccountChange::Nonexist(address) => { panic!("Unexpected nonexistent address: {:?}", address) }
        }
    }

    state
}

fn fire_transactions_and_update_state(
    transactions: &[ValidTransaction],
    state: &EthState
) -> (EthState, Vec<u8>) {
    let block_header = HeaderParams {
        beneficiary: Address::default(),
        timestamp: 0,
        number: U256::zero(),
        difficulty: U256::zero(),
        gas_limit: Gas::zero(),
    };

    let mut last_vm: Option<SeqTransactionVM<MainnetEIP160Patch>> = None;
    for t in transactions.iter() {
        let mut vm = if last_vm.is_none() {
            SeqTransactionVM::new(t.clone(), block_header.clone())
        } else {
            SeqTransactionVM::with_previous(t.clone(), block_header.clone(), last_vm.as_ref().unwrap())
        };

        handle_fire(&mut vm, state);
        last_vm = Some(vm);
    }

    let vm_result = last_vm.as_ref().unwrap().out();

    let new_state = update_state_from_vm(&last_vm.as_ref().unwrap(), state);
    (new_state, vm_result.to_vec())
}

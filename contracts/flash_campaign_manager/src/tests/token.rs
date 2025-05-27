
use soroban_sdk::{Env, Address, token, IntoVal, Vec, TryIntoVal, testutils::{MockAuth, MockAuthInvoke}, xdr::{AlphaNum4, Asset, AssetCode4, ContractExecutable, ContractIdPreimage, CreateContractArgs, HostFunction}};

use crate::tests::utils::{address_to_account_id};

pub fn create_token<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let wasm = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &wasm.address()),
        token::StellarAssetClient::new(e, &wasm.address()),
    )
}

pub fn register_custom_stellar_asset_contract<'a>(
    e: &Env,
    admin: &Address,
    code: &str, // up to 4 chars
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    // Pad or truncate code to 4 bytes
    let mut code_bytes = [0u8; 4];
    let code_slice = code.as_bytes();
    for (i, b) in code_slice.iter().take(4).enumerate() {
        code_bytes[i] = *b;
    }
    let issuer = address_to_account_id(e, admin);
    let asset = Asset::CreditAlphanum4(AlphaNum4 {
        asset_code: AssetCode4(code_bytes),
        issuer,
    });
    let create = HostFunction::CreateContract(CreateContractArgs {
        contract_id_preimage: ContractIdPreimage::Asset(asset),
        executable: ContractExecutable::StellarAsset,
    });

    // Call the host to create the contract
    let contract_id: Address = e
        .host()
        .invoke_function(create)
        .unwrap()
        .try_into_val(e)
        .unwrap();

    // Set admin
    let prev_auth_manager = e.host().snapshot_auth_manager().unwrap();
    e.host()
        .switch_to_recording_auth_inherited_from_snapshot(&prev_auth_manager)
        .unwrap();
    let client = token::StellarAssetClient::new(e, &contract_id);
    client.set_admin(admin);
    e.host().set_auth_manager(prev_auth_manager).unwrap();

    (
        token::Client::new(e, &contract_id),
        token::StellarAssetClient::new(e, &contract_id),
    )
}

#[allow(dead_code)]
fn set_sac_admin_to_contract(
    e: &Env,
    sac_admin: &Address,
    sac_client: &token::StellarAssetClient,
    contract_addr: &Address,
) {
    // Build args using Soroban Vec
    let mut args = Vec::new(e);
    args.push_back(contract_addr.clone().into_val(e));
    std::println!(
    "[set_sac_admin_to_contract] contract_addr={:?}",
    contract_addr);
    let dummy_invoke = MockAuthInvoke {
        contract: &sac_client.address, // the SAC contract address
        fn_name: "set_admin",
        args,
        sub_invokes: &[],
    };
    e.mock_auths(&[MockAuth {
        address: sac_admin,
        invoke: &dummy_invoke,
    }]);

    sac_client.set_admin(contract_addr);

    std::println!(
        "SAC admin changed from {:?} to contract address {:?}",
        sac_admin, contract_addr
    );
}
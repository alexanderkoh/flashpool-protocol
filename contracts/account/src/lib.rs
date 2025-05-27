#![no_std]

use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contractimpl, contracttype,
    Address, BytesN, Env, Symbol, Vec, crypto::Hash,
    panic_with_error,
};
use smart_wallet_interface::types::{
    Error, Signature, Signatures, SignerKey, SignerVal, SignerStorage, Signer, SignerExpiration, SignerLimits, Secp256r1Signature
};

// --- Signer management and verification logic (adapted from smart-wallet) ---

fn get_signer_val_storage(
    env: &Env,
    signer_key: &SignerKey,
    _extend_ttl: bool,
) -> Option<(SignerVal, SignerStorage)> {
    match env.storage().temporary().get::<SignerKey, SignerVal>(signer_key) {
        Some(signer_val) => Some((signer_val, SignerStorage::Temporary)),
        None => env.storage().persistent().get::<SignerKey, SignerVal>(signer_key).map(|signer_val| (signer_val, SignerStorage::Persistent)),
    }
}

fn verify_signer_expiration(env: &Env, expiration: &SignerExpiration) {
    if let Some(ledger) = expiration.0 {
        let current_ledger = env.ledger().sequence();
        if current_ledger > ledger {
            panic_with_error!(env, Error::SignerExpired);
        }
    }
}

fn verify_context(
    env: &Env,
    context: &Context,
    signer_key: &SignerKey,
    signer_limits: &SignerLimits,
    signatures: &Signatures,
) -> bool {
    match &signer_limits.0 {
        None => true, // No limits
        Some(signer_limits) => {
            if signer_limits.is_empty() {
                return true;
            }
            match context {
                Context::Contract(ctx) => {
                    match signer_limits.get(ctx.contract.clone()) {
                        None => false,
                        Some(_signer_limits_keys) => {
                            // For now, just allow if present. Expand for multisig/policy as needed.
                            true
                        }
                    }
                }
                _ => false,
            }
        }
    }
}

fn verify_secp256r1_signature(
    env: &Env,
    signature_payload: &Hash<32>,
    public_key: &BytesN<65>,
    signature: Secp256r1Signature,
) {
    // Minimal: just call the SDK's secp256r1_verify for now
    env.crypto().secp256r1_verify(
        public_key,
        &env.crypto().sha256(&signature.authenticator_data),
        &signature.signature,
    );
    // TODO: Add clientDataJson challenge check if needed
}

// --- Account contract ---

#[contract]
pub struct Account;

#[contracttype]
pub struct AccountSignature {
    pub signature: BytesN<64>,
}

#[contractimpl]
impl Account {
    pub fn deposit(_env: Env, _from: Address, _token: Address, _amount: i128) {
        // Transfer tokens from `from` to this contract.
    }

    pub fn withdraw(_env: Env, _to: Address, _token: Address, _amount: i128) {
        // Transfer tokens from this contract to `to`.
    }

    pub fn join_campaign(
        _env: Env,
        _campaign_manager: Address,
        _pool: Address,
        _token: Address,
        _amount: i128,
    ) {
        // Interact with campaign manager to join a campaign.
    }

    pub fn claim_campaign(
        _env: Env,
        _campaign_manager: Address,
        _campaign_id: u32,
    ) {
        // Call campaign manager's claim function.
    }

    pub fn set_lp_lock(_env: Env, _lp_token: Address, _locked: bool) {
        // Lock/unlock LP tokens.
    }
}

// Minimal CustomAccountInterface implementation
#[contractimpl]
impl CustomAccountInterface for Account {
    type Error = Error;
    type Signature = Signatures;

    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,
        signatures: Signatures,
        auth_contexts: Vec<Context>,
    ) -> Result<(), Error> {
        // Check all contexts for an authorizing signature
        for context in auth_contexts.iter() {
            'check: loop {
                for (signer_key, _) in signatures.0.iter() {
                    if let Some((signer_val, _)) = get_signer_val_storage(&env, &signer_key, false) {
                        let (signer_expiration, signer_limits) = match &signer_val {
                            SignerVal::Policy(exp, lim) => (exp, lim),
                            SignerVal::Ed25519(exp, lim) => (exp, lim),
                            SignerVal::Secp256r1(_, exp, lim) => (exp, lim),
                        };
                        verify_signer_expiration(&env, signer_expiration);
                        if verify_context(&env, &context, &signer_key, signer_limits, &signatures) {
                            break 'check;
                        } else {
                            continue;
                        }
                    }
                }
                panic_with_error!(env, Error::MissingContext);
            }
        }
        // Check all signatures for a matching context
        for (signer_key, signature) in signatures.0.iter() {
            match get_signer_val_storage(&env, &signer_key, true) {
                None => panic_with_error!(env, Error::NotFound),
                Some((signer_val, _)) => {
                    match signature {
                        Signature::Ed25519(sig) => {
                            if let SignerKey::Ed25519(public_key) = &signer_key {
                                env.crypto().ed25519_verify(
                                    &public_key,
                                    &signature_payload.clone().into(),
                                    &sig,
                                );
                                if let SignerVal::Ed25519(exp, _) = &signer_val {
                                    verify_signer_expiration(&env, exp);
                                }
                                continue;
                            }
                            panic_with_error!(env, Error::SignatureKeyValueMismatch)
                        }
                        Signature::Secp256r1(sig) => {
                            if let SignerVal::Secp256r1(public_key, exp, _) = &signer_val {
                                verify_secp256r1_signature(&env, &signature_payload, public_key, sig.clone());
                                verify_signer_expiration(&env, exp);
                                continue;
                            }
                            panic_with_error!(env, Error::SignatureKeyValueMismatch)
                        }
                        Signature::Policy => {
                            // Policy logic not implemented yet
                            panic_with_error!(env, Error::SignatureKeyValueMismatch)
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
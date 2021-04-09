use cosmwasm_std::{
    coin, coins, to_binary, BankMsg, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo,
    RecoverPubkeyError, Response, StdError, StdResult, Storage,
};
use cw0::NativeBalance;
use sha2::{Digest, Sha256};

use crate::{
    error::ContractError,
    msg::{HandleMsg, InitMsg, QueryMsg},
    state::{RegisteredChannelState, ASSETS, DENOM, STATES},
    types::{Account, ChannelID, ChannelParameters, ChannelState, Signature},
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    DENOM.save(deps.storage, &msg.denom)?;
    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<Response, ContractError> {
    match msg {
        HandleMsg::Deposit { channel, account } => deposit(deps, info, channel, account),
        HandleMsg::Register {
            params,
            state,
            sigs,
        } => register(deps, _env, &params, &state, sigs),
        HandleMsg::Withdraw {
            params,
            account_index,
            sig,
        } => withdraw(deps, _env, info, params, account_index, sig),
    }
}

pub fn deposit(
    deps: DepsMut,
    info: MessageInfo,
    channel: ChannelID,
    account: Account,
) -> Result<Response, ContractError> {
    ASSETS.update::<_, StdError>(deps.storage, (&account, &channel), |balance| {
        let mut balance = balance.unwrap_or_default();
        let sent_funds = NativeBalance(info.funds);
        balance += sent_funds;
        Ok(balance)
    })?;

    Ok(Response::default())
}

pub fn register(
    deps: DepsMut,
    _env: Env,
    params: &ChannelParameters,
    state: &ChannelState,
    sigs: [Signature; 2],
) -> Result<Response, ContractError> {
    if !verify_state(
        &deps,
        params,
        state,
        sigs[0].clone(),
        params.participants[0],
    )? {
        return Err(ContractError::InvalidSignature {});
    };
    if !verify_state(
        &deps,
        params,
        state,
        sigs[1].clone(),
        params.participants[1],
    )? {
        return Err(ContractError::InvalidSignature {});
    };

    STATES.update::<_, StdError>(
        deps.storage,
        &params.hash(),
        |state_option| match state_option {
            Some(state_before) => {
                if state_before.state_l2.version > state.version {
                    return Ok(state_before);
                } else if !state_before.timed_out(_env, &params) {
                    return Ok(state_before);
                }
                Ok(RegisteredChannelState {
                    state_l2: state.clone(),
                    timestamp: state_before.timestamp,
                    settled: false,
                })
            }
            None => Ok(RegisteredChannelState {
                state_l2: state.clone(),
                timestamp: _env.block.time,
                settled: false,
            }),
        },
    )?;

    Ok(Response::default())
}

pub fn withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    params: ChannelParameters,
    account_index: u16,
    sig: Signature,
) -> Result<Response, ContractError> {
    let account = params.participants[usize::from(account_index)];
    let channel_id = params.hash();
    let receiver = info.sender;
    let receiver_canonical = deps.api.canonical_address(&receiver)?;

    ensure_settled(deps.storage, _env, params)?;
    verify_withdrawal(&deps, channel_id, receiver_canonical, sig, account)?;

    let key: (&[u8], &[u8]) = (&account, &channel_id);
    let mut balance = ASSETS.may_load(deps.storage, key)?.unwrap_or_default();
    let amount = balance.clone();
    balance = (balance - amount.clone().into_vec()).expect("error converting balances");
    ASSETS.save(deps.storage, key, &balance)?;

    let mut res = Response::new();
    res.add_message(BankMsg::Send {
        to_address: receiver,
        amount: amount.into_vec(),
    });
    Ok(res)
}

fn verify_withdrawal(
    deps: &DepsMut,
    channel_id: ChannelID,
    receiver: CanonicalAddr,
    sig: Signature,
    account: Account,
) -> Result<(), ContractError> {
    let mut hasher = Sha256::new();
    hasher.update(channel_id);
    hasher.update(receiver.as_slice());
    let hash = hasher.finalize();

    let pk = deps
        .api
        .secp256k1_recover_pubkey(&hash, &sig.to_bytes(), sig.v)?;
    if Sha256::digest(&pk).as_slice()[..20] != account {
        return Err(ContractError::InvalidWithdrawal {});
    };
    Ok(())
}

fn ensure_settled(
    storage: &mut dyn Storage,
    _env: Env,
    params: ChannelParameters,
) -> Result<(), StdError> {
    let channel_id = params.hash();
    let mut state_l1 = STATES.load(storage, &channel_id)?;
    if !state_l1.timed_out(_env, &params) {
        return Err(StdError::GenericErr {
            msg: "not ready".to_string(),
        });
    };

    if state_l1.settled {
        return Ok(());
    }

    let k0: (&[u8], &[u8]) = (&params.participants[0], &channel_id);
    let k1: (&[u8], &[u8]) = (&params.participants[1], &channel_id);

    let assets0 = ASSETS.may_load(storage, k0)?.unwrap_or_default();
    let assets1 = ASSETS.may_load(storage, k1)?.unwrap_or_default();
    let sum_assets = assets0 + assets1;

    let bal0 = state_l1.state_l2.balance[0];
    let bal1 = state_l1.state_l2.balance[1];
    let sum_balances = bal0 + bal1;
    let denom = DENOM.load(storage)?;

    if sum_assets.has(&coin(sum_balances.u128(), &denom)) {
        ASSETS.save(storage, k0, &NativeBalance(coins(bal0.u128(), &denom)))?;
        ASSETS.save(storage, k1, &NativeBalance(coins(bal1.u128(), &denom)))?;
    };

    state_l1.settled = true;
    STATES.save(storage, &channel_id, &state_l1)
}

fn verify_state(
    deps: &DepsMut,
    params: &ChannelParameters,
    state: &ChannelState,
    sig: Signature,
    public_key: Account,
) -> Result<bool, RecoverPubkeyError> {
    let mut hasher = Sha256::new();
    hasher.update(params.hash());
    hasher.update(state.hash());
    let channel_hash = hasher.finalize();

    let pk = deps
        .api
        .secp256k1_recover_pubkey(&channel_hash, &sig.to_bytes(), sig.v)?;

    Ok(Sha256::digest(&pk).as_slice()[..20] == public_key)
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDenom {} => to_binary(&query_denom(deps)?),
        QueryMsg::GetFunding { channel, account } => {
            to_binary(&query_funds(deps, channel, account)?)
        }
    }
}

fn query_denom(deps: Deps) -> StdResult<String> {
    DENOM.load(deps.storage)
}

fn query_funds(deps: Deps, channel: ChannelID, account: Account) -> StdResult<NativeBalance> {
    ASSETS.load(deps.storage, (&account, &channel))
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::types::Nonce;

    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        HumanAddr, Uint128,
    };
    use k256::{
        ecdsa::{recoverable, signature::DigestSigner, SigningKey, VerifyingKey},
        elliptic_curve::sec1::ToEncodedPoint,
    };
    use rand_core::OsRng;

    const DENOM: &str = "umayo";

    fn generate_account() -> (SigningKey, Account) {
        let sk = SigningKey::random(&mut OsRng);
        let pk = VerifyingKey::from(&sk);
        (
            sk,
            Sha256::digest(pk.to_encoded_point(false).as_bytes()).as_slice()[..20]
                .try_into()
                .unwrap(),
        )
    }

    fn generate_nonce() -> Nonce {
        [0; 32]
    }

    fn test_get_denom(deps: Deps) {
        let denom_: String = query_denom(deps).unwrap();
        assert_eq!(DENOM.to_string(), denom_);
    }

    fn test_deposit(
        mut deps: DepsMut,
        _env: Env,
        amount: u128,
        channel_id: ChannelID,
        account: Account,
    ) {
        // deposit
        let funds = NativeBalance(coins(amount, DENOM.to_string()));
        let info = mock_info("alice", &funds.clone().into_vec());
        deposit(deps.branch(), info, channel_id, account).unwrap();

        // query funds
        let funds_ = query_funds(deps.branch().as_ref(), channel_id, account).unwrap();
        assert_eq!(funds, funds_);
    }

    fn test_register(
        deps: DepsMut,
        _env: Env,
        params: &ChannelParameters,
        state: &ChannelState,
        sigs: [super::Signature; 2],
    ) {
        register(deps, _env, params, state, sigs).unwrap();
    }

    fn test_withdraw(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        params: ChannelParameters,
        account_index: u16,
        sig: super::Signature,
    ) {
        withdraw(deps, _env, info, params, account_index, sig).unwrap();
    }

    #[test]
    fn test_all() {
        let mut deps = mock_dependencies(&[]);
        let mut _env = mock_env();

        // setup contract
        let denom = "umayo".to_string();
        let msg = InitMsg {
            denom: denom.clone(),
        };
        let info = mock_info("anyone", &[]);
        instantiate(deps.as_mut(), _env.clone(), info, msg).unwrap();

        // setup accounts
        let (sk1, acc1) = generate_account();
        let (sk2, acc2) = generate_account();

        // generate channel
        let bal1 = 123;
        let nonce = generate_nonce();
        let params = ChannelParameters {
            challenge_duration: 10,
            nonce,
            participants: [acc1, acc2],
        };
        let state_l2 = ChannelState {
            version: 0,
            finalized: false,
            balance: [Uint128(bal1), Uint128(0)],
        };
        let channel_id = params.hash();

        // get denomination
        test_get_denom(deps.as_ref());

        // deposit
        test_deposit(deps.as_mut(), _env.clone(), bal1, channel_id, acc1);

        // register
        let sigs = sign_state(&params, &state_l2, &sk1, &sk2);
        test_register(deps.as_mut(), _env.clone(), &params, &state_l2, sigs);

        // withdraw
        let account_index = 0;
        use cosmwasm_std::Api;
        let receiver = HumanAddr::from("alice");
        let receiver_canonical = deps.api.canonical_address(&receiver).unwrap();
        let sig = sign_withdrawal(params.hash(), receiver_canonical, &sk1);
        let info = mock_info("alice", &[]);
        _env.block.time += 60;
        test_withdraw(deps.as_mut(), _env, info, params, account_index, sig);
    }

    fn sign_state(
        params: &ChannelParameters,
        state: &ChannelState,
        sk1: &SigningKey,
        sk2: &SigningKey,
    ) -> [super::Signature; 2] {
        let mut hasher = Sha256::new();
        hasher.update(params.hash());
        hasher.update(state.hash());

        let sig1: recoverable::Signature = sk1.sign_digest(hasher.clone());
        let sig2: recoverable::Signature = sk2.sign_digest(hasher);

        [
            sig1.as_ref().try_into().unwrap(),
            sig2.as_ref().try_into().unwrap(),
        ]
    }

    fn sign_withdrawal(
        channel_id: ChannelID,
        receiver: CanonicalAddr,
        sk: &SigningKey,
    ) -> super::Signature {
        let mut hasher = Sha256::new();
        hasher.update(channel_id);
        hasher.update(receiver.as_slice());

        let sig: recoverable::Signature = sk.sign_digest(hasher);
        sig.as_ref().try_into().unwrap()
    }
}

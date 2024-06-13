use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response, StdResult};
mod contract;
pub mod msg;

use contract::{deposit, withdraw, instantiate as contract_instantiate};
use msg::{InstantiateMsg, ExecuteMsg, DepositMsg, WithdrawMsg};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract_instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Deposit { amount } => deposit(deps, env, info, DepositMsg { amount }),
        ExecuteMsg::Withdraw { amount } => withdraw(deps, env, info, WithdrawMsg { amount }),
    }
}

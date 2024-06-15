use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response, StdResult};
mod contract;
pub mod msg;
pub mod state;

use contract::{instantiate as contract_instantiate, execute as contract_execute};
use msg::{InstantiateMsg, ExecuteMsg};

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
    contract_execute(deps, env, info, msg)
}

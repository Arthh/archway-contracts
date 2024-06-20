#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,StdError
};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Config, CONFIG, COLLATERAL_STATE, Collateral};
use cw20::Cw20ExecuteMsg;
use crate::ContractError;
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:loan-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .and_then(|addr_string| deps.api.addr_validate(addr_string.as_str()).ok())
        .unwrap_or(info.sender);

    let config = Config {
        owner: owner.clone(),
    };

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DepositCollateral { amount, valuation } => {
            Ok(deposit_collateral(deps, env, info, amount, valuation)?)
        }
        ExecuteMsg::AdjustValuation { new_valuation } => Ok(adjust_valuation(deps, info, new_valuation)?),
        ExecuteMsg::PayTax {} => Ok(pay_tax(deps, env, info)?),
        ExecuteMsg::LiquidateCollateral { collateral_id } => Ok(liquidate_collateral(deps, info, collateral_id)?),
    }
}

fn deposit_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    valuation: Uint128,
) -> StdResult<Response> {
    let collateral_id = format!("{}-{}", env.block.height, info.sender); // Create a unique ID

    let collateral = Collateral {
        id: collateral_id.clone(),
        token: "CONST".to_string(), // Hardcoded to only accept CONST
        amount,
        valuation,
        last_tax_payment: env.block.time.seconds(),
        borrower: info.sender.clone(),
    };

    // Save the collateral into the state
    COLLATERAL_STATE.save(deps.storage, &collateral_id, &collateral)?;

    // Handle CONST tokens
    if let Some(fund) = info.funds.iter().find(|coin| coin.denom == "CONST") {
        if fund.amount < amount {
            return Err(StdError::generic_err("Insufficient CONST tokens sent"));
        }

        let transfer_msg = BankMsg::Send {
            to_address: env.contract.address.to_string(),
            amount: vec![Coin {
                denom: "CONST".to_string(),
                amount,
            }],
        };

        return Ok(Response::new()
            .add_message(transfer_msg)
            .add_attribute("method", "deposit_collateral")
            .add_attribute("collateral_id", collateral_id));
    } else {
        return Err(StdError::generic_err("No CONST tokens sent"));
    }
}

fn adjust_valuation(
    deps: DepsMut,
    info: MessageInfo,
    new_valuation: Uint128,
) -> StdResult<Response> {
    let mut collateral = COLLATERAL_STATE.load(deps.storage, &info.sender.to_string())?;
    collateral.valuation = new_valuation;
    COLLATERAL_STATE.save(deps.storage, &info.sender.to_string(), &collateral)?;

    Ok(Response::new().add_attribute("method", "adjust_valuation"))
}

fn pay_tax(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let mut collateral = COLLATERAL_STATE.load(deps.storage, &info.sender.to_string())?;
    let elapsed_time = env.block.time.seconds() - collateral.last_tax_payment;
    let config = CONFIG.load(deps.storage)?;
    // let tax_due = collateral.valuation.u128() * elapsed_time as u128 * config.tax_rate as u128 / 10000; // Simplified tax calculation
    // Logic to deduct tax from borrower
    collateral.last_tax_payment = env.block.time.seconds();
    COLLATERAL_STATE.save(deps.storage, &info.sender.to_string(), &collateral)?;

    Ok(Response::new().add_attribute("method", "pay_tax"))
}

fn liquidate_collateral(
    deps: DepsMut,
    info: MessageInfo,
    collateral_id: String,
) -> StdResult<Response> {
    let collateral = COLLATERAL_STATE.load(deps.storage, &collateral_id)?;
    COLLATERAL_STATE.remove(deps.storage, &collateral_id);

    // Transfer collateral to the liquidator
    let transfer_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: collateral.token.clone(),
            amount: collateral.amount,
        }],
    };

    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attribute("method", "liquidate_collateral")
        .add_attribute("status", "success"))
}

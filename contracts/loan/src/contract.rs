#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Config, CONFIG, COLLATERAL_STATE, Collateral};
use cw20::Cw20ExecuteMsg;
use crate::ContractError;
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:loan-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const TAX_RATE: f64 = 1.5; // Hardcoded tax rate of 1.5%

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
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
        .add_attribute("owner", owner)
        .add_attribute("tax_rate", TAX_RATE.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DepositCollateral { token, amount, valuation } => {
            Ok(deposit_collateral(deps, env, info, token, amount, valuation)?)
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
    token: String,
    amount: Uint128,
    valuation: Uint128,
) -> StdResult<Response> {
    let collateral_id = format!("{}-{}", env.block.height, info.sender); // Create a unique ID

    let collateral = Collateral {
        id: collateral_id.clone(),
        token: token.clone(),
        amount,
        valuation,
        last_tax_payment: env.block.time.seconds(),
        borrower: info.sender.clone(),
    };

    // Save the collateral into the state
    COLLATERAL_STATE.save(deps.storage, &collateral_id, &collateral)?;

    // Handle native tokens
    if info.funds.iter().any(|coin| coin.denom == token) {
        let transfer_msg = BankMsg::Send {
            to_address: env.contract.address.to_string(),
            amount: vec![Coin {
                denom: token.clone(),
                amount,
            }],
        };

        return Ok(Response::new()
            .add_message(transfer_msg)
            .add_attribute("method", "deposit_collateral")
            .add_attribute("collateral_id", collateral_id));
    }

    // Handle CW20 tokens
    let transfer_msg = WasmMsg::Execute {
        contract_addr: token.clone(),
        msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
            recipient: env.contract.address.to_string(),
            amount,
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attribute("method", "deposit_collateral")
        .add_attribute("collateral_id", collateral_id))
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
    
    let tax_due = Uint128::from(collateral.valuation.u128() * elapsed_time as u128 * TAX_RATE as u128 / 10000);
    
    let borrower_balance = deps.querier.query_balance(&info.sender, &collateral.token)?;
    
    if borrower_balance.amount < tax_due {
        return Err(StdError::generic_err("Insufficient funds to pay tax"));
    }
    
    let tax_payment = BankMsg::Send {
        to_address: env.contract.address.to_string(),
        amount: vec![Coin {
            denom: collateral.token.clone(),
            amount: tax_due,
        }],
    };
    
    collateral.last_tax_payment = env.block.time.seconds();
    
    COLLATERAL_STATE.save(deps.storage, &info.sender.to_string(), &collateral)?;
    
    Ok(Response::new()
        .add_message(tax_payment)
        .add_attribute("method", "pay_tax")
        .add_attribute("tax_due", tax_due.to_string()))
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

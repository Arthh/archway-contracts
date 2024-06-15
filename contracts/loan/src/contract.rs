use cosmwasm_std::{
    to_json_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Collateral, CollateralState, COLLATERAL_STATE};
use cw20::Cw20ExecuteMsg;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = CollateralState {
        collaterals: vec![],
        name: msg.name,
        symbol: msg.symbol,
        tax_rate: msg.tax_rate,
    };
    COLLATERAL_STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::DepositCollateral { token, amount, valuation } => {
            deposit_collateral(deps, env, info, token, amount, valuation)
        }
        ExecuteMsg::AdjustValuation { new_valuation } => adjust_valuation(deps, info, new_valuation),
        ExecuteMsg::PayTax {} => pay_tax(deps, env, info),
        ExecuteMsg::LiquidateCollateral { collateral_id } => {
            liquidate_collateral(deps, info, collateral_id)
        }
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

    // Handle native tokens
    if info.funds.iter().any(|coin| coin.denom == token) {
        let transfer_msg = BankMsg::Send {
            to_address: env.contract.address.to_string(),
            amount: vec![Coin {
                denom: token.clone(),
                amount,
            }],
        };
        COLLATERAL_STATE.update(deps.storage, |mut state| -> StdResult<_> {
            state.collaterals.push(collateral);
            Ok(state)
        })?;

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

    COLLATERAL_STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.collaterals.push(collateral);
        Ok(state)
    })?;

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
    COLLATERAL_STATE.update(deps.storage, |mut state| -> StdResult<_> {
        for collateral in &mut state.collaterals {
            if collateral.borrower == info.sender {
                collateral.valuation = new_valuation;
                break;
            }
        }
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "adjust_valuation"))
}

fn pay_tax(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    COLLATERAL_STATE.update(deps.storage, |mut state| -> StdResult<_> {
        for collateral in &mut state.collaterals {
            if collateral.borrower == info.sender {
                let elapsed_time = env.block.time.seconds() - collateral.last_tax_payment;
                let _tax_due = collateral.valuation.u128() * elapsed_time as u128 * state.tax_rate as u128 / 10000; // Simplified tax calculation
                // Logic to deduct tax from borrower
                collateral.last_tax_payment = env.block.time.seconds();
                break;
            }
        }
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "pay_tax"))
}

fn liquidate_collateral(
    deps: DepsMut,
    info: MessageInfo,
    collateral_id: String,
) -> StdResult<Response> {
    let response = COLLATERAL_STATE.update(deps.storage, |mut state| -> StdResult<CollateralState> {
        if let Some(index) = state.collaterals.iter().position(|c| c.id == collateral_id) {
            let collateral = state.collaterals.remove(index);

            // Transfer collateral to the liquidator
            let _transfer_msg = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin {
                    denom: collateral.token.clone(),
                    amount: collateral.amount,
                }],
            };

            Ok(state)
        } else {
            Ok(state)
        }
    });

    match response {
        Ok(_) => Ok(Response::new()
            .add_attribute("method", "liquidate_collateral")
            .add_attribute("status", "success")),
        Err(_) => Ok(Response::new()
            .add_attribute("method", "liquidate_collateral")
            .add_attribute("status", "collateral_not_found")),
    }
}

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    DepositCollateral { amount: Uint128, valuation: Uint128 },
    AdjustValuation { new_valuation: Uint128 },
    PayTax {},
    LiquidateCollateral { collateral_id: String },
}

#[cw_serde]
pub struct DepositMsg {
    pub amount: Uint128,
    pub valuation: Uint128,
}

#[cw_serde]
pub struct WithdrawMsg {
    pub amount: Uint128,
}

#[cw_serde]
pub struct QueryMsg {
    pub collateral_id: u64,
}

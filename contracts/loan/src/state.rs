use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollateralState {
    pub collaterals: Vec<Collateral>,
    pub name: String,
    pub symbol: String,
    pub tax_rate: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Collateral {
    pub id: String,
    pub token: String,
    pub amount: Uint128,
    pub valuation: Uint128,
    pub last_tax_payment: u64,
    pub borrower: Addr,
}

pub const COLLATERAL_STATE: Item<CollateralState> = Item::new("collateral_state");

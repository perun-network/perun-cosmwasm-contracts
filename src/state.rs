use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Env;
use cw0::NativeBalance;
use cw_storage_plus::{Item, Map};

use crate::msg::{ChannelParameters, ChannelState};

pub type BalanceID = [u8];
pub type Timestamp = u64;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RegisteredChannelState {
    pub state_l2: ChannelState,
    pub timestamp: Timestamp,
    pub settled: bool,
}

impl RegisteredChannelState {
    pub fn timed_out(&self, _env: Env, params: &ChannelParameters) -> bool {
        self.timestamp + params.challenge_duration < _env.block.time
    }
}

pub const DENOM: Item<String> = Item::new("denom");
pub const ASSETS: Map<(&[u8], &[u8]), NativeBalance> = Map::new("assets");
pub const STATES: Map<&[u8], RegisteredChannelState> = Map::new("states");

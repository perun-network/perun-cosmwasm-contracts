use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{Account, ChannelID, ChannelParameters, ChannelState, Signature};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Deposit deposits the attached assets into channel.
    Deposit {
        channel: ChannelID,
        account: Account,
    },
    // Register registers a channel for settlement. mutually-signed channel state.
    Register {
        params: ChannelParameters,
        state: ChannelState,
        sigs: [Signature; 2],
    },
    // Withdraw withdraws with
    Withdraw {
        params: ChannelParameters,
        account_index: usize,
        sig: Signature,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetDenom returns the coin denominator used for the channel funds.
    GetDenom {},
    // GetFunding returns the channel funding of the client.
    GetFunding {
        channel: ChannelID,
        account: Account,
    },
}

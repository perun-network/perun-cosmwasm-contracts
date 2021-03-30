use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::convert::TryInto;

pub type HashValue = [u8; 32];
pub type ChannelID = HashValue;
pub type Balance = Uint128;
pub type Account = [u8; 20];
pub type Nonce = [u8; 32];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Signature {
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub v: u8,
}

impl Signature {
    pub fn to_bytes(&self) -> [u8; 64] {
        [self.r, self.s].concat().try_into().unwrap()
    }
}

impl From<&[u8]> for Signature {
    fn from(bytes: &[u8]) -> Self {
        Signature {
            r: bytes[..32].try_into().unwrap(),
            s: bytes[32..64].try_into().unwrap(),
            v: bytes[64],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ChannelParameters {
    pub participants: [Account; 2],
    pub nonce: Nonce,
    pub challenge_duration: u64,
}

impl ChannelParameters {
    pub fn hash(&self) -> ChannelID {
        let mut hasher = Sha256::new();
        hasher.update(self.participants[0]);
        hasher.update(self.participants[1]);
        hasher.update(self.nonce);
        hasher.update(self.challenge_duration.to_be_bytes());
        let result = hasher.finalize();
        result.try_into().unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ChannelState {
    pub version: u64,
    pub balance: [Balance; 2],
    pub finalized: bool,
}

impl ChannelState {
    pub fn hash(&self) -> HashValue {
        let mut hasher = Sha256::new();
        hasher.update(self.version.to_be_bytes());
        hasher.update(self.balance[0].u128().to_be_bytes());
        hasher.update(self.balance[1].u128().to_be_bytes());
        hasher.update([self.finalized as u8]);
        let result = hasher.finalize();
        result.try_into().unwrap()
    }
}

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

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json;
use cosmwasm_std::to_binary;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Metadata {
    pub artist_name: String,
    pub album_name: String,
    pub album_artwork: String,
    pub album_year: i32,
    pub track_name: String,
    pub external_track_link: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NFT {
    pub owner: String,
    pub metadata: String,  // JSON string representation of Metadata
}

const NFT_KEY: &[u8] = b"nft";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum HandleMsg {
    Mint { metadata: Metadata },
    Transfer { recipient: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum QueryMsg {
    OwnerOf {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

pub fn init(deps: DepsMut, _env: Env, _info: MessageInfo, _msg: InitMsg) -> StdResult<Response> {
    Ok(Response::default())
}

pub fn handle(deps: DepsMut, _env: Env, info: MessageInfo, msg: HandleMsg) -> StdResult<Response> {
    match msg {
        HandleMsg::Mint { metadata } => {
            let metadata_str = serde_json::to_string(&metadata);
            let nft = NFT {
                owner: info.sender.into_string(),
                metadata: metadata_str.unwrap(),
            };
            deps.storage.set(NFT_KEY, &serde_json::to_vec(&nft).unwrap());
            Ok(Response::default())
        }
        HandleMsg::Transfer { recipient } => {
            let nft_data = deps.storage.get(NFT_KEY).unwrap();
            let mut nft: NFT = serde_json::from_slice(&nft_data).unwrap();
            nft.owner = recipient;
            deps.storage.set(NFT_KEY, &serde_json::to_vec(&nft).unwrap());
            Ok(Response::default())
        }
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::OwnerOf {} => {
            let nft_data = deps.storage.get(NFT_KEY).unwrap();
            let nft: NFT = serde_json::from_slice(&nft_data).unwrap();
            to_binary(&nft.owner)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let init_msg = InitMsg {};
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let res = init(deps.as_mut(), env, info, init_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn mint_nft() {
        let mut deps = mock_dependencies();

        let metadata = Metadata {
            artist_name: "Artist".into(),
            album_name: "Album".into(),
            album_artwork: "https://example.com/artwork".into(),
            album_year: 2023,
            track_name: "Track".into(),
            external_track_link: "https://example.com/track".into(),
        };

        let mint_msg = HandleMsg::Mint { metadata };
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let res = handle(deps.as_mut(), env, info, mint_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn transfer_nft() {
        let mut deps = mock_dependencies();

        let metadata = Metadata {
            artist_name: "Artist".to_string(),
            album_name: "Album".to_string(),
            album_artwork: "https://example.com/artwork".to_string(),
            album_year: 2023,
            track_name: "Track".to_string(),
            external_track_link: "https://example.com/track".to_string(),
        };

        let mint_msg = HandleMsg::Mint { metadata };
        let env = mock_env();
        let info = mock_info("creator", &[]);
        let _ = handle(deps.as_mut(), env.clone(), info.clone(), mint_msg).unwrap();

        let transfer_msg = HandleMsg::Transfer {
            recipient: "new_owner".into(),
        };

        let res = handle(deps.as_mut(), env, info, transfer_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}

use bincode;
use bincode::deserialize;
use bincode2;
use cosmwasm::serde::to_vec;
use cosmwasm::types::HumanAddr;
use cosmwasm_std::entry_point;
use cosmwasm_std::from_slice;
use cosmwasm_std::Empty;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, CosmosMsg, CustomMsg, Deps, DepsMut, Env, MessageInfo, Querier,
    Response, StdError, StdResult, Storage,
};
use cw721::{
    AllNftInfoResponse, Approval, ApprovalResponse, ApprovalsResponse, ContractInfoResponse, Cw721,
    Cw721Execute, Cw721ExecuteMsg, Cw721Query, Cw721QueryMsg, Cw721ReceiveMsg, Expiration,
    NftInfoResponse, NumTokensResponse, OperatorResponse, OperatorsResponse, OwnerOfResponse,
    TokensResponse,
};
use cw721_base::state::{Approval as MyApproval, Cw721Contract, TokenIndexes, TokenInfo};
use cw721_base::Extension;
use cw_ownable::initialize_owner;
use cw_storage_plus;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json;
use std::fmt::{self, Display};
use std::str::from_utf8;
const CONFIG_KEY: &[u8] = b"config";
const STATE_KEY: &[u8] = b"contract_state";

static STATE: Item<MyCw721ContractState> = Item::new("state");
static ARTIST: Item<String> = Item::new("artist");
static ALBUM: Item<String> = Item::new("album");
static ARTWORK_URL: Item<String> = Item::new("artwork_url");
static YEAR: Item<u32> = Item::new("year");
static TRACK_NAME: Item<String> = Item::new("track_name");
static AUDIO_TRACK_URL: Item<String> = Item::new("audio_track_url");

// Add this line near your imports or where you define your types
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;
type MyTokenInfo = TokenInfo<SongMetadata>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]

pub struct Config {
    pub version: String,
    pub artist: String,
    pub album: String,
    pub artwork_url: String,
    pub year: u32,
    pub track_name: String,
    pub audio_track_url: String,
    pub token_name: String,
    pub token_symbol: String,
    pub platform_fee: Option<u64>,
}

fn parse_custom_msg(msg: &Binary) -> StdResult<MyCustomMsg> {
    from_slice(&msg.0).map_err(|_| StdError::generic_err("Failed to deserialize custom message"))
}

pub fn read_token_info(storage: &dyn Storage, token_id: &str) -> StdResult<Option<MyTokenInfo>> {
    // Create a storage key based on token_id
    let storage_key = format!("token_info_{}", token_id);
    println!("Debug: Generated storage key: {}", &storage_key); // Debug line

    // Attempt to load data from storage
    let data = may_load::<MyTokenInfo>(storage, storage_key.as_bytes())?;

    // Debug: Print the retrieved data
    println!(
        "Debug: Retrieved token info for key: {:?}, Result: {:?}",
        &storage_key, &data
    );

    Ok(data)
}
pub fn query_owner(storage: &dyn Storage, token_id: &str) -> StdResult<Option<Addr>> {
    // Use the get_owner_of_token function to get the owner
    match get_owner_of_token(storage, token_id) {
        Ok(Some(owner)) => {
            // Debug: Print the retrieved owner
            println!(
                "Debug: Retrieved owner for token_id: {}, Owner: {:?}",
                token_id, owner
            );

            Ok(Some(owner))
        }
        Ok(None) => {
            // Debug: No owner found
            println!("Debug: No owner found for token_id: {}", token_id);

            Ok(None)
        }
        Err(e) => {
            // Debug: Print the error
            println!(
                "Debug: Error fetching owner for token_id: {}, Error: {:?}",
                token_id, e
            );

            // Convert MyStdError to StdError (this is just an example; you'll need to replace this with actual conversion logic)
            let converted_error = StdError::generic_err(format!("Error: {:?}", e));

            Err(converted_error)
        }
    }
}

pub fn may_load<T: DeserializeOwned + Serialize + std::fmt::Debug>(
    storage: &dyn Storage,
    key: &[u8],
) -> StdResult<Option<T>> {
    let key_str = from_utf8(key).map_err(|_| StdError::generic_err("Invalid UTF-8 sequence"))?;
    let item = Item::new(key_str);

    // Debug: Print the key being used for retrieval
    println!("Debug: Retrieving data with key: {}", key_str);

    match item.load(storage) {
        Ok(data) => {
            // Debug: Print the retrieved data
            println!("Debug: Retrieved data: {:?}", data);
            Ok(Some(data))
        }
        Err(e) => {
            // Debug: Print the error
            println!("Debug: Retrieval failed with error: {:?}", e);
            Ok(None)
        }
    }
}

fn get_owner_of_token(storage: &dyn Storage, token_id: &str) -> Result<Option<Addr>, MyStdError> {
    let storage_key = format!("token_info_{}", token_id); // Matching the key format
    println!(
        "Debug: Generated storage key for retrieval: {}",
        &storage_key
    ); // Debug line

    let data = may_load(storage, storage_key.as_bytes()); // Using the correct key format

    // Debug: Print the retrieved data
    println!(
        "Debug: Retrieved data for key: {:?}, Result: {:?}",
        &storage_key, &data
    );

    match data {
        Ok(owner) => Ok(owner),
        Err(_) => Err(MyStdError::NotFound),
    }
}
pub fn set_owner_of_token(
    storage: &mut dyn Storage,
    token_id: &String,
    owner: &Addr,
) -> StdResult<()> {
    save_token_owner(storage, &token_id, owner)?;

    Ok(())
}
pub fn save_token_owner(storage: &mut dyn Storage, token_id: &str, owner: &Addr) -> StdResult<()> {
    let key = format!("owner_of_{}", token_id);
    let item = Item::new(&key);

    item.save(storage, owner)
}
pub fn load_state(storage: &dyn Storage) -> StdResult<MyCw721ContractState> {
    let data = storage
        .get(CONFIG_KEY)
        .ok_or(StdError::generic_err("State not found"))?;
    let state: MyCw721ContractState = bincode::deserialize(&data)
        .map_err(|e| StdError::generic_err(format!("Failed to deserialize the state: {}", e)))?;
    Ok(state)
}
pub fn save_state(storage: &mut dyn Storage, state: &MyCw721ContractState) -> StdResult<()> {
    let data = bincode::serialize(state)
        .map_err(|e| StdError::generic_err(format!("Failed to serialize the state: {}", e)))?;
    storage.set(CONFIG_KEY, &data);
    Ok(())
}
fn update_state_after_transfer(
    storage: &mut dyn Storage,
    token_id: &String,
    new_owner: &String,
) -> StdResult<()> {
    let mut state = load_state(storage)?;
    save_state(storage, &state)?;
    Ok(())
}
fn update_state_after_mint(
    storage: &mut dyn Storage,
    token_id: &String,
    minter: &Addr,
) -> StdResult<()> {
    let mut state = load_state(storage)?;
    save_state(storage, &state)?;

    Ok(())
}
fn update_state_after_burn(
    storage: &mut dyn Storage,
    token_id: &String,
    owner: &Addr,
) -> StdResult<()> {
    let mut state = load_state(storage)?;
    save_state(storage, &state)?;
    Ok(())
}
pub fn get_all_tokens_by_owner(storage: &dyn Storage, owner: &Addr) -> StdResult<Vec<String>> {
    Ok(vec![])
}
pub fn save_all_approvals(
    storage: &mut dyn Storage,
    owner: &Addr,
    operator: &str,
    expires: Option<Expiration>,
) -> StdResult<()> {
    let all_tokens_owned = get_all_tokens_by_owner(storage, owner)?;

    for token_id in all_tokens_owned {
        save_approval(storage, &token_id, operator, expires.clone())?;
    }

    Ok(())
}

// Function to update the state (total supply, for example)
pub fn update_total_supply(storage: &mut dyn Storage, new_supply: u64) -> StdResult<()> {
    let mut state = load_state(storage)?;
    state.total_supply = new_supply;
    save_state(storage, &state)
}

pub fn save_approval(
    storage: &mut dyn Storage,
    token_id: &str,
    spender: &str,
    expires: Option<Expiration>,
) -> StdResult<()> {
    let key = format!("approval_{}_{}", token_id, spender);
    let expires_value = expires.unwrap_or(Expiration::default());
    let approval = Approval {
        spender: spender.to_string(),
        expires: expires_value,
    };

    let bin_data = bincode::serialize(&approval)
        .map_err(|e| StdError::generic_err(format!("Failed to serialize data: {}", e)))?;

    storage.set(key.as_bytes(), &bin_data);

    Ok(())
}
pub fn remove_approval(storage: &mut dyn Storage, token_id: &str, spender: &str) -> StdResult<()> {
    let key = format!("approval_{}_{}", token_id, spender);
    let key_bytes = key.as_bytes();
    storage.remove(key_bytes);

    Ok(())
}

pub fn save_token_info(
    storage: &mut dyn Storage,
    token_id: &str,
    token_info: &MyTokenInfo,
) -> StdResult<()> {
    println!("Debug: Entering save_token_info function");

    let storage_key = format!("token_info_{}", token_id);
    println!("Debug: Generated storage key: {}", &storage_key);

    let data = to_binary(token_info)?;
    println!("Debug: Serialized token_info data: {:?}", &data);

    storage.set(storage_key.as_bytes(), &data);
    println!("Debug: Data set in storage for key: {}", &storage_key);
    let data_check = storage.get(storage_key.as_bytes());
    println!("Debug: Data check right after saving: {:?}", data_check);

    Ok(())
}

fn query_number_of_tokens_owned_by(deps: Deps, owner: String) -> StdResult<u64> {
    let state = load_state(deps.storage)?;
    let count = 0;
    Ok(count)
}

// Function to remove all approvals for a specific token
fn remove_all_approvals_for_token(storage: &mut dyn Storage, token_id: &str) -> StdResult<()> {
    let all_spenders = get_all_approvals_for_token(storage, token_id)?;
    for spender in all_spenders {
        let key = format!("approval_{}_{}", token_id, spender);
        storage.remove(key.as_bytes());
    }
    Ok(())
}

// Function to remove token info and its owner mapping
fn remove_token_info_and_owner(storage: &mut dyn Storage, token_id: &str) -> StdResult<()> {
    let token_info_key = format!("token_info_{}", token_id);
    storage.remove(token_info_key.as_bytes());

    let owner_key = format!("owner_of_{}", token_id);
    storage.remove(owner_key.as_bytes());

    Ok(())
}

fn get_all_approvals_for_token(storage: &dyn Storage, token_id: &str) -> StdResult<Vec<String>> {
    // Convert the key to bytes
    let key = format!("approval:{}", token_id);
    let key_as_bytes = key.as_bytes();

    // Read from storage
    let data = storage
        .get(key_as_bytes)
        .ok_or_else(|| StdError::generic_err("Unable to read from storage"))?;

    // Deserialize data
    let approvals: Vec<String> = serde_json::from_slice(&data)
        .map_err(|_| StdError::generic_err("Error while deserializing data"))?;

    Ok(approvals)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DetailedNftInfoResponse {
    pub global_artist: String,
    pub global_album: String,
    pub token_track_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum MyCw721QueryMsg {
    ContractState {},
    NumberOfTokensOwnedBy { owner: String },
    DetailedNftInfo { token_id: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SongMetadata {
    pub artist: String,
    pub album: String,
    pub artwork_url: String,
    pub year: u32,
    pub track_name: String,
    pub audio_track_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MyCw721 {
    artist: String,
    album: String,
    artwork_url: String,
    year: u32,
    track_name: String,
    audio_track_url: String,
}

pub struct State<T, C, E>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: Display + ToString,
{
    cw721: Box<dyn Cw721<T, C, Err = E>>,
    artist: String,
    album: String,
    artwork_url: String,
    year: u32,
    track_name: String,
    audio_track_url: String,
}

impl<T, C, E> State<T, C, E>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: Display + ToString,
{
    pub fn burn_nft(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response<C>, E> {
        self.cw721.burn(deps, env, info, token_id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HumanAddrWrapper(HumanAddr);

impl JsonSchema for HumanAddrWrapper {
    fn schema_name() -> String {
        "HumanAddr".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(Default::default())
    }
}
impl HumanAddrWrapper {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
impl From<Addr> for HumanAddrWrapper {
    fn from(addr: Addr) -> Self {
        Self(HumanAddr::from(addr.as_str()))
    }
}

impl From<cosmwasm_std::StdError> for MyStdError {
    fn from(err: cosmwasm_std::StdError) -> Self {
        MyStdError::Other(err.to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub initial_owner: Option<HumanAddrWrapper>,
    pub artist: String,
    pub album: String,
    pub artwork_url: String,
    pub year: u32,
    pub track_name: String,
    pub audio_track_url: String,
    pub token_name: String,
    pub token_symbol: String,
    pub platform_fee: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CustomMsgType {}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MyCustomMsg {}
impl CustomMsg for MyCustomMsg {}

#[derive(Debug)]
pub enum ContractError {
    FailedToSaveState,
    FailedToLoadState,
}

impl MyCw721 {
    const STATE_KEY: &[u8] = b"contract_state";

    // Function to save the state into the contract's storage
    pub fn save_state(storage: &mut dyn Storage, state: &MyCw721ContractState) -> StdResult<()> {
        let data = bincode::serialize(state)
            .map_err(|_| StdError::generic_err("Failed to serialize the state"))?;
        storage.set(Self::STATE_KEY, &data);
        Ok(())
    }
    fn default() -> Self {
        let artist = "".to_string();
        let album = "".to_string();
        let artwork_url = "".to_string();
        let year = 1;
        let track_name = "".to_string();
        let audio_track_url = "".to_string();

        Self {
            artist,
            album,
            artwork_url,
            year,
            track_name,
            audio_track_url,
        }
    }
    pub fn init(
        cw721: Box<dyn Cw721<u64, MyCustomMsg, Err = MyStdError>>,
        artist: String,
        album: String,
        artwork_url: String,
        year: u32,
        track_name: String,
        audio_track_url: String,
    ) -> Self {
        Self {
            artist,
            album,
            artwork_url,
            year,
            track_name,
            audio_track_url,
        }
    }

    pub fn mint_nft(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        song_metadata: SongMetadata,
        token_id: String,
    ) -> Result<Response, MyStdError> {
        // Check permission
        if info.sender != env.contract.address {
            return Err(MyStdError::PermissionDenied);
        }

        // Debug: Print the token_id and sender
        println!("Debug: Minting token_id: {}", &token_id);
        println!("Debug: Sender: {}", &info.sender);

        // Clone the token_id to create a unique key for storage
        let unique_token_id = token_id.clone();
        println!(
            "Debug: Unique token_id used for storage: {}",
            &unique_token_id
        );

        // Create new token info
        let new_token_info = TokenInfo {
            owner: info.sender.clone(),
            approvals: vec![],
            token_uri: None,
            extension: song_metadata,
        };

        // Serialize new_token_info for storage
        let serialized_token_info = bincode2::serialize(&new_token_info).unwrap();

        // Create a storage key
        let storage_key = format!("token_info_{}", unique_token_id);

        deps.storage
            .set(storage_key.as_bytes(), &serialized_token_info);
        println!("Debug: Data saved under key: {}", storage_key);

        // TODO: Update your contract's state as required
        let mut state = load_state(deps.storage)?;
        save_state(deps.storage, &state)?;
        update_state_after_mint(deps.storage, &token_id, &info.sender)?;

        // Create a response
        let response = Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", &info.sender.to_string())
            .add_attribute("token_id", unique_token_id);

        // Debug: Retrieve and log the owner of the minted token
        let debug_owner = get_owner_of_token(deps.storage, &token_id)?;
        println!("Debug: Owner inside mint function: {:?}", debug_owner);

        Ok(response)
    }

    pub fn burn_nft(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response<MyCustomMsg>, MyStdError> {
        // Step 1: Verify the owner
        let current_owner = get_owner_of_token(deps.storage, &token_id)
            .map_err(|_| MyStdError::Other("Failed to fetch the owner".to_string()))?;

        if current_owner.is_none() {
            return Err(MyStdError::Other("Token does not exist".to_string()));
        }

        if current_owner.unwrap() != info.sender {
            return Err(MyStdError::PermissionDenied);
        }

        // Step 2: Remove all approvals related to this token
        remove_all_approvals_for_token(deps.storage, &token_id)?;

        // Step 3: Remove the token from the storage
        remove_token_info_and_owner(deps.storage, &token_id)?;

        // Step 4: Update total supply and state
        let mut state = load_state(deps.storage)?;
        state.total_supply = state.total_supply.saturating_sub(1);
        save_state(deps.storage, &state)?;

        // Step 5: Emit an event that the NFT has been burned
        let mut response = Response::new();
        response = response.add_attribute("action", "burn");
        response = response.add_attribute("burner", &info.sender.to_string());
        response = response.add_attribute("token_id", token_id);

        Ok(response)
    }
}
#[derive(Debug)]
pub enum MyStdError {
    NotFound,
    PermissionDenied,
    Other(String),
    Unauthorized,
}
struct MyStdErrorWrapper(cosmwasm_std::StdError);

impl std::fmt::Display for MyStdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            MyStdError::NotFound => write!(f, "Item not found"),
            MyStdError::PermissionDenied => write!(f, "Permission denied"),
            MyStdError::Other(ref s) => write!(f, "Other: {}", s),
            MyStdError::Unauthorized => todo!(),
        }
    }
}
impl std::fmt::Display for MyStdErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "My custom error wrapper: {}", self.0)
    }
}
impl std::fmt::Debug for MyStdErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyStdErrorWrapper: {:?}", self.0)
    }
}
impl Cw721Execute<u64, MyCustomMsg> for MyCw721 {
    type Err = MyStdError;

    fn transfer_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: String,
        token_id: String,
    ) -> Result<Response<MyCustomMsg>, MyStdError> {
        // let state = load_state(deps.storage)?;

        let current_owner = get_owner_of_token(deps.storage, &token_id)?;
        if current_owner.is_none() {
            return Err(MyStdError::NotFound); // or some other appropriate error
        }
        let current_owner = current_owner.unwrap();

        if current_owner != info.sender {
            return Err(MyStdError::Unauthorized); // or some other appropriate error
        }

        let recipient_addr = Addr::unchecked(recipient.clone());
        set_owner_of_token(deps.storage, &token_id, &recipient_addr)?;
        update_state_after_transfer(deps.storage, &token_id, &recipient)?;

        // save_state(deps.storage, &state)?;

        Ok(Response::new())
    }

    fn send_nft(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        recipient: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response<MyCustomMsg>, MyStdError> {
        let current_owner = get_owner_of_token(deps.storage, &token_id)?;
        if info.sender != current_owner.unwrap() {
            return Err(MyStdError::PermissionDenied);
        }
        let custom_msg: MyCustomMsg = parse_custom_msg(&msg)?;
        let recipient_addr = Addr::unchecked(recipient.clone());
        set_owner_of_token(deps.storage, &token_id, &recipient_addr)?;
        let mut response = Response::new();

        response = response.add_attribute("action", "send_nft");
        response = response.add_attribute("sender", info.sender);
        response = response.add_attribute("recipient", recipient);
        response = response.add_attribute("token_id", token_id);

        Ok(response)
    }

    fn approve(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    ) -> Result<Response<MyCustomMsg>, MyStdError> {
        let current_owner = get_owner_of_token(deps.storage, &token_id)?;
        if info.sender != current_owner.unwrap() {
            return Err(MyStdError::PermissionDenied);
        }

        // Save the approval information
        save_approval(deps.storage, &token_id, &spender, expires)?;

        // Create a response
        let mut response = Response::new();

        // Add some attributes to the response for tracking
        response = response.add_attribute("action", "approve");
        response = response.add_attribute("sender", info.sender);
        response = response.add_attribute("spender", spender);
        response = response.add_attribute("token_id", token_id);

        Ok(response)
    }

    fn revoke(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
    ) -> Result<Response<MyCustomMsg>, MyStdError> {
        let current_owner = get_owner_of_token(deps.storage, &token_id)?;
        if info.sender != current_owner.unwrap() {
            return Err(MyStdError::PermissionDenied);
        }
        remove_approval(deps.storage, &token_id, &spender)?;
        let mut response = Response::new();
        response = response.add_attribute("action", "revoke");
        response = response.add_attribute("sender", info.sender);
        response = response.add_attribute("spender", spender);
        response = response.add_attribute("token_id", token_id);

        Ok(response)
    }

    // Approves another address to transfer any NFT owned by the sender.
    fn approve_all(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: String,
        expires: Option<Expiration>,
    ) -> Result<Response<MyCustomMsg>, MyStdError> {
        // Your logic to save the approval for all tokens owned by the sender
        // Similar to save_approval, but for all tokens owned by the sender
        // Assume save_all_approvals is a function you've implemented to save approvals for all tokens
        save_all_approvals(deps.storage, &info.sender, &operator, expires)?;

        // Optionally, emit an event or create other side-effects
        // ...

        Ok(Response::new())
    }

    // Revokes approval for all NFTs owned by the sender.
    fn revoke_all(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        operator: String,
    ) -> Result<Response<MyCustomMsg>, MyStdError> {
        // Your logic here
        // ...
        Ok(Response::new())
    }

    // Deletes an NFT owned by the sender.
    fn burn(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response<MyCustomMsg>, MyStdError> {
        // Your logic here
        // ...
        Ok(Response::new())
    }
}

impl Cw721Query<u64> for MyCw721 {
    fn contract_info(&self, deps: Deps) -> StdResult<ContractInfoResponse> {
        let response = ContractInfoResponse {
            name: "MyToken".to_string(),
            symbol: "MTK".to_string(),
        };

        Ok(response)
    }

    fn num_tokens(&self, _deps: Deps) -> StdResult<NumTokensResponse> {
        Ok(NumTokensResponse { count: 0 }) // Placeholder
    }

    fn nft_info(&self, _deps: Deps, _token_id: String) -> StdResult<NftInfoResponse<u64>> {
        Ok(NftInfoResponse {
            token_uri: None,
            extension: 0,
        }) // Placeholder
    }

    fn owner_of(
        &self,
        deps: Deps,
        env: Env,
        token_id: String,
        _include_expired: bool,
    ) -> StdResult<OwnerOfResponse> {
        let owner = "owner".to_owned();
        let mut _approvals = Vec::new();

        let approval = Approval {
            spender: "operator_address".to_string(),
            expires: Expiration::default(),
        };

        _approvals.push(approval);

        let response = OwnerOfResponse {
            owner: owner.to_string(),
            approvals: _approvals,
        };

        Ok(response)
    }

    fn approval(
        &self,
        deps: Deps,
        env: Env,
        token_id: String,
        spender: String,
        include_expired: bool, // Note that this is a bool, not an Option<bool>
    ) -> StdResult<ApprovalResponse> {
        Err(cosmwasm_std::StdError::generic_err(
            "Method not implemented",
        ))
    }

    fn approvals(
        &self,
        _deps: Deps,
        _env: Env,
        _token_id: String,
        _include_expired: bool, // Changed from Option<bool> to bool
    ) -> StdResult<ApprovalsResponse> {
        Err(cosmwasm_std::StdError::generic_err(
            "Method not implemented",
        ))
    }

    fn operator(
        &self,
        _deps: Deps,
        _env: Env,
        _owner: String,
        _operator: String,
        _include_expired: bool, // Changed from Option<bool> to bool
    ) -> StdResult<OperatorResponse> {
        Ok(OperatorResponse {
            approval: Approval {
                spender: "operator_address".to_string(),
                expires: Expiration::default(),
            },
        })
    }

    fn operators(
        &self,
        _deps: Deps,
        _env: Env,
        _owner: String,
        _include_expired: bool, // Changed from Option<bool> to bool
        _start_after: Option<String>,
        _limit: Option<u32>,
    ) -> StdResult<OperatorsResponse> {
        Ok(OperatorsResponse {
            operators: vec![cw721::Approval {
                spender: "operator_address".to_string(),
                expires: Expiration::default(),
            }],
        })
    }

    fn tokens(
        &self,
        _deps: Deps,
        _owner: String,
        _start_after: Option<String>,
        _limit: Option<u32>,
    ) -> StdResult<TokensResponse> {
        Ok(TokensResponse { tokens: vec![] })
    }

    fn all_tokens(
        &self,
        _deps: Deps,
        _start_after: Option<String>,
        _limit: Option<u32>,
    ) -> StdResult<TokensResponse> {
        Ok(TokensResponse { tokens: vec![] })
    }

    fn all_nft_info(
        &self,
        _deps: Deps,
        _env: Env,
        _token_id: String,
        _include_expired: bool,
    ) -> StdResult<AllNftInfoResponse<u64>> {
        Ok(AllNftInfoResponse {
            access: OwnerOfResponse {
                owner: "owner_address".to_string(),
                approvals: vec![],
            },
            info: NftInfoResponse {
                token_uri: None,
                extension: 0,
            },
        }) // Placeholder
    }
}

impl Default for MyCw721 {
    fn default() -> Self {
        Self {
            artist: String::from("Unknown Artist"),
            album: String::from("Unknown Album"),
            artwork_url: String::from("https://example.com/default_artwork"),
            year: 0,
            track_name: String::from("Unknown Track"),
            audio_track_url: String::from("https://example.com/default_audio"),
        }
    }
}

pub fn query_state(deps: Deps) -> StdResult<MyCw721ContractState> {
    load_state(deps.storage)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MyCw721ContractState {
    pub version: String,
    pub total_supply: u64,
    // New fields
    pub artist: String,
    pub album: String,
    pub artwork_url: String,
    pub year: u32,
    pub track_name: String,
    pub audio_track_url: String,
    pub token_name: String,
    pub token_symbol: String,
    pub platform_fee: Option<u64>,
}
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, MyStdError> {
    let my_cw721_instance = MyCw721 {
        artist: msg.artist.clone(),
        album: msg.album.clone(),
        artwork_url: msg.artwork_url.clone(),
        year: msg.year,
        track_name: msg.track_name.clone(),
        audio_track_url: msg.audio_track_url.clone(),
    };

    let state = MyCw721ContractState {
        version: "1.0".to_string(),
        total_supply: 0,
        artist: msg.artist.clone(),
        album: msg.album.clone(),
        artwork_url: msg.artwork_url.clone(),
        year: msg.year,
        track_name: msg.track_name.clone(),
        audio_track_url: msg.audio_track_url.clone(),
        token_name: msg.token_name.clone(),
        token_symbol: msg.token_symbol.clone(),
        platform_fee: msg.platform_fee,
    };

    // Save the state
    save_state(deps.storage, &state)?;

    // Initialize contract owner
    // Assuming `info.sender` is the initial owner
    let _ownership = initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;

    Ok(Response::new())
}

fn init_cw721(
    storage: &mut dyn Storage,
    api: &dyn Api,
) -> Result<Box<dyn Cw721<u64, MyCustomMsg, Err = MyStdError>>, MyStdError> {
    // Your logic for initializing a Cw721 instance here...
    Ok(Box::new(MyCw721::default()))
}

impl Cw721<u64, MyCustomMsg> for MyCw721 {
    // Your implementation here...
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Uint128;
    use cosmwasm_std::{from_binary, CosmosMsg, WasmMsg};

    use super::*;

    use cosmwasm_std::Empty;
    use cosmwasm_std::{coins, Addr};
    use cw721_base::entry::execute;
    pub use cw721_base::{
        entry::{execute as _execute, query as _query},
        ContractError, Cw721Contract, ExecuteMsg, Extension,
        InstantiateMsg as Cw721BaseInstantiateMsg, MinterResponse,
    };

    use cosmwasm_std::BlockInfo;
    use cosmwasm_std::Timestamp;
    use cw721_base::Ownership;
    use cw_ownable::get_ownership;
    use cw_ownable::initialize_owner;
    use cw_ownable::update_ownership;
    use cw_ownable::Action;

    fn mock_block(height: u64) -> BlockInfo {
        BlockInfo {
            height,
            time: Timestamp::from_seconds(1000),
            chain_id: "testing".to_string(),
        }
    }

    fn setup_contract(deps: DepsMut) {
        // Initialization code here...
    }
    fn mock_block_info(height: u64, time: u64) -> BlockInfo {
        BlockInfo {
            height,
            time: Timestamp::from_seconds(time),
            chain_id: "mockchain".to_string(),
        }
    }

    // Mock addresses for testing
    fn mock_addresses() -> [Addr; 3] {
        [
            Addr::unchecked("addr0000"),
            Addr::unchecked("addr1111"),
            Addr::unchecked("addr2222"),
        ]
    }

    #[test]
    fn test_initialization() {
        // Setup mock dependencies
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Create initialization message
        let init_msg = InstantiateMsg {
            initial_owner: Some(HumanAddrWrapper::from(Addr::unchecked("creator"))),
            artist: "ArtistName".into(),
            album: "AlbumName".into(),
            artwork_url: "http://example.com/artwork.png".into(),
            year: 2021,
            track_name: "Track Name".into(),
            audio_track_url: "http://example.com/audio.mp3".into(),
            token_name: "TokenName".into(),
            token_symbol: "TKN".into(),
            platform_fee: Some(0),
            // ... any other fields you want to initialize
        };

        // Debug Step 1: Print init_msg to see if it is correct
        println!("Debug Step 1 - init_msg: {:?}", init_msg);

        // Instantiate the contract
        let instantiate_result = instantiate(deps.as_mut(), env.clone(), info, init_msg.clone());

        // Debug Step 2: Print the instantiate result
        println!(
            "Debug Step 2 - instantiate_result: {:?}",
            instantiate_result
        );

        assert!(
            instantiate_result.is_ok(),
            "Failed to instantiate the contract: {:?}",
            instantiate_result.unwrap_err()
        );

        // Load the config
        let config_result = load_state(deps.as_ref().storage);

        // Debug Step 3: Print the config result
        println!("Debug Step 3 - config_result: {:?}", config_result);

        assert!(
            config_result.is_ok(),
            "Failed to load config: {:?}",
            config_result.unwrap_err()
        );

        let real_config = config_result.unwrap(); // Assuming load_config returns Result<Option<T>, E>

        // Validate that the config has the expected values
        assert_eq!(real_config.artist, init_msg.artist);
        assert_eq!(real_config.album, init_msg.album);
        assert_eq!(real_config.artwork_url, init_msg.artwork_url);
        assert_eq!(real_config.year, init_msg.year as u32);

        assert_eq!(real_config.track_name, init_msg.track_name);
        assert_eq!(real_config.audio_track_url, init_msg.audio_track_url);
        assert_eq!(real_config.token_name, init_msg.token_name);
        assert_eq!(real_config.token_symbol, init_msg.token_symbol);
        assert_eq!(real_config.platform_fee, init_msg.platform_fee);

        // Load the ownership info
        let ownership_result = get_ownership(deps.as_ref().storage);

        // Debug Step 4: Print the ownership result
        println!("Debug Step 4 - ownership_result: {:?}", ownership_result);

        assert!(
            ownership_result.is_ok(),
            "Failed to get ownership: {:?}",
            ownership_result.unwrap_err()
        );

        let ownership = ownership_result.unwrap();
        assert_eq!(ownership.owner.unwrap(), Addr::unchecked("creator"));
    }

    #[test]
    fn test_serialization_deserialization() {
        let config = Config {
            version: "1.0".to_string(),
            artist: "ArtistName".to_string(),
            album: "AlbumName".to_string(),
            artwork_url: "http://example.com/artwork.png".to_string(),
            year: 2021,
            track_name: "Track Name".to_string(),
            audio_track_url: "http://example.com/audio.mp3".to_string(),
            token_name: "TokenName".to_string(),
            token_symbol: "TKN".to_string(),
            platform_fee: Some(0),
        };

        let data = to_vec(&config).unwrap();
        let deserialized_config: Config = from_slice(&data).unwrap();
        assert_eq!(config, deserialized_config);
    }
    #[test]
    fn test_initialize_ownership() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(0, "ust"));

        // Initialize the contract
        // Assume `instantiate` is your instantiate function and it initializes the owner
        // Instantiate the contract
        let init_msg = InstantiateMsg {
            initial_owner: Some(HumanAddrWrapper::from(Addr::unchecked("creator"))),
            artist: "ArtistName".into(),
            album: "AlbumName".into(),
            artwork_url: "http://example.com/artwork.png".into(),
            year: 2021,
            track_name: "Track Name".into(),
            audio_track_url: "http://example.com/audio.mp3".into(),
            token_name: "TokenName".into(),
            token_symbol: "TKN".into(),
            platform_fee: Some(0),
            // ... any other fields you want to initialize
        };

        let instantiate_res = instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg);
        assert!(instantiate_res.is_ok());

        // Check if the owner is initialized correctly
        let ownership = get_ownership(deps.as_ref().storage).unwrap();
        assert_eq!(ownership.owner.unwrap(), Addr::unchecked("creator"));
    }

    #[test]
    fn test_accept_ownership() {
        let mut deps = mock_dependencies();
        let [initial_owner, new_owner, _] = mock_addresses();

        // Step 1: Initialize the contract with an initial owner
        let ownership = Ownership {
            owner: Some(initial_owner.clone()),
            pending_owner: None,
            pending_expiry: None,
        };
        const OWNERSHIP: Item<Ownership<Addr>> = Item::new("ownership");
        OWNERSHIP.save(deps.as_mut().storage, &ownership).unwrap();

        // Step 2: Propose a new owner
        let transfer_action = Action::TransferOwnership {
            new_owner: new_owner.to_string(),
            expiry: None,
        };
        update_ownership(
            deps.as_mut(),
            &mock_block_info(1, 1000),
            &initial_owner,
            transfer_action,
        )
        .unwrap();

        // Step 3: Accept the new ownership
        let accept_action = Action::AcceptOwnership;
        let new_ownership = update_ownership(
            deps.as_mut(),
            &mock_block_info(2, 2000),
            &new_owner,
            accept_action,
        )
        .unwrap();

        // Validate that owner is now new_owner
        assert_eq!(new_ownership.owner, Some(new_owner));
        assert_eq!(new_ownership.pending_owner, None);
    }

    #[test]
    fn test_renounce_ownership() {
        let mut deps = mock_dependencies();
        let env: Env = mock_env(); // You might need to set specific fields of the Env
        let info: MessageInfo = mock_info("creator", &coins(0, "ust"));

        // Initialize contract with an owner
        let initial_owner = "creator";
        initialize_owner(&mut deps.storage, &deps.api, Some(initial_owner)).unwrap();

        // Renounce ownership
        let renounce_msg = Action::RenounceOwnership {
        // additional fields if any
    };

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            cw721_base::ExecuteMsg::UpdateOwnership(renounce_msg),
        );
        assert!(res.is_ok());

        // Check if ownership has been renounced correctly
        let ownership = get_ownership(deps.as_ref().storage).unwrap();
        assert_eq!(ownership.owner, None);
    }

    #[test]
fn test_mint() {
    // Setup
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(0, "ust"));

    // Initialize ownership
    let init_res = initialize_owner(&mut deps.storage, &deps.api, Some("creator"));
    assert!(init_res.is_ok(), "Initialization failed: {:?}", init_res.err());

    // Mint a new token
    let mint_msg = ExecuteMsg::Mint {
        token_id: "token123".into(),
        owner: "owner123".into(),
        token_uri: Some("uri".into()),
        extension: None,
    };

    let mint_res = execute(deps.as_mut(), env.clone(), info.clone(), mint_msg);
    assert!(mint_res.is_ok(), "Minting failed: {:?}", mint_res.err());

    // Validate token owner
    let owner = get_owner_of_token(&deps.storage, "token123").expect("Failed to get owner of token");
    assert_eq!(owner, Some(Addr::unchecked("owner123")), "Owner mismatch");
}

#[test]
fn test_transfer_nft() {
    // Setup
    let mut deps = mock_dependencies();
    let env = mock_env();
    let [owner, recipient, _] = mock_addresses();
    let info = mock_info(owner.as_str(), &coins(0, "ust"));

    // Initialize Ownership
    let init_res = initialize_owner(&mut deps.storage, &deps.api, Some(owner.as_str()));
    assert!(init_res.is_ok(), "Initialization failed: {:?}", init_res.err());

    // Mint a Token
    let mint_msg = ExecuteMsg::Mint {
        token_id: "token123".into(),
        owner: owner.to_string(),
        token_uri: Some("uri".into()),
        extension: None,
    };
    let mint_res = execute(deps.as_mut(), env.clone(), info.clone(), mint_msg);
    assert!(mint_res.is_ok(), "Minting failed: {:?}", mint_res.err());

    // Validate Initial Owner
    let initial_owner = get_owner_of_token(&deps.storage, "token123").expect("Failed to get initial owner");
    assert_eq!(initial_owner, Some(owner.clone()), "Initial owner mismatch");

    // Transfer Ownership
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: recipient.to_string(),
        token_id: "token123".into(),
    };
    let transfer_res = execute(deps.as_mut(), env.clone(), info.clone(), transfer_msg);
    assert!(transfer_res.is_ok(), "Transfer failed: {:?}", transfer_res.err());

    // Validate New Owner
    let new_owner = get_owner_of_token(&deps.storage, "token123").expect("Failed to get new owner");
    assert_eq!(new_owner, Some(recipient.clone()), "New owner mismatch");
}

}

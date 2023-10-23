# README.md

## Project Summary

This project is a Rust-based implementation of a Cosmos SDK contract for a NFT (Non-Fungible Token) standard, CW-721. The contract is written to provide a set of functionalities to create, transfer, and manage unique digital assets. These assets can be music tracks, identified by a unique token ID.

## Features

- **Token Minting**: Create new tokens with specified metadata such as artist, album, year, track name, and audio track URL.
- **Token Burning**: Destroy existing tokens.
- **Token Transfer**: Transfer tokens between accounts.
- **Ownership Query**: Query the current owner of a token.
- **Approval Mechanisms**: Approve and revoke permissions to transfer tokens on behalf of the owner.
- **State Management**: Maintain and update the contract's state.
- **Detailed Metadata**: Includes detailed metadata like artist, album, year, and so on for each token.

## Technologies Used

- Rust
- CosmWasm
- CW-721 Standard for NFTs

## Errors and Issues

The codebase contains several issues that need to be addressed for full functionality:

- **Deserialization Errors**: The code often fails to deserialize storage data into the desired types. This occurs in functions like `query_config`, `read_token_info`, and `get_owner_of_token`.
- **Data Consistency**: The data saved in storage sometimes is inconsistent with the data being used, as seen in the debug logs.
- **Incomplete Logic**: Functions like `query_number_of_tokens_owned_by`, `get_all_approvals_for_token`, and `get_all_tokens_by_owner` are incomplete and contain placeholder code.
- **State Management**: The contract's state isn't properly updated in several scenarios like minting, transferring, and burning tokens.
- **Debug Prints**: The code contains several debug prints which could be removed or converted to proper logging for production.
- **Error Handling**: The code does not have robust error handling and debugging mechanisms in place. For instance, generic errors are thrown, which do not provide detailed context for debugging.
- **Type Safety**: Some function parameters have loose types like `String` where more specific types like `Addr` could be used for better type safety.

## Recommendations for Future Work

- Fix the deserialization issues by ensuring that the data types and storage types match.
- Complete the logic for all unfinished functions.
- Implement robust error-handling and logging mechanisms.
- Update the contract's state properly after each transaction.
- Refactor code for better type safety and modularity.
- Add unit tests and integration tests to validate the contract's behavior.

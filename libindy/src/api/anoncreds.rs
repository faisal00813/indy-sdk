use indy_api_types::{ErrorCode, IndyHandle, CommandHandle, WalletHandle, SearchHandle};
use indy_api_types::errors::prelude::*;
use crate::commands::{Command, CommandExecutor};
use crate::commands::anoncreds::AnoncredsCommand;
use crate::commands::anoncreds::prover::ProverCommand;
use crate::commands::anoncreds::verifier::VerifierCommand;
use crate::domain::anoncreds::schema::{Schema, AttributeNames, Schemas};
use crate::domain::crypto::did::DidValue;
use crate::domain::anoncreds::credential_definition::{CredentialDefinition, CredentialDefinitionConfig, CredentialDefinitionId, CredentialDefinitions};
use crate::domain::anoncreds::credential_offer::CredentialOffer;
use crate::domain::anoncreds::credential_request::{CredentialRequest, CredentialRequestMetadata};
use crate::domain::anoncreds::credential_attr_tag_policy::CredentialAttrTagPolicy;
use crate::domain::anoncreds::credential::{Credential, CredentialValues};
use crate::domain::anoncreds::revocation_registry_definition::{RevocationRegistryConfig, RevocationRegistryDefinition, RevocationRegistryId, RevocationRegistryDefinitions};
use crate::domain::anoncreds::revocation_registry_delta::RevocationRegistryDelta;
use crate::domain::anoncreds::proof::Proof;
use crate::domain::anoncreds::proof_request::{ProofRequest, ProofRequestExtraQuery};
use crate::domain::anoncreds::requested_credential::RequestedCredentials;
use crate::domain::anoncreds::revocation_registry::RevocationRegistries;
use crate::domain::anoncreds::revocation_state::{RevocationState, RevocationStates};
use indy_utils::ctypes;

use libc::c_char;
use std::ptr;

use crate::indy_api_types::validation::Validatable;

/*
These functions wrap the Ursa algorithm as documented in this paper:
https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf

And is documented in this HIPE:
https://github.com/hyperledger/indy-hipe/blob/c761c583b1e01c1e9d3ceda2b03b35336fdc8cc1/text/anoncreds-protocol/README.md
*/


/// Creates a master secret with a given id and stores it in the wallet.
/// The id must be unique.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// master_secret_id: (optional, if not present random one will be generated) new master id
///
/// #Returns
/// out_master_secret_id: Id of generated master secret
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_create_master_secret(command_handle: CommandHandle,
                                               wallet_handle: WalletHandle,
                                               master_secret_id: *const c_char,
                                               cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                    out_master_secret_id: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_create_master_secret: >>> wallet_handle: {:?}, master_secret_id: {:?}", wallet_handle, master_secret_id);

    check_useful_opt_c_str!(master_secret_id, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_create_master_secret: entities >>> wallet_handle: {:?}, master_secret_id: {:?}", wallet_handle, master_secret_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::CreateMasterSecret(
                    wallet_handle,
                    master_secret_id,
                    boxed_callback_string!("indy_prover_create_master_secret", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_create_master_secret: <<< res: {:?}", res);

    res
}

/// Creates a credential request for the given credential offer.
///
/// The method creates a blinded master secret for a master secret identified by a provided name.
/// The master secret identified by the name must be already stored in the secure wallet (see prover_create_master_secret)
/// The blinded master secret is a part of the credential request.
///
/// #Params
/// command_handle: command handle to map callback to user context
/// wallet_handle: wallet handle (created by open_wallet)
/// prover_did: a DID of the prover
/// cred_offer_json: credential offer as a json containing information about the issuer and a credential
///     {
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///          ...
///         Other fields that contains data structures internal to Ursa.
///         These fields should not be parsed and are likely to change in future versions.
///     }
/// cred_def_json: credential definition json related to <cred_def_id> in <cred_offer_json>
/// master_secret_id: the id of the master secret stored in the wallet
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// cred_req_json: Credential request json for creation of credential by Issuer
///     {
///      "prover_did" : string,
///      "cred_def_id" : string,
///         // Fields below can depend on Cred Def type
///      "blinded_ms" : <blinded_master_secret>,
///                     (opaque type that contains data structures internal to Ursa.
///                      It should not be parsed and are likely to change in future versions).
///      "blinded_ms_correctness_proof" : <blinded_ms_correctness_proof>,
///                     (opaque type that contains data structures internal to Ursa.
///                      It should not be parsed and are likely to change in future versions).
///      "nonce": string
///    }
/// cred_req_metadata_json: Credential request metadata json for further processing of received form Issuer credential.
///     Credential request metadata contains data structures internal to Ursa.
///     Credential request metadata mustn't be shared with Issuer.
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_create_credential_req(command_handle: CommandHandle,
                                                wallet_handle: WalletHandle,
                                                prover_did: *const c_char,
                                                cred_offer_json: *const c_char,
                                                cred_def_json: *const c_char,
                                                master_secret_id: *const c_char,
                                                cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                     cred_req_json: *const c_char,
                                                                     cred_req_metadata_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_create_credential_req: >>> wallet_handle: {:?}, prover_did: {:?}, cred_offer_json: {:?}, cred_def_json: {:?}, master_secret_id: {:?}",
           wallet_handle, prover_did, cred_offer_json, cred_def_json, master_secret_id);

    check_useful_validatable_string!(prover_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_validatable_json!(cred_offer_json, ErrorCode::CommonInvalidParam4, CredentialOffer);
    check_useful_validatable_json!(cred_def_json, ErrorCode::CommonInvalidParam5, CredentialDefinition);
    check_useful_c_str!(master_secret_id, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_prover_create_credential_req: entities >>> wallet_handle: {:?}, prover_did: {:?}, cred_offer_json: {:?}, cred_def_json: {:?}, master_secret_id: {:?}",
           wallet_handle, prover_did, cred_offer_json, cred_def_json, master_secret_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::CreateCredentialRequest(
                    wallet_handle,
                    prover_did,
                    cred_offer_json,
                    cred_def_json,
                    master_secret_id,
                    Box::new(move |result| {
                        let (err, cred_req_json, cred_req_metadata_json) = prepare_result_2!(result, String::new(), String::new());
                        trace!("indy_prover_create_credential_req: cred_req_json: {:?}, cred_req_metadata_json: {:?}", cred_req_json, cred_req_metadata_json);
                        let cred_req_json = ctypes::string_to_cstring(cred_req_json);
                        let cred_req_metadata_json = ctypes::string_to_cstring(cred_req_metadata_json);
                        cb(command_handle, err, cred_req_json.as_ptr(), cred_req_metadata_json.as_ptr())
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_create_credential_req: <<< res: {:?}", res);

    res
}

/// Set credential attribute tagging policy.
/// Writes a non-secret record marking attributes to tag, and optionally
/// updates tags on existing credentials on the credential definition to match.
///
/// EXPERIMENTAL
///
/// The following tags are always present on write:
///     {
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///     }
///
/// The policy sets the following tags for each attribute it marks taggable, written to subsequent
/// credentials and (optionally) all existing credentials on the credential definition:
///     {
///         "attr::<attribute name>::marker": "1",
///         "attr::<attribute name>::value": <attribute raw value>,
///     }
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_def_id: credential definition id
/// tag_attrs_json: JSON array with names of attributes to tag by policy, or null for all
/// retroactive: boolean, whether to apply policy to existing credentials on credential definition identifier
/// cb: Callback that takes command result as parameter.
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_set_credential_attr_tag_policy(command_handle: CommandHandle,
                                                         wallet_handle: WalletHandle,
                                                         cred_def_id: *const c_char,
                                                         tag_attrs_json: *const c_char,
                                                         retroactive: bool,
                                                         cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode)>) -> ErrorCode {
    trace!("indy_prover_set_credential_attr_tag_policy: >>> wallet_handle: {:?}, cred_def_id: {:?}, tag_attrs_json: {:?}, retroactive: {:?}", wallet_handle, cred_def_id, tag_attrs_json, retroactive);

    check_useful_validatable_string!(cred_def_id, ErrorCode::CommonInvalidParam3, CredentialDefinitionId);
    check_useful_opt_json!(tag_attrs_json, ErrorCode::CommonInvalidParam4, CredentialAttrTagPolicy);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_prover_set_credential_attr_tag_policy: entities >>> wallet_handle: {:?}, cred_def_id: {:?}, tag_attrs_json: {:?}, retroactive: {:?}",
           wallet_handle, cred_def_id, tag_attrs_json, retroactive);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::SetCredentialAttrTagPolicy(
                    wallet_handle,
                    cred_def_id,
                    tag_attrs_json,
                    retroactive,
                    Box::new(move |result| {
                        let err = prepare_result!(result);
                        trace!("indy_prover_set_credential_attr_tag_policy: ");
                        cb(command_handle, err)
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_set_credential_attr_tag_policy: <<< res: {:?}", res);

    res
}

/// Get credential attribute tagging policy by credential definition id.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_def_id: credential definition id
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// JSON array with all attributes that current policy marks taggable;
/// null for default policy (tag all credential attributes).
/// 
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_get_credential_attr_tag_policy(command_handle: CommandHandle,
                                                         wallet_handle: WalletHandle,
                                                         cred_def_id: *const c_char,
                                                         cb: Option<extern fn(command_handle_: CommandHandle,
                                                                              err: ErrorCode,
                                                                              catpol_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_get_credential_attr_tag_policy: >>> wallet_handle: {:?}, cred_def_id: {:?}", wallet_handle, cred_def_id);

    check_useful_validatable_string!(cred_def_id, ErrorCode::CommonInvalidParam3, CredentialDefinitionId);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_get_credential_attr_tag_policy: entities >>> wallet_handle: {:?}, cred_def_id: {:?}", wallet_handle, cred_def_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::GetCredentialAttrTagPolicy(
                    wallet_handle,
                    cred_def_id,
                    boxed_callback_string!("indy_prover_get_credential_attr_tag_policy", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_get_credential_attr_tag_policy: <<< res: {:?}", res);

    res
}

/// Check credential provided by Issuer for the given credential request,
/// updates the credential by a master secret and stores in a secure wallet.
///
/// To support efficient and flexible search the following tags will be created for stored credential:
///     {
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///         // for every attribute in <credential values> that credential attribute tagging policy marks taggable
///         "attr::<attribute name>::marker": "1",
///         "attr::<attribute name>::value": <attribute raw value>,
///     }
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_id: (optional, default is a random one) identifier by which credential will be stored in the wallet
/// cred_req_metadata_json: a credential request metadata created by indy_prover_create_credential_req
/// cred_json: credential json received from issuer
///     {
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_def_id", Optional<string>, - identifier of revocation registry
///         "values": - credential values
///             {
///                 "attr1" : {"raw": "value1", "encoded": "value1_as_int" },
///                 "attr2" : {"raw": "value1", "encoded": "value1_as_int" }
///             }
///         // Fields below can depend on Cred Def type
///         Other fields that contains data structures internal to Ursa.
///         These fields should not be parsed and are likely to change in future versions.
///     }
/// cred_def_json: credential definition json related to <cred_def_id> in <cred_json>
/// rev_reg_def_json: revocation registry definition json related to <rev_reg_def_id> in <cred_json>
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// out_cred_id: identifier by which credential is stored in the wallet
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_store_credential(command_handle: CommandHandle,
                                           wallet_handle: WalletHandle,
                                           cred_id: *const c_char,
                                           cred_req_metadata_json: *const c_char,
                                           cred_json: *const c_char,
                                           cred_def_json: *const c_char,
                                           rev_reg_def_json: *const c_char,
                                           cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                out_cred_id: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_store_credential: >>> wallet_handle: {:?}, cred_id: {:?}, cred_req_metadata_json: {:?}, cred_json: {:?}, cred_def_json: {:?}, \
    cred_def_json: {:?}", wallet_handle, cred_id, cred_req_metadata_json, cred_json, cred_def_json, rev_reg_def_json);

    check_useful_opt_c_str!(cred_id, ErrorCode::CommonInvalidParam3);
    check_useful_validatable_json!(cred_req_metadata_json, ErrorCode::CommonInvalidParam4, CredentialRequestMetadata);
    check_useful_validatable_json!(cred_json, ErrorCode::CommonInvalidParam5, Credential);
    check_useful_validatable_json!(cred_def_json, ErrorCode::CommonInvalidParam6, CredentialDefinition);
    check_useful_opt_validatable_json!(rev_reg_def_json, ErrorCode::CommonInvalidParam7, RevocationRegistryDefinition);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    trace!("indy_prover_store_credential: entities >>> wallet_handle: {:?}, cred_id: {:?}, cred_req_metadata_json: {:?}, cred_json: {:?}, cred_def_json: {:?}, \
    rev_reg_def_json: {:?}", wallet_handle, cred_id, cred_req_metadata_json, cred_json, cred_def_json, rev_reg_def_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::StoreCredential(
                    wallet_handle,
                    cred_id,
                    cred_req_metadata_json,
                    cred_json,
                    cred_def_json,
                    rev_reg_def_json,
                    boxed_callback_string!("indy_prover_store_credential", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_store_credential: <<< res: {:?}", res);

    res
}

/// Gets human readable credential by the given id.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_id: Identifier by which requested credential is stored in the wallet
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// credential json:
///     {
///         "referent": string, - id of credential in the wallet
///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
///     }
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_get_credential(command_handle: CommandHandle,
                                         wallet_handle: WalletHandle,
                                         cred_id: *const c_char,
                                         cb: Option<extern fn(
                                             command_handle_: CommandHandle, err: ErrorCode,
                                             credential_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_get_credential: >>> wallet_handle: {:?}, cred_id: {:?}", wallet_handle, cred_id);

    check_useful_c_str!(cred_id, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_get_credential: entities >>> wallet_handle: {:?}, cred_id: {:?}", wallet_handle, cred_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::GetCredential(
                    wallet_handle,
                    cred_id,
                    boxed_callback_string!("indy_prover_get_credential", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_get_credential: <<< res: {:?}", res);

    res
}

/// Deletes credential by given id.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_id: Identifier by which requested credential is stored in the wallet
/// cb: Callback that takes command result as parameter.
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_delete_credential(command_handle: CommandHandle,
                                            wallet_handle: WalletHandle,
                                            cred_id: *const c_char,
                                            cb: Option<extern fn(
                                                command_handle_: CommandHandle,
                                                err: ErrorCode)>) -> ErrorCode {
    trace!("indy_prover_delete_credential: >>> wallet_handle: {:?}, cred_id: {:?}", wallet_handle, cred_id);

    check_useful_c_str!(cred_id, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::DeleteCredential(
                    wallet_handle,
                    cred_id,
                    Box::new(move |result| {
                        let err = prepare_result!(result);
                        trace!("indy_prover_delete_credential: ");
                        cb(command_handle, err)
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_delete_credential: <<< res: {:?}", res);

    res
}

/// Gets human readable credentials according to the filter.
/// If filter is NULL, then all credentials are returned.
/// Credentials can be filtered by Issuer, credential_def and/or Schema.
///
/// NOTE: This method is deprecated because immediately returns all fetched credentials.
/// Use <indy_prover_search_credentials> to fetch records by small batches.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// filter_json: filter for credentials
///        {
///            "schema_id": string, (Optional)
///            "schema_issuer_did": string, (Optional)
///            "schema_name": string, (Optional)
///            "schema_version": string, (Optional)
///            "issuer_did": string, (Optional)
///            "cred_def_id": string, (Optional)
///        }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// credentials json
///     [{
///         "referent": string, - id of credential in the wallet
///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
///     }]
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
#[deprecated(since = "1.6.1", note = "Please use indy_prover_search_credentials instead!")]
pub extern fn indy_prover_get_credentials(command_handle: CommandHandle,
                                          wallet_handle: WalletHandle,
                                          filter_json: *const c_char,
                                          cb: Option<extern fn(
                                              command_handle_: CommandHandle, err: ErrorCode,
                                              matched_credentials_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_get_credentials: >>> wallet_handle: {:?}, filter_json: {:?}", wallet_handle, filter_json);

    check_useful_opt_c_str!(filter_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_get_credentials: entities >>> wallet_handle: {:?}, filter_json: {:?}", wallet_handle, filter_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::GetCredentials(
                    wallet_handle,
                    filter_json,
                    boxed_callback_string!("indy_prover_get_credentials", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_get_credentials: <<< res: {:?}", res);

    res
}

/// Search for credentials stored in wallet.
/// Credentials can be filtered by tags created during saving of credential.
///
/// Instead of immediately returning of fetched credentials
/// this call returns search_handle that can be used later
/// to fetch records by small batches (with indy_prover_fetch_credentials).
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// query_json: Wql query filter for credentials searching based on tags.
///     where query: indy-sdk/docs/design/011-wallet-query-language/README.md
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// search_handle: Search handle that can be used later to fetch records by small batches (with indy_prover_fetch_credentials)
/// total_count: Total count of records
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_search_credentials(command_handle: CommandHandle,
                                             wallet_handle: WalletHandle,
                                             query_json: *const c_char,
                                             cb: Option<extern fn(
                                                 command_handle_: CommandHandle, err: ErrorCode,
                                                 search_handle: SearchHandle,
                                                 total_count: usize)>) -> ErrorCode {
    trace!("indy_prover_search_credentials: >>> wallet_handle: {:?}, query_json: {:?}", wallet_handle, query_json);

    check_useful_opt_c_str!(query_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_search_credentials: entities >>> wallet_handle: {:?}, query_json: {:?}", wallet_handle, query_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::SearchCredentials(
                    wallet_handle,
                    query_json,
                    Box::new(move |result| {
                        let (err, handle, total_count) = prepare_result_2!(result, 0, 0);
                        cb(command_handle, err, handle, total_count)
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_search_credentials: <<< res: {:?}", res);

    res
}

/// Fetch next credentials for search.
///
/// #Params
/// search_handle: Search handle (created by indy_prover_search_credentials)
/// count: Count of credentials to fetch
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// credentials_json: List of human readable credentials:
///     [{
///         "referent": string, - id of credential in the wallet
///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
///     }]
/// NOTE: The list of length less than the requested count means credentials search iterator is completed.
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_prover_fetch_credentials(command_handle: CommandHandle,
                                             search_handle: SearchHandle,
                                             count: usize,
                                             cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                  credentials_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_fetch_credentials: >>> search_handle: {:?}, count: {:?}", search_handle, count);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_fetch_credentials: entities >>> search_handle: {:?}, count: {:?}", search_handle, count);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::FetchCredentials(
                    search_handle,
                    count,
                    boxed_callback_string!("indy_prover_fetch_credentials", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_fetch_credentials: <<< res: {:?}", res);

    res
}

/// Close credentials search (make search handle invalid)
///
/// #Params
/// search_handle: Search handle (created by indy_prover_search_credentials)
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_prover_close_credentials_search(command_handle: CommandHandle,
                                                    search_handle: SearchHandle,
                                                    cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode)>) -> ErrorCode {
    trace!("indy_prover_close_credentials_search: >>> search_handle: {:?}", search_handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_prover_close_credentials_search: entities >>> search_handle: {:?}", search_handle);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::CloseCredentialsSearch(
                    search_handle,
                    Box::new(move |result| {
                        let err = prepare_result!(result);
                        trace!("indy_prover_close_credentials_search:");
                        cb(command_handle, err)
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_close_credentials_search: <<< res: {:?}", res);

    res
}

/// Gets human readable credentials matching the given proof request.
///
/// NOTE: This method is deprecated because immediately returns all fetched credentials.
/// Use <indy_prover_search_credentials_for_proof_req> to fetch records by small batches.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// proof_request_json: proof request json
///     {
///         "name": string,
///         "version": string,
///         "nonce": string, - a big number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
///         "requested_attributes": { // set of requested attributes
///              "<attr_referent>": <attr_info>, // see below
///              ...,
///         },
///         "requested_predicates": { // set of requested predicates
///              "<predicate_referent>": <predicate_info>, // see below
///              ...,
///          },
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (applies to every attribute and predicate but can be overridden on attribute level),
///         "ver": Optional<str>  - proof request version:
///             - omit or "1.0" to use unqualified identifiers for restrictions
///             - "2.0" to use fully qualified identifiers for restrictions
///     }
/// cb: Callback that takes command result as parameter.
///
/// where
/// attr_referent: Proof-request local identifier of requested attribute
/// attr_info: Describes requested attribute
///     {
///         "name": string, // attribute name, (case insensitive and ignore spaces)
///         "restrictions": Optional<filter_json>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// predicate_referent: Proof-request local identifier of requested attribute predicate
/// predicate_info: Describes requested attribute predicate
///     {
///         "name": attribute name, (case insensitive and ignore spaces)
///         "p_type": predicate type (">=", ">", "<=", "<")
///         "p_value": int predicate value
///         "restrictions": Optional<filter_json>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// non_revoc_interval: Defines non-revocation interval
///     {
///         "from": Optional<int>, // timestamp of interval beginning
///         "to": Optional<int>, // timestamp of interval ending
///     }
///  filter_json:
///     {
///        "schema_id": string, (Optional)
///        "schema_issuer_did": string, (Optional)
///        "schema_name": string, (Optional)
///        "schema_version": string, (Optional)
///        "issuer_did": string, (Optional)
///        "cred_def_id": string, (Optional)
///     }
///
/// #Returns
/// credentials_json: json with credentials for the given proof request.
///     {
///         "attrs": {
///             "<attr_referent>": [{ cred_info: <credential_info>, interval: Optional<non_revoc_interval> }],
///             ...,
///         },
///         "predicates": {
///             "requested_predicates": [{ cred_info: <credential_info>, timestamp: Optional<integer> }, { cred_info: <credential_2_info>, timestamp: Optional<integer> }],
///             "requested_predicate_2_referent": [{ cred_info: <credential_2_info>, timestamp: Optional<integer> }]
///         }
///     }, where <credential_info> is
///     {
///         "referent": string, - id of credential in the wallet
///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
///     }
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[deprecated(since = "1.6.1", note = "Please use indy_prover_search_credentials_for_proof_req instead!")]
#[no_mangle]
pub extern fn indy_prover_get_credentials_for_proof_req(command_handle: CommandHandle,
                                                        wallet_handle: WalletHandle,
                                                        proof_request_json: *const c_char,
                                                        cb: Option<extern fn(
                                                            command_handle_: CommandHandle, err: ErrorCode,
                                                            credentials_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_get_credentials_for_proof_req: >>> wallet_handle: {:?}, proof_request_json: {:?}", wallet_handle, proof_request_json);

    check_useful_validatable_json!(proof_request_json, ErrorCode::CommonInvalidParam3, ProofRequest);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_prover_get_credentials_for_proof_req: entities >>> wallet_handle: {:?}, proof_request_json: {:?}",
           wallet_handle, proof_request_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::GetCredentialsForProofReq(
                    wallet_handle,
                    proof_request_json,
                    boxed_callback_string!("indy_prover_get_credentials_for_proof_req", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_get_credentials_for_proof_req: <<< res: {:?}", res);

    res
}

/// Search for credentials matching the given proof request.
///
/// Instead of immediately returning of fetched credentials
/// this call returns search_handle that can be used later
/// to fetch records by small batches (with indy_prover_fetch_credentials_for_proof_req).
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// proof_request_json: proof request json
///     {
///         "name": string,
///         "version": string,
///         "nonce": string, - a big number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
///         "requested_attributes": { // set of requested attributes
///              "<attr_referent>": <attr_info>, // see below
///              ...,
///         },
///         "requested_predicates": { // set of requested predicates
///              "<predicate_referent>": <predicate_info>, // see below
///              ...,
///          },
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (applies to every attribute and predicate but can be overridden on attribute level)
///                        // (can be overridden on attribute level)
///         "ver": Optional<str>  - proof request version:
///             - omit or "1.0" to use unqualified identifiers for restrictions
///             - "2.0" to use fully qualified identifiers for restrictions
///     }
///
/// where
/// attr_info: Describes requested attribute
///     {
///         "name": string, // attribute name, (case insensitive and ignore spaces)
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// predicate_referent: Proof-request local identifier of requested attribute predicate
/// predicate_info: Describes requested attribute predicate
///     {
///         "name": attribute name, (case insensitive and ignore spaces)
///         "p_type": predicate type (">=", ">", "<=", "<")
///         "p_value": predicate value
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// non_revoc_interval: Defines non-revocation interval
///     {
///         "from": Optional<int>, // timestamp of interval beginning
///         "to": Optional<int>, // timestamp of interval ending
///     }
/// extra_query_json:(Optional) List of extra queries that will be applied to correspondent attribute/predicate:
///     {
///         "<attr_referent>": <wql query>,
///         "<predicate_referent>": <wql query>,
///     }
/// where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
///     The list of allowed fields:
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// search_handle: Search handle that can be used later to fetch records by small batches (with indy_prover_fetch_credentials_for_proof_req)
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_search_credentials_for_proof_req(command_handle: CommandHandle,
                                                           wallet_handle: WalletHandle,
                                                           proof_request_json: *const c_char,
                                                           extra_query_json: *const c_char,
                                                           cb: Option<extern fn(
                                                               command_handle_: CommandHandle, err: ErrorCode,
                                                               search_handle: SearchHandle)>) -> ErrorCode {
    trace!("indy_prover_search_credentials_for_proof_req: >>> wallet_handle: {:?}, proof_request_json: {:?}, extra_query_json: {:?}", wallet_handle, proof_request_json, extra_query_json);

    check_useful_validatable_json!(proof_request_json, ErrorCode::CommonInvalidParam3, ProofRequest);
    check_useful_opt_json!(extra_query_json, ErrorCode::CommonInvalidParam4, ProofRequestExtraQuery);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_prover_search_credentials_for_proof_req: entities >>> wallet_handle: {:?}, proof_request_json: {:?}, extra_query_json: {:?}",
           wallet_handle, proof_request_json, extra_query_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::SearchCredentialsForProofReq(
                    wallet_handle,
                    proof_request_json,
                    extra_query_json,
                    Box::new(move |result| {
                        let (err, search_handle) = prepare_result_1!(result, 0);
                        trace!("indy_prover_search_credentials_for_proof_req: search_handle: {:?}", search_handle);
                        cb(command_handle, err, search_handle)
                    }),
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_search_credentials_for_proof_req: <<< res: {:?}", res);

    res
}

/// Fetch next credentials for the requested item using proof request search
/// handle (created by indy_prover_search_credentials_for_proof_req).
///
/// #Params
/// search_handle: Search handle (created by indy_prover_search_credentials_for_proof_req)
/// item_referent: Referent of attribute/predicate in the proof request
/// count: Count of credentials to fetch
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// credentials_json: List of credentials for the given proof request.
///     [{
///         cred_info: <credential_info>,
///         interval: Optional<non_revoc_interval>
///     }]
/// where
/// credential_info:
///     {
///         "referent": string, - id of credential in the wallet
///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
///     }
/// non_revoc_interval:
///     {
///         "from": Optional<int>, // timestamp of interval beginning
///         "to": Optional<int>, // timestamp of interval ending
///     }
/// NOTE: The list of length less than the requested count means that search iterator
/// correspondent to the requested <item_referent> is completed.
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_prover_fetch_credentials_for_proof_req(command_handle: CommandHandle,
                                                           search_handle: SearchHandle,
                                                           item_referent: *const c_char,
                                                           count: usize,
                                                           cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                                credentials_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_fetch_credentials_for_proof_req: >>> search_handle: {:?}, count: {:?}", search_handle, count);

    check_useful_c_str!(item_referent, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_prover_fetch_credentials_for_proof_req: entities >>> search_handle: {:?}, count: {:?}", search_handle, count);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::FetchCredentialForProofReq(
                    search_handle,
                    item_referent,
                    count,
                    boxed_callback_string!("indy_prover_fetch_credentials_for_proof_request", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_fetch_credentials_for_proof_req: <<< res: {:?}", res);

    res
}

/// Close credentials search for proof request (make search handle invalid)
///
/// #Params
/// search_handle: Search handle (created by indy_prover_search_credentials_for_proof_req)
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_prover_close_credentials_search_for_proof_req(command_handle: CommandHandle,
                                                                  search_handle: SearchHandle,
                                                                  cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode)>) -> ErrorCode {
    trace!("indy_prover_close_credentials_search_for_proof_req: >>> search_handle: {:?}", search_handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_prover_close_credentials_search_for_proof_req: entities >>> search_handle: {:?}", search_handle);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::CloseCredentialsSearchForProofReq(
                    search_handle,
                    Box::new(move |result| {
                        let err = prepare_result!(result);
                        trace!("indy_prover_close_credentials_search:");
                        cb(command_handle, err)
                    }),
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_close_credentials_search_for_proof_req: <<< res: {:?}", res);

    res
}

/// Creates a proof according to the given proof request
/// Either a corresponding credential with optionally revealed attributes or self-attested attribute must be provided
/// for each requested attribute (see indy_prover_get_credentials_for_pool_req).
/// A proof request may request multiple credentials from different schemas and different issuers.
/// All required schemas, public keys and revocation registries must be provided.
/// The proof request also contains nonce.
/// The proof contains either proof or self-attested attribute value for each requested attribute.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// proof_request_json: proof request json
///     {
///         "name": string,
///         "version": string,
///         "nonce": string, - a big number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
///         "requested_attributes": { // set of requested attributes
///              "<attr_referent>": <attr_info>, // see below
///              ...,
///         },
///         "requested_predicates": { // set of requested predicates
///              "<predicate_referent>": <predicate_info>, // see below
///              ...,
///          },
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (applies to every attribute and predicate but can be overridden on attribute level)
///                        // (can be overridden on attribute level)
///         "ver": Optional<str>  - proof request version:
///             - omit or "1.0" to use unqualified identifiers for restrictions
///             - "2.0" to use fully qualified identifiers for restrictions
///     }
/// requested_credentials_json: either a credential or self-attested attribute for each requested attribute
///     {
///         "self_attested_attributes": {
///             "self_attested_attribute_referent": string
///         },
///         "requested_attributes": {
///             "requested_attribute_referent_1": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }},
///             "requested_attribute_referent_2": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }}
///         },
///         "requested_predicates": {
///             "requested_predicates_referent_1": {"cred_id": string, "timestamp": Optional<number> }},
///         }
///     }
/// master_secret_id: the id of the master secret stored in the wallet
/// schemas_json: all schemas participating in the proof request
///     {
///         <schema1_id>: <schema1>,
///         <schema2_id>: <schema2>,
///         <schema3_id>: <schema3>,
///     }
/// credential_defs_json: all credential definitions participating in the proof request
///     {
///         "cred_def1_id": <credential_def1>,
///         "cred_def2_id": <credential_def2>,
///         "cred_def3_id": <credential_def3>,
///     }
/// rev_states_json: all revocation states participating in the proof request
///     {
///         "rev_reg_def1_id": {
///             "timestamp1": <rev_state1>,
///             "timestamp2": <rev_state2>,
///         },
///         "rev_reg_def2_id": {
///             "timestamp3": <rev_state3>
///         },
///         "rev_reg_def3_id": {
///             "timestamp4": <rev_state4>
///         },
///     }
/// cb: Callback that takes command result as parameter.
///
/// where
/// attr_referent: Proof-request local identifier of requested attribute
/// attr_info: Describes requested attribute
///     {
///         "name": string, // attribute name, (case insensitive and ignore spaces)
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// predicate_referent: Proof-request local identifier of requested attribute predicate
/// predicate_info: Describes requested attribute predicate
///     {
///         "name": attribute name, (case insensitive and ignore spaces)
///         "p_type": predicate type (">=", ">", "<=", "<")
///         "p_value": predicate value
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// non_revoc_interval: Defines non-revocation interval
///     {
///         "from": Optional<int>, // timestamp of interval beginning
///         "to": Optional<int>, // timestamp of interval ending
///     }
/// where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
///     The list of allowed fields:
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///
/// #Returns
/// Proof json
/// For each requested attribute either a proof (with optionally revealed attribute value) or
/// self-attested attribute value is provided.
/// Each proof is associated with a credential and corresponding schema_id, cred_def_id, rev_reg_id and timestamp.
/// There is also aggregated proof part common for all credential proofs.
///     {
///         "requested_proof": {
///             "revealed_attrs": {
///                 "requested_attr1_id": {sub_proof_index: number, raw: string, encoded: string},
///                 "requested_attr4_id": {sub_proof_index: number: string, encoded: string},
///             },
///             "unrevealed_attrs": {
///                 "requested_attr3_id": {sub_proof_index: number}
///             },
///             "self_attested_attrs": {
///                 "requested_attr2_id": self_attested_value,
///             },
///             "predicates": {
///                 "requested_predicate_1_referent": {sub_proof_index: int},
///                 "requested_predicate_2_referent": {sub_proof_index: int},
///             }
///         }
///         "proof": {
///             "proofs": [ <credential_proof>, <credential_proof>, <credential_proof> ],
///             "aggregated_proof": <aggregated_proof>
///         } (opaque type that contains data structures internal to Ursa.
///           It should not be parsed and are likely to change in future versions).
///         "identifiers": [{schema_id, cred_def_id, Optional<rev_reg_id>, Optional<timestamp>}]
///     }
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_create_proof(command_handle: CommandHandle,
                                       wallet_handle: WalletHandle,
                                       proof_req_json: *const c_char,
                                       requested_credentials_json: *const c_char,
                                       master_secret_id: *const c_char,
                                       schemas_json: *const c_char,
                                       credential_defs_json: *const c_char,
                                       rev_states_json: *const c_char,
                                       cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                            proof_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_create_proof: >>> wallet_handle: {:?}, proof_req_json: {:?}, requested_credentials_json: {:?}, master_secret_id: {:?}, \
    schemas_json: {:?}, credential_defs_json: {:?}, rev_states_json: {:?}",
           wallet_handle, proof_req_json, requested_credentials_json, master_secret_id, schemas_json, credential_defs_json, rev_states_json);

    check_useful_validatable_json!(proof_req_json, ErrorCode::CommonInvalidParam3, ProofRequest);
    check_useful_validatable_json!(requested_credentials_json, ErrorCode::CommonInvalidParam4, RequestedCredentials);
    check_useful_c_str!(master_secret_id, ErrorCode::CommonInvalidParam5);
    check_useful_json!(schemas_json, ErrorCode::CommonInvalidParam6, Schemas);
    check_useful_json!(credential_defs_json, ErrorCode::CommonInvalidParam7, CredentialDefinitions);
    check_useful_json!(rev_states_json, ErrorCode::CommonInvalidParam8, RevocationStates);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam9);

    trace!("indy_prover_create_proof: entities >>> wallet_handle: {:?}, proof_req_json: {:?}, requested_credentials_json: {:?}, master_secret_id: {:?}, \
    schemas_json: {:?}, credential_defs_json: {:?}, rev_states_json: {:?}",
           wallet_handle, proof_req_json, requested_credentials_json, master_secret_id, schemas_json, credential_defs_json, rev_states_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::CreateProof(
            wallet_handle,
            proof_req_json,
            requested_credentials_json,
            master_secret_id,
            schemas_json,
            credential_defs_json,
            rev_states_json,
            boxed_callback_string!("indy_prover_create_proof", cb, command_handle)
        ))));

    let res = prepare_result!(result);

    trace!("indy_prover_create_proof: <<< res: {:?}", res);

    res
}

/// Verifies a proof (of multiple credential).
/// All required schemas, public keys and revocation registries must be provided.
///
/// IMPORTANT: You must use *_id's (`schema_id`, `cred_def_id`, `rev_reg_id`) listed in `proof[identifiers]`
/// as the keys for corresponding `schemas_json`, `credential_defs_json`, `rev_reg_defs_json`, `rev_regs_json` objects.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// proof_request_json: proof request json
///     {
///         "name": string,
///         "version": string,
///         "nonce": string, - a big number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
///         "requested_attributes": { // set of requested attributes
///              "<attr_referent>": <attr_info>, // see below
///              ...,
///         },
///         "requested_predicates": { // set of requested predicates
///              "<predicate_referent>": <predicate_info>, // see below
///              ...,
///          },
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (can be overridden on attribute level)
///         "ver": Optional<str>  - proof request version:
///             - omit or "1.0" to use unqualified identifiers for restrictions
///             - "2.0" to use fully qualified identifiers for restrictions
///     }
/// proof_json: created for request proof json
///     {
///         "requested_proof": {
///             "revealed_attrs": {
///                 "requested_attr1_id": {sub_proof_index: number, raw: string, encoded: string}, // NOTE: check that `encoded` value match to `raw` value on application level
///                 "requested_attr4_id": {sub_proof_index: number: string, encoded: string}, // NOTE: check that `encoded` value match to `raw` value on application level
///             },
///             "unrevealed_attrs": {
///                 "requested_attr3_id": {sub_proof_index: number}
///             },
///             "self_attested_attrs": {
///                 "requested_attr2_id": self_attested_value,
///             },
///             "requested_predicates": {
///                 "requested_predicate_1_referent": {sub_proof_index: int},
///                 "requested_predicate_2_referent": {sub_proof_index: int},
///             }
///         }
///         "proof": {
///             "proofs": [ <credential_proof>, <credential_proof>, <credential_proof> ],
///             "aggregated_proof": <aggregated_proof>
///         }
///         "identifiers": [{schema_id, cred_def_id, Optional<rev_reg_id>, Optional<timestamp>}]
///     }
/// schemas_json: all schemas participating in the proof
///     {
///         <schema1_id>: <schema1>,
///         <schema2_id>: <schema2>,
///         <schema3_id>: <schema3>,
///     }
/// credential_defs_json: all credential definitions participating in the proof
///     {
///         "cred_def1_id": <credential_def1>,
///         "cred_def2_id": <credential_def2>,
///         "cred_def3_id": <credential_def3>,
///     }
/// rev_reg_defs_json: all revocation registry definitions participating in the proof
///     {
///         "rev_reg_def1_id": <rev_reg_def1>,
///         "rev_reg_def2_id": <rev_reg_def2>,
///         "rev_reg_def3_id": <rev_reg_def3>,
///     }
/// rev_regs_json: all revocation registries participating in the proof
///     {
///         "rev_reg_def1_id": {
///             "timestamp1": <rev_reg1>,
///             "timestamp2": <rev_reg2>,
///         },
///         "rev_reg_def2_id": {
///             "timestamp3": <rev_reg3>
///         },
///         "rev_reg_def3_id": {
///             "timestamp4": <rev_reg4>
///         },
///     }
/// where
/// attr_referent: Proof-request local identifier of requested attribute
/// attr_info: Describes requested attribute
///     {
///         "name": string, // attribute name, (case insensitive and ignore spaces)
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// predicate_referent: Proof-request local identifier of requested attribute predicate
/// predicate_info: Describes requested attribute predicate
///     {
///         "name": attribute name, (case insensitive and ignore spaces)
///         "p_type": predicate type (">=", ">", "<=", "<")
///         "p_value": predicate value
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// non_revoc_interval: Defines non-revocation interval
///     {
///         "from": Optional<int>, // timestamp of interval beginning
///         "to": Optional<int>, // timestamp of interval ending
///     }
/// where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
///     The list of allowed fields:
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// valid: true - if signature is valid, false - otherwise
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_verifier_verify_proof(command_handle: CommandHandle,
                                         proof_request_json: *const c_char,
                                         proof_json: *const c_char,
                                         schemas_json: *const c_char,
                                         credential_defs_json: *const c_char,
                                         rev_reg_defs_json: *const c_char,
                                         rev_regs_json: *const c_char,
                                         cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                              valid: bool)>) -> ErrorCode {
    trace!("indy_verifier_verify_proof: >>> proof_request_json: {:?}, proof_json: {:?}, schemas_json: {:?}, credential_defs_json: {:?}, \
    rev_reg_defs_json: {:?}, rev_regs_json: {:?}", proof_request_json, proof_json, schemas_json, credential_defs_json, rev_reg_defs_json, rev_regs_json);

    check_useful_validatable_json!(proof_request_json, ErrorCode::CommonInvalidParam2, ProofRequest);
    check_useful_validatable_json!(proof_json, ErrorCode::CommonInvalidParam3, Proof);
    check_useful_json!(schemas_json, ErrorCode::CommonInvalidParam4, Schemas);
    check_useful_json!(credential_defs_json, ErrorCode::CommonInvalidParam5, CredentialDefinitions);
    check_useful_json!(rev_reg_defs_json, ErrorCode::CommonInvalidParam6, RevocationRegistryDefinitions);
    check_useful_json!(rev_regs_json, ErrorCode::CommonInvalidParam7, RevocationRegistries);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    trace!("indy_verifier_verify_proof: entities >>> proof_request_json: {:?}, proof_json: {:?}, schemas_json: {:?}, credential_defs_json: {:?}, \
    rev_reg_defs_json: {:?}, rev_regs_json: {:?}", proof_request_json, proof_json, schemas_json, credential_defs_json, rev_reg_defs_json, rev_regs_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Verifier(VerifierCommand::VerifyProof(
            proof_request_json,
            proof_json,
            schemas_json,
            credential_defs_json,
            rev_reg_defs_json,
            rev_regs_json,
            Box::new(move |result| {
                let (err, valid) = prepare_result_1!(result, false);
                trace!("indy_verifier_verify_proof: valid: {:?}", valid);

                cb(command_handle, err, valid)
            })
        ))));

    let res = prepare_result!(result);

    trace!("indy_verifier_verify_proof: <<< res: {:?}", res);

    res
}


///  Generates 80-bit numbers that can be used as a nonce for proof request.
///
/// #Params
/// command_handle: command handle to map callback to user context
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// nonce: generated number as a string
///
#[no_mangle]
pub extern fn indy_generate_nonce(command_handle: CommandHandle,
                                  cb: Option<extern fn(
                                      command_handle_: CommandHandle, err: ErrorCode,
                                      nonce: *const c_char)>) -> ErrorCode {
    trace!("indy_generate_nonce: >>> ");

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam2);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Verifier(
            VerifierCommand::GenerateNonce(
                boxed_callback_string!("indy_generate_nonce", cb, command_handle)
            ))));

    let res = prepare_result!(result);

    trace!("indy_generate_nonce: <<< res: {:?}", res);

    res
}

/// Get unqualified form (short form without method) of a fully qualified entity like DID.
///
/// This function should be used to the proper casting of fully qualified entity to unqualified form in the following cases:
///     Issuer, which works with fully qualified identifiers, creates a Credential Offer for Prover, which doesn't support fully qualified identifiers.
///     Verifier prepares a Proof Request based on fully qualified identifiers or Prover, which doesn't support fully qualified identifiers.
///     another case when casting to unqualified form needed
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// entity: string - target entity to disqualify. Can be one of:
///             Did
///             SchemaId
///             CredentialDefinitionId
///             RevocationRegistryId
///             Schema
///             CredentialDefinition
///             RevocationRegistryDefinition
///             CredentialOffer
///             CredentialRequest
///             ProofRequest
///
/// #Returns
///   res: entity either in unqualified form or original if casting isn't possible
#[no_mangle]
pub  extern fn indy_to_unqualified(command_handle: CommandHandle,
                                   entity: *const c_char,
                                   cb: Option<extern fn(command_handle_: CommandHandle,
                                                        err: ErrorCode,
                                                        res: *const c_char)>) -> ErrorCode {
    trace!("indy_to_unqualified: >>> entity: {:?}", entity);

    check_useful_c_str!(entity, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_to_unqualified: entities >>> entity: {:?}", entity);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::ToUnqualified(
            entity,
            Box::new(move |result| {
                let (err, res) = prepare_result_1!(result, String::new());
                trace!("indy_to_unqualified: did: {:?}", res);
                let res = ctypes::string_to_cstring(res);
                cb(command_handle, err, res.as_ptr())
            }),
        )));

    let res = prepare_result!(result);

    trace!("indy_to_unqualified: <<< res: {:?}", res);

    res
}


use crate::{get_peers_content, SignalType};
use hdk::prelude::*;
use hdk_crud::{crud, retrieval::{inputs::FetchOptions, fetch_entries::FetchEntries, fetch_links::FetchLinks, get_latest_for_entry::GetLatestEntry}, chain_actions::{fetch_action::FetchAction, delete_action::DeleteAction}};
use holo_hash::{AgentPubKeyB64, HeaderHashB64};

// a relationship between a Goal and an Agent
// representing roughly the idea of someone being "assigned to"
// or "responsible for" or "working on"
#[hdk_entry(id = "goal_member")]
#[derive(Clone, PartialEq)]
pub struct GoalMember {
    pub goal_address: HeaderHashB64,
    // the "assignee"
    pub agent_address: AgentPubKeyB64,
    // the person who authored this entry
    pub user_edit_hash: AgentPubKeyB64,
    pub unix_timestamp: f64,
    pub is_imported: bool,
}

impl GoalMember {
    pub fn new(
        goal_address: HeaderHashB64,
        agent_address: AgentPubKeyB64,
        user_edit_hash: AgentPubKeyB64,
        unix_timestamp: f64,
        is_imported: bool,
    ) -> Self {
        Self {
            goal_address,
            agent_address,
            user_edit_hash,
            unix_timestamp,
            is_imported,
        }
    }
}

crud!(
    GoalMember,
    goal_member,
    "goal_member",
    get_peers_content,
    SignalType
);

// DELETE
// clear all members
pub fn archive_goal_members(address: HeaderHashB64) -> ExternResult<Vec<HeaderHashB64>> {
    let fetch_action = FetchAction {};
    let delete_action = DeleteAction {};
    let fetch_entries = FetchEntries {};
    let fetch_links = FetchLinks {};
    let get_latest = GetLatestEntry {};
    Ok(
        fetch_action.fetch_action::<GoalMember, WasmError>(
           &fetch_entries,
           &fetch_links,
           &get_latest,
           FetchOptions::All,
           GetOptions::content(),
           get_goal_member_path(),
        )?
            .into_iter()
            .filter(|wire_element| {
                // check whether the parent_address or child_address is equal to the given address.
                // If so, the edge is connected to the goal being archived.
                wire_element.entry.goal_address == address.clone()
            })
            .map(|wire_element| {
                let goal_member_address = wire_element.header_hash;
                // archive the edge with this address
                // this will also trigger signals
                match delete_action.delete_action::<GoalMember, WasmError, SignalType>(
                    goal_member_address.clone(),
                    "goal_member".to_string(),
                    Some(get_peers_content()?),
                ) {
                    Ok(_) => Ok(goal_member_address),
                    Err(e) => Err(e),
                }
            })
            // filter out errors
            .filter_map(Result::ok)
            .collect(),
    )
}

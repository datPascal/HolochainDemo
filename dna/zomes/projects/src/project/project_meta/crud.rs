use crate::{
    get_peers_content,
    project::{error::Error, goal::crud::UIEnum},
    SignalType,
};
use hdk::prelude::*;
use hdk_crud::{
    crud, retrieval::{inputs::FetchOptions, fetch_entries::FetchEntries, fetch_links::FetchLinks, get_latest_for_entry::GetLatestEntry}, wire_element::WireElement, chain_actions::fetch_action::FetchAction,
};
use holo_hash::{AgentPubKeyB64, EntryHashB64, HeaderHashB64};
use std::*;

#[hdk_entry(id = "project_meta")]
#[derive(Clone, PartialEq)]
pub struct ProjectMeta {
    pub creator_address: AgentPubKeyB64,
    pub created_at: f64,
    pub name: String,
    pub image: Option<String>,
    pub passphrase: String,
    pub is_imported: bool,
    pub priority_mode: PriorityMode,
    pub top_priority_goals: Vec<HeaderHashB64>,
}

impl ProjectMeta {
    pub fn new(
        creator_address: AgentPubKeyB64,
        created_at: f64,
        name: String,
        image: Option<String>,
        passphrase: String,
        is_imported: bool,
        priority_mode: PriorityMode,
        top_priority_goals: Vec<HeaderHashB64>,
    ) -> Self {
        Self {
            creator_address,
            created_at,
            name,
            image,
            passphrase,
            is_imported,
            priority_mode,
            top_priority_goals,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
#[serde(from = "UIEnum")]
#[serde(into = "UIEnum")]
pub enum PriorityMode {
    Universal,
    Vote,
}
impl From<UIEnum> for PriorityMode {
    fn from(ui_enum: UIEnum) -> Self {
        match ui_enum.0.as_str() {
            "Universal" => Self::Universal,
            "Vote" => Self::Vote,
            _ => Self::Vote,
        }
    }
}
impl From<PriorityMode> for UIEnum {
    fn from(priority_mode: PriorityMode) -> Self {
        Self(priority_mode.to_string())
    }
}
impl fmt::Display for PriorityMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

crud!(
    ProjectMeta,
    project_meta,
    "project_meta",
    get_peers_content,
    SignalType
);

#[hdk_extern]
pub fn simple_create_project_meta(entry: ProjectMeta) -> ExternResult<WireElement<ProjectMeta>> {
    // no project_meta entry should exist at least
    // that we can know about
    let fetch_action = FetchAction {};
    let fetch_entries = FetchEntries {};
    let fetch_links = FetchLinks {};
    let get_latest = GetLatestEntry {};
    match fetch_action.fetch_action::<ProjectMeta, WasmError>(
        &fetch_entries,
        &fetch_links,
        &get_latest,
        FetchOptions::All,
        GetOptions::latest(),
        get_project_meta_path(),
    )?.len() {
        0 => {}
        _ => return Err(WasmError::Guest(Error::OnlyOneOfEntryType.to_string())),
    };
    let address = create_entry(&entry)?;
    let entry_hash = hash_entry(&entry)?;
    let path = Path::from(PROJECT_META_PATH);
    path.ensure()?;
    let path_hash = path.hash()?;
    create_link(path_hash, entry_hash.clone(), ())?;
    let wire_entry: WireElement<ProjectMeta> = WireElement {
        entry,
        header_hash: HeaderHashB64::new(address),
        entry_hash: EntryHashB64::new(entry_hash),
    };
    Ok(wire_entry)
}

// READ
#[hdk_extern]
pub fn fetch_project_meta(_: ()) -> ExternResult<WireElement<ProjectMeta>> {
    let fetch_action = FetchAction {};
    let fetch_entries = FetchEntries {};
    let fetch_links = FetchLinks {};
    let get_latest = GetLatestEntry {};
    match fetch_action.fetch_action::<ProjectMeta, WasmError>(
        &fetch_entries,
        &fetch_links,
        &get_latest,
        FetchOptions::All,
        GetOptions::latest(),
        get_project_meta_path(),
    )?.first() {
        Some(wire_entry) => Ok(wire_entry.to_owned()),
        None => Err(WasmError::Guest("no project meta exists".into())),
    }
}

// Since get_links can't be controlled with GetOptions right
// now, we need to check the Path instead, and use GetOptions::latest
// this is used while trying to join a project
#[hdk_extern]
pub fn check_project_meta_exists(_: ()) -> ExternResult<bool> {
    let path = Path::from(PROJECT_META_PATH);
    Ok(get(path.hash()?, GetOptions::latest())?.is_some())
}

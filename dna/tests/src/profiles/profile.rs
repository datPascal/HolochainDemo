#[cfg(test)]
pub mod tests {
    use crate::fixtures::fixtures::ProfileFixturator;
    use crate::test_lib::*;
    use ::fixt::prelude::*;
    use hdk::prelude::*;
    use hdk_crud::signals::ActionType;
    use hdk_crud::wire_element::WireElement;
    use holo_hash::{EntryHashB64, HeaderHashB64};
    use profiles::profile::{
        agent_signal_entry_type, create_imported_profile, inner_create_whoami, inner_update_whoami,
        AgentSignal, Profile, SignalData,
    };

    #[test]
    fn test_create_whoami() {
        let mut mock_hdk = MockHdkT::new();
        let mock_hdk_ref = &mut mock_hdk;

        let wire_element = generate_wire_element();
        let profile = wire_element.clone().entry;
        let profile_entry = CreateInput::try_from(profile.clone()).unwrap();
        let profile_header_hash = wire_element.clone().header_hash;
        mock_create(
            mock_hdk_ref,
            profile_entry,
            Ok(profile_header_hash.clone().into()),
        );

        let profile_hash: EntryHash = wire_element.entry_hash.clone().into();
        mock_hash_entry(
            mock_hdk_ref,
            Entry::try_from(profile.clone()).unwrap(),
            Ok(profile_hash.clone()),
        );

        let agent_path = Path::from("agents");
        let agent_path_hash = fixt!(EntryHash);
        mock_hash_entry(
            mock_hdk_ref,
            Entry::try_from(agent_path).unwrap(),
            Ok(agent_path_hash.clone()),
        );

        let create_link_input = CreateLinkInput::new(
            agent_path_hash.clone(),
            profile_hash.clone(),
            LinkTag::from(()),
            ChainTopOrdering::default(),
        );
        let link_header_hash = fixt!(HeaderHash);
        mock_create_link(
            mock_hdk_ref,
            create_link_input,
            Ok(link_header_hash.clone()),
        );

        let agent_info = fixt!(AgentInfo);
        let agent_entry_hash = EntryHash::from(agent_info.clone().agent_initial_pubkey);
        mock_agent_info(mock_hdk_ref, Ok(agent_info));
        let create_link_input = CreateLinkInput::new(
            agent_entry_hash,
            profile_hash,
            LinkTag::from(()),
            ChainTopOrdering::default(),
        );
        mock_create_link(
            mock_hdk_ref,
            create_link_input,
            Ok(link_header_hash.clone()),
        );

        let signal = AgentSignal {
            entry_type: agent_signal_entry_type(),
            action: ActionType::Create,
            data: SignalData::Create(wire_element.clone()),
        };

        let payload = ExternIO::encode(signal).unwrap();
        let agents = vec![];
        let remote_signal = RemoteSignal {
            signal: ExternIO::encode(payload).unwrap(),
            agents,
        };
        mock_remote_signal(mock_hdk_ref, remote_signal, Ok(()));

        set_hdk(mock_hdk);
        fn get_peers() -> ExternResult<Vec<AgentPubKey>> {
            Ok(vec![])
        }
        let result = inner_create_whoami(profile, get_peers);
        assert_eq!(result, Ok(wire_element));
    }
    #[test]
    fn test_create_imported_profile() {
        let mut mock_hdk = MockHdkT::new();
        let mock_hdk_ref = &mut mock_hdk;

        let wire_element = generate_wire_element();
        let profile = wire_element.clone().entry;
        let profile_entry = CreateInput::try_from(profile.clone()).unwrap();
        let profile_header_hash = wire_element.clone().header_hash;
        let profile_entry_hash = wire_element.clone().entry_hash;

        mock_create(
            mock_hdk_ref,
            profile_entry,
            Ok(profile_header_hash.clone().into()),
        );

        mock_hash_entry(
            mock_hdk_ref,
            Entry::try_from(profile.clone()).unwrap(),
            Ok(profile_entry_hash.clone().into()),
        );

        let agent_path = Path::from("agents");
        let agent_path_hash = fixt!(EntryHash);
        mock_hash_entry(
            mock_hdk_ref,
            Entry::try_from(agent_path).unwrap(),
            Ok(agent_path_hash.clone()),
        );

        let create_link_input = CreateLinkInput::new(
            agent_path_hash.clone(),
            profile_entry_hash.into(),
            LinkTag::from(()),
            ChainTopOrdering::default(),
        );
        let link_header_hash = fixt!(HeaderHash);
        mock_create_link(
            mock_hdk_ref,
            create_link_input,
            Ok(link_header_hash.clone()),
        );

        set_hdk(mock_hdk);
        let result = create_imported_profile(profile);
        assert_eq!(result, Ok(wire_element));
    }
    #[test]
    fn test_update_whoami() {
        let mut mock_hdk = MockHdkT::new();
        let mock_hdk_ref = &mut mock_hdk;

        let mut wire_element = generate_wire_element();
        let profile_address = wire_element.header_hash.clone();
        let profile = wire_element.entry.clone();
        let update_input = UpdateInput::new(
            profile_address.clone().into(),
            CreateInput::try_from(profile.clone()).unwrap(),
        );
        let update_header_hash = fixt!(HeaderHash);
        mock_update(mock_hdk_ref, update_input, Ok(update_header_hash));

        let update_entry_hash = fixt!(EntryHash);
        mock_hash_entry(
            mock_hdk_ref,
            CreateInput::try_from(profile.clone()).unwrap().entry,
            Ok(update_entry_hash.clone()),
        );

        wire_element.entry_hash = EntryHashB64::new(update_entry_hash);

        let signal = AgentSignal {
            entry_type: agent_signal_entry_type(),
            action: ActionType::Update,
            data: SignalData::Update(wire_element.clone()),
        };

        let payload = ExternIO::encode(signal).unwrap();
        let agents = vec![];
        let remote_signal = RemoteSignal {
            signal: ExternIO::encode(payload).unwrap(),
            agents,
        };
        mock_remote_signal(mock_hdk_ref, remote_signal, Ok(()));

        set_hdk(mock_hdk);
        fn get_peers() -> ExternResult<Vec<AgentPubKey>> {
            Ok(vec![])
        }
        let result = inner_update_whoami(
            profiles::profile::UpdateInput {
                header_hash: profile_address,
                entry: profile,
            },
            get_peers,
        );
        assert_eq!(result, Ok(wire_element));
    }
    // #[test]
    // fn test_whoami() {}
    // #[test]
    // fn test_fetch_agents() {}

    /// generate an arbitrary `WireEntry` for unit testing
    fn generate_wire_element() -> WireElement<Profile> {
        let profile = fixt!(Profile);
        let profile_header_hash = fixt!(HeaderHash);
        let profile_entry_hash = fixt!(EntryHash);
        let wire_element = WireElement {
            entry: profile,
            header_hash: HeaderHashB64::new(profile_header_hash),
            entry_hash: EntryHashB64::new(profile_entry_hash),
        };
        wire_element
    }
}

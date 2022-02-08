use crate::project::{
  error::Error,
  project_meta::crud::ProjectMeta,
  validate::validate_value_matches_create_author,
};
use hdk_crud::{resolve_dependency, ResolvedDependency};
use hdk::prelude::*;

#[hdk_extern]
fn validate_create_entry_project_meta(
  validate_data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
  Ok(
    // element must have an entry that must deserialize correctly
    match ProjectMeta::try_from(&validate_data.element) {
      Ok(proposed_entry) => {
        // `address` must match header author
        validate_value_matches_create_author(&proposed_entry.creator_address.0, &validate_data)
      }
      Err(_e) => Error::EntryMissing.into(),
    },
  )
}

#[hdk_extern]
fn validate_update_entry_project_meta(
  validate_data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
  Ok(
    // element must have an entry that must deserialize correctly
    match ProjectMeta::try_from(&validate_data.element) {
      Ok(project_meta) => match validate_data.element.header() {
        Header::Update(header) => {
          match resolve_dependency::<ProjectMeta>(header.original_header_address.clone().into())? {
            Ok(ResolvedDependency(_, orig_project_meta)) => {
              if project_meta.creator_address == orig_project_meta.creator_address
                && project_meta.created_at == orig_project_meta.created_at
                && project_meta.passphrase == orig_project_meta.passphrase
              {
                ValidateCallbackResult::Valid
              } else {
                Error::ProjectMetaEditableFields.into()
              }
            }
            Err(e) => e,
          }
        }
        _ => {
          // Holochain passed the wrong header!
          #[allow(unreachable_code)]
          return unreachable!();
        }
      },
      Err(_e) => Error::EntryMissing.into(),
    },
  )
}

#[hdk_extern]
/// Deletes aren't allowed
fn validate_delete_entry_project_meta(_: ValidateData) -> ExternResult<ValidateCallbackResult> {
  Ok(Error::DeleteAttempted.into())
}

#[cfg(test)]
pub mod tests {
  use crate::project::error::Error;
  use crate::project::fixtures::fixtures::ProjectMetaFixturator;
  use crate::project::project_meta::crud::ProjectMeta;
  use ::fixt::prelude::*;
  use hdk_crud::WrappedAgentPubKey;
  use hdk::prelude::*;
  use holochain_types::prelude::ElementFixturator;
  use holochain_types::prelude::ValidateDataFixturator;

  #[test]
  fn test_validate_create_entry_project_meta() {
    let mut validate_data = fixt!(ValidateData);
    let create_header = fixt!(Create);
    let mut project_meta = fixt!(ProjectMeta);
    *validate_data.element.as_header_mut() = Header::Create(create_header.clone());

    // without an Element containing an Entry, validation will fail
    assert_eq!(
      super::validate_create_entry_project_meta(validate_data.clone()),
      Error::EntryMissing.into(),
    );

    // with an entry with a random
    // `address` it will fail (not the agent committing)
    *validate_data.element.as_entry_mut() =
      ElementEntry::Present(project_meta.clone().try_into().unwrap());
    assert_eq!(
      super::validate_create_entry_project_meta(validate_data.clone()),
      Error::CorruptCreateAgentPubKeyReference.into(),
    );

    // success case:
    // make the `address` field valid by making it equal the
    // AgentPubKey of the agent committing
    
    project_meta.creator_address = WrappedAgentPubKey::new(create_header.author.as_hash().clone());
    *validate_data.element.as_entry_mut() =
    ElementEntry::Present(project_meta.clone().try_into().unwrap());

    // we should see that the ValidateCallbackResult
    // is valid
    assert_eq!(
      super::validate_create_entry_project_meta(validate_data.clone()),
      Ok(ValidateCallbackResult::Valid),
    );
  }

  #[test]
  fn test_validate_update_entry_project_meta() {
    let mut validate_data = fixt!(ValidateData);
    let update_header = fixt!(Update);
    *validate_data.element.as_header_mut() = Header::Update(update_header.clone());
    // without an Element containing an Entry, validation will fail
    assert_eq!(
      super::validate_update_entry_project_meta(validate_data.clone()),
      Error::EntryMissing.into(),
    );

    // make it pass first step by adding a project meta
    let mut project_meta = fixt!(ProjectMeta);
    // do this because for some reason the fixturator was occasionally 
    // producing a NaN for the f64 for created_at field. Will
    // have to watch out for that weird behaviour
    project_meta.created_at = 102391293.0;
    *validate_data.element.as_entry_mut() =
      ElementEntry::Present(project_meta.clone().try_into().unwrap());

    // now, since validation is dependent on other entries, we begin
    // to have to mock `get` calls to the HDK

    let mut mock_hdk = MockHdkT::new();
    // the resolve_dependencies `get` call of the original ProjectMeta
    mock_hdk
      .expect_get()
      .with(mockall::predicate::eq(vec![GetInput::new(
        update_header.original_header_address.clone().into(),
        GetOptions::content(),
      )]))
      .times(1)
      // act as if not present / not found
      .return_const(Ok(vec![None]));

    set_hdk(mock_hdk);

    // we should see that the ValidateCallbackResult is that there are UnresolvedDependencies
    // equal to the Hash of the original_header_address of the Update
    assert_eq!(
      super::validate_update_entry_project_meta(validate_data.clone()),
      Ok(ValidateCallbackResult::UnresolvedDependencies(vec![
        update_header.original_header_address.clone().into()
      ])),
    );

    let invalid_original_project_meta = ProjectMeta {
      //make it invalid can't edit passphrase
      passphrase: "test".to_string(),
      ..project_meta.clone()
    };
    let mut invalid_original_project_meta_element = fixt!(Element);
    *invalid_original_project_meta_element.as_entry_mut() =
      ElementEntry::Present(invalid_original_project_meta.clone().try_into().unwrap());

    let mut mock_hdk = MockHdkT::new();
    // the resolve_dependencies `get` call of the original_header_address
    mock_hdk
      .expect_get()
      .with(mockall::predicate::eq(vec![GetInput::new(
        update_header.original_header_address.clone().into(),
        GetOptions::content(),
      )]))
      .times(1)
      .return_const(Ok(vec![Some(invalid_original_project_meta_element)]));

    set_hdk(mock_hdk);

    // without an Element containing an Entry, validation will fail
    assert_eq!(
      super::validate_update_entry_project_meta(validate_data.clone()),
      Error::ProjectMetaEditableFields.into(),
    );

    // SUCCESS CASE, can edit name and image (only)

    let valid_original_project_meta = ProjectMeta {
      //make it valid, by matching the rest, but name and image
      name: "test".to_string(),
      image: Some("hi".to_string()),
      ..project_meta
    };
    let mut valid_original_project_meta_element = fixt!(Element);
    *valid_original_project_meta_element.as_entry_mut() =
      ElementEntry::Present(valid_original_project_meta.clone().try_into().unwrap());

    let mut mock_hdk = MockHdkT::new();
    // the resolve_dependencies `get` call of the original_header_address
    mock_hdk
      .expect_get()
      .with(mockall::predicate::eq(vec![GetInput::new(
        update_header.original_header_address.clone().into(),
        GetOptions::content(),
      )]))
      .times(1)
      .return_const(Ok(vec![Some(valid_original_project_meta_element)]));

    set_hdk(mock_hdk);

    // valid!
    assert_eq!(
      super::validate_update_entry_project_meta(validate_data.clone()),
      Ok(ValidateCallbackResult::Valid),
    );
  }

  #[test]
  fn test_validate_delete_entry_project_meta() {
    let mut validate_data = fixt!(ValidateData);
    let delete_header = fixt!(Delete);
    *validate_data.element.as_header_mut() = Header::Delete(delete_header.clone());
    assert_eq!(
      super::validate_delete_entry_project_meta(validate_data),
      Error::DeleteAttempted.into(),
    );
  }
}

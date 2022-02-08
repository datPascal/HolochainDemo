/*
  There should be an actions.js file in every
  feature folder, and it should start with a list
  of constants defining all the types of actions
  that can be taken within that feature.
*/

const SET_ACTIVE_ENTRY_POINTS = 'SET_ACTIVE_ENTRY_POINTS'

/* action creator functions */

const setActiveEntryPoints = entryPointAddresses => {
  return {
    type: SET_ACTIVE_ENTRY_POINTS,
    payload: entryPointAddresses,
  }
}

export { SET_ACTIVE_ENTRY_POINTS, setActiveEntryPoints }

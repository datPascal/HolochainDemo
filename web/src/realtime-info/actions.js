const UPDATE_PEER_STATE = 'UPDATE_PEER_STATE'
const REMOVE_PEER_STATE = 'REMOVE_PEER_STATE'

function updatePeerState(realtimeInfo) {
  return {
    type: UPDATE_PEER_STATE,
    payload: realtimeInfo,
  }
}
function removePeerState(realtimeInfo) {
  return {
    type: REMOVE_PEER_STATE,
    payload: realtimeInfo,
  }
}

export {
  UPDATE_PEER_STATE,
  REMOVE_PEER_STATE,
  updatePeerState,
  removePeerState,
}

const OPEN_EXPANDED_VIEW = 'OPEN_EXPANDED_VIEW'
const CLOSE_EXPANDED_VIEW = 'CLOSE_EXPANDED_VIEW'

function openExpandedView(goalAddress) {
  return {
    type: OPEN_EXPANDED_VIEW,
    payload: {
      goalAddress,
    },
  }
}

function closeExpandedView() {
  return {
    type: CLOSE_EXPANDED_VIEW,
  }
}

export {
  OPEN_EXPANDED_VIEW,
  CLOSE_EXPANDED_VIEW,
  openExpandedView,
  closeExpandedView,
}

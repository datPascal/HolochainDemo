import TWEEN from '@tweenjs/tween.js'
import {
    createGoalWithEdge,
    fetchGoals,
    archiveGoalFully
} from '../projects/goals/actions'
import {
    createEdge,
    fetchEdges,
    AFFECT_LAYOUT_ARCHIVE_EDGE,
    PREVIEW_EDGES,
    CLEAR_EDGES_PREVIEW,
} from '../projects/edges/actions'
import {
  TRIGGER_UPDATE_LAYOUT,
} from '../layout/actions'
import panZoomToFrame from '../animations/pan-and-zoom'
import { ANIMATE_PAN_AND_ZOOM } from '../viewport/actions'
import performLayoutAnimation from '../animations/layout'


const isOneOfLayoutAffectingActions = (action) => {
    const { type } = action
    return type === TRIGGER_UPDATE_LAYOUT
        || type === PREVIEW_EDGES
        || type === CLEAR_EDGES_PREVIEW
        || type === createGoalWithEdge.success().type
        || type === fetchGoals.success().type
        || type === archiveGoalFully.success().type
        || type === createEdge.success().type
        || type === fetchEdges.success().type
        || (action.type === AFFECT_LAYOUT_ARCHIVE_EDGE && action.affectLayout)
}

const isOneOfViewportAffectingActions = (action) => {
  const { type } = action
  return type === ANIMATE_PAN_AND_ZOOM
}

// watch for actions that will affect the layout (create/fetches/deletes)
// and update the layout when we see an action like that
// it is useful to have this layout cached in the state for performance reasons

const layoutWatcher = store => {

  // when this middleware is called
  // run an animation loop which will performantly handle 
  // the animation of layouts
  animate()
  function animate() {
    requestAnimationFrame(animate)
    TWEEN.update()
  }

  // return the action handler middleware
  return next => async action => {

      // in many cases, we just skip right over this middleware, 
      // and it has no effect. it is only during 'layout affecting actions'
      // (anything that deletes or adds Goals or Edges which form the graph we
      // pass to dagre to generate a layout) that this kicks in, and handles the creation of an animation
      // from the current layout to the new layout, by using the TWEENJS library

      // catch and handle this uniquely special action
      // which has a special case of being a delete firing very close in time to
      // a create action (since we delete an Edge then immediately create one)
      // and allow preventing that action in special cases from causing a layout animation
      // to occur
      const specialLayoutAffectingArchiveEdge = action.type === AFFECT_LAYOUT_ARCHIVE_EDGE
      if (specialLayoutAffectingArchiveEdge) {
        if (!action.affectLayout) {
          // just dispatch the "real" action and return that
          return store.dispatch(action.asyncAction)
        } else {
          // wait for the async action to complete before running re-layout
          await store.dispatch(action.asyncAction)
          // now the store will have the edge removed from state
          // meaning the end calculated layout will be proper
        }
      }

      let currentState
      const shouldReLayout = isOneOfLayoutAffectingActions(action)
      const shouldAnimateViewport = isOneOfViewportAffectingActions(action)
      // don't call and getState if we don't have to
      if (shouldReLayout || shouldAnimateViewport) {
        currentState = store.getState()
      }
      // perform the usual (next) action ->
      // first integrate the new data
      // that way the layout recalculation
      // will include it
      let result = next(action)

      // in the case of a special "layout affecting"
      // action, we should also recalculate the layout
      // based on the new graph of Goals and Edges
      // and animate to it
      if (shouldReLayout) {
        performLayoutAnimation(store, action, currentState)
      } else if (shouldAnimateViewport) {
        panZoomToFrame(store, action, currentState)
      }

      return result
  }
}


export {
    layoutWatcher
}
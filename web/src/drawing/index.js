/*
  This file is the entry point for how to render the redux state visually
  onto the screen, using the HTML5 canvas APIs.
  It will iterate through each part of the state that needs rendering
  and use well defined functions for rendering those specific parts
  to the canvas.
*/
import layoutFormula from './layoutFormula'
import drawGoalCard from './drawGoalCard'
import drawEdge, { calculateEdgeCoordsByGoalCoords } from './drawEdge'
import drawOverlay from './drawOverlay'
import drawSelectBox from '../drawing/drawSelectBox'
import drawEntryPoints from './drawEntryPoints'
import {
  RELATION_AS_PARENT,
  RELATION_AS_CHILD,
} from '../edge-connector/actions'
import { CONNECTOR_VERTICAL_SPACING, firstZoomThreshold } from './dimensions'

function setupCanvas(canvas) {
  // Get the device pixel ratio, falling back to 1.
  const dpr = window.devicePixelRatio || 1
  // Get the size of the canvas in CSS pixels.
  const rect = canvas.getBoundingClientRect()
  // Give the canvas pixel dimensions of their CSS
  // size * the device pixel ratio.
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr
  const ctx = canvas.getContext('2d')
  return ctx
}

// `store` is a redux store
// `canvas` is a reference to an HTML5 canvas DOM element
// render the state contained in store onto the canvas
function render(store, canvas) {
  // Get the 2 dimensional drawing context of the canvas (there is also 3 dimensional, e.g.)
  const ctx = setupCanvas(canvas)

  // pull the current state from the store
  const state = store.getState()

  // scale x, skew x, skew y, scale y, translate x, and translate y
  const {
    ui: {
      viewport: { translate, scale },
    },
  } = state
  ctx.setTransform(1, 0, 0, 1, 0, 0) // normalize
  // clear the entirety of the canvas
  ctx.clearRect(0, 0, state.ui.screensize.width, state.ui.screensize.height)

  // Scale all drawing operations by the dpr, as well as the zoom, so you
  // don't have to worry about the difference.
  const dpr = window.devicePixelRatio || 1
  ctx.setTransform(
    scale * dpr,
    0,
    0,
    scale * dpr,
    translate.x * dpr,
    translate.y * dpr
  )

  const projectId = state.ui.activeProject
  const activeEntryPoints = state.ui.activeEntryPoints
  if (!projectId) return
  const goals = state.projects.goals[projectId]
  const edges = state.projects.edges[projectId]
  const goalMembers = state.projects.goalMembers[projectId]
  const entryPoints = state.projects.entryPoints[projectId]

  // draw things relating to the project, if the project has fully loaded
  if (goals && edges && goalMembers && entryPoints) {
    // converts the goals object to an array
    const goalsAsArray = Object.keys(goals).map(address => goals[address])
    // convert the edges object to an array
    const edgesAsArray = Object.keys(edges).map(address => edges[address])

    const coordinates = layoutFormula(state.ui.screensize.width, state)

    const activeEntryPointsObjects = activeEntryPoints.map(
      entryPointAddress => entryPoints[entryPointAddress]
    )
    drawEntryPoints(
      ctx,
      activeEntryPointsObjects,
      goals,
      edgesAsArray,
      coordinates
    )

    // render each edge to the canvas, basing it off the rendering coordinates of the parent and child nodes
    edgesAsArray.forEach(function (edge) {
      const childCoords = coordinates[edge.child_address]
      const parentCoords = coordinates[edge.parent_address]
      const parentGoalText = goals[edge.parent_address]
        ? goals[edge.parent_address].content
        : ''
      if (childCoords && parentCoords) {
        const [edge1port, edge2port] = calculateEdgeCoordsByGoalCoords(
          childCoords,
          parentCoords,
          parentGoalText,
          ctx
        )
        const isHovered = state.ui.hover.hoveredEdge === edge.address
        const isSelected = state.ui.selection.selectedEdges.includes(
          edge.address
        )
        drawEdge(edge1port, edge2port, ctx, isHovered, isSelected)
      }
    })

    // create layers behind and in front of the editing highlight overlay
    const unselectedGoals = goalsAsArray.filter(goal => {
      return (
        state.ui.selection.selectedGoals.indexOf(goal.address) === -1 &&
        state.ui.goalForm.editAddress !== goal.address
      )
    })
    const selectedGoals = goalsAsArray.filter(goal => {
      return (
        state.ui.selection.selectedGoals.indexOf(goal.address) > -1 &&
        state.ui.goalForm.editAddress !== goal.address
      )
    })

    // render each unselected goal to the canvas
    unselectedGoals.forEach(goal => {
      // use the set of coordinates at the same index
      // in the coordinates array
      const isHovered = state.ui.hover.hoveredGoal === goal.address
      const isSelected = false
      const isEditing = false
      const membersOfGoal = Object.keys(goalMembers)
        .map(address => goalMembers[address])
        .filter(goalMember => goalMember.goal_address === goal.address)
        .map(goalMember => state.agents[goalMember.agent_address])
      drawGoalCard(
        scale,
        goal,
        membersOfGoal,
        coordinates[goal.address],
        isEditing,
        '',
        isSelected,
        isHovered,
        ctx
      )
    })
    if (
      state.ui.keyboard.shiftKeyDown &&
      state.ui.mouse.mousedown &&
      state.ui.mouse.coordinate.x !== 0
    ) {
      drawSelectBox(
        state.ui.mouse.coordinate,
        state.ui.mouse.size,
        canvas.getContext('2d')
      )
    }
    // draw the editing highlight overlay
    /* if shift key not held down and there are more than 1 Goals selected */
    if (
      state.ui.goalForm.editAddress ||
      (state.ui.selection.selectedGoals.length > 1 &&
        !state.ui.keyboard.shiftKeyDown)
    ) {
      // counteract the translation
      ctx.save()
      ctx.setTransform(1, 0, 0, 1, 0, 0)
      drawOverlay(
        ctx,
        0,
        0,
        state.ui.screensize.width,
        state.ui.screensize.height
      )
      ctx.restore()
    }

    // render each selected goal to the canvas
    selectedGoals.forEach(goal => {
      // use the set of coordinates at the same index
      // in the coordinates array
      const isHovered = state.ui.hover.hoveredGoal === goal.address
      const isSelected = true
      const isEditing = false
      const membersOfGoal = Object.keys(goalMembers)
        .map(address => goalMembers[address])
        .filter(goalMember => goalMember.goal_address === goal.address)
        .map(goalMember => state.agents[goalMember.agent_address])
      drawGoalCard(
        scale,
        goal,
        membersOfGoal,
        coordinates[goal.address],
        isEditing,
        '',
        isSelected,
        isHovered,
        ctx
      )
    })

    // render the edge that is pending to be created to the open goal form
    if (state.ui.goalForm.isOpen) {
      if (state.ui.goalForm.parentAddress) {
        const parentCoords = coordinates[state.ui.goalForm.parentAddress]
        const newGoalCoords = {
          x: state.ui.goalForm.leftEdgeXPosition,
          y: state.ui.goalForm.topEdgeYPosition,
        }
        const parentGoalText = state.projects.goals[
          state.ui.goalForm.parentAddress
        ]
          ? goals[state.ui.goalForm.parentAddress].content
          : ''
        const [edge1port, edge2port] = calculateEdgeCoordsByGoalCoords(
          newGoalCoords,
          parentCoords,
          parentGoalText,
          ctx
        )
        drawEdge(edge1port, edge2port, ctx)
      }
    }

    // render the edge that is pending to be created between existing Goals
    if (state.ui.edgeConnector.fromAddress) {
      const { fromAddress, relation, toAddress } = state.ui.edgeConnector
      const { liveCoordinate } = state.ui.mouse
      const fromCoords = coordinates[fromAddress]
      const fromContent = goals[fromAddress].content
      const [
        fromAsChildCoord,
        fromAsParentCoord,
      ] = calculateEdgeCoordsByGoalCoords(
        fromCoords,
        fromCoords,
        fromContent,
        ctx
      )

      // if there's a goal this is pending
      // as being "to", then we will be drawing the edge to its correct
      // upper or lower port
      // the opposite of whichever the "from" port is connected to
      let toCoords, toContent, toAsChildCoord, toAsParentCoord
      if (toAddress) {
        toCoords = coordinates[toAddress]
        toContent = goals[toAddress].content
        ;[toAsChildCoord, toAsParentCoord] = calculateEdgeCoordsByGoalCoords(
          toCoords,
          toCoords,
          toContent,
          ctx
        )
      }

      // in drawEdge, it draws at exactly the two coordinates given,
      // so we could pass them in either order/position
      const fromEdgeCoord =
        relation === RELATION_AS_PARENT ? fromAsParentCoord : fromAsChildCoord

      // use the current mouse coordinate position, liveCoordinate, by default
      let toEdgeCoord = liveCoordinate

      // use the coordinates relating to a Goal which it is pending that
      // this edge will connect the "from" Goal "to"
      if (toAddress) {
        toEdgeCoord =
          relation === RELATION_AS_PARENT ? toAsChildCoord : toAsParentCoord
      }

      if (relation === RELATION_AS_CHILD) {
        fromEdgeCoord.y = fromEdgeCoord.y - CONNECTOR_VERTICAL_SPACING
        // only modify if we're dealing with an actual goal being connected to
        if (toAddress)
          toEdgeCoord.y = toEdgeCoord.y + CONNECTOR_VERTICAL_SPACING
      } else if (relation === RELATION_AS_PARENT) {
        fromEdgeCoord.y = fromEdgeCoord.y + CONNECTOR_VERTICAL_SPACING
        // only modify if we're dealing with an actual goal being connected to
        if (toAddress)
          toEdgeCoord.y = toEdgeCoord.y - CONNECTOR_VERTICAL_SPACING
      }

      drawEdge(fromEdgeCoord, toEdgeCoord, ctx)
    }

    // draw the editing goal in front of the overlay as well
    if (state.ui.goalForm.editAddress) {
      // editing an existing Goal
      const editingGoal = goals[state.ui.goalForm.editAddress]
      // we only allow this goal
      // to be edited using 'quickedit'
      // above the first zoom threshold
      const isEditing = scale >= firstZoomThreshold
      const editText = state.ui.goalForm.content
      const membersOfGoal = Object.keys(goalMembers)
        .map(address => goalMembers[address])
        .filter(goalMember => goalMember.goal_address === editingGoal.address)
        .map(goalMember => state.agents[goalMember.agent_address])
      drawGoalCard(
        scale,
        editingGoal,
        membersOfGoal,
        coordinates[editingGoal.address],
        isEditing,
        editText,
        false,
        false,
        ctx
      )
    }
  }

  // creating a new Goal
  if (!state.ui.goalForm.editAddress && state.ui.goalForm.isOpen) {
    const isHovered = false
    const isSelected = false
    const isEditing = true
    drawGoalCard(
      scale,
      { status: 'Uncertain' },
      [],
      { x: state.ui.goalForm.leftEdgeXPosition, y: state.ui.goalForm.topEdgeYPosition },
      isEditing,
      state.ui.goalForm.content,
      isSelected,
      isHovered,
      ctx
    )
  }
}

export default render

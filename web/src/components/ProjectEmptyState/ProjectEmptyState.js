import React from 'react'
import './ProjectEmptyState.css'

function ProjectEmptyState() {
  return (
    <div className='project-empty-state-wrapper'>
      <div className='project-empty-state'>
        <div className='project-empty-state-text'>
          <h4>
            Hold G and left click anywhere on canvas to create your first goal
            card.
          </h4>
        </div>
      </div>
    </div>
  )
}

export default ProjectEmptyState

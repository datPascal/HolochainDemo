import { AppWebsocket } from '@holochain/conductor-api/lib/websocket/app'
import { AdminWebsocket } from '@holochain/conductor-api/lib/websocket/admin'

// export for use by holochainMiddleware (redux)
// @ts-ignore
export const APP_WS_URL = `ws://localhost:${__APP_PORT__}`
// @ts-ignore
const ADMIN_WS_URL = `ws://localhost:${__ADMIN_PORT__}`

let appWs: AppWebsocket
let adminWs: AdminWebsocket
let agentPubKey

export async function getAdminWs(): Promise<AdminWebsocket> {
  if (adminWs) {
    return adminWs
  } else {
    adminWs = await AdminWebsocket.connect(ADMIN_WS_URL)
    return adminWs
  }
}

export async function getAppWs(signalsHandler): Promise<AppWebsocket> {
  if (appWs) {
    return appWs
  } else {
    // undefined is for default timeout
    appWs = await AppWebsocket.connect(APP_WS_URL, undefined, signalsHandler)
    return appWs
  }
}

export function getAgentPubKey() {
  return agentPubKey
}

export function setAgentPubKey(setAs) {
  agentPubKey = setAs
}

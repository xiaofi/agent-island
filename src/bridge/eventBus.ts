import type { AgentBridgeSubscriptions } from "@/domain/taskTypes";
import { subscribeAgentEvents } from "@/bridge/tauriApi";

export function connectAgentEventBus(subscriptions: AgentBridgeSubscriptions) {
  return subscribeAgentEvents(subscriptions);
}

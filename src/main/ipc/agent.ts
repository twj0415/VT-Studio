import type { AgentSocketInfo } from '@shared/types/socket';
import { getAgentSocketInfo } from '../services/socket';
import { handleIpc } from './handle';

export function registerAgentIpc(): void {
  handleIpc<AgentSocketInfo>('agent:get-socket-info', () => getAgentSocketInfo());
}

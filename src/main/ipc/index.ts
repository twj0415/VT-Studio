import { registerAgentIpc } from './agent';
import { registerAppIpc } from './app';
import { registerAuthIpc } from './auth';
import { registerMediaIpc } from './media';
import { registerSettingsIpc } from './settings';

export function registerIpc(): void {
  registerAppIpc();
  registerAgentIpc();
  registerAuthIpc();
  registerMediaIpc();
  registerSettingsIpc();
}

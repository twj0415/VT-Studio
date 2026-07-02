export type AgentNamespace = 'scriptAgent' | 'productionAgent';

export type AgentMessageType =
  | 'text'
  | 'markdown'
  | 'thinking'
  | 'toolcall'
  | 'search'
  | 'reasoning'
  | 'suggestion';

export type AgentMessageStatus = 'pending' | 'streaming' | 'complete' | 'stop' | 'error';

export interface AgentToolCallInfo {
  name: string;
  args?: unknown;
  result?: unknown;
}

export interface AgentMessage {
  id: string;
  type: AgentMessageType;
  content: string;
  status: AgentMessageStatus;
  toolcall?: AgentToolCallInfo;
}

export interface AgentMessageUpdatePayload {
  id: string;
  role?: 'assistant' | 'user';
  type?: AgentMessageType;
  content?: string;
  status: AgentMessageStatus;
}

export interface AgentContentBlock {
  id: string;
  messageId: string;
  type: AgentMessageType;
  content: string;
  status: AgentMessageStatus;
  toolcall?: AgentToolCallInfo;
}

export interface AgentContentAddPayload {
  messageId: string;
  content: AgentContentBlock;
}

export interface AgentContentUpdatePayload {
  messageId: string;
  contentId: string;
  patch: Partial<Pick<AgentContentBlock, 'type' | 'content' | 'status' | 'toolcall'>>;
}

export interface AgentSocketAuth {
  token: string;
  isolationKey: string;
  projectId: string | number;
  scriptId?: string | number;
}

export interface AgentChatPayload {
  content: string;
  role?: 'user';
}

export interface AgentRegeneratePayload {
  messageId: string;
}

export interface AgentThinkConfigPayload {
  think: boolean;
  thinkLevel?: 0 | 1 | 2 | 3;
}

export interface AgentContextPayload {
  context: unknown;
}

export interface AgentSocketErrorPayload {
  code: string;
  msg: string;
}

export interface AgentSocketInfo {
  url: string;
  token: string;
}

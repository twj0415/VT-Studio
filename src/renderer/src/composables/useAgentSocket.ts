import { ref } from 'vue';
import { io, type Socket } from 'socket.io-client';
import { VT_STATUS } from '@shared/constants/status';
import type {
  AgentChatPayload,
  AgentContentAddPayload,
  AgentContentBlock,
  AgentContentUpdatePayload,
  AgentContextPayload,
  AgentMessage,
  AgentMessageUpdatePayload,
  AgentNamespace,
  AgentRegeneratePayload,
  AgentSocketAuth,
  AgentSocketErrorPayload,
  AgentThinkConfigPayload,
} from '@shared/types/socket';

interface UseAgentSocketOptions {
  namespace: AgentNamespace;
  auth: Omit<AgentSocketAuth, 'token'>;
}

type SocketEventHandler = (...args: unknown[]) => void;
type LooseSocket = Socket & {
  on(event: string, listener: SocketEventHandler): LooseSocket;
  once(event: string, listener: SocketEventHandler): LooseSocket;
  off(event: string, listener?: SocketEventHandler): LooseSocket;
};

export function useAgentSocket() {
  const socket = ref<Socket | null>(null);
  const isConnected = ref(false);
  const messages = ref<AgentMessage[]>([]);
  const messageUpdates = ref<AgentMessageUpdatePayload[]>([]);
  const contentBlocks = ref<Record<string, AgentContentBlock[]>>({});
  const lastError = ref<AgentSocketErrorPayload | null>(null);

  async function connect(options: UseAgentSocketOptions): Promise<void> {
    disconnect();

    const response = await window.vtStudio.agent.getSocketInfo();
    if (response.code !== VT_STATUS.OK) {
      throw new Error(response.msg);
    }

    const client = io(`${response.data.url}/socket/${options.namespace}`, {
      autoConnect: false,
      transports: ['websocket'],
      auth: {
        ...options.auth,
        token: response.data.token,
      },
    });

    client.on('connect', () => {
      isConnected.value = true;
      lastError.value = null;
    });

    client.on('disconnect', () => {
      isConnected.value = false;
    });

    client.on('message', (message: AgentMessage) => {
      messages.value.push(message);
    });

    client.on('message:update', (payload: AgentMessageUpdatePayload) => {
      messageUpdates.value.push(payload);
      const existing = messages.value.find((message) => message.id === payload.id);
      if (existing) {
        if (payload.type) {
          existing.type = payload.type;
        }
        if (typeof payload.content === 'string') {
          existing.content = payload.content;
        }
        existing.status = payload.status;
      }
    });

    client.on('content:add', (payload: AgentContentAddPayload) => {
      const blocks = contentBlocks.value[payload.messageId] ?? [];
      contentBlocks.value = {
        ...contentBlocks.value,
        [payload.messageId]: [...blocks, payload.content],
      };
    });

    client.on('content:update', (payload: AgentContentUpdatePayload) => {
      const blocks = contentBlocks.value[payload.messageId] ?? [];
      contentBlocks.value = {
        ...contentBlocks.value,
        [payload.messageId]: blocks.map((block) =>
          block.id === payload.contentId ? { ...block, ...payload.patch } : block,
        ),
      };
    });

    client.on('error', (error: AgentSocketErrorPayload) => {
      lastError.value = error;
    });

    client.on('connect_error', (error) => {
      lastError.value = {
        code: String(VT_STATUS.AGENT_ERROR),
        msg: error.message,
      };
    });

    socket.value = client;
    client.connect();
  }

  function disconnect(): void {
    if (!socket.value) {
      return;
    }

    socket.value.disconnect();
    socket.value = null;
    isConnected.value = false;
  }

  function sendChat(content: string): void {
    const payload: AgentChatPayload = { content, role: 'user' };
    socket.value?.emit('chat', payload);
  }

  function stop(): void {
    socket.value?.emit('stop');
  }

  function regenerate(messageId: string): void {
    const payload: AgentRegeneratePayload = { messageId };
    socket.value?.emit('regenerate', payload);
  }

  function updateThinkConfig(payload: AgentThinkConfigPayload): void {
    socket.value?.emit('updateThinkConfig', payload);
  }

  function updateContext(payload: AgentContextPayload): void {
    socket.value?.emit('updateContext', payload);
  }

  function clearMessages(): void {
    messages.value = [];
    messageUpdates.value = [];
    contentBlocks.value = {};
    lastError.value = null;
  }

  function reconnect(): void {
    socket.value?.disconnect();
    socket.value?.connect();
  }

  function on(event: string, handler: SocketEventHandler): void {
    (socket.value as LooseSocket | null)?.on(event, handler);
  }

  function once(event: string, handler: SocketEventHandler): void {
    (socket.value as LooseSocket | null)?.once(event, handler);
  }

  function off(event: string, handler?: SocketEventHandler): void {
    (socket.value as LooseSocket | null)?.off(event, handler);
  }

  return {
    isConnected,
    messages,
    messageUpdates,
    contentBlocks,
    lastError,
    connect,
    reconnect,
    disconnect,
    sendChat,
    stop,
    regenerate,
    updateThinkConfig,
    updateContext,
    clearMessages,
    on,
    once,
    off,
  };
}

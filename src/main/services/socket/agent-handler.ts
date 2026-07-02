import { randomUUID } from 'node:crypto';
import type { Namespace } from 'socket.io';
import { VT_STATUS } from '@shared/constants/status';
import type {
  AgentContentBlock,
  AgentChatPayload,
  AgentContextPayload,
  AgentMessage,
  AgentMessageStatus,
  AgentNamespace,
  AgentRegeneratePayload,
  AgentSocketErrorPayload,
  AgentThinkConfigPayload,
} from '@shared/types/socket';
import { normalizeUnknownError } from '@shared/errors';
import { streamModelText } from '../model';
import { ThinkStreamParser } from './stripThink';
import type { AgentSessionState, AgentSocket } from './types';

const DEFAULT_THINK_CONFIG: Required<AgentThinkConfigPayload> = {
  think: false,
  thinkLevel: 0,
};

function emitSocketError(socket: AgentSocket, msg: string, code = String(VT_STATUS.AGENT_ERROR)): void {
  const payload: AgentSocketErrorPayload = { code, msg };
  socket.emit('error', payload);
}

function emitMessage(socket: AgentSocket, message: AgentMessage): void {
  socket.emit('message', message);
}

function emitMessageUpdate(socket: AgentSocket, message: AgentMessage): void {
  socket.emit('message:update', {
    id: message.id,
    type: message.type,
    content: message.content,
    status: message.status,
    role: 'assistant',
  });
}

function emitContentBlock(
  socket: AgentSocket,
  content: AgentContentBlock,
  emittedContentIds: Set<string>,
): void {
  if (emittedContentIds.has(content.id)) {
    socket.emit('content:update', {
      messageId: content.messageId,
      contentId: content.id,
      patch: {
        content: content.content,
        status: content.status,
        type: content.type,
        toolcall: content.toolcall,
      },
    });
    return;
  }

  emittedContentIds.add(content.id);
  socket.emit('content:add', {
    messageId: content.messageId,
    content,
  });
}

function normalizeChatPayload(payload: unknown): AgentChatPayload {
  if (!payload || typeof payload !== 'object') {
    throw new Error('chat 参数无效');
  }

  const content = (payload as AgentChatPayload).content;
  if (typeof content !== 'string' || content.trim().length === 0) {
    throw new Error('消息内容不能为空');
  }

  return { content, role: 'user' };
}

function normalizeThinkConfig(payload: unknown): Required<AgentThinkConfigPayload> {
  if (!payload || typeof payload !== 'object') {
    return DEFAULT_THINK_CONFIG;
  }

  const think = Boolean((payload as AgentThinkConfigPayload).think);
  const rawThinkLevel = (payload as AgentThinkConfigPayload).thinkLevel;
  const thinkLevel = rawThinkLevel === 1 || rawThinkLevel === 2 || rawThinkLevel === 3 ? rawThinkLevel : 0;

  return { think, thinkLevel };
}

async function consumeModelStream(
  socket: AgentSocket,
  namespace: AgentNamespace,
  content: string,
  state: AgentSessionState,
): Promise<void> {
  const messageId = randomUUID();
  const emittedContentIds = new Set<string>();
  const markdownContentId = `${messageId}:markdown`;
  const thinkingContentId = `${messageId}:thinking`;
  let markdownContent = '';
  let thinkingContent = '';
  const parser = new ThinkStreamParser();

  state.abortController = new AbortController();

  const pendingMessage: AgentMessage = {
    id: messageId,
    type: 'markdown',
    content: '',
    status: 'pending',
  };
  emitMessage(socket, pendingMessage);
  emitMessageUpdate(socket, pendingMessage);

  try {
    const result = streamModelText({
      modelKey: namespace,
      messages: [{ role: 'user', content }],
      think: state.thinkConfig.think,
      thinkLevel: state.thinkConfig.thinkLevel,
      abortSignal: state.abortController.signal,
    });

    for await (const delta of result.textStream) {
      if (state.abortController.signal.aborted) {
        break;
      }

      const parsed = parser.feed(delta);
      if (parsed.thinking) {
        thinkingContent += parsed.thinking;
        const thinkingMessage: AgentMessage = {
          id: messageId,
          type: 'thinking',
          content: parsed.thinking,
          status: 'streaming',
        };
        emitMessage(socket, thinkingMessage);
        emitContentBlock(
          socket,
          {
            id: thinkingContentId,
            messageId,
            type: 'thinking',
            content: thinkingContent,
            status: 'streaming',
          },
          emittedContentIds,
        );
      }
      if (parsed.content) {
        markdownContent += parsed.content;
        const markdownMessage: AgentMessage = {
          id: messageId,
          type: 'markdown',
          content: parsed.content,
          status: 'streaming',
        };
        emitMessage(socket, markdownMessage);
        emitContentBlock(
          socket,
          {
            id: markdownContentId,
            messageId,
            type: 'markdown',
            content: markdownContent,
            status: 'streaming',
          },
          emittedContentIds,
        );
      }
    }

    const flushed = parser.flush();
    if (flushed.thinking) {
      thinkingContent += flushed.thinking;
      const thinkingMessage: AgentMessage = {
        id: messageId,
        type: 'thinking',
        content: flushed.thinking,
        status: 'streaming',
      };
      emitMessage(socket, thinkingMessage);
      emitContentBlock(
        socket,
        {
          id: thinkingContentId,
          messageId,
          type: 'thinking',
          content: thinkingContent,
          status: 'streaming',
        },
        emittedContentIds,
      );
    }
    if (flushed.content) {
      markdownContent += flushed.content;
      const markdownMessage: AgentMessage = {
        id: messageId,
        type: 'markdown',
        content: flushed.content,
        status: 'streaming',
      };
      emitMessage(socket, markdownMessage);
      emitContentBlock(
        socket,
        {
          id: markdownContentId,
          messageId,
          type: 'markdown',
          content: markdownContent,
          status: 'streaming',
        },
        emittedContentIds,
      );
    }

    const finalStatus: AgentMessageStatus = state.abortController.signal.aborted ? 'stop' : 'complete';
    const finalMessage: AgentMessage = {
      id: messageId,
      type: 'markdown',
      content: '',
      status: finalStatus,
    };
    emitMessage(socket, finalMessage);
    emitMessageUpdate(socket, finalMessage);
    for (const contentId of emittedContentIds) {
      socket.emit('content:update', {
        messageId,
        contentId,
        patch: { status: finalStatus },
      });
    }
  } catch (error) {
    const normalized = normalizeUnknownError(error);
    const errorMessage: AgentMessage = {
      id: messageId,
      type: 'markdown',
      content: normalized.message,
      status: 'error',
    };
    emitMessage(socket, errorMessage);
    emitMessageUpdate(socket, errorMessage);
    emitSocketError(socket, normalized.message);
  } finally {
    state.abortController = null;
  }
}

export function registerAgentNamespace(ioNamespace: Namespace, namespace: AgentNamespace): void {
  const sessions = new Map<string, AgentSessionState>();

  ioNamespace.on('connection', (rawSocket) => {
    const socket = rawSocket as AgentSocket;
    const state: AgentSessionState = {
      abortController: null,
      thinkConfig: DEFAULT_THINK_CONFIG,
    };
    sessions.set(socket.id, state);

    socket.on('updateThinkConfig', (payload: unknown) => {
      state.thinkConfig = normalizeThinkConfig(payload);
    });

    socket.on('updateContext', (payload: unknown) => {
      if (namespace !== 'productionAgent') {
        return;
      }
      state.context = (payload as AgentContextPayload | undefined)?.context;
    });

    socket.on('stop', () => {
      state.abortController?.abort();
    });

    socket.on('regenerate', (payload: unknown) => {
      const messageId = (payload as AgentRegeneratePayload | undefined)?.messageId;
      if (typeof messageId !== 'string' || messageId.trim().length === 0) {
        emitSocketError(socket, 'regenerate 参数无效', String(VT_STATUS.INVALID_PARAMS));
        return;
      }

      emitSocketError(socket, '业务重生成待剧本/生产 Agent 接入历史消息后实现', String(VT_STATUS.AGENT_ERROR));
    });

    socket.on('chat', (payload: unknown) => {
      void (async () => {
        if (state.abortController) {
          emitSocketError(socket, '已有 Agent 消息正在生成中，请先停止或等待完成', String(VT_STATUS.CONFLICT));
          return;
        }

        try {
          const chatPayload = normalizeChatPayload(payload);
          await consumeModelStream(socket, namespace, chatPayload.content, state);
        } catch (error) {
          const normalized = normalizeUnknownError(error);
          emitSocketError(socket, normalized.message);
        }
      })();
    });

    socket.on('disconnect', () => {
      state.abortController?.abort();
      sessions.delete(socket.id);
    });
  });
}

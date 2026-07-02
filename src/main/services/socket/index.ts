import type { Server as HttpServer } from 'node:http';
import { Server as SocketIOServer } from 'socket.io';
import { VT_STATUS } from '@shared/constants/status';
import type { AgentSocketInfo } from '@shared/types/socket';
import { getDatabase } from '../database';
import { logger } from '../logger';
import { authenticateAgentSocket } from './auth';
import { registerAgentNamespace } from './agent-handler';
import type { AgentSocket } from './types';

let io: SocketIOServer | null = null;
let socketUrl: string | null = null;

function getTokenKey(): string {
  const row = getDatabase()
    .prepare<[], { value: string }>("SELECT value FROM app_settings WHERE key = 'tokenKey' LIMIT 1")
    .get();

  if (!row?.value) {
    throw new Error('tokenKey 未初始化');
  }

  return row.value;
}

export function startSocketService(server: HttpServer, localServerUrl: string): void {
  if (io) {
    socketUrl = localServerUrl;
    return;
  }

  io = new SocketIOServer(server, {
    serveClient: false,
    cors: { origin: false },
  });
  socketUrl = localServerUrl;

  const scriptAgentNamespace = io.of('/socket/scriptAgent');
  scriptAgentNamespace.use((rawSocket, next) => {
    try {
      const socket = rawSocket as AgentSocket;
      socket.data = authenticateAgentSocket(socket.handshake.auth, 'scriptAgent');
      next();
    } catch (error) {
      next(Object.assign(error instanceof Error ? error : new Error('Socket 鉴权失败'), { data: { code: VT_STATUS.UNAUTHORIZED } }));
    }
  });
  registerAgentNamespace(scriptAgentNamespace, 'scriptAgent');

  const productionAgentNamespace = io.of('/socket/productionAgent');
  productionAgentNamespace.use((rawSocket, next) => {
    try {
      const socket = rawSocket as AgentSocket;
      socket.data = authenticateAgentSocket(socket.handshake.auth, 'productionAgent');
      next();
    } catch (error) {
      next(Object.assign(error instanceof Error ? error : new Error('Socket 鉴权失败'), { data: { code: VT_STATUS.UNAUTHORIZED } }));
    }
  });
  registerAgentNamespace(productionAgentNamespace, 'productionAgent');

  logger.info('Agent 连接', '已就绪');
  logger.detail('Agent 连接', 'Socket 服务详情', { url: localServerUrl });
}

export function getAgentSocketInfo(): AgentSocketInfo {
  if (!socketUrl) {
    throw new Error('Socket 服务未启动');
  }

  return {
    url: socketUrl,
    token: getTokenKey(),
  };
}

export async function stopSocketService(): Promise<void> {
  if (!io) {
    return;
  }

  const currentIo = io;
  io = null;
  socketUrl = null;

  await new Promise<void>((resolve) => {
    currentIo.close(() => resolve());
  });

  logger.info('Agent 连接', '已停止');
}

import { createServer, type Server as HttpServer } from 'node:http';
import { logger } from '../services/logger';
import { handleMediaRequest } from '../services/media/request-handler';

let server: HttpServer | null = null;
let serverUrl: string | null = null;

export interface LocalServerInfo {
  server: HttpServer;
  url: string;
  port: number;
}

export async function startLocalServer(): Promise<LocalServerInfo> {
  if (server && serverUrl) {
    const address = server.address();
    const port = typeof address === 'object' && address ? address.port : 0;
    return { server, url: serverUrl, port };
  }

  server = createServer(async (req, res) => {
    if (await handleMediaRequest(req, res)) {
      return;
    }

    res.statusCode = 404;
    res.setHeader('content-type', 'application/json; charset=utf-8');
    res.end(JSON.stringify({ code: 404, msg: 'Not Found' }));
  });

  await new Promise<void>((resolve, reject) => {
    const onError = (error: Error) => {
      server?.off('listening', onListening);
      reject(error);
    };
    const onListening = () => {
      server?.off('error', onError);
      resolve();
    };

    server!.once('error', onError);
    server!.once('listening', onListening);
    server!.listen(0, '127.0.0.1');
  });

  const address = server.address();
  if (!address || typeof address !== 'object') {
    throw new Error('本地服务启动失败');
  }

  serverUrl = `http://127.0.0.1:${address.port}`;
  logger.info('本地服务', `已启动：${serverUrl}`);
  logger.detail('本地服务', '本地服务详情', { url: serverUrl, port: address.port });

  return { server, url: serverUrl, port: address.port };
}

export function getLocalServerUrl(): string {
  if (!serverUrl) {
    throw new Error('本地服务未启动');
  }

  return serverUrl;
}

export async function stopLocalServer(): Promise<void> {
  if (!server) {
    return;
  }

  const currentServer = server;
  server = null;
  serverUrl = null;

  await new Promise<void>((resolve, reject) => {
    currentServer.close((error: NodeJS.ErrnoException | undefined) => {
      if (!error || error.code === 'ERR_SERVER_NOT_RUNNING') {
        resolve();
        return;
      }

      reject(error);
    });
  });

  logger.info('本地服务', '已停止');
}

import { appendFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { normalizeUnknownError } from '@shared/errors';
import { getUserDataPath } from '../app/runtime';

export type LogLevel = 'debug' | 'info' | 'warn' | 'error' | 'fatal';

export interface LogMeta {
  detail?: unknown;
  terminal?: boolean;
}

const SENSITIVE_KEYS = ['apikey', 'api_key', 'authorization', 'password', 'secret', 'token'];
const SCOPE_WIDTH = 12;
const TERMINAL_SPACING = true;
const LEVEL_LABEL: Record<LogLevel, string> = {
  debug: 'DEBUG',
  info: 'INFO',
  warn: 'WARN',
  error: 'ERROR',
  fatal: 'FATAL',
};
const COLORS = {
  reset: '\x1b[0m',
  dim: '\x1b[2m',
  cyan: '\x1b[36m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  magenta: '\x1b[35m',
} as const;

function getLogFilePath(): string {
  const logsDirectory = join(getUserDataPath(), 'logs');
  mkdirSync(logsDirectory, { recursive: true });
  return join(logsDirectory, 'main.log');
}

function shouldMaskKey(key: string): boolean {
  const normalized = key.toLowerCase();
  return SENSITIVE_KEYS.some((sensitiveKey) => normalized.includes(sensitiveKey));
}

function maskSensitive(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(maskSensitive);
  }

  if (!value || typeof value !== 'object') {
    return value;
  }

  if (value instanceof Error) {
    return normalizeUnknownError(value);
  }

  const result: Record<string, unknown> = {};
  for (const [key, item] of Object.entries(value as Record<string, unknown>)) {
    result[key] = shouldMaskKey(key) ? '[已隐藏]' : maskSensitive(item);
  }

  return result;
}

function supportsColor(): boolean {
  return Boolean(process.stdout.isTTY && !process.env.NO_COLOR);
}

function colorize(value: string, color: keyof typeof COLORS): string {
  if (!supportsColor()) {
    return value;
  }

  return `${COLORS[color]}${value}${COLORS.reset}`;
}

function getDisplayWidth(value: string): number {
  let width = 0;

  for (const char of value) {
    width += char.charCodeAt(0) > 0x7f ? 2 : 1;
  }

  return width;
}

function padDisplayEnd(value: string, targetWidth: number): string {
  const width = getDisplayWidth(value);
  return width >= targetWidth ? value : `${value}${' '.repeat(targetWidth - width)}`;
}

function getTimeLabel(): string {
  const date = new Date();
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');
  return `${hours}:${minutes}:${seconds}`;
}

function getLevelColor(level: LogLevel): keyof typeof COLORS {
  if (level === 'warn') {
    return 'yellow';
  }

  if (level === 'error' || level === 'fatal') {
    return 'red';
  }

  if (level === 'debug') {
    return 'magenta';
  }

  return 'green';
}

function writeLogFile(level: LogLevel, scope: string, message: string, detail?: unknown): void {
  const entry = {
    time: new Date().toISOString(),
    level,
    scope,
    message,
    detail: detail === undefined ? undefined : maskSensitive(detail),
  };

  try {
    appendFileSync(getLogFilePath(), `${JSON.stringify(entry)}\n`, 'utf8');
  } catch {
    // 日志写入失败不能影响应用启动。
  }
}

function writeTerminal(level: LogLevel, scope: string, message: string): void {
  const time = colorize(getTimeLabel(), 'dim');
  const levelLabel = colorize(padDisplayEnd(LEVEL_LABEL[level], 5), getLevelColor(level));
  const scopeLabel = colorize(padDisplayEnd(scope, SCOPE_WIDTH), 'cyan');
  const separator = colorize('│', 'dim');
  const line = `${time} ${levelLabel} ${separator} ${scopeLabel} ${separator} ${message}${TERMINAL_SPACING ? '\n' : ''}`;

  if (level === 'error' || level === 'fatal') {
    console.error(line);
    return;
  }

  if (level === 'warn') {
    console.warn(line);
    return;
  }

  console.info(line);
}

function writeTerminalSection(title: string): void {
  const lineWidth = 72;
  const titleText = ` ${title} `;
  const titleWidth = getDisplayWidth(titleText);
  const sideWidth = Math.max(6, Math.floor((lineWidth - titleWidth) / 2));
  const rightWidth = Math.max(6, lineWidth - titleWidth - sideWidth);
  const line = colorize(`${'─'.repeat(sideWidth)}${titleText}${'─'.repeat(rightWidth)}`, 'dim');
  console.info(TERMINAL_SPACING ? `\n${line}\n` : line);
}

export function log(level: LogLevel, scope: string, message: string, meta: LogMeta = {}): void {
  writeLogFile(level, scope, message, meta.detail);

  if (meta.terminal !== false) {
    writeTerminal(level, scope, message);
  }
}

export const logger = {
  debug(scope: string, message: string, detail?: unknown): void {
    log('debug', scope, message, { detail, terminal: false });
  },
  info(scope: string, message: string, detail?: unknown): void {
    log('info', scope, message, { detail });
  },
  detail(scope: string, message: string, detail?: unknown): void {
    log('debug', scope, message, { detail, terminal: false });
  },
  warn(scope: string, message: string, detail?: unknown): void {
    log('warn', scope, message, { detail });
  },
  error(scope: string, message: string, error?: unknown): void {
    const detail = error instanceof Error ? normalizeUnknownError(error) : error;
    log('error', scope, message, { detail });
  },
  fatal(scope: string, message: string, error?: unknown): void {
    const detail = error instanceof Error ? normalizeUnknownError(error) : error;
    log('fatal', scope, message, { detail });
  },
  section(title: string): void {
    writeLogFile('info', 'section', title);
    writeTerminalSection(title);
  },
};

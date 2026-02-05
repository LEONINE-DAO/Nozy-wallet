/**
 * Simple logging utility for NozyWallet Desktop Client
 * Logs errors and important events without exposing sensitive data
 */

type LogLevel = 'info' | 'warn' | 'error' | 'debug';

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  context?: Record<string, any>;
  error?: {
    message: string;
    stack?: string;
  };
}

class Logger {
  private logs: LogEntry[] = [];
  private maxLogs = 1000; // Keep last 1000 log entries
  private enabled = true;

  private sanitizeContext(context?: Record<string, any>): Record<string, any> | undefined {
    if (!context) return undefined;

    const sanitized: Record<string, any> = {};
    const sensitiveKeys = ['password', 'secret', 'key', 'mnemonic', 'private', 'token'];

    for (const [key, value] of Object.entries(context)) {
      const lowerKey = key.toLowerCase();
      if (sensitiveKeys.some(sk => lowerKey.includes(sk))) {
        sanitized[key] = '[REDACTED]';
      } else if (typeof value === 'object' && value !== null) {
        sanitized[key] = this.sanitizeContext(value as Record<string, any>);
      } else {
        sanitized[key] = value;
      }
    }

    return sanitized;
  }

  private addLog(level: LogLevel, message: string, context?: Record<string, any>, error?: Error) {
    if (!this.enabled) return;

    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      context: this.sanitizeContext(context),
      error: error
        ? {
            message: error.message,
            stack: error.stack,
          }
        : undefined,
    };

    this.logs.push(entry);

    // Keep only last maxLogs entries
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }

    // Also log to console in development
    if (import.meta.env.DEV) {
      const consoleMethod = level === 'error' ? console.error : level === 'warn' ? console.warn : console.log;
      consoleMethod(`[${level.toUpperCase()}] ${message}`, context || '', error || '');
    }
  }

  info(message: string, context?: Record<string, any>) {
    this.addLog('info', message, context);
  }

  warn(message: string, context?: Record<string, any>) {
    this.addLog('warn', message, context);
  }

  error(message: string, error?: Error, context?: Record<string, any>) {
    this.addLog('error', message, context, error);
  }

  debug(message: string, context?: Record<string, any>) {
    if (import.meta.env.DEV) {
      this.addLog('debug', message, context);
    }
  }

  getLogs(level?: LogLevel, limit?: number): LogEntry[] {
    let filtered = this.logs;
    if (level) {
      filtered = this.logs.filter(log => log.level === level);
    }
    if (limit) {
      return filtered.slice(-limit);
    }
    return filtered;
  }

  exportLogs(): string {
    return JSON.stringify(this.logs, null, 2);
  }

  clearLogs() {
    this.logs = [];
  }

  enable() {
    this.enabled = true;
  }

  disable() {
    this.enabled = false;
  }
}

// Singleton instance
export const logger = new Logger();

// Export for use in components
export default logger;

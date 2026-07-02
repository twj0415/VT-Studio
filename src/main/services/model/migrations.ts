import type { Migration } from '../database/migrations';

export const modelMigrations: Migration[] = [
  {
    id: '0003_create_model_tables',
    name: 'create model vendor and agent config tables',
    statements: [
      `
      CREATE TABLE IF NOT EXISTS model_vendors (
        id TEXT PRIMARY KEY,
        input_values TEXT NOT NULL DEFAULT '{}',
        models TEXT NOT NULL DEFAULT '[]',
        enabled INTEGER NOT NULL DEFAULT 0,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
      `,
      `
      CREATE TABLE IF NOT EXISTS agent_model_configs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        key TEXT NOT NULL UNIQUE,
        name TEXT NULL,
        description TEXT NULL,
        model_label TEXT NULL,
        model_id TEXT NULL,
        vendor_id TEXT NULL,
        temperature REAL NULL,
        max_output_tokens INTEGER NULL,
        disabled INTEGER NOT NULL DEFAULT 0,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
      `,
      `
      CREATE TABLE IF NOT EXISTS app_settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
      `,
      'CREATE INDEX IF NOT EXISTS idx_model_vendors_enabled ON model_vendors(enabled)',
      'CREATE INDEX IF NOT EXISTS idx_agent_model_configs_key ON agent_model_configs(key)',
      'CREATE INDEX IF NOT EXISTS idx_agent_model_configs_vendor_id ON agent_model_configs(vendor_id)',
    ],
  },
];

import type { Migration } from '../database/migrations';

export const modelPromptMigrations: Migration[] = [
  {
    id: '0006_create_model_prompt_tables',
    name: 'create model prompt template and mapping tables',
    statements: [
      `
      CREATE TABLE IF NOT EXISTS model_prompt_templates (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        name       TEXT    NOT NULL,
        type       TEXT    NOT NULL,
        content    TEXT    NOT NULL DEFAULT '',
        is_builtin INTEGER NOT NULL DEFAULT 0,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
      `,
      `
      CREATE TABLE IF NOT EXISTS model_prompt_mappings (
        id            INTEGER PRIMARY KEY AUTOINCREMENT,
        connection_id TEXT    NOT NULL,
        model_name    TEXT    NOT NULL,
        model_type    TEXT    NOT NULL,
        model_mode    TEXT    NOT NULL DEFAULT '',
        template_id   INTEGER NOT NULL,
        created_at    INTEGER NOT NULL,
        updated_at    INTEGER NOT NULL
      )
      `,
      'CREATE UNIQUE INDEX IF NOT EXISTS idx_model_prompt_templates_type_name ON model_prompt_templates(type, name)',
      'CREATE UNIQUE INDEX IF NOT EXISTS idx_model_prompt_mappings_target ON model_prompt_mappings(connection_id, model_name, model_type, model_mode)',
      'CREATE INDEX IF NOT EXISTS idx_model_prompt_mappings_template_id ON model_prompt_mappings(template_id)',
    ],
  },
];

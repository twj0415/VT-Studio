import type { Migration } from './migrations';

/**
 * CORE-009：新增业务基础表
 * users / prompts / skill_list / skill_attributions
 * model_vendors / agent_model_configs / app_settings 已在 model/migrations 中创建
 */
export const seedMigrations: Migration[] = [
  {
    id: '0004_create_seed_tables',
    name: 'create users, prompts, skill_list, skill_attributions tables',
    statements: [
      `
      CREATE TABLE IF NOT EXISTS users (
        id       INTEGER PRIMARY KEY,
        name     TEXT    NOT NULL,
        password TEXT    NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
      `,
      `
      CREATE TABLE IF NOT EXISTS prompts (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        name       TEXT    NOT NULL,
        type       TEXT    NOT NULL,
        data       TEXT    NOT NULL DEFAULT '',
        use_data   TEXT    NOT NULL DEFAULT '',
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
      `,
      'CREATE UNIQUE INDEX IF NOT EXISTS idx_prompts_type ON prompts(type)',
      `
      CREATE TABLE IF NOT EXISTS skill_list (
        id          TEXT    PRIMARY KEY,
        md5         TEXT    NOT NULL,
        path        TEXT    NOT NULL,
        name        TEXT    NOT NULL,
        description TEXT    NOT NULL DEFAULT '',
        embedding   TEXT    NOT NULL DEFAULT '',
        type        TEXT    NOT NULL,
        created_at  INTEGER NOT NULL,
        updated_at  INTEGER NOT NULL,
        state       INTEGER NOT NULL DEFAULT -1
      )
      `,
      'CREATE INDEX IF NOT EXISTS idx_skill_list_type  ON skill_list(type)',
      'CREATE INDEX IF NOT EXISTS idx_skill_list_state ON skill_list(state)',
      `
      CREATE TABLE IF NOT EXISTS skill_attributions (
        skill_id    TEXT NOT NULL,
        attribution TEXT NOT NULL,
        PRIMARY KEY (skill_id, attribution),
        FOREIGN KEY (skill_id) REFERENCES skill_list(id) ON DELETE CASCADE
      )
      `,
      'CREATE INDEX IF NOT EXISTS idx_skill_attributions_attribution ON skill_attributions(attribution)',
    ],
  },
];

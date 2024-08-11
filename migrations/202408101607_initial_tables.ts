import { Knex } from 'knex';

export const up = (knex: Knex) =>
  knex.schema
    .createTable('thread', (t) => {
      t.uuid('id').primary().notNullable();
      t.string('title', 400).notNullable();
      t.specificType('visibleToUserIds', 'varchar(100)[]');
      t.timestamps();
    })
    .raw(
      'create index thread_visibletouserids_idx on thread using GIN ("visible_to_user_ids")',
    )
    .createTable('message', (t) => {
      t.uuid('id').primary().notNullable();
      t.uuid('threadId').notNullable().references('id').inTable('thread');
      t.string('senderId').notNullable(); // string not uuid so we can describe the llm bots separately
      t.text('content').notNullable();
      t.timestamps();
      t.index('threadId');
      t.index('senderId');
    });

export const down = (knex: Knex) =>
  knex.schema.dropTable('message').dropTable('thread');

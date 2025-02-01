import { Knex } from 'knex';

export const up = async (knex: Knex) => {
  await knex.schema.table('message', (t) => {
    t.string('role', 20);
  });
  await knex.raw(`UPDATE message SET role = 'user' WHERE length(sender_id) = 36`);
  await knex.raw(`UPDATE message SET role = 'assistant' WHERE length(sender_id) <> 36`);
  await knex.schema.alterTable('message', (t) => {
    t.string('role').notNullable().alter();
  });
}

export const down = (knex: Knex) =>
  knex.schema.table('message', (t) => t.dropColumn('role'));

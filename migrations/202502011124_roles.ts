import { Knex } from 'knex';

export const up = async (knex: Knex) => {
  await knex.schema.table('message', (t) => {
    t.string('role', 20);
    t.jsonb('toolCallJson');
  });
  await knex.raw(
    `UPDATE message SET role = 'user' WHERE length(sender_id) = 36`,
  );
  await knex.raw(
    `UPDATE message SET role = 'assistant' WHERE length(sender_id) <> 36`,
  );
};

export const down = (knex: Knex) =>
  knex.schema.table('message', (t) => {
    t.dropColumn('role');
    t.dropColumn('toolCallJson');
  });

import { Knex } from 'knex';

export const up = async (knex: Knex) => {
  await knex.schema.table('message', (t) => {
    t.renameColumn('toolCallJson', 'toolCalls');
    t.string('toolResultCallId', 36).nullable();
    t.dropColumn('role');
  });
};
export const down = async (knex: Knex) => {
  await knex.schema.table('message', (t) => {
    t.renameColumn('toolCalls', 'toolCallJson');
    t.dropColumn('toolResultCallId');
    t.string('role', 20);
  });
};
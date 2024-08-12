import { Knex } from 'knex';

export interface Thread {
  id: string;
  title: string;
  visibleToUserIds?: string[];
  botId?: string;
}

const insert = (db: Knex) => (thread: Thread) =>
  db
    .insert({
      ...thread,
      // visibleToUserIds:
      //   thread.visibleToUserIds === null
      //     ? null
      //     : JSON.stringify(thread.visibleToUserIds),
      createdAt: new Date(),
    })
    .into('thread')
    .returning('*')
    .then((r) => r[0]);

const upsert = (db: Knex) => (thread: Thread) =>
  db
    .insert({
      ...thread,
      // visibleToUserIds:
      //   thread.visibleToUserIds === null
      //     ? null
      //     : JSON.stringify(thread.visibleToUserIds),
      createdAt: new Date(),
    })
    .into('thread')
    .onConflict('id')
    .merge({
      ...thread,
      // visibleToUserIds:
      //   thread.visibleToUserIds === null
      //     ? null
      //     : JSON.stringify(thread.visibleToUserIds),
      updatedAt: new Date(),
    })
    .returning('*')
    .then((r) => r[0]);

const get =
  (db: Knex) =>
  (id: string): Promise<Thread> =>
    db
      .select()
      .from('thread')
      .where('id', id)
      .then((r) => (r.length ? r[0] : null));

const getAll = (db: Knex) => () =>
  db.select().from('thread').orderBy('createdAt', 'asc');

const getAllByUserId = (db: Knex) => (userId: string) =>
  db
    .select()
    .from('thread')
    .whereNull('visibleToUserIds')
    .orWhere(db.raw('? = any(visible_to_user_ids)', userId))
    .orderBy('createdAt', 'asc');

export default (db: Knex) => ({
  insert: insert(db),
  upsert: upsert(db),
  get: get(db),
  getAll: getAll(db),
  getAllByUserId: getAllByUserId(db),
});

import { Knex } from 'knex';

export interface Message {
  id: string;
  threadId: string;
  senderId: string;
  content: string;
  role?: string;
  toolCallJson?: string; // only applicable when role = assistant
}

const insert = (db: Knex) => (message: Message) =>
  db
    .insert({ ...message, createdAt: new Date() })
    .into('message')
    .returning('*')
    .then((r) => r[0]);

const upsert = (db: Knex) => (message: Message) =>
  db
    .insert({ ...message, createdAt: new Date() })
    .into('message')
    .onConflict('id')
    .merge({ ...message, updatedAt: new Date() })
    .returning('*')
    .then((r) => r[0]);

const get =
  (db: Knex) =>
  (id: string): Promise<Message> =>
    db
      .select()
      .from('message')
      .where('id', id)
      .then((r) => (r.length ? r[0] : null));

const getAll = (db: Knex) => () =>
  db.select().from('message').orderBy('createdAt', 'asc');

const getAllByThreadId = (db: Knex) => (threadId: string) =>
  db
    .select()
    .from('message')
    .where('threadId', threadId)
    .orderBy('createdAt', 'asc');

const getSenderIdsByThreadId = (db: Knex) => (threadId: string) =>
  db
    .select('senderId')
    .from('message')
    .where('threadId', threadId)
    .groupBy('senderId')
    .then((rows) => rows.map((row: any) => row.senderId));

const del = (db: Knex) => (id: string) => db.del().where('id', id);

export default (db: Knex) => ({
  insert: insert(db),
  upsert: upsert(db),
  get: get(db),
  getAll: getAll(db),
  getAllByThreadId: getAllByThreadId(db),
  getSenderIdsByThreadId: getSenderIdsByThreadId(db),
  del: del(db),
});

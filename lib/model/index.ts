import { knex, Knex } from 'knex';
import knexFile from '../knexfile';
import thread from './thread';
import message from './message';

const db: Knex = knex(knexFile.main);

export const healthcheck = () => db.select().from('knex_migrations').limit(1);

export const shutdown = () => db.destroy();

export const threadModel = thread(db);
export const messageModel = message(db);

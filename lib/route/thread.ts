import express from 'express';
import { v4 } from 'uuid';
import { logging } from '@hikari-systems/hs.utils';

import { threadModel } from '../model';

const log = logging('routes:site');

const router = express.Router();
// const jsonParser = express.json();

router.get('/:threadId', async (req, res, next) => {
  const threadId = req.params.threadId as string;
  if (!threadId) {
    return res.status(400).send(`No threadId provided`);
  }
  try {
    const thread = await threadModel.get(threadId);
    if (!thread) {
      log.debug(`no thread found for id ${threadId}`);
      return res.status(204).end();
    }
    return res.status(200).json(thread);
  } catch (e) {
    log.error(`Error fetching thread for id ${threadId}`, e);
    return next(e);
  }
});

router.get('/byUserId/:userId', async (req, res, next) => {
  const userId = req.params.userId as string;
  if (!userId) {
    return res.status(400).send(`No userId provided`);
  }
  try {
    const threads = await threadModel.getAllByUserId(userId);
    if (!threads) {
      log.debug(`no threads found for userId ${userId}`);
      return res.status(200).json([]);
    }
    return res.status(200).json(threads);
  } catch (e) {
    log.error(`Error fetching threads for userId ${userId}`, e);
    return next(e);
  }
});

router.post('/', express.json(), async (req, res, next) => {
  const { title, visibleToUserIds, botId } = req.body as {
    title: string;
    visibleToUserIds?: string[];
    botId?: string;
  };
  try {
    const thread = await threadModel.insert({
      id: v4(),
      title,
      visibleToUserIds,
      botId,
    });
    return res.status(201).json(thread);
  } catch (e) {
    log.error(`Error adding thread for ${JSON.stringify(req.body)}`, e);
    return next(e);
  }
});

export default router;

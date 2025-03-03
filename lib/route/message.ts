import express from 'express';
import { v4 } from 'uuid';
import { logging } from '@hikari-systems/hs.utils';
import { messageModel } from '../model';

const log = logging('routes:treatment_item');

const router = express.Router();
// const jsonParser = express.json();

router.get('/byThreadId/:threadId', async (req, res, next) => {
  const threadId = req.params.threadId as string;
  if (!threadId) {
    return res.status(400).send(`No threadId provided`);
  }
  try {
    const messages = await messageModel.getAllByThreadId(threadId);
    if (!messages) {
      log.debug(`no messages found for threadId ${threadId}`);
      return res.status(200).json([]);
    }
    return res.status(200).json(messages);
  } catch (e) {
    log.error(`Error fetching messages for threadId ${threadId}`, e);
    return next(e);
  }
});

router.get('/senderIdsByThreadId/:threadId', async (req, res, next) => {
  const threadId = req.params.threadId as string;
  if (!threadId) {
    return res.status(400).send(`No threadId provided`);
  }
  try {
    const senderIds = await messageModel.getSenderIdsByThreadId(threadId);
    if (!senderIds) {
      log.debug(`No senderIds found for threadId ${threadId}`);
      return res.status(200).json([]);
    }
    return res.status(200).json(senderIds);
  } catch (e) {
    log.error(`Error fetching senderIds for threadId ${threadId}`, e);
    return next(e);
  }
});

router.post('/', express.json(), async (req, res, next) => {
  const { content, threadId, senderId, role, toolCallJson } = req.body as {
    threadId: string;
    senderId: string;
    content: string;
    role?: string;
    toolCallJson?: string;
  };
  try {
    const msg = await messageModel.insert({
      id: v4(),
      content,
      senderId,
      threadId,
      role,
      toolCallJson,
    });
    return res.status(201).json(msg);
  } catch (e) {
    log.error(`Error adding msg for ${JSON.stringify(req.body)}`, e);
    return next(e);
  }
});

export default router;

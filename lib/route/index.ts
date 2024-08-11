import express from 'express';
import threadRoutes from './thread';
import messageRoutes from './message';

const router = express.Router();

router.use('/thread', threadRoutes);
router.use('/message', messageRoutes);

export default router;

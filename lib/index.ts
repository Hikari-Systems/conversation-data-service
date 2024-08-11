import express from 'express';
import path from 'path';
import routes from './route';

const app = express.Router();

app.get('/healthcheck', (_req, res) => res.status(200).send('OK'));
app.use('/api', routes);
app.use('/', express.static(path.join(__dirname, '../static')));

export default app;

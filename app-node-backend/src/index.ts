// src/index.ts

import express from 'express';
import routes from './routes';
import errorHandler from './middleware/errorHandler';
import { logger } from './utils/logger';
import prisma from './config/database';

const app = express();
const PORT = process.env.PORT || 3000;

app.use(express.json());

// Routes
app.use('/api', routes);

// Error handling middleware
app.use(errorHandler);

// Graceful shutdown
process.on('SIGINT', async () => {
  await prisma.$disconnect();
  process.exit();
});

app.listen(PORT, () => {
  logger.info(`Server is running on port ${PORT}`);
});
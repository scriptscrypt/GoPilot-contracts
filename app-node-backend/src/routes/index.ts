// src/routes/index.ts

import { Router } from 'express';
import userRoutes from './userRoutes';
import imageKitRoutes from './imageKitRoutes';

const router = Router();

router.use('/users', userRoutes);
router.use('/ik', imageKitRoutes);

export default router;
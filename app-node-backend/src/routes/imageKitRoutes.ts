// src/imageKitRoute.ts
import express from 'express';
import ImageKit from 'imagekit';
// env
import dotenv from 'dotenv';
import { ENV_IK_PRIVATE_KEY, ENV_IK_PUBLIC_KEY, ENV_IK_URL_ENDPOINT } from '../config/secrets';
dotenv.config();

const router = express.Router();

const imagekit = new ImageKit({
  publicKey: ENV_IK_PUBLIC_KEY || '',
  privateKey: ENV_IK_PRIVATE_KEY || '',
  urlEndpoint: ENV_IK_URL_ENDPOINT || ''
});

router.get('/imagekit-auth', (req, res) => {
  console.log(req)
  const authenticationParameters = imagekit.getAuthenticationParameters();
  res.json(authenticationParameters);
});


export default router;
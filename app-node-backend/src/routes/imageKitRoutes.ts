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

router.post('/upload', express.json({ limit: '10mb' }), async (req, res) => {
  try {
    const { image, fileName } = req.body;

    if (!image || !fileName) {
      return res.status(400).json({ error: 'Image data and file name are required' });
    }

    // Remove the data:image/[type];base64, part
    const base64Image = image.split(';base64,').pop();

    const uploadResponse = await imagekit.upload({
      file: base64Image,
      fileName: fileName,
    });

    res.json({
      success: true,
      url: uploadResponse.url,
      fileId: uploadResponse.fileId
    });
  } catch (error) {
    console.error('Upload error:', error);
    res.status(500).json({ error: 'Failed to upload image' });
  }
});

export default router;
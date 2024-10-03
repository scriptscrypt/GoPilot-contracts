// src/config/secrets.ts

import dotenv from 'dotenv';
dotenv.config();

export const ENV_IK_PUBLIC_KEY = process.env.IK_PUBLIC_KEY;
export const ENV_IK_PRIVATE_KEY = process.env.IK_PRIVATE_KEY;
export const ENV_IK_URL_ENDPOINT = process.env.IK_URL_ENDPOINT;
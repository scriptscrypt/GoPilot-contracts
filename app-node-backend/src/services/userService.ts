// src/services/userService.ts

import prisma from '../config/database';
import { CreateUserDto, UpdateUserDto } from '../models/user';

export const createUser = async (data: CreateUserDto) => {
  return prisma.user.create({ data });
};

export const getUsers = async () => {
  return prisma.user.findMany();
};

export const getUserById = async (id: number) => {
  return prisma.user.findUnique({ where: { id } });
};

export const updateUser = async (id: number, data: UpdateUserDto) => {
  return prisma.user.update({ where: { id }, data });
};

export const deleteUser = async (id: number) => {
  return prisma.user.delete({ where: { id } });
};
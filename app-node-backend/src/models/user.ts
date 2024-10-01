// src/models/user.ts

export interface User {
    id: number;
    name: string;
    email: string;
  }
  
  export interface CreateUserDto {
    name: string;
    email: string;
  }
  
  export interface UpdateUserDto {
    name?: string;
    email?: string;
  }
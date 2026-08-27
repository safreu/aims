export type RegisterRequest = {
  email: string;
  display_name: string;
  password: string;
};

export type RegisterResponse = {
  id: string;
};

export type LoginRequest = {
  email: string;
  password: string;
};

export type LoginResponse = {
  id: string;
};

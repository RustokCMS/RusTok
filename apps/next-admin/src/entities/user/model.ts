export type UserRole = 'SUPER_ADMIN' | 'ADMIN' | 'MANAGER' | 'CUSTOMER';
export type UserStatus = 'ACTIVE' | 'INACTIVE' | 'BANNED' | 'PENDING';

export interface User {
  id: string;
  email: string;
  name: string | null;
  role: UserRole;
  status: UserStatus;
  createdAt: string;
  tenantName: string | null;
}

export interface UserEdge {
  node: User;
}

export interface UsersConnection {
  edges: UserEdge[];
  pageInfo: { totalCount: number };
}

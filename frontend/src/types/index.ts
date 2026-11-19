export interface User {
  id: string;
  email: string;
  displayName: string;
  avatarUrl?: string;
  role: 'admin' | 'member' | 'viewer';
  isActive: boolean;
}

export interface Team {
  id: string;
  name: string;
  slug: string;
  description?: string;
  ownerId: string;
  memberCount: number;
}

export interface Channel {
  id: string;
  teamId: string;
  name: string;
  slug: string;
  type: 'text' | 'issues' | 'notes' | 'general';
  description?: string;
  isPrivate: boolean;
  requiredRole?: 'admin' | 'member' | 'viewer';
}

export interface Issue {
  id: string;
  teamId: string;
  channelId: string;
  title: string;
  description?: string;
  status: 'open' | 'in_progress' | 'review' | 'done' | 'closed';
  priority: 'low' | 'medium' | 'high' | 'critical';
  assigneeId?: string;
  reporterId: string;
  githubIssueNumber?: number;
  githubPrNumber?: number;
}

export interface Note {
  id: string;
  teamId: string;
  channelId: string;
  title: string;
  content: string;
  color: string;
  positionX: number;
  positionY: number;
  authorId: string;
  isPinned: boolean;
}

export interface Message {
  id: string;
  channelId: string;
  userId: string;
  username: string;
  displayName: string;
  content: string;
  messageType: 'text' | 'system' | 'github_commit' | 'github_pr';
  githubCommitSha?: string;
  githubPrNumber?: number;
  parentId?: string;
  createdAt: string;
}

export interface GithubCommit {
  sha: string;
  message: string;
  author: string;
  url: string;
  timestamp: string;
}

export interface GithubPullRequest {
  number: number;
  title: string;
  state: string;
  url: string;
  author: string;
  createdAt: string;
}

export interface AuthResponse {
  token: string;
  user: UserResponse;
}

export interface UserResponse {
  id: string;
  email: string;
  displayName: string;
  avatarUrl?: string;
  role: string;
  isActive: boolean;
}
export interface FileObj {
  id: string;
  filename: string;
  size: number;
}

export interface StatisticsObj {
  downloads: string[];
  views: string[];
}

export interface EmailEntry {
  email: string;
}

export interface Transfer {
  id: string;
  name: string;
  description: string | null;
  expiresAt: string;
  createdAt: string;
  secretCode: string;
  status: string;
  files: FileObj[];
  statistics: StatisticsObj;
  emailsSharedWith: EmailEntry[];
  finishedUploading: boolean;
  hasTransferRequest: boolean;
  brandProfile?: any; // kept for legacy support if needed, though removed from UI
}

export interface TransferRequest {
  id: string;
  name: string | null;
  description: string | null;
  secretCode: string;
  active: boolean;
  createdAt: string;
}

export interface User {
  id: string;
  email: string;
  createdAt: string;
  plan: string;
}

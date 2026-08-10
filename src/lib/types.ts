export type AppStatus = {
  has_admin: boolean;
  db_path: string;
  app_data_dir: string;
};

export type AuthResult = { session_token: string };

export type PasswordReplaceChallenge = {
  code: string;
  expires_at: string;
};

export type Donor = {
  id: number;
  name: string;
  notes?: string | null;
  created_at: string;
};

export type Category = {
  id: number;
  name: string;
  created_at: string;
};

export type Project = {
  id: number;
  name: string;
  description?: string | null;
  target_amount_cents: number;
  status: string;
  start_date?: string | null;
  end_date?: string | null;
  created_at: string;
};

export type DocumentationRecord = {
  id: number;
  event_name: string;
  event_date: string;
  registration_collected_cents: number;
  expenses_cents: number;
  balance_cents: number;
  notes?: string | null;
  created_at: string;
};

export type DocumentationExpenseRow = {
  id: number;
  spent_at: string;
  amount_cents: number;
  payee?: string | null;
  notes?: string | null;
  created_at: string;
};

export type DocumentationDetail = {
  documentation: DocumentationRecord;
  expenses: DocumentationExpenseRow[];
};

export type Donation = {
  id: number;
  donated_at: string;
  amount_cents: number;
  donor_id?: number | null;
  anonymous: boolean;
  notes?: string | null;
  project_id?: number | null;
  created_at: string;
};

export type Expense = {
  id: number;
  spent_at: string;
  amount_cents: number;
  category_id?: number | null;
  payee?: string | null;
  notes?: string | null;
  project_id?: number | null;
  created_at: string;
};

export type DateRangeFilter = {
  from?: string | null;
  to?: string | null;
  project_id?: number | null;
};

export type LedgerSummary = {
  total_donations_cents: number;
  total_expenses_cents: number;
  balance_cents: number;
};

export type ProjectBalanceRow = {
  project_id: number;
  project_name: string;
  donations_cents: number;
  expenses_cents: number;
  balance_cents: number;
};

export type DonationRow = {
  id: number;
  donated_at: string;
  amount_cents: number;
  donor_name?: string | null;
  anonymous: boolean;
  notes?: string | null;
};

export type ExpenseRow = {
  id: number;
  spent_at: string;
  amount_cents: number;
  category_name?: string | null;
  payee?: string | null;
  notes?: string | null;
};

export type TopDonorRow = {
  donor_name: string;
  total_cents: number;
};

export type ProjectReport = {
  project: Project;
  donations_cents: number;
  expenses_cents: number;
  balance_cents: number;
  target_amount_cents: number;
  remaining_to_target_cents: number;
  donations: DonationRow[];
  expenses: ExpenseRow[];
  top_donors: TopDonorRow[];
};

export type BackupInfo = {
  file_name: string;
  full_path: string;
  created_at: string;
  bytes: number;
};

export type DatabaseHealth = {
  integrity_ok: boolean;
  checked_at: string;
  record_count: number;
};

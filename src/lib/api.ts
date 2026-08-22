import { invoke } from "@tauri-apps/api/core";
import { demoInvoke, isDemoMode } from "./demo-api";
import type {
  AppStatus,
  AuditEvent,
  AuthResult,
  BackupInfo,
  Category,
  DocumentationDetail,
  DocumentationRecord,
  DateRangeFilter,
  Donation,
  Donor,
  Expense,
  DatabaseHealth,
  LedgerSummary,
  ProjectReport,
  Project,
  ProjectBalanceRow,
  PasswordReplaceChallenge,
} from "./types";

function isUnauthorizedError(e: unknown): boolean {
  const message =
    typeof e === "string"
      ? e
      : (e as any)?.message
        ? String((e as any).message)
        : String(e);
  return message.toLowerCase().includes("unauthorized");
}

async function invokeAuthed<T>(command: string, args: Record<string, unknown>): Promise<T> {
  try {
    return await invokeCommand<T>(command, args);
  } catch (e) {
    if (isUnauthorizedError(e)) {
      localStorage.removeItem("pft_session_token");
      window.dispatchEvent(new CustomEvent("pft:unauthorized"));
    }
    throw e;
  }
}

async function invokeCommand<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  return isDemoMode ? demoInvoke<T>(command, args) : invoke<T>(command, args);
}

export async function appStatus(): Promise<AppStatus> {
  return await invokeCommand("app_status");
}

export async function bootstrapAdmin(password: string): Promise<void> {
  await invokeCommand("bootstrap_admin", { password });
}

export async function requestAdminPasswordReplace(): Promise<PasswordReplaceChallenge> {
  return await invokeCommand("request_admin_password_replace");
}

export async function completeAdminPasswordReplace(
  code: string,
  new_password: string,
): Promise<void> {
  await invokeCommand("complete_admin_password_replace", { code, newPassword: new_password });
}

export async function login(password: string): Promise<AuthResult> {
  return await invokeCommand("login", { password });
}

export async function logout(session_token: string): Promise<void> {
  await invokeAuthed("logout", { sessionToken: session_token });
}

export async function auditTrailList(session_token: string): Promise<AuditEvent[]> {
  return await invokeAuthed("audit_trail_list", { sessionToken: session_token });
}

export async function donorsList(session_token: string): Promise<Donor[]> {
  return await invokeAuthed("donors_list", { sessionToken: session_token });
}

export async function donorsCreate(
  session_token: string,
  payload: { name: string; notes?: string | null },
): Promise<{ id: number }> {
  return await invokeAuthed("donors_create", { sessionToken: session_token, payload });
}

export async function donorsDelete(session_token: string, id: number): Promise<void> {
  await invokeAuthed("donors_delete", { sessionToken: session_token, id });
}

export async function categoriesList(session_token: string): Promise<Category[]> {
  return await invokeAuthed("categories_list", { sessionToken: session_token });
}

export async function categoriesCreate(
  session_token: string,
  payload: { name: string },
): Promise<{ id: number }> {
  return await invokeAuthed("categories_create", { sessionToken: session_token, payload });
}

export async function projectsList(session_token: string): Promise<Project[]> {
  return await invokeAuthed("projects_list", { sessionToken: session_token });
}

export async function projectsCreate(
  session_token: string,
  payload: {
    name: string;
    description?: string | null;
    target_amount_cents: number;
    status?: string | null;
    start_date?: string | null;
    end_date?: string | null;
  },
): Promise<{ id: number }> {
  return await invokeAuthed("projects_create", { sessionToken: session_token, payload });
}

export async function projectsUpdate(
  session_token: string,
  id: number,
  payload: {
    name: string;
    description?: string | null;
    target_amount_cents: number;
    status: string;
    start_date?: string | null;
    end_date?: string | null;
  },
): Promise<void> {
  await invokeAuthed("projects_update", { sessionToken: session_token, id, payload });
}

export async function projectsDelete(session_token: string, id: number): Promise<void> {
  await invokeAuthed("projects_delete", { sessionToken: session_token, id });
}

export async function documentationsList(session_token: string): Promise<DocumentationRecord[]> {
  return await invokeAuthed("documentations_list", { sessionToken: session_token });
}

export async function documentationDetail(
  session_token: string,
  id: number,
): Promise<DocumentationDetail> {
  return await invokeAuthed("documentation_detail", { sessionToken: session_token, id });
}

export async function documentationsCreate(
  session_token: string,
  payload: {
    event_name: string;
    event_date: string;
    registration_collected_cents: number;
    notes?: string | null;
  },
): Promise<{ id: number }> {
  return await invokeAuthed("documentations_create", { sessionToken: session_token, payload });
}

export async function documentationExpensesCreate(
  session_token: string,
  payload: {
    documentation_id: number;
    spent_at: string;
    amount_cents: number;
    payee?: string | null;
    notes?: string | null;
  },
): Promise<{ id: number }> {
  return await invokeAuthed("documentation_expenses_create", { sessionToken: session_token, payload });
}

export async function documentationExpensesDelete(session_token: string, id: number): Promise<void> {
  await invokeAuthed("documentation_expenses_delete", { sessionToken: session_token, id });
}

export async function documentationsDelete(session_token: string, id: number): Promise<void> {
  await invokeAuthed("documentations_delete", { sessionToken: session_token, id });
}

export async function donationsList(
  session_token: string,
  filter: DateRangeFilter,
): Promise<Donation[]> {
  return await invokeAuthed("donations_list", { sessionToken: session_token, filter });
}

export async function donationsCreate(
  session_token: string,
  payload: {
    donated_at: string;
    amount_cents: number;
    donor_id?: number | null;
    anonymous: boolean;
    notes?: string | null;
    project_id?: number | null;
  },
): Promise<{ id: number }> {
  return await invokeAuthed("donations_create", { sessionToken: session_token, payload });
}

export async function donationsUpdate(
  session_token: string,
  id: number,
  payload: {
    donated_at: string;
    amount_cents: number;
    donor_id?: number | null;
    anonymous: boolean;
    notes?: string | null;
    project_id?: number | null;
  },
): Promise<void> {
  await invokeAuthed("donations_update", { sessionToken: session_token, id, payload });
}

export async function donationsDelete(session_token: string, id: number): Promise<void> {
  await invokeAuthed("donations_delete", { sessionToken: session_token, id });
}

export async function expensesList(
  session_token: string,
  filter: DateRangeFilter,
): Promise<Expense[]> {
  return await invokeAuthed("expenses_list", { sessionToken: session_token, filter });
}

export async function expensesCreate(
  session_token: string,
  payload: {
    spent_at: string;
    amount_cents: number;
    category_id?: number | null;
    payee?: string | null;
    notes?: string | null;
    project_id?: number | null;
  },
): Promise<{ id: number }> {
  return await invokeAuthed("expenses_create", { sessionToken: session_token, payload });
}

export async function expensesUpdate(
  session_token: string,
  id: number,
  payload: {
    spent_at: string;
    amount_cents: number;
    category_id?: number | null;
    payee?: string | null;
    notes?: string | null;
    project_id?: number | null;
  },
): Promise<void> {
  await invokeAuthed("expenses_update", { sessionToken: session_token, id, payload });
}

export async function expensesDelete(session_token: string, id: number): Promise<void> {
  await invokeAuthed("expenses_delete", { sessionToken: session_token, id });
}

export async function ledgerSummary(
  session_token: string,
  filter: DateRangeFilter,
): Promise<LedgerSummary> {
  return await invokeAuthed("ledger_summary", { sessionToken: session_token, filter });
}

export async function projectBalances(
  session_token: string,
  filter: DateRangeFilter,
): Promise<ProjectBalanceRow[]> {
  return await invokeAuthed("project_balances", { sessionToken: session_token, filter });
}

export async function projectReport(
  session_token: string,
  project_id: number,
  filter: DateRangeFilter,
): Promise<ProjectReport> {
  return await invokeAuthed("project_report", {
    sessionToken: session_token,
    projectId: project_id,
    filter,
  });
}

export async function exportCsv(
  session_token: string,
  req: { kind: string; filter: DateRangeFilter; dest_path: string },
): Promise<void> {
  await invokeAuthed("export_csv_command", { sessionToken: session_token, req });
}

export async function exportPdf(
  session_token: string,
  req: { title: string; filter: DateRangeFilter; dest_path: string },
): Promise<void> {
  await invokeAuthed("export_pdf_command", { sessionToken: session_token, req });
}

export async function backupList(session_token: string): Promise<BackupInfo[]> {
  return await invokeAuthed("backup_list", { sessionToken: session_token });
}

export async function backupCreate(session_token: string): Promise<string> {
  return await invokeAuthed("backup_create", { sessionToken: session_token });
}

export async function backupRestore(session_token: string, src_path: string): Promise<void> {
  await invokeAuthed("backup_restore", { sessionToken: session_token, srcPath: src_path });
}

export async function databaseHealth(session_token: string): Promise<DatabaseHealth> {
  return await invokeAuthed("database_health", { sessionToken: session_token });
}

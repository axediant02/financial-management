import type {
  AuthResult,
  AuditEvent,
  BackupInfo,
  Category,
  DocumentationDetail,
  DocumentationExpenseRow,
  DocumentationRecord,
  Donation,
  Donor,
  Expense,
  Project,
  ProjectReport,
} from "./types";

export const isDemoMode =
  typeof window !== "undefined" && new URLSearchParams(window.location.search).get("demo") === "1";

const demoSessionToken = "demo-session";

const donors: Donor[] = [
  { id: 1, name: "Maria Santos", notes: "Monthly contributor", created_at: "2026-01-05T09:00:00Z" },
  { id: 2, name: "Ramon and Liza Cruz", notes: null, created_at: "2026-01-11T09:00:00Z" },
  { id: 3, name: "Sister Angela Reyes", notes: "Choir ministry", created_at: "2026-02-02T09:00:00Z" },
];

const categories: Category[] = [
  { id: 1, name: "Utilities", created_at: "2026-01-02T09:00:00Z" },
  { id: 2, name: "Supplies", created_at: "2026-01-02T09:00:00Z" },
  { id: 3, name: "Outreach", created_at: "2026-01-02T09:00:00Z" },
  { id: 4, name: "Maintenance", created_at: "2026-01-02T09:00:00Z" },
];

const projects: Project[] = [
  {
    id: 1,
    name: "Community Pantry Drive",
    description: "Monthly food support for families in the parish community.",
    target_amount_cents: 25000000,
    status: "active",
    start_date: "2026-01-10",
    end_date: "2026-12-31",
    created_at: "2026-01-10T09:00:00Z",
  },
  {
    id: 2,
    name: "Youth Retreat Fund",
    description: "Transportation, meals, and materials for the annual youth retreat.",
    target_amount_cents: 12000000,
    status: "active",
    start_date: "2026-03-01",
    end_date: "2026-09-30",
    created_at: "2026-03-01T09:00:00Z",
  },
  {
    id: 3,
    name: "Building Repair Fund",
    description: "Completed roof and electrical repairs for the parish hall.",
    target_amount_cents: 18000000,
    status: "completed",
    start_date: "2025-08-01",
    end_date: "2026-02-28",
    created_at: "2025-08-01T09:00:00Z",
  },
];

const donations: Donation[] = [
  { id: 1, donated_at: "2026-08-16", amount_cents: 1250000, donor_id: 1, anonymous: false, notes: "Sunday offering", project_id: 1, created_at: "2026-08-16T10:00:00Z" },
  { id: 2, donated_at: "2026-08-09", amount_cents: 850000, donor_id: 2, anonymous: false, notes: "Pantry sponsorship", project_id: 1, created_at: "2026-08-09T10:00:00Z" },
  { id: 3, donated_at: "2026-08-02", amount_cents: 500000, donor_id: null, anonymous: true, notes: null, project_id: 2, created_at: "2026-08-02T10:00:00Z" },
  { id: 4, donated_at: "2026-07-26", amount_cents: 1500000, donor_id: 3, anonymous: false, notes: "Building fund balance", project_id: 3, created_at: "2026-07-26T10:00:00Z" },
  { id: 5, donated_at: "2026-07-19", amount_cents: 950000, donor_id: 1, anonymous: false, notes: null, project_id: 1, created_at: "2026-07-19T10:00:00Z" },
  { id: 6, donated_at: "2026-07-12", amount_cents: 700000, donor_id: null, anonymous: true, notes: "General fund", project_id: null, created_at: "2026-07-12T10:00:00Z" },
];

const expenses: Expense[] = [
  { id: 1, spent_at: "2026-08-14", amount_cents: 425000, category_id: 3, payee: "Barangay Community Store", notes: "Pantry rice and canned goods", project_id: 1, created_at: "2026-08-14T10:00:00Z" },
  { id: 2, spent_at: "2026-08-05", amount_cents: 180000, category_id: 2, payee: "Parish Supplies Co.", notes: "Retreat materials", project_id: 2, created_at: "2026-08-05T10:00:00Z" },
  { id: 3, spent_at: "2026-07-24", amount_cents: 650000, category_id: 4, payee: "Santos Construction", notes: "Final repair invoice", project_id: 3, created_at: "2026-07-24T10:00:00Z" },
  { id: 4, spent_at: "2026-07-10", amount_cents: 95000, category_id: 1, payee: "City Electric", notes: "July parish hall power", project_id: null, created_at: "2026-07-10T10:00:00Z" },
  { id: 5, spent_at: "2026-06-28", amount_cents: 230000, category_id: 3, payee: "Community Kitchen", notes: "Outreach meal supplies", project_id: 1, created_at: "2026-06-28T10:00:00Z" },
];

const documentationExpenses = new Map<number, DocumentationExpenseRow[]>([
  [1, [
    { id: 1, spent_at: "2026-06-15", amount_cents: 185000, payee: "Parish Events Team", notes: "Registration materials", created_at: "2026-06-15T10:00:00Z" },
  ]],
  [2, [
    { id: 2, spent_at: "2026-07-21", amount_cents: 90000, payee: "Youth Ministry", notes: "Retreat welcome kits", created_at: "2026-07-21T10:00:00Z" },
  ]],
]);

const documentations: DocumentationRecord[] = [
  { id: 1, event_name: "Annual Registration Sunday", event_date: "2026-06-14", registration_collected_cents: 850000, expenses_cents: 185000, balance_cents: 665000, notes: "Annual parish registration collection.", created_at: "2026-06-14T10:00:00Z" },
  { id: 2, event_name: "Youth Retreat Registration", event_date: "2026-07-20", registration_collected_cents: 420000, expenses_cents: 90000, balance_cents: 330000, notes: "Registration fees for the youth retreat.", created_at: "2026-07-20T10:00:00Z" },
];

const backups: BackupInfo[] = [
  { file_name: "church-ledger-2026-08-15.sqlite3", full_path: "App data\\backups\\church-ledger-2026-08-15.sqlite3", created_at: "2026-08-15T23:30:00Z", bytes: 184320 },
  { file_name: "church-ledger-2026-08-08.sqlite3", full_path: "App data\\backups\\church-ledger-2026-08-08.sqlite3", created_at: "2026-08-08T23:30:00Z", bytes: 176128 },
  { file_name: "church-ledger-2026-08-01.sqlite3", full_path: "App data\\backups\\church-ledger-2026-08-01.sqlite3", created_at: "2026-08-01T23:30:00Z", bytes: 163840 },
];

const auditEvents: AuditEvent[] = [
  {
    id: 3,
    actor: "Admin",
    action: "backup",
    entity: "database",
    record_id: null,
    summary: "Created database backup",
    created_at: "2026-08-15T23:30:00Z",
  },
  {
    id: 2,
    actor: "Admin",
    action: "export",
    entity: "report",
    record_id: null,
    summary: "Exported donations CSV report",
    created_at: "2026-08-15T10:15:00Z",
  },
  {
    id: 1,
    actor: "Admin",
    action: "create",
    entity: "project",
    record_id: 2,
    summary: "Created project record #2",
    created_at: "2026-08-14T09:00:00Z",
  },
];

function recordDemoAudit(action: string, entity: string, recordId: number | null, summary: string) {
  auditEvents.unshift({
    id: Math.max(0, ...auditEvents.map((event) => event.id)) + 1,
    actor: "Admin",
    action,
    entity,
    record_id: recordId,
    summary,
    created_at: new Date().toISOString(),
  });
}

function matchesFilter(date: string, projectId: number | null | undefined, filter?: { from?: string | null; to?: string | null; project_id?: number | null }) {
  if (filter?.from && date < filter.from) return false;
  if (filter?.to && date > filter.to) return false;
  return filter?.project_id == null || projectId === filter.project_id;
}

function sum(values: number[]) {
  return values.reduce((total, value) => total + value, 0);
}

function summary(filter?: { from?: string | null; to?: string | null; project_id?: number | null }) {
  const filteredDonations = donations.filter((item) => matchesFilter(item.donated_at, item.project_id, filter));
  const filteredExpenses = expenses.filter((item) => matchesFilter(item.spent_at, item.project_id, filter));
  const totalDonations = sum(filteredDonations.map((item) => item.amount_cents));
  const totalExpenses = sum(filteredExpenses.map((item) => item.amount_cents));
  return {
    total_donations_cents: totalDonations,
    total_expenses_cents: totalExpenses,
    balance_cents: totalDonations - totalExpenses,
  };
}

function documentationDetail(id: number): DocumentationDetail {
  const documentation = documentations.find((item) => item.id === id) || documentations[0];
  return { documentation, expenses: documentationExpenses.get(documentation.id) || [] };
}

function projectReport(id: number): ProjectReport {
  const project = projects.find((item) => item.id === id) || projects[0];
  const projectDonations = donations.filter((item) => item.project_id === project.id);
  const projectExpenses = expenses.filter((item) => item.project_id === project.id);
  const donorTotals = new Map<string, number>();
  projectDonations.forEach((item) => {
    const name = item.anonymous ? "Anonymous" : donors.find((donor) => donor.id === item.donor_id)?.name || "Contribution";
    donorTotals.set(name, (donorTotals.get(name) || 0) + item.amount_cents);
  });
  const donationsTotal = sum(projectDonations.map((item) => item.amount_cents));
  const expensesTotal = sum(projectExpenses.map((item) => item.amount_cents));
  return {
    project,
    donations_cents: donationsTotal,
    expenses_cents: expensesTotal,
    balance_cents: donationsTotal - expensesTotal,
    target_amount_cents: project.target_amount_cents,
    remaining_to_target_cents: Math.max(0, project.target_amount_cents - donationsTotal),
    donations: projectDonations.map((item) => ({
      id: item.id,
      donated_at: item.donated_at,
      amount_cents: item.amount_cents,
      donor_name: item.anonymous ? null : donors.find((donor) => donor.id === item.donor_id)?.name || null,
      anonymous: item.anonymous,
      notes: item.notes,
    })),
    expenses: projectExpenses.map((item) => ({
      id: item.id,
      spent_at: item.spent_at,
      amount_cents: item.amount_cents,
      category_name: categories.find((category) => category.id === item.category_id)?.name || null,
      payee: item.payee,
      notes: item.notes,
    })),
    top_donors: [...donorTotals.entries()]
      .map(([donor_name, total_cents]) => ({ donor_name, total_cents }))
      .sort((a, b) => b.total_cents - a.total_cents),
  };
}

export async function demoInvoke<T>(command: string, args: Record<string, any> = {}): Promise<T> {
  const payload = args.payload || {};

  switch (command) {
    case "app_status":
      return { has_admin: true, db_path: "Demo data (read-only)", app_data_dir: "App data" } as T;
    case "login":
      recordDemoAudit("login", "session", null, "Administrator logged in");
      return ({ session_token: demoSessionToken } satisfies AuthResult) as T;
    case "logout":
      recordDemoAudit("logout", "session", null, "Administrator logged out");
      return undefined as T;
    case "bootstrap_admin":
    case "complete_admin_password_replace":
      return undefined as T;
    case "audit_trail_list":
      if (args.sessionToken !== demoSessionToken) throw new Error("unauthorized");
      return [...auditEvents].sort((a, b) => b.created_at.localeCompare(a.created_at) || b.id - a.id).slice(0, 500) as T;
    case "export_csv_command":
      recordDemoAudit("export", "report", null, `Exported ${args.req?.kind || "ledger"} CSV report`);
      return undefined as T;
    case "export_pdf_command":
      recordDemoAudit("export", "report", null, "Exported PDF report");
      return undefined as T;
    case "backup_restore":
      recordDemoAudit("restore", "database", null, "Restored database backup");
      return undefined as T;
    case "request_admin_password_replace":
      return { code: "DEMO-1234", expires_at: "2026-12-31T23:59:00Z" } as T;
    case "donors_list":
      return [...donors] as T;
    case "donors_create": {
      const id = Math.max(0, ...donors.map((item) => item.id)) + 1;
      donors.push({ id, name: payload.name, notes: payload.notes || null, created_at: new Date().toISOString() });
      recordDemoAudit("create", "donor", id, `Created donor record #${id}`);
      return { id } as T;
    }
    case "donors_delete": {
      const index = donors.findIndex((item) => item.id === args.id);
      if (index < 0) throw new Error("donor record not found");
      donors.splice(index, 1);
      recordDemoAudit("delete", "donor", args.id, `Deleted donor record #${args.id}`);
      return undefined as T;
    }
    case "categories_list":
      return [...categories] as T;
    case "categories_create": {
      const id = Math.max(0, ...categories.map((item) => item.id)) + 1;
      categories.push({ id, name: payload.name, created_at: new Date().toISOString() });
      recordDemoAudit("create", "category", id, `Created category record #${id}`);
      return { id } as T;
    }
    case "projects_list":
      return [...projects] as T;
    case "projects_create": {
      const id = Math.max(0, ...projects.map((item) => item.id)) + 1;
      projects.push({ ...payload, id, status: payload.status || "active", created_at: new Date().toISOString() });
      recordDemoAudit("create", "project", id, `Created project record #${id}`);
      return { id } as T;
    }
    case "projects_update": {
      const item = projects.find((project) => project.id === args.id);
      if (!item) throw new Error("project record not found");
      Object.assign(item, payload);
      recordDemoAudit("update", "project", args.id, `Updated project record #${args.id}`);
      return undefined as T;
    }
    case "projects_delete": {
      const index = projects.findIndex((item) => item.id === args.id);
      if (index < 0) throw new Error("project record not found");
      projects.splice(index, 1);
      recordDemoAudit("delete", "project", args.id, `Deleted project record #${args.id}`);
      return undefined as T;
    }
    case "documentations_list":
      return [...documentations] as T;
    case "documentation_detail":
      return documentationDetail(args.id) as T;
    case "documentations_create": {
      const id = Math.max(0, ...documentations.map((item) => item.id)) + 1;
      documentations.push({ ...payload, id, expenses_cents: 0, balance_cents: payload.registration_collected_cents, created_at: new Date().toISOString() });
      documentationExpenses.set(id, []);
      recordDemoAudit("create", "documentation", id, `Created documentation record #${id}`);
      return { id } as T;
    }
    case "documentation_expenses_create": {
      const id = Math.max(0, ...[...documentationExpenses.values()].flat().map((item) => item.id)) + 1;
      const item = { ...payload, id, created_at: new Date().toISOString() } as DocumentationExpenseRow;
      const rows = documentationExpenses.get(payload.documentation_id) || [];
      rows.push(item);
      documentationExpenses.set(payload.documentation_id, rows);
      const documentation = documentations.find((record) => record.id === payload.documentation_id);
      if (documentation) {
        documentation.expenses_cents += payload.amount_cents;
        documentation.balance_cents -= payload.amount_cents;
      }
      recordDemoAudit("create", "documentation expense", id, `Created documentation expense record #${id}`);
      return { id } as T;
    }
    case "documentation_expenses_delete":
      for (const [documentationId, rows] of documentationExpenses.entries()) {
        const index = rows.findIndex((item) => item.id === args.id);
        if (index >= 0) {
          const [removed] = rows.splice(index, 1);
          const documentation = documentations.find((record) => record.id === documentationId);
          if (documentation) {
            documentation.expenses_cents -= removed.amount_cents;
            documentation.balance_cents += removed.amount_cents;
          }
        }
      }
      recordDemoAudit("delete", "documentation expense", args.id, `Deleted documentation expense record #${args.id}`);
      return undefined as T;
    case "documentations_delete": {
      const index = documentations.findIndex((item) => item.id === args.id);
      if (index < 0) throw new Error("documentation record not found");
      documentations.splice(index, 1);
      documentationExpenses.delete(args.id);
      recordDemoAudit("delete", "documentation", args.id, `Deleted documentation record #${args.id}`);
      return undefined as T;
    }
    case "donations_list":
      return donations.filter((item) => matchesFilter(item.donated_at, item.project_id, args.filter)) as T;
    case "donations_create": {
      const id = Math.max(0, ...donations.map((item) => item.id)) + 1;
      donations.push({ ...payload, id, created_at: new Date().toISOString() });
      recordDemoAudit("create", "donation", id, `Created contribution record #${id}`);
      return { id } as T;
    }
    case "donations_update": {
      const item = donations.find((donation) => donation.id === args.id);
      if (!item) throw new Error("contribution record not found");
      Object.assign(item, payload);
      recordDemoAudit("update", "donation", args.id, `Updated contribution record #${args.id}`);
      return undefined as T;
    }
    case "donations_delete": {
      const index = donations.findIndex((item) => item.id === args.id);
      if (index < 0) throw new Error("contribution record not found");
      donations.splice(index, 1);
      recordDemoAudit("delete", "donation", args.id, `Deleted contribution record #${args.id}`);
      return undefined as T;
    }
    case "expenses_list":
      return expenses.filter((item) => matchesFilter(item.spent_at, item.project_id, args.filter)) as T;
    case "expenses_create": {
      const id = Math.max(0, ...expenses.map((item) => item.id)) + 1;
      expenses.push({ ...payload, id, created_at: new Date().toISOString() });
      recordDemoAudit("create", "expense", id, `Created expense record #${id}`);
      return { id } as T;
    }
    case "expenses_update": {
      const item = expenses.find((expense) => expense.id === args.id);
      if (!item) throw new Error("expense record not found");
      Object.assign(item, payload);
      recordDemoAudit("update", "expense", args.id, `Updated expense record #${args.id}`);
      return undefined as T;
    }
    case "expenses_delete": {
      const index = expenses.findIndex((item) => item.id === args.id);
      if (index < 0) throw new Error("expense record not found");
      expenses.splice(index, 1);
      recordDemoAudit("delete", "expense", args.id, `Deleted expense record #${args.id}`);
      return undefined as T;
    }
    case "ledger_summary":
      return summary(args.filter) as T;
    case "project_balances":
      return projects.filter((project) => !args.filter?.project_id || project.id === args.filter.project_id).map((project) => {
        const donationsTotal = sum(donations.filter((item) => item.project_id === project.id && matchesFilter(item.donated_at, item.project_id, args.filter)).map((item) => item.amount_cents));
        const expensesTotal = sum(expenses.filter((item) => item.project_id === project.id && matchesFilter(item.spent_at, item.project_id, args.filter)).map((item) => item.amount_cents));
        return { project_id: project.id, project_name: project.name, donations_cents: donationsTotal, expenses_cents: expensesTotal, balance_cents: donationsTotal - expensesTotal };
      }) as T;
    case "project_report":
      return projectReport(args.projectId) as T;
    case "backup_list":
      return [...backups] as T;
    case "backup_create": {
      const fileName = `church-ledger-demo-${new Date().toISOString().slice(0, 10)}.sqlite3`;
      backups.unshift({ file_name: fileName, full_path: `App data\\backups\\${fileName}`, created_at: new Date().toISOString(), bytes: 192512 });
      recordDemoAudit("backup", "database", null, "Created database backup");
      return backups[0].full_path as T;
    }
    case "database_health":
      return { integrity_ok: true, checked_at: "2026-08-16T08:30:00Z", record_count: donations.length + expenses.length + projects.length + documentations.length } as T;
    default:
      return undefined as T;
  }
}

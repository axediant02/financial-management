use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub has_admin: bool,
    pub db_path: String,
    pub app_data_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResult {
    pub session_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PasswordReplaceChallenge {
    pub code: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdResult {
    pub id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Donor {
    pub id: i64,
    pub name: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DonorCreate {
    pub name: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryCreate {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub target_amount_cents: i64,
    pub status: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectCreate {
    pub name: String,
    pub description: Option<String>,
    pub target_amount_cents: i64,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectUpdate {
    pub name: String,
    pub description: Option<String>,
    pub target_amount_cents: i64,
    pub status: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentationRecord {
    pub id: i64,
    pub event_name: String,
    pub event_date: String,
    pub registration_collected_cents: i64,
    pub expenses_cents: i64,
    pub balance_cents: i64,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentationCreate {
    pub event_name: String,
    pub event_date: String,
    pub registration_collected_cents: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentationExpenseRow {
    pub id: i64,
    pub spent_at: String,
    pub amount_cents: i64,
    pub payee: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentationExpenseCreate {
    pub documentation_id: i64,
    pub spent_at: String,
    pub amount_cents: i64,
    pub payee: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentationDetail {
    pub documentation: DocumentationRecord,
    pub expenses: Vec<DocumentationExpenseRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Donation {
    pub id: i64,
    pub donated_at: String,
    pub amount_cents: i64,
    pub donor_id: Option<i64>,
    pub anonymous: bool,
    pub notes: Option<String>,
    pub project_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DonationCreate {
    pub donated_at: String, // YYYY-MM-DD
    pub amount_cents: i64,
    pub donor_id: Option<i64>,
    pub anonymous: bool,
    pub notes: Option<String>,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DonationUpdate {
    pub donated_at: String,
    pub amount_cents: i64,
    pub donor_id: Option<i64>,
    pub anonymous: bool,
    pub notes: Option<String>,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Expense {
    pub id: i64,
    pub spent_at: String, // YYYY-MM-DD
    pub amount_cents: i64,
    pub category_id: Option<i64>,
    pub payee: Option<String>,
    pub notes: Option<String>,
    pub project_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExpenseCreate {
    pub spent_at: String,
    pub amount_cents: i64,
    pub category_id: Option<i64>,
    pub payee: Option<String>,
    pub notes: Option<String>,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExpenseUpdate {
    pub spent_at: String,
    pub amount_cents: i64,
    pub category_id: Option<i64>,
    pub payee: Option<String>,
    pub notes: Option<String>,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LedgerSummary {
    pub total_donations_cents: i64,
    pub total_expenses_cents: i64,
    pub balance_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectBalanceRow {
    pub project_id: i64,
    pub project_name: String,
    pub donations_cents: i64,
    pub expenses_cents: i64,
    pub balance_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DonationRow {
    pub id: i64,
    pub donated_at: String,
    pub amount_cents: i64,
    pub donor_name: Option<String>,
    pub anonymous: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpenseRow {
    pub id: i64,
    pub spent_at: String,
    pub amount_cents: i64,
    pub category_name: Option<String>,
    pub payee: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopDonorRow {
    pub donor_name: String,
    pub total_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectReport {
    pub project: Project,
    pub donations_cents: i64,
    pub expenses_cents: i64,
    pub balance_cents: i64,
    pub target_amount_cents: i64,
    pub remaining_to_target_cents: i64,
    pub donations: Vec<DonationRow>,
    pub expenses: Vec<ExpenseRow>,
    pub top_donors: Vec<TopDonorRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DateRangeFilter {
    pub from: Option<String>, // YYYY-MM-DD inclusive
    pub to: Option<String>,   // YYYY-MM-DD inclusive
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportCsvRequest {
    pub kind: String, // "donations" | "expenses" | "projects"
    pub filter: DateRangeFilter,
    pub dest_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportPdfRequest {
    pub title: String,
    pub filter: DateRangeFilter,
    pub dest_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub file_name: String,
    pub full_path: String,
    pub created_at: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseHealth {
    pub integrity_ok: bool,
    pub checked_at: String,
    pub record_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub id: i64,
    pub actor: String,
    pub action: String,
    pub entity: String,
    pub record_id: Option<i64>,
    pub summary: String,
    pub created_at: String,
}

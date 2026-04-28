use std::fs;
use std::path::Path;
use std::io::BufWriter;

use printpdf::{BuiltinFont, Mm, PdfDocument};
use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::models::{DateRangeFilter, ExportCsvRequest, ExportPdfRequest, LedgerSummary};

pub fn export_csv(conn: &Connection, req: ExportCsvRequest) -> AppResult<()> {
    let path = Path::new(&req.dest_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut wtr = csv::Writer::from_path(path)?;

    match req.kind.as_str() {
        "donations" => {
            wtr.write_record([
                "id",
                "donated_at",
                "amount_cents",
                "donor_id",
                "anonymous",
                "project_id",
                "notes",
            ])?;
            let mut stmt = conn.prepare(
                r#"
SELECT id, donated_at, amount_cents, donor_id, anonymous, project_id, notes
FROM donations
WHERE (?1 IS NULL OR donated_at >= ?1)
  AND (?2 IS NULL OR donated_at <= ?2)
  AND (?3 IS NULL OR project_id = ?3)
ORDER BY donated_at DESC, id DESC
"#,
            )?;
            let rows = stmt.query_map(
                params![req.filter.from, req.filter.to, req.filter.project_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, Option<i64>>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, Option<i64>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                    ))
                },
            )?;
            for r in rows {
                let (id, donated_at, amount_cents, donor_id, anonymous, project_id, notes) = r?;
                wtr.write_record([
                    id.to_string(),
                    donated_at,
                    amount_cents.to_string(),
                    donor_id.map(|v| v.to_string()).unwrap_or_default(),
                    (anonymous != 0).to_string(),
                    project_id.map(|v| v.to_string()).unwrap_or_default(),
                    notes.unwrap_or_default(),
                ])?;
            }
        }
        "expenses" => {
            wtr.write_record([
                "id",
                "spent_at",
                "amount_cents",
                "category_id",
                "project_id",
                "payee",
                "notes",
            ])?;
            let mut stmt = conn.prepare(
                r#"
SELECT id, spent_at, amount_cents, category_id, project_id, payee, notes
FROM expenses
WHERE (?1 IS NULL OR spent_at >= ?1)
  AND (?2 IS NULL OR spent_at <= ?2)
  AND (?3 IS NULL OR project_id = ?3)
ORDER BY spent_at DESC, id DESC
"#,
            )?;
            let rows = stmt.query_map(
                params![req.filter.from, req.filter.to, req.filter.project_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, Option<i64>>(3)?,
                        row.get::<_, Option<i64>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                    ))
                },
            )?;
            for r in rows {
                let (id, spent_at, amount_cents, category_id, project_id, payee, notes) = r?;
                wtr.write_record([
                    id.to_string(),
                    spent_at,
                    amount_cents.to_string(),
                    category_id.map(|v| v.to_string()).unwrap_or_default(),
                    project_id.map(|v| v.to_string()).unwrap_or_default(),
                    payee.unwrap_or_default(),
                    notes.unwrap_or_default(),
                ])?;
            }
        }
        "projects" => {
            wtr.write_record([
                "id",
                "name",
                "target_amount_cents",
                "status",
                "start_date",
                "end_date",
                "description",
            ])?;
            let mut stmt = conn.prepare(
                r#"
SELECT id, name, target_amount_cents, status, start_date, end_date, description
FROM projects
ORDER BY status ASC, name ASC
"#,
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            })?;
            for r in rows {
                let (id, name, target, status, start, end, desc) = r?;
                wtr.write_record([
                    id.to_string(),
                    name,
                    target.to_string(),
                    status,
                    start.unwrap_or_default(),
                    end.unwrap_or_default(),
                    desc.unwrap_or_default(),
                ])?;
            }
        }
        _ => {
            return Err(AppError::InvalidInput(
                "invalid export kind (use donations|expenses|projects)".to_string(),
            ));
        }
    }

    wtr.flush()?;
    Ok(())
}

pub fn export_pdf_summary(
    conn: &Connection,
    req: ExportPdfRequest,
    summary: LedgerSummary,
) -> AppResult<()> {
    let path = Path::new(&req.dest_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let (doc, page1, layer1) = PdfDocument::new(&req.title, Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    let mut y = 280.0;
    let x = 20.0;
    let font_size = 14.0;

    current_layer.use_text(&req.title, font_size + 4.0, Mm(x), Mm(y), &font);
    y -= 12.0;

    let filter_line = format!(
        "Filter: from={} to={} project_id={}",
        req.filter.from.clone().unwrap_or_else(|| "-".to_string()),
        req.filter.to.clone().unwrap_or_else(|| "-".to_string()),
        req.filter
            .project_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    current_layer.use_text(filter_line, font_size, Mm(x), Mm(y), &font);
    y -= 14.0;

    current_layer.use_text(
        format!(
            "Total Balance: {}",
            format_currency_php(summary.total_donations_cents)
        ),
        font_size,
        Mm(x),
        Mm(y),
        &font,
    );
    y -= 10.0;
    current_layer.use_text(
        format!(
            "Total expenses:  {}",
            format_currency_php(summary.total_expenses_cents)
        ),
        font_size,
        Mm(x),
        Mm(y),
        &font,
    );
    y -= 10.0;
    current_layer.use_text(
        format!(
            "Balance:        {}",
            format_currency_php(summary.balance_cents)
        ),
        font_size,
        Mm(x),
        Mm(y),
        &font,
    );
    y -= 16.0;

    // Top 10 recent contributions for the same filter
    current_layer.use_text(
        "Recent contributions (top 10):",
        font_size,
        Mm(x),
        Mm(y),
        &font,
    );
    y -= 10.0;
    let mut stmt = conn.prepare(
        r#"
SELECT donated_at, amount_cents, anonymous, donor_id, project_id
FROM donations
WHERE (?1 IS NULL OR donated_at >= ?1)
  AND (?2 IS NULL OR donated_at <= ?2)
  AND (?3 IS NULL OR project_id = ?3)
ORDER BY donated_at DESC, id DESC
LIMIT 10
"#,
    )?;
    let rows = stmt.query_map(
        params![req.filter.from, req.filter.to, req.filter.project_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, Option<i64>>(4)?,
            ))
        },
    )?;
    for r in rows {
        if y < 30.0 {
            break;
        }
        let (date, amount, anonymous, donor_id, project_id) = r?;
        current_layer.use_text(
            format!(
                "- {}  {}  name={}  anon={}  project={}",
                date,
                format_currency_php(amount),
                donor_id
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                (anonymous != 0),
                project_id
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string())
            ),
            10.0,
            Mm(x),
            Mm(y),
            &font,
        );
        y -= 8.0;
    }

    let file = fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)?;
    Ok(())
}

pub fn compute_summary(conn: &Connection, filter: &DateRangeFilter) -> AppResult<LedgerSummary> {
    let donations: i64 = conn.query_row(
        r#"
SELECT COALESCE(SUM(amount_cents), 0)
FROM donations
WHERE (?1 IS NULL OR donated_at >= ?1)
  AND (?2 IS NULL OR donated_at <= ?2)
  AND (?3 IS NULL OR project_id = ?3)
"#,
        params![filter.from, filter.to, filter.project_id],
        |row| row.get(0),
    )?;
    let expenses: i64 = conn.query_row(
        r#"
SELECT COALESCE(SUM(amount_cents), 0)
FROM expenses
WHERE (?1 IS NULL OR spent_at >= ?1)
  AND (?2 IS NULL OR spent_at <= ?2)
  AND (?3 IS NULL OR project_id = ?3)
"#,
        params![filter.from, filter.to, filter.project_id],
        |row| row.get(0),
    )?;

    Ok(LedgerSummary {
        total_donations_cents: donations,
        total_expenses_cents: expenses,
        balance_cents: donations - expenses,
    })
}

fn format_currency_php(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let cents = cents.abs();
    format!("{sign}PHP {:.2}", (cents as f64) / 100.0)
}

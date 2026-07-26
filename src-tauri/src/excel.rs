use crate::commands::{fields_for_course, roster};
use crate::db::{self, DbState};
use crate::models::{
    ExcelMatchPreview, ExcelPreflight, RosterImportRow, TemplateRow, WorkbookSheet,
};
use regex::Regex;
use rusqlite::{params, OptionalExtension};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::State;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Clone)]
struct Package {
    entries: Vec<PackageEntry>,
}

#[derive(Debug, Clone)]
struct PackageEntry {
    name: String,
    data: Vec<u8>,
    compression: CompressionMethod,
    unix_mode: Option<u32>,
    is_dir: bool,
}

fn read_package(path: &Path) -> Result<Package, String> {
    let file = File::open(path).map_err(|e| format!("Unable to open workbook: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid XLSX package: {e}"))?;
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let mut item = archive.by_index(index).map_err(|e| e.to_string())?;
        let mut data = Vec::new();
        if !item.is_dir() {
            item.read_to_end(&mut data).map_err(|e| e.to_string())?;
        }
        entries.push(PackageEntry {
            name: item.name().to_string(),
            data,
            compression: item.compression(),
            unix_mode: item.unix_mode(),
            is_dir: item.is_dir(),
        });
    }
    Ok(Package { entries })
}

fn package_text<'a>(package: &'a Package, name: &str) -> Result<&'a str, String> {
    let entry = package
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .ok_or_else(|| format!("Workbook package is missing {name}."))?;
    std::str::from_utf8(&entry.data).map_err(|e| format!("{name} is not valid UTF-8 XML: {e}"))
}

fn xml_attribute(tag: &str, name: &str) -> Option<String> {
    let pattern = format!(r#"(?:^|\s){}="([^"]*)""#, regex::escape(name));
    Regex::new(&pattern)
        .ok()?
        .captures(tag)?
        .get(1)
        .map(|value| xml_decode(value.as_str()))
}

fn xml_decode(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn workbook_sheets(package: &Package) -> Result<Vec<WorkbookSheet>, String> {
    let workbook = package_text(package, "xl/workbook.xml")?;
    let relationships = package_text(package, "xl/_rels/workbook.xml.rels")?;
    let tag = Regex::new(r"(?s)<Relationship\b([^>]*)/?>").map_err(|e| e.to_string())?;
    let mut targets = HashMap::new();
    for capture in tag.captures_iter(relationships) {
        let attrs = capture
            .get(1)
            .map(|value| value.as_str())
            .unwrap_or_default();
        if let (Some(id), Some(target)) =
            (xml_attribute(attrs, "Id"), xml_attribute(attrs, "Target"))
        {
            let target = if target.starts_with('/') {
                target.trim_start_matches('/').to_string()
            } else {
                format!("xl/{}", target.trim_start_matches("./"))
            };
            targets.insert(id, target.replace('\\', "/"));
        }
    }
    let sheet_tag = Regex::new(r"(?s)<sheet\b([^>]*)/?>").map_err(|e| e.to_string())?;
    let mut sheets = Vec::new();
    for capture in sheet_tag.captures_iter(workbook) {
        let attrs = capture
            .get(1)
            .map(|value| value.as_str())
            .unwrap_or_default();
        let name = xml_attribute(attrs, "name").ok_or("A worksheet is missing its name.")?;
        let relationship = xml_attribute(attrs, "r:id").ok_or("A worksheet is missing r:id.")?;
        let path = targets
            .get(&relationship)
            .cloned()
            .ok_or_else(|| format!("Worksheet relationship {relationship} is missing."))?;
        sheets.push(WorkbookSheet { name, path });
    }
    if sheets.is_empty() {
        return Err("The workbook contains no worksheets.".into());
    }
    Ok(sheets)
}

fn shared_strings(package: &Package) -> Result<Vec<String>, String> {
    let Some(entry) = package
        .entries
        .iter()
        .find(|entry| entry.name == "xl/sharedStrings.xml")
    else {
        return Ok(vec![]);
    };
    let xml = std::str::from_utf8(&entry.data).map_err(|e| e.to_string())?;
    let item_regex = Regex::new(r"(?s)<si\b[^>]*>(.*?)</si>").map_err(|e| e.to_string())?;
    let text_regex = Regex::new(r"(?s)<t\b[^>]*>(.*?)</t>").map_err(|e| e.to_string())?;
    Ok(item_regex
        .captures_iter(xml)
        .map(|item| {
            text_regex
                .captures_iter(item.get(1).map(|value| value.as_str()).unwrap_or_default())
                .filter_map(|text| text.get(1))
                .map(|value| xml_decode(value.as_str()))
                .collect::<String>()
        })
        .collect())
}

#[derive(Debug, Clone)]
struct Cell {
    reference: String,
    cell_type: Option<String>,
    raw_value: Option<String>,
}

fn worksheet_cells(xml: &str) -> Result<Vec<Cell>, String> {
    let cell_regex = Regex::new(r"(?s)<c\b([^>]*)>(.*?)</c>").map_err(|e| e.to_string())?;
    let value_regex = Regex::new(r"(?s)<v\b[^>]*>(.*?)</v>").map_err(|e| e.to_string())?;
    let inline_regex = Regex::new(r"(?s)<t\b[^>]*>(.*?)</t>").map_err(|e| e.to_string())?;
    let mut cells = Vec::new();
    for capture in cell_regex.captures_iter(xml) {
        let attrs = capture
            .get(1)
            .map(|value| value.as_str())
            .unwrap_or_default();
        let body = capture
            .get(2)
            .map(|value| value.as_str())
            .unwrap_or_default();
        let Some(reference) = xml_attribute(attrs, "r") else {
            continue;
        };
        let cell_type = xml_attribute(attrs, "t");
        let raw_value = if cell_type.as_deref() == Some("inlineStr") {
            inline_regex
                .captures(body)
                .and_then(|capture| capture.get(1))
                .map(|value| xml_decode(value.as_str()))
        } else {
            value_regex
                .captures(body)
                .and_then(|capture| capture.get(1))
                .map(|value| xml_decode(value.as_str()))
        };
        cells.push(Cell {
            reference,
            cell_type,
            raw_value,
        });
    }
    Ok(cells)
}

fn display_value(cell: &Cell, strings: &[String]) -> String {
    match (cell.cell_type.as_deref(), cell.raw_value.as_deref()) {
        (Some("s"), Some(index)) => index
            .parse::<usize>()
            .ok()
            .and_then(|index| strings.get(index))
            .cloned()
            .unwrap_or_default(),
        (_, Some(value)) => value.to_string(),
        _ => String::new(),
    }
}

fn split_reference(reference: &str) -> (String, usize) {
    let mut column = String::new();
    let mut row = String::new();
    for character in reference.chars() {
        if character.is_ascii_alphabetic() {
            column.push(character);
        } else if character.is_ascii_digit() {
            row.push(character);
        }
    }
    (column, row.parse().unwrap_or(0))
}

fn inspect_workbook(path: &Path, requested_sheet: Option<&str>) -> Result<ExcelPreflight, String> {
    let package = read_package(path)?;
    let sheets = workbook_sheets(&package)?;
    let selected = match requested_sheet {
        Some(name) => sheets
            .iter()
            .find(|sheet| sheet.name == name)
            .cloned()
            .ok_or_else(|| format!("Worksheet “{name}” was not found."))?,
        None => sheets.first().expect("checked non-empty").clone(),
    };
    let xml = package_text(&package, &selected.path)?;
    let strings = shared_strings(&package)?;
    let cells = worksheet_cells(xml)?;
    let mut first_row = HashMap::new();
    for cell in &cells {
        let (column, row) = split_reference(&cell.reference);
        if row == 1 {
            first_row.insert(column, display_value(cell, &strings));
        }
    }
    let student_header = first_row.get("A").map(|value| value.trim().to_lowercase());
    if student_header.as_deref() != Some("student id") {
        return Err("Strict template validation failed: cell A1 must be “Student ID”.".into());
    }
    let mark_column = first_row
        .iter()
        .find_map(|(column, value)| {
            value
                .trim()
                .eq_ignore_ascii_case("mark")
                .then(|| column.clone())
        })
        .ok_or("The selected worksheet does not contain a “Mark” column in row 1.")?;
    let mut by_ref = HashMap::new();
    for cell in &cells {
        by_ref.insert(cell.reference.clone(), cell);
    }
    let max_row = cells
        .iter()
        .map(|cell| split_reference(&cell.reference).1)
        .max()
        .unwrap_or(1);
    let mut rows = Vec::new();
    for row in 2..=max_row {
        let id_ref = format!("A{row}");
        let mark_ref = format!("{mark_column}{row}");
        let identifier = by_ref
            .get(&id_ref)
            .map(|cell| display_value(cell, &strings))
            .unwrap_or_default();
        if identifier.trim().is_empty() {
            continue;
        }
        let current_mark = by_ref
            .get(&mark_ref)
            .map(|cell| display_value(cell, &strings))
            .and_then(|value| value.parse::<f64>().ok());
        rows.push(TemplateRow {
            row_number: row,
            student_identifier: identifier,
            current_mark,
        });
    }
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();
    let detected_term = if filename.contains("midterm") || filename.contains("mid term") {
        Some("mid".into())
    } else if filename.contains("finalterm") || filename.contains("final term") {
        Some("final".into())
    } else {
        None
    };
    Ok(ExcelPreflight {
        sheets,
        selected_sheet: selected.name,
        student_id_column: "A".into(),
        mark_column,
        rows,
        detected_term,
    })
}

pub fn read_roster_rows(
    path: &Path,
    requested_sheet: Option<&str>,
) -> Result<(String, Vec<RosterImportRow>), String> {
    let package = read_package(path)?;
    let sheets = workbook_sheets(&package)?;
    let selected = match requested_sheet {
        Some(name) => sheets
            .iter()
            .find(|sheet| sheet.name == name)
            .cloned()
            .ok_or_else(|| format!("Worksheet “{name}” was not found."))?,
        None => sheets.first().expect("checked non-empty").clone(),
    };
    let xml = package_text(&package, &selected.path)?;
    let strings = shared_strings(&package)?;
    let cells = worksheet_cells(xml)?;
    let mut header_by_column = HashMap::new();
    let mut by_ref = HashMap::new();
    for cell in &cells {
        let (column, row) = split_reference(&cell.reference);
        if row == 1 {
            header_by_column.insert(column, display_value(cell, &strings).trim().to_lowercase());
        }
        by_ref.insert(cell.reference.clone(), cell);
    }
    let id_column = header_by_column
        .iter()
        .find_map(|(column, header)| {
            ["student id", "student_id", "id"]
                .contains(&header.as_str())
                .then(|| column.clone())
        })
        .ok_or("Roster import requires a “Student ID” column in row 1.")?;
    let name_column = header_by_column
        .iter()
        .find_map(|(column, header)| {
            ["name", "student name", "student_name"]
                .contains(&header.as_str())
                .then(|| column.clone())
        })
        .ok_or("Roster import requires a “Name” column in row 1.")?;
    let email_column = header_by_column.iter().find_map(|(column, header)| {
        ["email", "e-mail"]
            .contains(&header.as_str())
            .then(|| column.clone())
    });
    let max_row = cells
        .iter()
        .map(|cell| split_reference(&cell.reference).1)
        .max()
        .unwrap_or(1);
    let mut rows = Vec::new();
    for source_row in 2..=max_row {
        let value_at = |column: &str| {
            by_ref
                .get(&format!("{column}{source_row}"))
                .map(|cell| display_value(cell, &strings).trim().to_string())
                .unwrap_or_default()
        };
        let student_identifier = value_at(&id_column);
        let name = value_at(&name_column);
        if student_identifier.is_empty() && name.is_empty() {
            continue;
        }
        if student_identifier.is_empty() || name.is_empty() {
            return Err(format!(
                "Roster row {source_row} must contain both Student ID and Name."
            ));
        }
        let email = email_column
            .as_deref()
            .map(value_at)
            .filter(|value| !value.is_empty());
        rows.push(RosterImportRow {
            source_row,
            student_identifier,
            name,
            email,
            status: "ready".into(),
        });
    }
    if rows.is_empty() {
        return Err("The selected worksheet contains no roster rows.".into());
    }
    Ok((selected.name, rows))
}

#[tauri::command]
pub fn preflight_excel(path: String, sheet_name: Option<String>) -> Result<ExcelPreflight, String> {
    inspect_workbook(Path::new(&path), sheet_name.as_deref())
}

fn normalize_id(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

pub fn output_filename(
    export_name: &str,
    section: &str,
    term: &str,
    session: &str,
    season: &str,
) -> String {
    let section = if section
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .count()
        > 0
    {
        format!("[{}]", section.trim_matches(['[', ']']))
    } else {
        section.trim_matches(['[', ']']).to_string()
    };
    let term_label = match term {
        "mid" => "Midterm",
        "final" => "Finalterm",
        "semester" => "Semester Result",
        other => other,
    };
    format!(
        "{} {} {} for {} {}.xlsx",
        export_name.trim(),
        section,
        term_label,
        session.trim(),
        season.trim()
    )
}

fn build_preview(
    connection: &rusqlite::Connection,
    template_path: &Path,
    sheet_name: Option<&str>,
    section_id: i64,
    term: &str,
    final_field_id: Option<i64>,
) -> Result<ExcelMatchPreview, String> {
    let preflight = inspect_workbook(template_path, sheet_name)?;
    let section: (i64, String, String, String, String) = connection
        .query_row(
            "SELECT s.course_id, s.label, c.export_name, sem.session, sem.season
             FROM sections s
             JOIN courses c ON c.id = s.course_id
             JOIN semesters sem ON sem.id = c.semester_id
             WHERE s.id = ?1",
            params![section_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|_| "Section not found.".to_string())?;
    let field_id = match final_field_id {
        Some(id) => id,
        None => fields_for_course(connection, section.0)?
            .into_iter()
            .find(|field| field.term == term && field.is_final && !field.archived)
            .map(|field| field.id)
            .ok_or_else(|| {
                format!("No final {term} result field is configured for this course.")
            })?,
    };
    let roster = roster(connection, section_id)?;
    let mut gradia_ids = HashMap::new();
    for enrollment in &roster {
        gradia_ids.insert(
            normalize_id(&enrollment.student_identifier),
            (
                enrollment.student_identifier.clone(),
                enrollment.enrollment_id,
            ),
        );
    }
    let mut seen = HashSet::new();
    let mut duplicate_template_ids = Vec::new();
    let mut template_ids_not_found = Vec::new();
    let mut missing_final_marks = Vec::new();
    let mut marks_by_row = HashMap::new();
    let mut matched_ids = HashSet::new();
    for row in &preflight.rows {
        let normalized = normalize_id(&row.student_identifier);
        if !seen.insert(normalized.clone()) {
            duplicate_template_ids.push(row.student_identifier.clone());
            continue;
        }
        let Some((display_id, enrollment_id)) = gradia_ids.get(&normalized) else {
            template_ids_not_found.push(row.student_identifier.clone());
            continue;
        };
        matched_ids.insert(normalized);
        let mark: Option<f64> = connection
            .query_row(
                "SELECT numeric_value FROM grade_entries
                 WHERE field_id = ?1 AND enrollment_id = ?2 AND state = 'value'",
                params![field_id, enrollment_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();
        match mark {
            Some(mark) => {
                marks_by_row.insert(row.row_number, mark);
            }
            None => missing_final_marks.push(display_id.clone()),
        }
    }
    let gradia_students_not_in_template = gradia_ids
        .iter()
        .filter(|(normalized, _)| !matched_ids.contains(*normalized))
        .map(|(_, (display, _))| display.clone())
        .collect::<Vec<_>>();
    Ok(ExcelMatchPreview {
        template_rows: preflight.rows.len(),
        matched: matched_ids.len(),
        changed: marks_by_row.len(),
        template_ids_not_found,
        gradia_students_not_in_template,
        missing_final_marks,
        duplicate_template_ids,
        output_filename: output_filename(&section.2, &section.1, term, &section.3, &section.4),
        marks_by_row,
    })
}

#[tauri::command]
pub fn preview_excel_export(
    state: State<'_, DbState>,
    template_path: String,
    sheet_name: Option<String>,
    section_id: i64,
    term: String,
    final_field_id: Option<i64>,
) -> Result<ExcelMatchPreview, String> {
    let connection = db::open(&state.path)?;
    build_preview(
        &connection,
        Path::new(&template_path),
        sheet_name.as_deref(),
        section_id,
        &term,
        final_field_id,
    )
}

fn replace_cell_values(
    xml: &str,
    mark_column: &str,
    marks_by_row: &HashMap<usize, f64>,
) -> Result<String, String> {
    let mut output = xml.to_string();
    for (row, mark) in marks_by_row {
        let reference = format!("{mark_column}{row}");
        let pattern = format!(
            r#"(?s)(<c\b[^>]*\br="{}"[^>]*>.*?<v\b[^>]*>)(.*?)(</v>)"#,
            regex::escape(&reference)
        );
        let regex = Regex::new(&pattern).map_err(|e| e.to_string())?;
        if !regex.is_match(&output) {
            return Err(format!(
                "Mark cell {reference} does not contain a writable numeric value."
            ));
        }
        let replacement = format!("${{1}}{}${{3}}", format_number(*mark));
        output = regex.replace(&output, replacement.as_str()).to_string();
    }
    Ok(output)
}

fn format_number(value: f64) -> String {
    if value.fract().abs() < 1e-12 {
        format!("{value:.0}")
    } else {
        let mut value = format!("{value:.8}");
        while value.ends_with('0') {
            value.pop();
        }
        value
    }
}

fn write_package(
    package: &Package,
    output: &Path,
    changed: &HashMap<String, Vec<u8>>,
) -> Result<(), String> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file =
        File::create(output).map_err(|e| format!("Unable to create output workbook: {e}"))?;
    let mut writer = ZipWriter::new(file);
    for entry in &package.entries {
        let mut options = SimpleFileOptions::default().compression_method(entry.compression);
        if let Some(mode) = entry.unix_mode {
            options = options.unix_permissions(mode);
        }
        if entry.is_dir {
            writer
                .add_directory(&entry.name, options)
                .map_err(|e| e.to_string())?;
        } else {
            writer
                .start_file(&entry.name, options)
                .map_err(|e| e.to_string())?;
            writer
                .write_all(changed.get(&entry.name).unwrap_or(&entry.data))
                .map_err(|e| e.to_string())?;
        }
    }
    writer.finish().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn export_excel(
    state: State<'_, DbState>,
    template_path: String,
    sheet_name: Option<String>,
    section_id: i64,
    term: String,
    final_field_id: Option<i64>,
    output_path: String,
) -> Result<ExcelMatchPreview, String> {
    let template = PathBuf::from(&template_path);
    let output = PathBuf::from(&output_path);
    if template == output {
        return Err(
            "Gradia never overwrites the source template. Choose a new output path.".into(),
        );
    }
    let mut connection = db::open(&state.path)?;
    let preview = build_preview(
        &connection,
        &template,
        sheet_name.as_deref(),
        section_id,
        &term,
        final_field_id,
    )?;
    if !preview.duplicate_template_ids.is_empty() {
        return Err(
            "Export is blocked because the template contains duplicate Student IDs.".into(),
        );
    }
    let preflight = inspect_workbook(&template, sheet_name.as_deref())?;
    let package = read_package(&template)?;
    let selected_path = preflight
        .sheets
        .iter()
        .find(|sheet| sheet.name == preflight.selected_sheet)
        .map(|sheet| sheet.path.clone())
        .ok_or("Selected worksheet package path is missing.")?;
    let original = package_text(&package, &selected_path)?;
    let updated = replace_cell_values(original, &preflight.mark_column, &preview.marks_by_row)?;
    let changed = HashMap::from([(selected_path.clone(), updated.into_bytes())]);
    write_package(&package, &output, &changed)?;

    // Reopen and validate the exported values and package member equality.
    let exported = read_package(&output)?;
    for original_entry in &package.entries {
        if original_entry.name == selected_path {
            continue;
        }
        let exported_entry = exported
            .entries
            .iter()
            .find(|entry| entry.name == original_entry.name)
            .ok_or_else(|| format!("Export lost package member {}.", original_entry.name))?;
        if exported_entry.data != original_entry.data {
            return Err(format!(
                "Workbook fidelity check failed: {} changed unexpectedly.",
                original_entry.name
            ));
        }
    }
    inspect_workbook(&output, Some(&preflight.selected_sheet))?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    transaction
        .execute(
            "INSERT INTO export_jobs(
                section_id, term, template_path, output_path, matched_count,
                changed_count, summary_json, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                section_id,
                term,
                template_path,
                output_path,
                preview.matched as i64,
                preview.changed as i64,
                serde_json::to_string(&preview).map_err(|e| e.to_string())?,
                db::now()
            ],
        )
        .map_err(|e| e.to_string())?;
    db::audit(
        &transaction,
        "export_job",
        Some(transaction.last_insert_rowid()),
        "excel_export",
        None,
        Some(&serde_json::to_string(&preview).map_err(|e| e.to_string())?),
        None,
    )?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(preview)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_template(path: &Path) {
        let file = File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        let parts = [
            (
                "[Content_Types].xml",
                r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"></Types>"#,
            ),
            (
                "xl/workbook.xml",
                r#"<?xml version="1.0"?><workbook xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Marks" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
            ),
            (
                "xl/_rels/workbook.xml.rels",
                r#"<?xml version="1.0"?><Relationships><Relationship Id="rId1" Type="x" Target="worksheets/sheet1.xml"/></Relationships>"#,
            ),
            (
                "xl/sharedStrings.xml",
                r#"<?xml version="1.0"?><sst><si><t>Student ID</t></si><si><t>Name</t></si><si><t>Mark</t></si><si><t>26-1</t></si><si><t>Ada</t></si></sst>"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                r#"<?xml version="1.0"?><worksheet><sheetData><row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c><c r="C1" t="s"><v>2</v></c></row><row r="2"><c r="A2" t="s"><v>3</v></c><c r="B2" t="s"><v>4</v></c><c r="C2" t="n"><v>0</v></c></row></sheetData></worksheet>"#,
            ),
        ];
        for (name, body) in parts {
            zip.start_file(name, options).unwrap();
            zip.write_all(body.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn preflight_and_surgical_update_preserve_other_parts() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("Course B7 Midterm for  Summer.xlsx");
        let output = directory.path().join("output.xlsx");
        make_template(&source);
        let preflight = inspect_workbook(&source, None).unwrap();
        assert_eq!(preflight.mark_column, "C");
        assert_eq!(preflight.rows[0].student_identifier, "26-1");
        let package = read_package(&source).unwrap();
        let xml = package_text(&package, "xl/worksheets/sheet1.xml").unwrap();
        let updated = replace_cell_values(xml, "C", &HashMap::from([(2, 87.5)])).unwrap();
        write_package(
            &package,
            &output,
            &HashMap::from([("xl/worksheets/sheet1.xml".into(), updated.into_bytes())]),
        )
        .unwrap();
        let output_package = read_package(&output).unwrap();
        assert_eq!(
            package_text(&package, "xl/sharedStrings.xml").unwrap(),
            package_text(&output_package, "xl/sharedStrings.xml").unwrap()
        );
        assert!(package_text(&output_package, "xl/worksheets/sheet1.xml")
            .unwrap()
            .contains("<v>87.5</v>"));
    }

    #[test]
    fn filename_brackets_sections_ending_in_digits() {
        assert_eq!(
            output_filename(
                "INTRODUCTION TO PROGRAMMING LAB",
                "B7",
                "mid",
                "2025-2026",
                "Fall"
            ),
            "INTRODUCTION TO PROGRAMMING LAB [B7] Midterm for 2025-2026 Fall.xlsx"
        );
        assert_eq!(
            output_filename("DATA STRUCTURE LAB", "G", "mid", "2025-2026", "Spring"),
            "DATA STRUCTURE LAB G Midterm for 2025-2026 Spring.xlsx"
        );
    }

    #[test]
    fn supplied_institution_templates_pass_real_preflight() {
        let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/project_info");
        let marks_template =
            project.join("INTRODUCTION TO PROGRAMMING LAB B Midterm for  Summer.xlsx");
        let gradebook = project.join("DS Lab [ G ].xlsx");

        // These user-supplied workbooks contain private institutional data and are
        // deliberately excluded from the public repository. Validate them whenever
        // they are present locally; public CI exercises the same package invariants
        // with the synthetic fixture above.
        if !marks_template.is_file() || !gradebook.is_file() {
            eprintln!("Private institutional workbook fixtures are not available; skipping local-only preflight.");
            return;
        }

        let inspected = inspect_workbook(&marks_template, None).unwrap();
        assert_eq!(inspected.student_id_column, "A");
        assert_eq!(inspected.mark_column, "C");
        assert_eq!(inspected.rows.len(), 39);
        assert_eq!(inspected.detected_term.as_deref(), Some("mid"));
        let (sheet, roster_rows) = read_roster_rows(&marks_template, None).unwrap();
        assert_eq!(sheet, inspected.selected_sheet);
        assert_eq!(roster_rows.len(), 39);
        assert!(!roster_rows[0].student_identifier.is_empty());
        assert!(!roster_rows[0].name.is_empty());

        let gradebook_package = read_package(&gradebook).unwrap();
        let sheets = workbook_sheets(&gradebook_package).unwrap();
        assert_eq!(sheets.len(), 7);
    }
}

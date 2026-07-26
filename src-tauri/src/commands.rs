use crate::calc::{
    descriptive_statistics, evaluate_rule, lookup_grade, referenced_fields, validate_bands,
    validate_dependency_graph, RuleNode,
};
use crate::db::{self, DbState};
use crate::excel;
use crate::models::*;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tauri::State;

fn connection(state: &State<'_, DbState>) -> Result<Connection, String> {
    db::open(&state.path)
}

fn clean_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        Err(format!("{label} is required."))
    } else {
        Ok(value.to_string())
    }
}

fn query_semesters(connection: &Connection) -> Result<Vec<Semester>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, season, session, is_active, created_at
             FROM semesters ORDER BY is_active DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(Semester {
                id: row.get(0)?,
                season: row.get(1)?,
                session: row.get(2)?,
                is_active: row.get::<_, i64>(3)? == 1,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn query_courses(connection: &Connection, semester_id: Option<i64>) -> Result<Vec<Course>, String> {
    let sql = if semester_id.is_some() {
        "SELECT id, semester_id, grading_policy_id, code, name, export_name, color_hex, created_at
         FROM courses WHERE semester_id = ?1 ORDER BY code"
    } else {
        "SELECT id, semester_id, grading_policy_id, code, name, export_name, color_hex, created_at
         FROM courses ORDER BY code"
    };
    let mut statement = connection.prepare(sql).map_err(|e| e.to_string())?;
    let mapper = |row: &rusqlite::Row<'_>| {
        Ok(Course {
            id: row.get(0)?,
            semester_id: row.get(1)?,
            grading_policy_id: row.get(2)?,
            code: row.get(3)?,
            name: row.get(4)?,
            export_name: row.get(5)?,
            color_hex: row.get(6)?,
            created_at: row.get(7)?,
        })
    };
    let rows = match semester_id {
        Some(id) => statement.query_map(params![id], mapper),
        None => statement.query_map([], mapper),
    }
    .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn query_sections(connection: &Connection, course_id: Option<i64>) -> Result<Vec<Section>, String> {
    let sql = if course_id.is_some() {
        "SELECT id, course_id, label, order_index, archived, created_at
         FROM sections WHERE course_id = ?1 ORDER BY archived, order_index, label"
    } else {
        "SELECT id, course_id, label, order_index, archived, created_at
         FROM sections ORDER BY archived, order_index, label"
    };
    let mut statement = connection.prepare(sql).map_err(|e| e.to_string())?;
    let mapper = |row: &rusqlite::Row<'_>| {
        Ok(Section {
            id: row.get(0)?,
            course_id: row.get(1)?,
            label: row.get(2)?,
            order_index: row.get(3)?,
            archived: row.get::<_, i64>(4)? == 1,
            created_at: row.get(5)?,
        })
    };
    let rows = match course_id {
        Some(id) => statement.query_map(params![id], mapper),
        None => statement.query_map([], mapper),
    }
    .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn query_policies(connection: &Connection) -> Result<Vec<GradingPolicy>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, name, description, is_default, version
             FROM grading_policies ORDER BY is_default DESC, name",
        )
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(GradingPolicy {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                is_default: row.get::<_, i64>(3)? == 1,
                version: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_bootstrap(state: State<'_, DbState>) -> Result<BootstrapData, String> {
    let connection = connection(&state)?;
    Ok(BootstrapData {
        semesters: query_semesters(&connection)?,
        courses: query_courses(&connection, None)?,
        sections: query_sections(&connection, None)?,
        policies: query_policies(&connection)?,
    })
}

#[tauri::command]
pub fn create_semester(
    state: State<'_, DbState>,
    season: String,
    session: String,
) -> Result<Semester, String> {
    let season = clean_required(&season, "Season")?;
    let session = clean_required(&session, "Session")?;
    let mut connection = connection(&state)?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    transaction
        .execute("UPDATE semesters SET is_active = 0", [])
        .map_err(|e| e.to_string())?;
    let timestamp = db::now();
    transaction
        .execute(
            "INSERT INTO semesters(season, session, is_active, created_at) VALUES (?1, ?2, 1, ?3)",
            params![season, session, timestamp],
        )
        .map_err(|e| format!("Unable to create semester: {e}"))?;
    let id = transaction.last_insert_rowid();
    db::audit(
        &transaction,
        "semester",
        Some(id),
        "create",
        None,
        Some(&json!({"season": season, "session": session}).to_string()),
        None,
    )?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(Semester {
        id,
        season,
        session,
        is_active: true,
        created_at: timestamp,
    })
}

#[tauri::command]
pub fn set_active_semester(state: State<'_, DbState>, id: i64) -> Result<(), String> {
    let mut connection = connection(&state)?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let exists: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM semesters WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if exists == 0 {
        return Err("Semester not found.".into());
    }
    transaction
        .execute(
            "UPDATE semesters SET is_active = CASE WHEN id = ?1 THEN 1 ELSE 0 END",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    db::audit(
        &transaction,
        "semester",
        Some(id),
        "activate",
        None,
        None,
        None,
    )?;
    transaction.commit().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_course(
    state: State<'_, DbState>,
    semester_id: i64,
    code: String,
    name: String,
    export_name: String,
    color_hex: String,
    grading_policy_id: Option<i64>,
) -> Result<Course, String> {
    let code = clean_required(&code, "Course code")?;
    let name = clean_required(&name, "Course name")?;
    let export_name = if export_name.trim().is_empty() {
        name.to_uppercase()
    } else {
        export_name.trim().to_string()
    };
    let color_hex = if color_hex.trim().is_empty() {
        "#8b5cf6".into()
    } else {
        color_hex
    };
    let mut connection = connection(&state)?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let policy_id = match grading_policy_id {
        Some(id) => Some(id),
        None => db::default_policy_id(&transaction)?,
    };
    let timestamp = db::now();
    transaction
        .execute(
            "INSERT INTO courses(semester_id, grading_policy_id, code, name, export_name, color_hex, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                semester_id,
                policy_id,
                code,
                name,
                export_name,
                color_hex,
                timestamp
            ],
        )
        .map_err(|e| format!("Unable to create course: {e}"))?;
    let id = transaction.last_insert_rowid();
    for (index, (view_name, term)) in [
        ("Midterm", "mid"),
        ("Final", "final"),
        ("Semester Result", "semester"),
        ("Attendance", "custom"),
    ]
    .iter()
    .enumerate()
    {
        transaction
            .execute(
                "INSERT INTO gradebook_views(course_id, name, term, order_index, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, view_name, term, index as i64, timestamp],
            )
            .map_err(|e| e.to_string())?;
    }
    db::audit(
        &transaction,
        "course",
        Some(id),
        "create",
        None,
        Some(&json!({"code": code, "name": name}).to_string()),
        None,
    )?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(Course {
        id,
        semester_id,
        grading_policy_id: policy_id,
        code,
        name,
        export_name,
        color_hex,
        created_at: timestamp,
    })
}

#[tauri::command]
pub fn create_section(
    state: State<'_, DbState>,
    course_id: i64,
    label: String,
) -> Result<Section, String> {
    let label = clean_required(&label, "Section label")?;
    let connection = connection(&state)?;
    let order: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(order_index), -1) + 1 FROM sections WHERE course_id = ?1",
            params![course_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let timestamp = db::now();
    connection
        .execute(
            "INSERT INTO sections(course_id, label, order_index, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![course_id, label, order, timestamp],
        )
        .map_err(|e| format!("Unable to create section: {e}"))?;
    let id = connection.last_insert_rowid();
    db::audit(
        &connection,
        "section",
        Some(id),
        "create",
        None,
        Some(&json!({"label": label}).to_string()),
        None,
    )?;
    Ok(Section {
        id,
        course_id,
        label,
        order_index: order,
        archived: false,
        created_at: timestamp,
    })
}

#[tauri::command]
pub fn add_student(
    state: State<'_, DbState>,
    section_id: i64,
    student_identifier: String,
    name: String,
    email: Option<String>,
) -> Result<EnrollmentRow, String> {
    let student_identifier = clean_required(&student_identifier, "Student ID")?;
    let name = clean_required(&name, "Student name")?;
    let mut connection = connection(&state)?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let timestamp = db::now();
    let student_id = match transaction
        .query_row(
            "SELECT id FROM students WHERE student_identifier = ?1",
            params![student_identifier],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
    {
        Some(id) => {
            transaction
                .execute(
                    "UPDATE students SET name = ?1, email = COALESCE(?2, email), updated_at = ?3 WHERE id = ?4",
                    params![name, email, timestamp, id],
                )
                .map_err(|e| e.to_string())?;
            id
        }
        None => {
            transaction
                .execute(
                    "INSERT INTO students(student_identifier, name, email, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![student_identifier, name, email, timestamp],
                )
                .map_err(|e| format!("Unable to create student: {e}"))?;
            transaction.last_insert_rowid()
        }
    };
    let roll_order: i64 = transaction
        .query_row(
            "SELECT COALESCE(MAX(roll_order), -1) + 1 FROM enrollments WHERE section_id = ?1",
            params![section_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    transaction
        .execute(
            "INSERT INTO enrollments(section_id, student_id, status, roll_order, created_at)
             VALUES (?1, ?2, 'active', ?3, ?4)",
            params![section_id, student_id, roll_order, timestamp],
        )
        .map_err(|e| {
            format!("Student is already enrolled in this section or cannot be enrolled: {e}")
        })?;
    let enrollment_id = transaction.last_insert_rowid();
    db::audit(
        &transaction,
        "enrollment",
        Some(enrollment_id),
        "create",
        None,
        Some(
            &json!({"section_id": section_id, "student_identifier": student_identifier, "name": name})
                .to_string(),
        ),
        None,
    )?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(EnrollmentRow {
        enrollment_id,
        section_id,
        student_id,
        student_identifier,
        name,
        email,
        status: "active".into(),
        roll_order,
    })
}

fn roster_import_preview(
    connection: &Connection,
    section_id: i64,
    path: &Path,
    sheet_name: Option<&str>,
) -> Result<RosterImportPreview, String> {
    let (sheet, mut rows) = excel::read_roster_rows(path, sheet_name)?;
    let mut student_statement = connection
        .prepare(
            "SELECT s.student_identifier,
                    EXISTS(SELECT 1 FROM enrollments e WHERE e.student_id = s.id AND e.section_id = ?1)
             FROM students s",
        )
        .map_err(|e| e.to_string())?;
    let existing = student_statement
        .query_map(params![section_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? == 1))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(identifier, enrolled)| {
            (
                identifier
                    .chars()
                    .filter(|character| !character.is_whitespace())
                    .collect::<String>()
                    .to_uppercase(),
                enrolled,
            )
        })
        .collect::<HashMap<_, _>>();
    let mut seen = HashSet::new();
    let mut duplicates = HashSet::new();
    let mut new_students = 0;
    let mut existing_students = 0;
    let mut already_enrolled = 0;
    for row in &mut rows {
        let normalized = row
            .student_identifier
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
            .to_uppercase();
        if !seen.insert(normalized.clone()) {
            duplicates.insert(row.student_identifier.clone());
            row.status = "duplicate".into();
        } else if existing.get(&normalized).copied() == Some(true) {
            already_enrolled += 1;
            row.status = "already_enrolled".into();
        } else if existing.contains_key(&normalized) {
            existing_students += 1;
            row.status = "existing_student".into();
        } else {
            new_students += 1;
            row.status = "new_student".into();
        }
    }
    let mut duplicate_ids = duplicates.into_iter().collect::<Vec<_>>();
    duplicate_ids.sort();
    Ok(RosterImportPreview {
        sheet,
        rows,
        new_students,
        existing_students,
        already_enrolled,
        duplicate_ids,
    })
}

#[tauri::command]
pub fn preview_roster_import(
    state: State<'_, DbState>,
    section_id: i64,
    path: String,
    sheet_name: Option<String>,
) -> Result<RosterImportPreview, String> {
    roster_import_preview(
        &connection(&state)?,
        section_id,
        Path::new(&path),
        sheet_name.as_deref(),
    )
}

#[tauri::command]
pub fn import_roster(
    state: State<'_, DbState>,
    section_id: i64,
    path: String,
    sheet_name: Option<String>,
) -> Result<RosterImportResult, String> {
    let mut connection = connection(&state)?;
    let preview = roster_import_preview(
        &connection,
        section_id,
        Path::new(&path),
        sheet_name.as_deref(),
    )?;
    if !preview.duplicate_ids.is_empty() {
        return Err(format!(
            "Import is blocked because the workbook repeats Student ID(s): {}.",
            preview.duplicate_ids.join(", ")
        ));
    }
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let timestamp = db::now();
    let mut next_roll: i64 = transaction
        .query_row(
            "SELECT COALESCE(MAX(roll_order), -1) + 1 FROM enrollments WHERE section_id = ?1",
            params![section_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let mut students_created = 0;
    let mut students_updated = 0;
    let mut enrollments_added = 0;
    let mut already_enrolled = 0;
    for row in &preview.rows {
        let existing_id = transaction
            .query_row(
                "SELECT id FROM students WHERE UPPER(REPLACE(student_identifier, ' ', '')) =
                    UPPER(REPLACE(?1, ' ', ''))",
                params![row.student_identifier],
                |query| query.get::<_, i64>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let student_id =
            if let Some(id) = existing_id {
                transaction
                .execute(
                    "UPDATE students SET name = ?1, email = COALESCE(?2, email), updated_at = ?3
                     WHERE id = ?4",
                    params![row.name, row.email, timestamp, id],
                )
                .map_err(|e| e.to_string())?;
                students_updated += 1;
                id
            } else {
                transaction
                .execute(
                    "INSERT INTO students(student_identifier, name, email, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![row.student_identifier.trim(), row.name.trim(), row.email, timestamp],
                )
                .map_err(|e| format!("Unable to create {}: {e}", row.student_identifier))?;
                students_created += 1;
                transaction.last_insert_rowid()
            };
        let inserted = transaction
            .execute(
                "INSERT OR IGNORE INTO enrollments(section_id, student_id, status, roll_order, created_at)
                 VALUES (?1, ?2, 'active', ?3, ?4)",
                params![section_id, student_id, next_roll, timestamp],
            )
            .map_err(|e| e.to_string())?;
        if inserted == 1 {
            enrollments_added += 1;
            next_roll += 1;
        } else {
            already_enrolled += 1;
        }
    }
    db::audit(
        &transaction,
        "section",
        Some(section_id),
        "roster_import",
        None,
        Some(
            &json!({
                "source": path,
                "sheet": preview.sheet,
                "students_created": students_created,
                "students_updated": students_updated,
                "enrollments_added": enrollments_added
            })
            .to_string(),
        ),
        None,
    )?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(RosterImportResult {
        students_created,
        students_updated,
        enrollments_added,
        already_enrolled,
    })
}

pub fn roster(connection: &Connection, section_id: i64) -> Result<Vec<EnrollmentRow>, String> {
    let mut statement = connection
        .prepare(
            "SELECT e.id, e.section_id, s.id, s.student_identifier, s.name, s.email, e.status, e.roll_order
             FROM enrollments e
             JOIN students s ON s.id = e.student_id
             WHERE e.section_id = ?1
             ORDER BY e.roll_order, s.student_identifier",
        )
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(params![section_id], |row| {
            Ok(EnrollmentRow {
                enrollment_id: row.get(0)?,
                section_id: row.get(1)?,
                student_id: row.get(2)?,
                student_identifier: row.get(3)?,
                name: row.get(4)?,
                email: row.get(5)?,
                status: row.get(6)?,
                roll_order: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_roster(
    state: State<'_, DbState>,
    section_id: i64,
) -> Result<Vec<EnrollmentRow>, String> {
    roster(&connection(&state)?, section_id)
}

#[tauri::command]
pub fn search_students(
    state: State<'_, DbState>,
    section_id: i64,
    query: String,
) -> Result<Vec<EnrollmentRow>, String> {
    let mut rows = roster(&connection(&state)?, section_id)?;
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        rows.truncate(12);
        return Ok(rows);
    }
    rows.retain(|row| {
        row.student_identifier.to_lowercase().contains(&needle)
            || row.name.to_lowercase().contains(&needle)
    });
    rows.sort_by_key(|row| {
        let id = row.student_identifier.to_lowercase();
        let name = row.name.to_lowercase();
        if id == needle {
            0
        } else if id.starts_with(&needle) {
            1
        } else if name.starts_with(&needle) {
            2
        } else if name
            .split_whitespace()
            .any(|word| word.starts_with(&needle))
        {
            3
        } else {
            4
        }
    });
    rows.truncate(20);
    Ok(rows)
}

#[tauri::command]
pub fn create_gradebook_view(
    state: State<'_, DbState>,
    course_id: i64,
    name: String,
    term: String,
) -> Result<GradebookView, String> {
    let name = clean_required(&name, "View name")?;
    if !["mid", "final", "semester", "custom"].contains(&term.as_str()) {
        return Err("Term must be mid, final, semester, or custom.".into());
    }
    let connection = connection(&state)?;
    let order_index: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(order_index), -1) + 1 FROM gradebook_views WHERE course_id = ?1",
            params![course_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    connection
        .execute(
            "INSERT INTO gradebook_views(course_id, name, term, order_index, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![course_id, name, term, order_index, db::now()],
        )
        .map_err(|e| format!("Unable to create gradebook view: {e}"))?;
    Ok(GradebookView {
        id: connection.last_insert_rowid(),
        course_id,
        name,
        term,
        order_index,
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_assessment_field(
    state: State<'_, DbState>,
    course_id: i64,
    view_id: Option<i64>,
    stable_key: String,
    label: String,
    term: String,
    field_type: String,
    max_mark: Option<f64>,
    contribution: Option<f64>,
    rule_json: Option<String>,
    is_final: bool,
) -> Result<AssessmentField, String> {
    let stable_key = clean_required(&stable_key, "Stable key")?
        .to_lowercase()
        .replace(' ', "_");
    let label = clean_required(&label, "Field label")?;
    if !["mid", "final", "semester", "custom"].contains(&term.as_str()) {
        return Err("Invalid term.".into());
    }
    if let Some(maximum) = max_mark {
        if maximum <= 0.0 || !maximum.is_finite() {
            return Err("Maximum mark must be a positive number.".into());
        }
    }
    let parsed_rule = rule_json
        .as_deref()
        .map(|rule| {
            serde_json::from_str::<RuleNode>(rule)
                .map_err(|e| format!("Calculation rule is invalid: {e}"))
        })
        .transpose()?;
    let connection = connection(&state)?;
    if let Some(rule) = &parsed_rule {
        for dependency in referenced_fields(rule) {
            let belongs_to_course = connection
                .query_row(
                    "SELECT EXISTS(
                        SELECT 1 FROM assessment_fields
                        WHERE id = ?1 AND course_id = ?2 AND archived = 0
                     )",
                    params![dependency, course_id],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|e| e.to_string())?
                == 1;
            if !belongs_to_course {
                return Err(format!(
                    "Calculation source field {dependency} is not an active field in this course."
                ));
            }
        }
        let mut rules = HashMap::from([(-1, rule.clone())]);
        let mut statement = connection
            .prepare(
                "SELECT id, rule_json FROM assessment_fields
                 WHERE course_id = ?1 AND rule_json IS NOT NULL AND archived = 0",
            )
            .map_err(|e| e.to_string())?;
        for row in statement
            .query_map(params![course_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?
        {
            let (id, json) = row.map_err(|e| e.to_string())?;
            rules.insert(id, serde_json::from_str(&json).map_err(|e| e.to_string())?);
        }
        validate_dependency_graph(&rules)?;
    }
    let order_index: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(order_index), -1) + 1 FROM assessment_fields WHERE course_id = ?1",
            params![course_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let timestamp = db::now();
    connection
        .execute(
            "INSERT INTO assessment_fields(
                course_id, view_id, stable_key, label, term, field_type, max_mark,
                contribution, rule_json, is_final, order_index, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
            params![
                course_id,
                view_id,
                stable_key,
                label,
                term,
                field_type,
                max_mark,
                contribution,
                rule_json,
                is_final as i64,
                order_index,
                timestamp
            ],
        )
        .map_err(|e| format!("Unable to create assessment field: {e}"))?;
    let id = connection.last_insert_rowid();
    db::audit(
        &connection,
        "assessment_field",
        Some(id),
        "create",
        None,
        Some(&json!({"label": label, "max_mark": max_mark}).to_string()),
        None,
    )?;
    Ok(AssessmentField {
        id,
        course_id,
        view_id,
        stable_key,
        label,
        term,
        field_type,
        max_mark,
        contribution,
        rule_json,
        is_final,
        order_index,
        archived: false,
    })
}

fn field_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssessmentField> {
    Ok(AssessmentField {
        id: row.get(0)?,
        course_id: row.get(1)?,
        view_id: row.get(2)?,
        stable_key: row.get(3)?,
        label: row.get(4)?,
        term: row.get(5)?,
        field_type: row.get(6)?,
        max_mark: row.get(7)?,
        contribution: row.get(8)?,
        rule_json: row.get(9)?,
        is_final: row.get::<_, i64>(10)? == 1,
        order_index: row.get(11)?,
        archived: row.get::<_, i64>(12)? == 1,
    })
}

pub fn fields_for_course(
    connection: &Connection,
    course_id: i64,
) -> Result<Vec<AssessmentField>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, course_id, view_id, stable_key, label, term, field_type, max_mark,
                    contribution, rule_json, is_final, order_index, archived
             FROM assessment_fields WHERE course_id = ?1
             ORDER BY archived, order_index, id",
        )
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(params![course_id], field_from_row)
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_grade_entry(
    state: State<'_, DbState>,
    field_id: i64,
    enrollment_id: i64,
    numeric_value: Option<f64>,
    text_value: Option<String>,
    entry_state: String,
    note: Option<String>,
) -> Result<GradeEntry, String> {
    let allowed = [
        "missing",
        "value",
        "absent",
        "excused",
        "withdrawn",
        "incomplete",
        "not_applicable",
    ];
    if !allowed.contains(&entry_state.as_str()) {
        return Err("Invalid grade-entry state.".into());
    }
    if entry_state == "value"
        && numeric_value.is_none()
        && text_value.as_deref().unwrap_or("").is_empty()
    {
        return Err("A value entry needs a numeric or text value.".into());
    }
    let mut connection = connection(&state)?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let maximum: Option<f64> = transaction
        .query_row(
            "SELECT max_mark FROM assessment_fields WHERE id = ?1",
            params![field_id],
            |row| row.get(0),
        )
        .map_err(|_| "Assessment field not found.".to_string())?;
    if let Some(value) = numeric_value {
        if !value.is_finite() {
            return Err("Mark must be a finite number.".into());
        }
        if value < 0.0 {
            return Err("Mark cannot be negative. Use a penalty field for deductions.".into());
        }
        if let Some(maximum) = maximum {
            if value > maximum {
                return Err(format!(
                    "Mark {value} exceeds the allowed maximum {maximum}."
                ));
            }
        }
    }
    let old_json: Option<String> = transaction
        .query_row(
            "SELECT json_object('numeric_value', numeric_value, 'text_value', text_value, 'state', state, 'note', note)
             FROM grade_entries WHERE field_id = ?1 AND enrollment_id = ?2",
            params![field_id, enrollment_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let timestamp = db::now();
    transaction
        .execute(
            "INSERT INTO grade_entries(
                field_id, enrollment_id, numeric_value, text_value, state, note, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
             ON CONFLICT(field_id, enrollment_id) DO UPDATE SET
                numeric_value = excluded.numeric_value,
                text_value = excluded.text_value,
                state = excluded.state,
                note = excluded.note,
                updated_at = excluded.updated_at",
            params![
                field_id,
                enrollment_id,
                numeric_value,
                text_value,
                entry_state,
                note,
                timestamp
            ],
        )
        .map_err(|e| format!("Unable to save mark: {e}"))?;
    let id: i64 = transaction
        .query_row(
            "SELECT id FROM grade_entries WHERE field_id = ?1 AND enrollment_id = ?2",
            params![field_id, enrollment_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    db::audit(
        &transaction,
        "grade_entry",
        Some(id),
        if old_json.is_some() {
            "update"
        } else {
            "create"
        },
        old_json.as_deref(),
        Some(
            &json!({
                "numeric_value": numeric_value,
                "text_value": text_value,
                "state": entry_state,
                "note": note
            })
            .to_string(),
        ),
        None,
    )?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(GradeEntry {
        id,
        field_id,
        enrollment_id,
        numeric_value,
        text_value,
        state: entry_state,
        note,
        updated_at: timestamp,
    })
}

fn course_by_id(connection: &Connection, id: i64) -> Result<Course, String> {
    connection
        .query_row(
            "SELECT id, semester_id, grading_policy_id, code, name, export_name, color_hex, created_at
             FROM courses WHERE id = ?1",
            params![id],
            |row| {
                Ok(Course {
                    id: row.get(0)?,
                    semester_id: row.get(1)?,
                    grading_policy_id: row.get(2)?,
                    code: row.get(3)?,
                    name: row.get(4)?,
                    export_name: row.get(5)?,
                    color_hex: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        )
        .map_err(|_| "Course not found.".into())
}

fn section_by_id(connection: &Connection, id: i64) -> Result<Section, String> {
    connection
        .query_row(
            "SELECT id, course_id, label, order_index, archived, created_at FROM sections WHERE id = ?1",
            params![id],
            |row| {
                Ok(Section {
                    id: row.get(0)?,
                    course_id: row.get(1)?,
                    label: row.get(2)?,
                    order_index: row.get(3)?,
                    archived: row.get::<_, i64>(4)? == 1,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|_| "Section not found.".into())
}

#[tauri::command]
pub fn get_gradebook(state: State<'_, DbState>, section_id: i64) -> Result<GradebookData, String> {
    let connection = connection(&state)?;
    let section = section_by_id(&connection, section_id)?;
    let course = course_by_id(&connection, section.course_id)?;
    let enrollments = roster(&connection, section_id)?;
    let fields = fields_for_course(&connection, course.id)?;
    let mut view_statement = connection
        .prepare(
            "SELECT id, course_id, name, term, order_index
             FROM gradebook_views WHERE course_id = ?1 ORDER BY order_index, id",
        )
        .map_err(|e| e.to_string())?;
    let views = view_statement
        .query_map(params![course.id], |row| {
            Ok(GradebookView {
                id: row.get(0)?,
                course_id: row.get(1)?,
                name: row.get(2)?,
                term: row.get(3)?,
                order_index: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut entry_statement = connection
        .prepare(
            "SELECT ge.id, ge.field_id, ge.enrollment_id, ge.numeric_value, ge.text_value,
                    ge.state, ge.note, ge.updated_at
             FROM grade_entries ge
             JOIN enrollments e ON e.id = ge.enrollment_id
             WHERE e.section_id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let entries = entry_statement
        .query_map(params![section_id], |row| {
            Ok(GradeEntry {
                id: row.get(0)?,
                field_id: row.get(1)?,
                enrollment_id: row.get(2)?,
                numeric_value: row.get(3)?,
                text_value: row.get(4)?,
                state: row.get(5)?,
                note: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let computed = compute_results(&connection, section_id)?;
    Ok(GradebookData {
        course,
        section,
        enrollments,
        views,
        fields,
        entries,
        computed,
    })
}

pub fn bands_for_policy(connection: &Connection, policy_id: i64) -> Result<Vec<GradeBand>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, policy_id, min_percent, max_percent, min_inclusive, max_inclusive,
                    grade_label, grade_point, result_label, color_hex, order_index
             FROM grade_bands WHERE policy_id = ?1 ORDER BY order_index, max_percent DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(params![policy_id], |row| {
            Ok(GradeBand {
                id: row.get(0)?,
                policy_id: row.get(1)?,
                min_percent: row.get(2)?,
                max_percent: row.get(3)?,
                min_inclusive: row.get::<_, i64>(4)? == 1,
                max_inclusive: row.get::<_, i64>(5)? == 1,
                grade_label: row.get(6)?,
                grade_point: row.get(7)?,
                result_label: row.get(8)?,
                color_hex: row.get(9)?,
                order_index: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_grade_bands(
    state: State<'_, DbState>,
    policy_id: i64,
) -> Result<Vec<GradeBand>, String> {
    bands_for_policy(&connection(&state)?, policy_id)
}

#[tauri::command]
pub fn save_grading_policy(
    state: State<'_, DbState>,
    policy_id: Option<i64>,
    name: String,
    description: String,
    make_default: bool,
    mut bands: Vec<GradeBand>,
) -> Result<GradingPolicy, String> {
    let name = clean_required(&name, "Policy name")?;
    for (index, band) in bands.iter_mut().enumerate() {
        band.order_index = index as i64;
    }
    validate_bands(&bands)?;
    let mut connection = connection(&state)?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    if make_default {
        transaction
            .execute("UPDATE grading_policies SET is_default = 0", [])
            .map_err(|e| e.to_string())?;
    }
    let timestamp = db::now();
    let id = match policy_id {
        Some(id) => {
            transaction
                .execute(
                    "UPDATE grading_policies
                     SET name = ?1, description = ?2, is_default = ?3, version = version + 1, updated_at = ?4
                     WHERE id = ?5",
                    params![name, description, make_default as i64, timestamp, id],
                )
                .map_err(|e| e.to_string())?;
            transaction
                .execute("DELETE FROM grade_bands WHERE policy_id = ?1", params![id])
                .map_err(|e| e.to_string())?;
            id
        }
        None => {
            transaction
                .execute(
                    "INSERT INTO grading_policies(name, description, is_default, version, created_at, updated_at)
                     VALUES (?1, ?2, ?3, 1, ?4, ?4)",
                    params![name, description, make_default as i64, timestamp],
                )
                .map_err(|e| e.to_string())?;
            transaction.last_insert_rowid()
        }
    };
    for band in &bands {
        transaction
            .execute(
                "INSERT INTO grade_bands(
                    policy_id, min_percent, max_percent, min_inclusive, max_inclusive,
                    grade_label, grade_point, result_label, color_hex, order_index
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    id,
                    band.min_percent,
                    band.max_percent,
                    band.min_inclusive as i64,
                    band.max_inclusive as i64,
                    band.grade_label,
                    band.grade_point,
                    band.result_label,
                    band.color_hex,
                    band.order_index
                ],
            )
            .map_err(|e| e.to_string())?;
    }
    db::audit(
        &transaction,
        "grading_policy",
        Some(id),
        if policy_id.is_some() {
            "update"
        } else {
            "create"
        },
        None,
        Some(&json!({"name": name, "bands": bands}).to_string()),
        None,
    )?;
    let version: i64 = transaction
        .query_row(
            "SELECT version FROM grading_policies WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(GradingPolicy {
        id,
        name,
        description,
        is_default: make_default,
        version,
    })
}

fn compute_results(connection: &Connection, section_id: i64) -> Result<Vec<StudentResult>, String> {
    let section = section_by_id(connection, section_id)?;
    let course = course_by_id(connection, section.course_id)?;
    let fields = fields_for_course(connection, course.id)?;
    let enrollments = roster(connection, section_id)?;
    let policy_id = course
        .grading_policy_id
        .or(db::default_policy_id(connection)?);
    let bands = match policy_id {
        Some(id) => bands_for_policy(connection, id)?,
        None => vec![],
    };
    let mut results = Vec::new();
    for enrollment in enrollments {
        let mut values = HashMap::<i64, Option<f64>>::new();
        let mut statement = connection
            .prepare(
                "SELECT field_id, numeric_value, state FROM grade_entries WHERE enrollment_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map(params![enrollment.enrollment_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<f64>>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (field_id, value, state) = row.map_err(|e| e.to_string())?;
            values.insert(field_id, if state == "value" { value } else { None });
        }
        for field in &fields {
            values.entry(field.id).or_insert(None);
        }
        for _ in 0..fields.len().max(1) {
            let mut changed = false;
            for field in &fields {
                if let Some(rule_json) = field.rule_json.as_deref() {
                    let rule: RuleNode =
                        serde_json::from_str(rule_json).map_err(|e| e.to_string())?;
                    let (value, _) = evaluate_rule(&rule, &values);
                    if values.get(&field.id).copied().flatten() != value {
                        values.insert(field.id, value);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        let final_field = fields
            .iter()
            .find(|field| field.is_final && !field.archived);
        let final_percentage = final_field.and_then(|field| {
            let value = values.get(&field.id).copied().flatten()?;
            let maximum = field.max_mark.unwrap_or(100.0);
            (maximum > 0.0).then_some(value / maximum * 100.0)
        });
        let grade = final_percentage.and_then(|value| lookup_grade(value, &bands));
        results.push(StudentResult {
            enrollment_id: enrollment.enrollment_id,
            student_identifier: enrollment.student_identifier,
            name: enrollment.name,
            values,
            final_percentage,
            grade,
        });
    }
    Ok(results)
}

#[tauri::command]
pub fn get_section_analytics(
    state: State<'_, DbState>,
    section_id: i64,
) -> Result<AnalyticsSummary, String> {
    let connection = connection(&state)?;
    let results = compute_results(&connection, section_id)?;
    let values: Vec<f64> = results
        .iter()
        .filter_map(|result| result.final_percentage)
        .collect();
    let stats = (!values.is_empty()).then(|| descriptive_statistics(&values));
    let mut grade_frequency = HashMap::new();
    let mut pass_count = 0;
    let mut fail_count = 0;
    for result in &results {
        if let Some(grade) = &result.grade {
            *grade_frequency
                .entry(grade.grade_label.clone())
                .or_insert(0) += 1;
            if grade.result_label.eq_ignore_ascii_case("fail") {
                fail_count += 1;
            } else {
                pass_count += 1;
            }
        }
    }
    Ok(AnalyticsSummary {
        section_id,
        count: results.len(),
        completed: values.len(),
        missing: results.len().saturating_sub(values.len()),
        mean: stats.map(|s| s.0),
        median: stats.map(|s| s.1),
        minimum: stats.map(|s| s.2),
        maximum: stats.map(|s| s.3),
        standard_deviation: stats.map(|s| s.4),
        pass_count,
        fail_count,
        grade_frequency,
        results,
    })
}

#[tauri::command]
pub fn create_attendance_session(
    state: State<'_, DbState>,
    section_id: i64,
    held_on: String,
    title: String,
    note: Option<String>,
) -> Result<AttendanceSession, String> {
    let held_on = clean_required(&held_on, "Attendance date")?;
    let title = if title.trim().is_empty() {
        "Class".into()
    } else {
        title.trim().to_string()
    };
    let mut connection = connection(&state)?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let timestamp = db::now();
    transaction
        .execute(
            "INSERT INTO attendance_sessions(section_id, held_on, title, note, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![section_id, held_on, title, note, timestamp],
        )
        .map_err(|e| format!("Unable to create attendance session: {e}"))?;
    let id = transaction.last_insert_rowid();
    transaction
        .execute(
            "INSERT INTO attendance_records(session_id, enrollment_id, status, updated_at)
             SELECT ?1, id, 'present', ?2 FROM enrollments
             WHERE section_id = ?3 AND status = 'active'",
            params![id, timestamp, section_id],
        )
        .map_err(|e| e.to_string())?;
    let present: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM attendance_records WHERE session_id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    db::audit(
        &transaction,
        "attendance_session",
        Some(id),
        "create_default_present",
        None,
        Some(&json!({"section_id": section_id, "held_on": held_on}).to_string()),
        None,
    )?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(AttendanceSession {
        id,
        section_id,
        held_on,
        title,
        note,
        present,
        absent: 0,
        late: 0,
        excused: 0,
        left_early: 0,
    })
}

#[tauri::command]
pub fn set_attendance_status(
    state: State<'_, DbState>,
    session_id: i64,
    enrollment_id: i64,
    status: String,
    note: Option<String>,
) -> Result<AttendanceRecord, String> {
    if !["present", "absent", "late", "excused", "left_early"].contains(&status.as_str()) {
        return Err("Invalid attendance status.".into());
    }
    let connection = connection(&state)?;
    let old: Option<String> = connection
        .query_row(
            "SELECT status FROM attendance_records WHERE session_id = ?1 AND enrollment_id = ?2",
            params![session_id, enrollment_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    connection
        .execute(
            "INSERT INTO attendance_records(session_id, enrollment_id, status, note, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(session_id, enrollment_id) DO UPDATE SET
                status = excluded.status, note = excluded.note, updated_at = excluded.updated_at",
            params![session_id, enrollment_id, status, note, db::now()],
        )
        .map_err(|e| e.to_string())?;
    db::audit(
        &connection,
        "attendance_record",
        Some(enrollment_id),
        "update",
        old.as_deref(),
        Some(&json!({"session_id": session_id, "status": status}).to_string()),
        None,
    )?;
    Ok(AttendanceRecord {
        enrollment_id,
        status,
        note,
    })
}

#[tauri::command]
pub fn get_attendance(
    state: State<'_, DbState>,
    section_id: i64,
) -> Result<(Vec<AttendanceSession>, HashMap<i64, Vec<AttendanceRecord>>), String> {
    let connection = connection(&state)?;
    let mut statement = connection
        .prepare(
            "SELECT s.id, s.section_id, s.held_on, s.title, s.note,
                    SUM(CASE WHEN r.status='present' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN r.status='absent' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN r.status='late' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN r.status='excused' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN r.status='left_early' THEN 1 ELSE 0 END)
             FROM attendance_sessions s
             LEFT JOIN attendance_records r ON r.session_id = s.id
             WHERE s.section_id = ?1
             GROUP BY s.id ORDER BY s.held_on DESC, s.id DESC",
        )
        .map_err(|e| e.to_string())?;
    let sessions = statement
        .query_map(params![section_id], |row| {
            Ok(AttendanceSession {
                id: row.get(0)?,
                section_id: row.get(1)?,
                held_on: row.get(2)?,
                title: row.get(3)?,
                note: row.get(4)?,
                present: row.get(5)?,
                absent: row.get(6)?,
                late: row.get(7)?,
                excused: row.get(8)?,
                left_early: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut records = HashMap::new();
    for session in &sessions {
        let mut record_statement = connection
            .prepare(
                "SELECT enrollment_id, status, note FROM attendance_records
                 WHERE session_id = ?1 ORDER BY enrollment_id",
            )
            .map_err(|e| e.to_string())?;
        let values = record_statement
            .query_map(params![session.id], |row| {
                Ok(AttendanceRecord {
                    enrollment_id: row.get(0)?,
                    status: row.get(1)?,
                    note: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        records.insert(session.id, values);
    }
    Ok((sessions, records))
}

#[tauri::command]
pub fn toggle_pipeline(
    state: State<'_, DbState>,
    field_id: i64,
    section_id: i64,
    stage: String,
    value: bool,
) -> Result<PipelineStatus, String> {
    let (column, timestamp_column) = match stage.as_str() {
        "evaluated" => ("evaluated", "evaluated_at"),
        "marks_recorded" => ("marks_recorded", "marks_recorded_at"),
        "portal_uploaded" => ("portal_uploaded", "portal_uploaded_at"),
        _ => return Err("Invalid pipeline stage.".into()),
    };
    let connection = connection(&state)?;
    connection
        .execute(
            "INSERT INTO pipeline_status(field_id, section_id) VALUES (?1, ?2)
             ON CONFLICT(field_id, section_id) DO NOTHING",
            params![field_id, section_id],
        )
        .map_err(|e| e.to_string())?;
    let sql = format!(
        "UPDATE pipeline_status SET {column} = ?1, {timestamp_column} = ?2
         WHERE field_id = ?3 AND section_id = ?4"
    );
    let timestamp = value.then(db::now);
    connection
        .execute(&sql, params![value as i64, timestamp, field_id, section_id])
        .map_err(|e| e.to_string())?;
    pipeline_by_key(&connection, field_id, section_id)
}

#[tauri::command]
pub fn get_pipeline(
    state: State<'_, DbState>,
    section_id: i64,
) -> Result<Vec<PipelineStatus>, String> {
    let connection = connection(&state)?;
    let course_id = section_by_id(&connection, section_id)?.course_id;
    let fields = fields_for_course(&connection, course_id)?;
    fields
        .into_iter()
        .filter(|field| !field.archived)
        .map(|field| {
            pipeline_by_key(&connection, field.id, section_id).or_else(|_| {
                Ok(PipelineStatus {
                    field_id: field.id,
                    section_id,
                    evaluated: false,
                    evaluated_at: None,
                    marks_recorded: false,
                    marks_recorded_at: None,
                    portal_uploaded: false,
                    portal_uploaded_at: None,
                    pending_note: None,
                })
            })
        })
        .collect()
}

fn pipeline_by_key(
    connection: &Connection,
    field_id: i64,
    section_id: i64,
) -> Result<PipelineStatus, String> {
    connection
        .query_row(
            "SELECT field_id, section_id, evaluated, evaluated_at, marks_recorded,
                    marks_recorded_at, portal_uploaded, portal_uploaded_at, pending_note
             FROM pipeline_status WHERE field_id = ?1 AND section_id = ?2",
            params![field_id, section_id],
            |row| {
                Ok(PipelineStatus {
                    field_id: row.get(0)?,
                    section_id: row.get(1)?,
                    evaluated: row.get::<_, i64>(2)? == 1,
                    evaluated_at: row.get(3)?,
                    marks_recorded: row.get::<_, i64>(4)? == 1,
                    marks_recorded_at: row.get(5)?,
                    portal_uploaded: row.get::<_, i64>(6)? == 1,
                    portal_uploaded_at: row.get(7)?,
                    pending_note: row.get(8)?,
                })
            },
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_dashboard(
    state: State<'_, DbState>,
    semester_id: i64,
) -> Result<Vec<DashboardCourse>, String> {
    let connection = connection(&state)?;
    let courses = query_courses(&connection, Some(semester_id))?;
    let mut dashboard = Vec::new();
    for course in courses {
        let sections = query_sections(&connection, Some(course.id))?;
        let mut section_rows = Vec::new();
        for section in sections.into_iter().filter(|section| !section.archived) {
            let students: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM enrollments WHERE section_id = ?1 AND status = 'active'",
                    params![section.id],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            let fields: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM assessment_fields WHERE course_id = ?1 AND archived = 0",
                    params![course.id],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            let possible = students * fields;
            let entered: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM grade_entries ge
                     JOIN enrollments e ON e.id = ge.enrollment_id
                     JOIN assessment_fields f ON f.id = ge.field_id
                     WHERE e.section_id = ?1 AND f.archived = 0 AND ge.state != 'missing'",
                    params![section.id],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            section_rows.push(DashboardSection {
                section,
                students,
                fields,
                entered,
                possible,
                completion_percent: if possible == 0 {
                    0.0
                } else {
                    entered as f64 / possible as f64 * 100.0
                },
            });
        }
        dashboard.push(DashboardCourse {
            course,
            sections: section_rows,
        });
    }
    Ok(dashboard)
}

#[tauri::command]
pub fn finalize_results(
    state: State<'_, DbState>,
    section_id: i64,
    label: String,
) -> Result<i64, String> {
    let label = clean_required(&label, "Snapshot label")?;
    let mut connection = connection(&state)?;
    let results = compute_results(&connection, section_id)?;
    if results
        .iter()
        .any(|result| result.final_percentage.is_none())
    {
        return Err("Cannot finalize while one or more students have no final result.".into());
    }
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let revision: i64 = transaction
        .query_row(
            "SELECT COALESCE(MAX(revision), 0) + 1 FROM result_snapshots
             WHERE section_id = ?1 AND label = ?2",
            params![section_id, label],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let policy_json: String = transaction
        .query_row(
            "SELECT json_object('policy_id', c.grading_policy_id)
             FROM sections s JOIN courses c ON c.id = s.course_id WHERE s.id = ?1",
            params![section_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let rules_json = json!(fields_for_course(
        &transaction,
        section_by_id(&transaction, section_id)?.course_id
    )?)
    .to_string();
    transaction
        .execute(
            "INSERT INTO result_snapshots(section_id, label, revision, policy_json, rules_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![section_id, label, revision, policy_json, rules_json, db::now()],
        )
        .map_err(|e| e.to_string())?;
    let snapshot_id = transaction.last_insert_rowid();
    for result in &results {
        transaction
            .execute(
                "INSERT INTO snapshot_entries(snapshot_id, enrollment_id, result_json)
                 VALUES (?1, ?2, ?3)",
                params![
                    snapshot_id,
                    result.enrollment_id,
                    serde_json::to_string(result).map_err(|e| e.to_string())?
                ],
            )
            .map_err(|e| e.to_string())?;
    }
    db::audit(
        &transaction,
        "result_snapshot",
        Some(snapshot_id),
        "finalize",
        None,
        Some(&json!({"label": label, "revision": revision}).to_string()),
        None,
    )?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(snapshot_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn active_semester_invariant_and_grade_validation_work() {
        let connection = db::open_memory().unwrap();
        let now = db::now();
        connection
            .execute(
                "INSERT INTO semesters(season, session, is_active, created_at) VALUES ('Fall','2025-2026',1,?1)",
                params![now],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO semesters(season, session, is_active, created_at) VALUES ('Spring','2025-2026',0,?1)",
                params![now],
            )
            .unwrap();
        connection
            .execute(
                "UPDATE semesters SET is_active = CASE WHEN season='Spring' THEN 1 ELSE 0 END",
                [],
            )
            .unwrap();
        let active: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM semesters WHERE is_active=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(active, 1);
    }
}

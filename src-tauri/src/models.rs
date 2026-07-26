use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Semester {
    pub id: i64,
    pub season: String,
    pub session: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Course {
    pub id: i64,
    pub semester_id: i64,
    pub grading_policy_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub export_name: String,
    pub color_hex: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Section {
    pub id: i64,
    pub course_id: i64,
    pub label: String,
    pub order_index: i64,
    pub archived: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnrollmentRow {
    pub enrollment_id: i64,
    pub section_id: i64,
    pub student_id: i64,
    pub student_identifier: String,
    pub name: String,
    pub email: Option<String>,
    pub status: String,
    pub roll_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradebookView {
    pub id: i64,
    pub course_id: i64,
    pub name: String,
    pub term: String,
    pub order_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssessmentField {
    pub id: i64,
    pub course_id: i64,
    pub view_id: Option<i64>,
    pub stable_key: String,
    pub label: String,
    pub term: String,
    pub field_type: String,
    pub max_mark: Option<f64>,
    pub contribution: Option<f64>,
    pub rule_json: Option<String>,
    pub is_final: bool,
    pub order_index: i64,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradeEntry {
    pub id: i64,
    pub field_id: i64,
    pub enrollment_id: i64,
    pub numeric_value: Option<f64>,
    pub text_value: Option<String>,
    pub state: String,
    pub note: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradebookData {
    pub course: Course,
    pub section: Section,
    pub enrollments: Vec<EnrollmentRow>,
    pub views: Vec<GradebookView>,
    pub fields: Vec<AssessmentField>,
    pub entries: Vec<GradeEntry>,
    pub computed: Vec<StudentResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradingPolicy {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradeBand {
    pub id: i64,
    pub policy_id: i64,
    pub min_percent: f64,
    pub max_percent: f64,
    pub min_inclusive: bool,
    pub max_inclusive: bool,
    pub grade_label: String,
    pub grade_point: Option<f64>,
    pub result_label: String,
    pub color_hex: String,
    pub order_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradeResult {
    pub percentage: f64,
    pub grade_label: String,
    pub grade_point: Option<f64>,
    pub result_label: String,
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalculationTrace {
    pub op: String,
    pub value: Option<f64>,
    pub detail: String,
    pub children: Vec<CalculationTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StudentResult {
    pub enrollment_id: i64,
    pub student_identifier: String,
    pub name: String,
    pub values: HashMap<i64, Option<f64>>,
    pub final_percentage: Option<f64>,
    pub grade: Option<GradeResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttendanceSession {
    pub id: i64,
    pub section_id: i64,
    pub held_on: String,
    pub title: String,
    pub note: Option<String>,
    pub present: i64,
    pub absent: i64,
    pub late: i64,
    pub excused: i64,
    pub left_early: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttendanceRecord {
    pub enrollment_id: i64,
    pub status: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineStatus {
    pub field_id: i64,
    pub section_id: i64,
    pub evaluated: bool,
    pub evaluated_at: Option<String>,
    pub marks_recorded: bool,
    pub marks_recorded_at: Option<String>,
    pub portal_uploaded: bool,
    pub portal_uploaded_at: Option<String>,
    pub pending_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyticsSummary {
    pub section_id: i64,
    pub count: usize,
    pub completed: usize,
    pub missing: usize,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub standard_deviation: Option<f64>,
    pub pass_count: usize,
    pub fail_count: usize,
    pub grade_frequency: HashMap<String, usize>,
    pub results: Vec<StudentResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardCourse {
    pub course: Course,
    pub sections: Vec<DashboardSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardSection {
    pub section: Section,
    pub students: i64,
    pub fields: i64,
    pub entered: i64,
    pub possible: i64,
    pub completion_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BootstrapData {
    pub semesters: Vec<Semester>,
    pub courses: Vec<Course>,
    pub sections: Vec<Section>,
    pub policies: Vec<GradingPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkbookSheet {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RosterImportRow {
    pub source_row: usize,
    pub student_identifier: String,
    pub name: String,
    pub email: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RosterImportPreview {
    pub sheet: String,
    pub rows: Vec<RosterImportRow>,
    pub new_students: usize,
    pub existing_students: usize,
    pub already_enrolled: usize,
    pub duplicate_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RosterImportResult {
    pub students_created: usize,
    pub students_updated: usize,
    pub enrollments_added: usize,
    pub already_enrolled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateRow {
    pub row_number: usize,
    pub student_identifier: String,
    pub current_mark: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExcelPreflight {
    pub sheets: Vec<WorkbookSheet>,
    pub selected_sheet: String,
    pub student_id_column: String,
    pub mark_column: String,
    pub rows: Vec<TemplateRow>,
    pub detected_term: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExcelMatchPreview {
    pub template_rows: usize,
    pub matched: usize,
    pub changed: usize,
    pub template_ids_not_found: Vec<String>,
    pub gradia_students_not_in_template: Vec<String>,
    pub missing_final_marks: Vec<String>,
    pub duplicate_template_ids: Vec<String>,
    pub output_filename: String,
    pub marks_by_row: HashMap<usize, f64>,
}

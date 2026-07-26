PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS grading_policies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0,1)),
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS grade_bands (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    policy_id INTEGER NOT NULL REFERENCES grading_policies(id) ON DELETE CASCADE,
    min_percent REAL NOT NULL,
    max_percent REAL NOT NULL,
    min_inclusive INTEGER NOT NULL DEFAULT 1 CHECK (min_inclusive IN (0,1)),
    max_inclusive INTEGER NOT NULL DEFAULT 1 CHECK (max_inclusive IN (0,1)),
    grade_label TEXT NOT NULL,
    grade_point REAL,
    result_label TEXT NOT NULL DEFAULT 'Pass',
    color_hex TEXT NOT NULL DEFAULT '#7c5cff',
    order_index INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS semesters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    season TEXT NOT NULL,
    session TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0,1)),
    created_at TEXT NOT NULL,
    UNIQUE(season, session)
);

CREATE TABLE IF NOT EXISTS courses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    semester_id INTEGER NOT NULL REFERENCES semesters(id) ON DELETE CASCADE,
    grading_policy_id INTEGER REFERENCES grading_policies(id) ON DELETE SET NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    export_name TEXT NOT NULL,
    color_hex TEXT NOT NULL DEFAULT '#8b5cf6',
    created_at TEXT NOT NULL,
    UNIQUE(semester_id, code)
);

CREATE TABLE IF NOT EXISTS sections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0,1)),
    created_at TEXT NOT NULL,
    UNIQUE(course_id, label)
);

CREATE TABLE IF NOT EXISTS students (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_identifier TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    email TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS enrollments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    section_id INTEGER NOT NULL REFERENCES sections(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','withdrawn','incomplete','archived')),
    roll_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    UNIQUE(section_id, student_id)
);

CREATE TABLE IF NOT EXISTS gradebook_views (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    term TEXT NOT NULL DEFAULT 'custom' CHECK (term IN ('mid','final','semester','custom')),
    order_index INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    UNIQUE(course_id, name)
);

CREATE TABLE IF NOT EXISTS assessment_fields (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    view_id INTEGER REFERENCES gradebook_views(id) ON DELETE SET NULL,
    stable_key TEXT NOT NULL,
    label TEXT NOT NULL,
    term TEXT NOT NULL DEFAULT 'custom' CHECK (term IN ('mid','final','semester','custom')),
    field_type TEXT NOT NULL DEFAULT 'score' CHECK (field_type IN ('score','number','text','checkbox','date','attendance','bonus','penalty','calculated','percentage','grade','note')),
    max_mark REAL,
    contribution REAL,
    rule_json TEXT,
    is_final INTEGER NOT NULL DEFAULT 0 CHECK (is_final IN (0,1)),
    order_index INTEGER NOT NULL DEFAULT 0,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0,1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(course_id, stable_key)
);

CREATE TABLE IF NOT EXISTS grade_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    field_id INTEGER NOT NULL REFERENCES assessment_fields(id) ON DELETE CASCADE,
    enrollment_id INTEGER NOT NULL REFERENCES enrollments(id) ON DELETE CASCADE,
    numeric_value REAL,
    text_value TEXT,
    state TEXT NOT NULL DEFAULT 'missing' CHECK (state IN ('missing','value','absent','excused','withdrawn','incomplete','not_applicable')),
    note TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(field_id, enrollment_id)
);

CREATE TABLE IF NOT EXISTS calculation_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    rule_json TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS attendance_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    section_id INTEGER NOT NULL REFERENCES sections(id) ON DELETE CASCADE,
    held_on TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT 'Class',
    note TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(section_id, held_on, title)
);

CREATE TABLE IF NOT EXISTS attendance_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES attendance_sessions(id) ON DELETE CASCADE,
    enrollment_id INTEGER NOT NULL REFERENCES enrollments(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'present' CHECK (status IN ('present','absent','late','excused','left_early')),
    note TEXT,
    updated_at TEXT NOT NULL,
    UNIQUE(session_id, enrollment_id)
);

CREATE TABLE IF NOT EXISTS pipeline_status (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    field_id INTEGER NOT NULL REFERENCES assessment_fields(id) ON DELETE CASCADE,
    section_id INTEGER NOT NULL REFERENCES sections(id) ON DELETE CASCADE,
    evaluated INTEGER NOT NULL DEFAULT 0 CHECK (evaluated IN (0,1)),
    evaluated_at TEXT,
    marks_recorded INTEGER NOT NULL DEFAULT 0 CHECK (marks_recorded IN (0,1)),
    marks_recorded_at TEXT,
    portal_uploaded INTEGER NOT NULL DEFAULT 0 CHECK (portal_uploaded IN (0,1)),
    portal_uploaded_at TEXT,
    pending_note TEXT,
    UNIQUE(field_id, section_id)
);

CREATE TABLE IF NOT EXISTS export_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER REFERENCES courses(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    path TEXT NOT NULL,
    sheet_name TEXT,
    student_id_column TEXT NOT NULL DEFAULT 'A',
    mark_column TEXT NOT NULL DEFAULT 'C',
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS export_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    section_id INTEGER NOT NULL REFERENCES sections(id) ON DELETE CASCADE,
    term TEXT NOT NULL CHECK (term IN ('mid','final','semester','custom')),
    template_path TEXT NOT NULL,
    output_path TEXT NOT NULL,
    matched_count INTEGER NOT NULL DEFAULT 0,
    changed_count INTEGER NOT NULL DEFAULT 0,
    summary_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS result_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    section_id INTEGER NOT NULL REFERENCES sections(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 1,
    policy_json TEXT NOT NULL,
    rules_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(section_id, label, revision)
);

CREATE TABLE IF NOT EXISTS snapshot_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_id INTEGER NOT NULL REFERENCES result_snapshots(id) ON DELETE CASCADE,
    enrollment_id INTEGER NOT NULL REFERENCES enrollments(id) ON DELETE RESTRICT,
    result_json TEXT NOT NULL,
    UNIQUE(snapshot_id, enrollment_id)
);

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id INTEGER,
    action TEXT NOT NULL,
    old_json TEXT,
    new_json TEXT,
    reason TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_courses_semester ON courses(semester_id);
CREATE INDEX IF NOT EXISTS idx_sections_course ON sections(course_id);
CREATE INDEX IF NOT EXISTS idx_enrollments_section ON enrollments(section_id, roll_order);
CREATE INDEX IF NOT EXISTS idx_students_search ON students(student_identifier, name);
CREATE INDEX IF NOT EXISTS idx_fields_course ON assessment_fields(course_id, order_index);
CREATE INDEX IF NOT EXISTS idx_entries_enrollment ON grade_entries(enrollment_id);
CREATE INDEX IF NOT EXISTS idx_attendance_section_date ON attendance_sessions(section_id, held_on);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_log(entity_type, entity_id);

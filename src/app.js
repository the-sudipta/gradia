import { call, chooseOpenFile, chooseSaveFile, isDesktop } from "./api.js";
import { barChart, distributionChart, donutChart } from "./charts.js";
import gradiaLogoUrl from "./assets/gradia-logo-transparent.png";
import packageMetadata from "../package.json";
import {
  assessmentTypeGuide,
  calculationOperationGuide,
  entryKey,
  escapeHtml,
  formatNumber,
  gradebookViewGuide,
  gradeHeatColor,
  initials,
  insightSentences,
  optimisticGradeUpdate,
  percentage,
  rankStudents,
  studentWindowDetails,
  validateMark
} from "./core.js";

const app = document.querySelector("#app");
const toastRegion = document.querySelector("#toast-region");
const APP_VERSION = packageMetadata.version;

const state = {
  bootstrap: null,
  semesterId: null,
  courseId: null,
  sectionId: null,
  route: "dashboard",
  dashboard: null,
  gradebook: null,
  analytics: null,
  attendance: null,
  pipeline: null,
  activeAttendanceId: null,
  activeTerm: "mid",
  gradeFilter: "",
  quickQuery: "",
  quickStudentId: null,
  excel: { path: null, preflight: null, preview: null, sheet: null, term: "mid" },
  showWelcome: false,
  busy: false
};

const routeMeta = {
  dashboard: ["Overview", "Dashboard"],
  gradebook: ["Assessment", "Gradebook"],
  quick: ["Assessment", "Quick entry"],
  attendance: ["Classroom", "Attendance"],
  pipeline: ["Workflow", "Completion pipeline"],
  insights: ["Analysis", "Insights"],
  excel: ["Institutional workflow", "Excel bridge"],
  setup: ["Workspace", "Setup"],
  settings: ["Gradia", "Settings"]
};

const navItems = [
  ["dashboard", "◫", "Dashboard"],
  ["gradebook", "▦", "Gradebook"],
  ["quick", "⌕", "Quick entry"],
  ["attendance", "✓", "Attendance"],
  ["pipeline", "⇢", "Pipeline"],
  ["insights", "⌁", "Insights"],
  ["excel", "↗", "Excel bridge"],
  ["setup", "＋", "Setup"]
];

function toast(message, type = "") {
  const element = document.createElement("div");
  element.className = `toast ${type}`;
  element.textContent = message;
  toastRegion.append(element);
  setTimeout(() => element.remove(), 4200);
}

function errorMessage(error) {
  return typeof error === "string" ? error : error?.message || "An unexpected error occurred.";
}

function activeSemester() {
  return state.bootstrap?.semesters.find((item) => item.id === Number(state.semesterId));
}

function activeCourse() {
  return state.bootstrap?.courses.find((item) => item.id === Number(state.courseId));
}

function activeSection() {
  return state.bootstrap?.sections.find((item) => item.id === Number(state.sectionId));
}

function selectInitialContext() {
  const semesters = state.bootstrap.semesters;
  if (!semesters.length) return;
  const semester =
    semesters.find((item) => item.is_active) ??
    semesters.find((item) => item.id === Number(state.semesterId)) ??
    semesters[0];
  state.semesterId = semester.id;
  const courses = state.bootstrap.courses.filter((item) => item.semester_id === semester.id);
  if (!courses.some((item) => item.id === Number(state.courseId))) state.courseId = courses[0]?.id ?? null;
  const sections = state.bootstrap.sections.filter(
    (item) => item.course_id === Number(state.courseId) && !item.archived
  );
  if (!sections.some((item) => item.id === Number(state.sectionId))) state.sectionId = sections[0]?.id ?? null;
}

async function refreshBootstrap({ preserve = true } = {}) {
  const previous = preserve
    ? { semesterId: state.semesterId, courseId: state.courseId, sectionId: state.sectionId }
    : {};
  state.bootstrap = await call("get_bootstrap");
  Object.assign(state, previous);
  selectInitialContext();
}

async function loadRoute() {
  if (!state.bootstrap?.semesters.length) {
    render();
    return;
  }
  state.busy = true;
  render();
  try {
    if (state.route === "dashboard") {
      state.dashboard = await call("get_dashboard", { semesterId: Number(state.semesterId) });
    }
    if (["gradebook", "quick", "pipeline", "insights", "excel", "setup"].includes(state.route) && state.sectionId) {
      state.gradebook = await call("get_gradebook", { sectionId: Number(state.sectionId) });
    }
    if (state.route === "insights" && state.sectionId) {
      state.analytics = await call("get_section_analytics", { sectionId: Number(state.sectionId) });
    }
    if (state.route === "attendance" && state.sectionId) {
      const [sessions, records] = await call("get_attendance", { sectionId: Number(state.sectionId) });
      state.attendance = { sessions, records };
      if (!sessions.some((session) => session.id === state.activeAttendanceId)) {
        state.activeAttendanceId = sessions[0]?.id ?? null;
      }
      if (!state.gradebook) {
        state.gradebook = await call("get_gradebook", { sectionId: Number(state.sectionId) });
      }
    }
    if (state.route === "pipeline" && state.sectionId) {
      state.pipeline = await call("get_pipeline", { sectionId: Number(state.sectionId) });
    }
  } catch (error) {
    toast(errorMessage(error), "error");
  } finally {
    state.busy = false;
    render();
    if (state.route === "quick") {
      setTimeout(() => document.querySelector("#student-search")?.focus(), 0);
    }
  }
}

function selectorOptions(items, selected, label) {
  if (!items.length) return `<option value="">No ${label}</option>`;
  return items
    .map(
      (item) =>
        `<option value="${item.id}" ${item.id === Number(selected) ? "selected" : ""}>${escapeHtml(
          label === "semesters"
            ? `${item.season} ${item.session}`
            : label === "courses"
              ? `${item.code} · ${item.name}`
              : item.label
        )}</option>`
    )
    .join("");
}

function renderShell() {
  const semester = activeSemester();
  const course = activeCourse();
  const section = activeSection();
  const courses = state.bootstrap.courses.filter((item) => item.semester_id === Number(state.semesterId));
  const sections = state.bootstrap.sections.filter(
    (item) => item.course_id === Number(state.courseId) && !item.archived
  );
  const [kicker, title] = routeMeta[state.route];
  return `
    <div class="app-shell" style="--course-color:${course?.color_hex || "#9b7bff"}">
      <aside class="sidebar">
        <div class="brand">
          <img src="${gradiaLogoUrl}" alt="" />
          <div class="brand-copy">
            <div class="brand-name"><strong>Gradia</strong><span class="version-chip">v${escapeHtml(APP_VERSION)}</span></div>
            <span>Smarter academic assessment.</span>
          </div>
        </div>
        <div class="context-stack">
          <span class="context-label">Academic context</span>
          <select class="context-select" id="semester-select" aria-label="Semester">
            ${selectorOptions(state.bootstrap.semesters, state.semesterId, "semesters")}
          </select>
          <select class="context-select" id="course-select" aria-label="Course">
            ${selectorOptions(courses, state.courseId, "courses")}
          </select>
          <select class="context-select" id="section-select" aria-label="Section">
            ${selectorOptions(sections, state.sectionId, "sections")}
          </select>
        </div>
        <nav class="nav">
          ${navItems
            .map(
              ([route, icon, label]) => `
                <button class="nav-button ${state.route === route ? "active" : ""}" data-route="${route}">
                  <span class="nav-icon">${icon}</span>${label}
                </button>`
            )
            .join("")}
        </nav>
        <div class="nav-spacer"></div>
        <button class="nav-button ${state.route === "settings" ? "active" : ""}" data-route="settings">
          <span class="nav-icon">⚙</span>Settings
        </button>
        <div class="privacy-note">
          <strong>Private by design</strong>
          Student data stays on this device. Gradia makes no normal runtime network requests.
        </div>
      </aside>
      <section class="workspace">
        <header class="topbar">
          <div>
            <h1>${escapeHtml(title)}</h1>
            <div class="breadcrumb">
              <span>${escapeHtml(kicker)}</span>
              ${course ? `<span>•</span><span class="course-dot"></span><span>${escapeHtml(course.code)}</span>` : ""}
              ${section ? `<span>•</span><span>Section ${escapeHtml(section.label)}</span>` : ""}
            </div>
          </div>
          <div class="top-actions">
            ${!isDesktop ? `<span class="browser-banner">Preview data · desktop actions disabled</span>` : ""}
            <button class="shortcut" id="global-search"><span>Find student</span><kbd>Ctrl K</kbd></button>
          </div>
        </header>
        <main class="content">
          ${state.busy ? loadingPage() : routeContent()}
        </main>
      </section>
    </div>`;
}

function loadingPage() {
  return `
    <div class="page">
      <div class="page-header loading-line"><div><div class="page-kicker">Loading</div><h2>Preparing this view…</h2></div></div>
      <div class="summary-grid cards">
        ${Array.from({ length: 4 }, () => `<div class="stat-card loading-line"></div>`).join("")}
      </div>
    </div>`;
}

function routeContent() {
  switch (state.route) {
    case "dashboard":
      return renderDashboard();
    case "gradebook":
      return renderGradebook();
    case "quick":
      return renderQuickEntry();
    case "attendance":
      return renderAttendance();
    case "pipeline":
      return renderPipeline();
    case "insights":
      return renderInsights();
    case "excel":
      return renderExcel();
    case "setup":
      return renderSetup();
    case "settings":
      return renderSettings();
    default:
      return renderDashboard();
  }
}

function renderDashboard() {
  const courses = state.dashboard ?? [];
  const allSections = courses.flatMap((item) => item.sections);
  const students = allSections.reduce((sum, section) => sum + section.students, 0);
  const entered = allSections.reduce((sum, section) => sum + section.entered, 0);
  const possible = allSections.reduce((sum, section) => sum + section.possible, 0);
  const completion = percentage(entered, possible);
  const semester = activeSemester();
  return `
    <div class="page">
      <div class="page-header">
        <div>
          <div class="page-kicker">Semester pulse</div>
          <h2>${escapeHtml(semester ? `${semester.season} ${semester.session}` : "Academic overview")}</h2>
          <p>One reliable view of recorded marks, active rosters, and section readiness.</p>
        </div>
        <button class="button primary" data-route="quick">＋ Record marks</button>
      </div>
      <div class="summary-grid cards">
        <div class="stat-card"><i class="indicator" style="--indicator:var(--accent)"></i><span class="label">Courses</span><strong class="value">${courses.length}</strong><span class="meta">in this semester</span></div>
        <div class="stat-card"><i class="indicator" style="--indicator:var(--mint)"></i><span class="label">Active students</span><strong class="value">${students}</strong><span class="meta">across ${allSections.length} sections</span></div>
        <div class="stat-card"><i class="indicator" style="--indicator:var(--warning)"></i><span class="label">Recorded cells</span><strong class="value">${entered}</strong><span class="meta">of ${possible} possible entries</span></div>
        <div class="stat-card"><i class="indicator" style="--indicator:var(--success)"></i><span class="label">Completion</span><strong class="value">${formatNumber(completion, 0)}%</strong><span class="meta">current gradebook coverage</span></div>
      </div>
      <div class="section-grid">
        <section>
          <div class="panel-header"><h3>Course readiness</h3><span>${courses.length} courses</span></div>
          <div class="course-list">
            ${
              courses.length
                ? courses
                    .map((item) => {
                      const courseEntered = item.sections.reduce((sum, section) => sum + section.entered, 0);
                      const coursePossible = item.sections.reduce((sum, section) => sum + section.possible, 0);
                      const percent = percentage(courseEntered, coursePossible);
                      return `
                        <article class="course-card" style="--course-color:${item.course.color_hex}">
                          <div class="course-card-head">
                            <div><h3>${escapeHtml(item.course.code)} · ${escapeHtml(item.course.name)}</h3><p>${item.sections.length} sections · ${item.sections.reduce((sum, section) => sum + section.students, 0)} students</p></div>
                            ${donutChart(percent, item.course.color_hex)}
                          </div>
                          <div class="section-breakdown">
                            ${item.sections
                              .map(
                                (section) => `
                                <div class="section-row">
                                  <strong>Section ${escapeHtml(section.section.label)}</strong>
                                  <div class="progress-track"><span style="width:${section.completion_percent}%"></span></div>
                                  <span>${section.entered}/${section.possible}</span>
                                  <button class="button small ghost" data-open-section="${section.section.id}">Open</button>
                                </div>`
                              )
                              .join("")}
                          </div>
                        </article>`;
                    })
                    .join("")
                : emptyInline("No courses yet", "Use Setup to create your first course and section.")
            }
          </div>
        </section>
        <aside class="panel">
          <div class="panel-header"><h3>What Gradia is watching</h3><span>Live checks</span></div>
          <div class="activity-list">
            <div class="activity-item"><div class="activity-icon">✓</div><div><strong>Missing stays missing</strong><span>Blank marks never silently become zero during calculations.</span></div></div>
            <div class="activity-item"><div class="activity-icon">⌁</div><div><strong>Policy-driven grades</strong><span>Grade ranges come from the selected institute policy, not hard-coded AIUB rules.</span></div></div>
            <div class="activity-item"><div class="activity-icon">↗</div><div><strong>Safe Excel export</strong><span>Institution templates are matched by Student ID and never overwritten.</span></div></div>
            <div class="activity-item"><div class="activity-icon">◉</div><div><strong>Local workspace</strong><span>Normal operation remains entirely on this computer.</span></div></div>
          </div>
        </aside>
      </div>
    </div>`;
}

function emptyInline(title, description) {
  return `<div class="empty-state panel"><div><div class="empty-icon">＋</div><h3>${escapeHtml(title)}</h3><p>${escapeHtml(description)}</p></div></div>`;
}

function entriesMap() {
  return new Map(
    (state.gradebook?.entries ?? []).map((entry) => [entryKey(entry.enrollment_id, entry.field_id), entry])
  );
}

function computedValue(enrollmentId, fieldId) {
  const result = state.gradebook?.computed?.find((item) => item.enrollment_id === Number(enrollmentId));
  return result?.values?.[fieldId] ?? result?.values?.[String(fieldId)] ?? null;
}

function visibleFields() {
  return (state.gradebook?.fields ?? []).filter(
    (field) => !field.archived && (state.activeTerm === "all" || field.term === state.activeTerm)
  );
}

function renderGradebook() {
  if (!state.sectionId) return emptyInline("Select a section", "Create or select a course section before opening the gradebook.");
  const book = state.gradebook;
  if (!book) return emptyInline("Gradebook unavailable", "The selected section could not be loaded.");
  const fields = visibleFields();
  const map = entriesMap();
  const query = state.gradeFilter.toLowerCase();
  const enrollments = book.enrollments.filter(
    (student) =>
      !query ||
      student.name.toLowerCase().includes(query) ||
      student.student_identifier.toLowerCase().includes(query)
  );
  return `
    <div class="page">
      <div class="page-header">
        <div><div class="page-kicker">Flexible gradebook</div><h2>${escapeHtml(book.course.name)} · ${escapeHtml(book.section.label)}</h2><p>Enter raw marks directly. Gradia validates maximums, distinguishes missing from zero, and saves each cell immediately.</p></div>
        <button class="button primary" id="add-field">＋ Add assessment</button>
      </div>
      <div class="toolbar">
        <div class="tabs">
          ${[
            ["mid", "Midterm"],
            ["final", "Final"],
            ["semester", "Semester"],
            ["all", "All fields"]
          ]
            .map(([value, label]) => `<button class="tab ${state.activeTerm === value ? "active" : ""}" data-term="${value}">${label}</button>`)
            .join("")}
        </div>
        <div class="toolbar-group">
          <input class="search-input" id="grade-filter" value="${escapeHtml(state.gradeFilter)}" placeholder="Search name or Student ID…" />
          <button class="button" data-route="quick">Focused entry</button>
        </div>
      </div>
      ${
        fields.length
          ? `<div class="gradebook-wrap">
              <table class="gradebook-table">
                <thead><tr>
                  <th class="sticky id-col">Student ID</th>
                  <th class="sticky name-col">Student</th>
                  ${fields
                    .map(
                      (field) =>
                        `<th><button class="field-head field-edit-button" type="button" data-edit-field="${field.id}" aria-label="Edit ${escapeHtml(field.label)}"><span>${escapeHtml(field.label)}</span><small>${field.max_mark ? `out of ${formatNumber(field.max_mark)}` : field.field_type}${field.is_final ? " · final" : ""} · Edit</small></button></th>`
                    )
                    .join("")}
                </tr></thead>
                <tbody>
                  ${enrollments
                    .map(
                      (student) => `<tr>
                        <td class="sticky id-col">${escapeHtml(student.student_identifier)}</td>
                        <td class="sticky name-col"><div class="student-name">${escapeHtml(student.name)}<small>${escapeHtml(student.status)}</small></div></td>
                        ${fields
                           .map((field) => {
                             const entry = map.get(entryKey(student.enrollment_id, field.id));
                             if (field.field_type === "calculated" || field.field_type === "grade") {
                               const value = computedValue(student.enrollment_id, field.id);
                               return `<td style="text-align:center"><span class="state-pill ${value === null ? "muted" : ""}">${value === null ? "Missing inputs" : formatNumber(value)}</span></td>`;
                             }
                             const textual = ["text", "note"].includes(field.field_type);
                             return `<td style="text-align:center">
                               <input
                                 class="mark-input"
                                 type="${textual ? "text" : "number"}"
                                 ${textual ? "" : 'step="0.01" min="0"'}
                                 ${!textual && field.max_mark ? `max="${field.max_mark}"` : ""}
                                 value="${escapeHtml(entry?.state === "value" ? textual ? entry.text_value ?? "" : entry.numeric_value ?? "" : "")}"
                                 placeholder="—"
                                 data-grade-cell
                                 data-enrollment="${student.enrollment_id}"
                                 data-field="${field.id}"
                                 data-maximum="${textual ? "" : field.max_mark ?? ""}"
                                 data-value-kind="${textual ? "text" : "number"}"
                                 aria-label="${escapeHtml(field.label)} for ${escapeHtml(student.name)}"
                               />
                             </td>`;
                          })
                          .join("")}
                      </tr>`
                    )
                    .join("")}
                </tbody>
              </table>
            </div>`
          : emptyInline("No fields in this view", "Add an assessment field or select another term.")
      }
    </div>`;
}

function quickSelectedStudent() {
  return state.gradebook?.enrollments.find((student) => student.enrollment_id === Number(state.quickStudentId));
}

function renderQuickEntry() {
  if (!state.gradebook) return emptyInline("Select a section", "Quick entry needs an active section and roster.");
  const searchResults = rankStudents(state.gradebook.enrollments, state.quickQuery);
  const selected = quickSelectedStudent() ?? searchResults[0] ?? state.gradebook.enrollments[0] ?? null;
  if (selected && selected.enrollment_id !== Number(state.quickStudentId)) {
    state.quickStudentId = selected.enrollment_id;
  }
  const rosterWindow = studentWindowDetails(
    state.gradebook.enrollments,
    state.quickStudentId
  );
  const ranked = state.quickQuery.trim() ? searchResults : rosterWindow.students;
  const finderStatus = state.quickQuery.trim()
    ? `${searchResults.length} match${searchResults.length === 1 ? "" : "es"} · ${state.gradebook.enrollments.length} enrolled`
    : `Showing ${rosterWindow.start}–${rosterWindow.end} of ${rosterWindow.total}`;
  const map = entriesMap();
  const fields = state.gradebook.fields.filter(
    (field) => !field.archived && !["calculated", "grade"].includes(field.field_type)
  );
  return `
    <div class="page">
      <div class="page-header">
        <div><div class="page-kicker">Keyboard-first workflow</div><h2>Find, record, continue.</h2><p>Search any part of a Student ID or name, enter the selected student's marks, then move directly to the next incomplete record.</p></div>
      </div>
      <div class="quick-layout">
        <section class="panel student-finder">
          <div class="panel-header"><h3>Student finder</h3><span id="finder-status" aria-live="polite">${finderStatus}</span></div>
          <input class="search-input" style="width:100%" id="student-search" value="${escapeHtml(state.quickQuery)}" placeholder="Type ID or any part of a name…" />
          <div class="finder-results" aria-describedby="finder-status">
            ${ranked
              .map(
                (student) => `<button class="student-result ${student.enrollment_id === Number(state.quickStudentId) ? "active" : ""}" data-student="${student.enrollment_id}" ${student.enrollment_id === Number(state.quickStudentId) ? 'aria-current="true"' : ""}>
                  <span class="avatar">${initials(student.name)}</span>
                  <span><strong>${escapeHtml(student.name)}</strong><small>${escapeHtml(student.student_identifier)}</small></span>
                  <span>›</span>
                </button>`
              )
              .join("") || `<p style="color:var(--muted);font-size:11px">No matching student.</p>`}
          </div>
        </section>
        <section class="panel entry-form-panel">
          ${
            selected
              ? `<div class="selected-student"><span class="avatar">${initials(selected.name)}</span><div><h3>${escapeHtml(selected.name)}</h3><p>${escapeHtml(selected.student_identifier)} · Section ${escapeHtml(state.gradebook.section.label)}</p></div></div>
                 <form id="quick-entry-form">
                   <div class="field-form-grid">
                     ${fields
                       .map((field) => {
                         const entry = map.get(entryKey(selected.enrollment_id, field.id));
                         const textual = ["text", "note"].includes(field.field_type);
                         return `<div class="form-group">
                           <label>${escapeHtml(field.label)} <small>${textual ? field.field_type : field.max_mark ? `Max ${formatNumber(field.max_mark)}` : field.term}</small></label>
                           <input class="form-control" type="${textual ? "text" : "number"}" ${textual ? "" : `step="0.01" min="0" ${field.max_mark ? `max="${field.max_mark}"` : ""}`} name="field-${field.id}" value="${escapeHtml(entry?.state === "value" ? textual ? entry.text_value ?? "" : entry.numeric_value ?? "" : "")}" placeholder="Missing" />
                         </div>`;
                       })
                       .join("")}
                     <div class="form-group full"><label>Entry note</label><textarea class="form-control" name="entry-note" placeholder="Optional context, makeup, verification, or exception note…"></textarea></div>
                   </div>
                   <div class="form-actions">
                     <button type="button" class="button" id="next-student">Next student</button>
                     <button type="submit" class="button primary">Save & next unmarked</button>
                   </div>
                 </form>`
              : `<div class="empty-state"><div><div class="empty-icon">⌕</div><h3>No student selected</h3><p>Search the roster to begin focused entry.</p></div></div>`
          }
        </section>
      </div>
    </div>`;
}

function currentAttendanceRecords() {
  if (!state.attendance || !state.activeAttendanceId) return [];
  return (
    state.attendance.records[state.activeAttendanceId] ??
    state.attendance.records[String(state.activeAttendanceId)] ??
    []
  );
}

function renderAttendance() {
  const sessions = state.attendance?.sessions ?? [];
  const roster = state.gradebook?.enrollments ?? [];
  const active = sessions.find((session) => session.id === Number(state.activeAttendanceId));
  const records = new Map(currentAttendanceRecords().map((record) => [record.enrollment_id, record]));
  return `
    <div class="page">
      <div class="page-header">
        <div><div class="page-kicker">Exception-first attendance</div><h2>Mark everyone present. Change only exceptions.</h2><p>A new session starts with every active student present; tap A, L, E, or LE only where needed.</p></div>
        <button class="button primary" id="new-attendance">＋ New session</button>
      </div>
      <div class="attendance-layout">
        <aside class="panel">
          <div class="panel-header"><h3>Sessions</h3><span>${sessions.length}</span></div>
          <div class="session-list">
            ${sessions
              .map(
                (session) => `<button class="session-card ${session.id === Number(state.activeAttendanceId) ? "active" : ""}" data-session="${session.id}">
                  <strong>${escapeHtml(session.held_on)} · ${escapeHtml(session.title)}</strong>
                  <small>${session.present} present · ${session.absent} absent · ${session.late} late</small>
                </button>`
              )
              .join("") || `<p style="color:var(--muted);font-size:11px">No attendance sessions yet.</p>`}
          </div>
        </aside>
        <section class="panel">
          ${
            active
              ? `<div class="panel-header"><h3>${escapeHtml(active.held_on)} · ${escapeHtml(active.title)}</h3><div class="panel-header-actions"><span>Changes save immediately</span><button class="button compact" id="edit-attendance-session">Edit session</button></div></div>
                 <div class="attendance-summary">
                   <span class="status-chip present">${active.present} Present</span>
                   <span class="status-chip absent">${active.absent} Absent</span>
                   <span class="status-chip late">${active.late} Late</span>
                   <span class="status-chip excused">${active.excused} Excused</span>
                   <span class="status-chip left_early">${active.left_early} Left early</span>
                 </div>
                 <div class="attendance-roster">
                   ${roster
                     .map((student) => {
                       const status = records.get(student.enrollment_id)?.status ?? "present";
                       return `<div class="attendance-row">
                         <span class="avatar">${initials(student.name)}</span>
                         <div class="student-name">${escapeHtml(student.name)}<small>${escapeHtml(student.student_identifier)}</small></div>
                         <div class="status-options">
                           ${[
                             ["present", "P"],
                             ["absent", "A"],
                             ["late", "L"],
                             ["excused", "E"],
                             ["left_early", "LE"]
                           ]
                             .map(([value, label]) => `<button class="status-option ${status === value ? "active" : ""}" data-attendance-status="${value}" data-enrollment="${student.enrollment_id}">${label}</button>`)
                             .join("")}
                         </div>
                       </div>`;
                     })
                     .join("")}
                 </div>`
              : `<div class="empty-state"><div><div class="empty-icon">✓</div><h3>Create today's session</h3><p>All active students will begin as Present. You only record exceptions.</p><button class="button primary" id="new-attendance-empty" style="margin-top:16px">Create session</button></div></div>`
          }
        </section>
      </div>
    </div>`;
}

function renderPipeline() {
  if (!state.gradebook) return emptyInline("Select a section", "The completion pipeline is tracked per assessment and section.");
  const statuses = new Map((state.pipeline ?? []).map((status) => [status.field_id, status]));
  const fields = state.gradebook.fields.filter((field) => !field.archived);
  return `
    <div class="page">
      <div class="page-header">
        <div><div class="page-kicker">Administrative certainty</div><h2>Evaluation → recording → portal</h2><p>Each checkpoint is independent, timestamped, and attached to this assessment and section.</p></div>
      </div>
      <section class="panel">
        <div class="panel-header"><h3>${escapeHtml(state.gradebook.course.name)} · Section ${escapeHtml(state.gradebook.section.label)}</h3><span>${fields.length} assessments</span></div>
        <div class="attendance-roster">
          ${fields
            .map((field) => {
              const item = statuses.get(field.id) ?? {};
              return `<div class="attendance-row" style="grid-template-columns:minmax(0,1fr) auto">
                <div class="student-name">${escapeHtml(field.label)}<small>${escapeHtml(field.term)} · ${field.max_mark ? `out of ${formatNumber(field.max_mark)}` : field.field_type}</small></div>
                <div class="status-options">
                  ${[
                    ["evaluated", "Evaluated"],
                    ["marks_recorded", "Recorded"],
                    ["portal_uploaded", "Portal"]
                  ]
                    .map(
                      ([stage, label]) => `<button class="status-chip ${item[stage] ? "present" : ""}" data-pipeline-stage="${stage}" data-field="${field.id}" data-value="${item[stage] ? "true" : "false"}">${item[stage] ? "✓ " : "○ "}${label}</button>`
                    )
                    .join("")}
                </div>
              </div>`;
            })
            .join("") || `<p style="color:var(--muted)">Add assessment fields first.</p>`}
        </div>
      </section>
    </div>`;
}

function renderInsights() {
  const data = state.analytics;
  if (!data) return emptyInline("No analytics yet", "Select a section containing a final-result field.");
  const frequency = Object.entries(data.grade_frequency).map(([label, value]) => ({ label, value }));
  return `
    <div class="page">
      <div class="page-header">
        <div><div class="page-kicker">Academic intelligence</div><h2>Understand more than the total.</h2><p>Gradia turns finalized results into descriptive evidence. Associations are reported as observations, never as causal conclusions.</p></div>
        <button class="button" id="refresh-insights">↻ Refresh</button>
      </div>
      <div class="summary-grid cards" style="margin-bottom:16px">
        <div class="stat-card"><span class="label">Mean</span><strong class="value">${formatNumber(data.mean)}%</strong><span class="meta">section average</span></div>
        <div class="stat-card"><span class="label">Median</span><strong class="value">${formatNumber(data.median)}%</strong><span class="meta">middle finalized result</span></div>
        <div class="stat-card"><span class="label">Spread</span><strong class="value">${formatNumber(data.standard_deviation)}</strong><span class="meta">population standard deviation</span></div>
        <div class="stat-card"><span class="label">Ready</span><strong class="value">${data.completed}/${data.count}</strong><span class="meta">${data.missing} results missing</span></div>
      </div>
      <div class="analytics-grid">
        <section class="panel span-7"><div class="panel-header"><h3>Result distribution</h3><span>10-point bands</span></div>${distributionChart(data.results)}</section>
        <section class="panel span-5"><div class="panel-header"><h3>Grade frequency</h3><span>${data.pass_count} pass · ${data.fail_count} fail</span></div>${barChart(frequency, { color: "#62f0bd" })}</section>
        <section class="panel span-8"><div class="panel-header"><h3>Student result heatmap</h3><span>Policy-neutral percentage color</span></div>
          <div class="heatmap-grid">
            <div class="heatmap-cell name"><strong>Student</strong></div><div class="heatmap-cell">Result</div><div class="heatmap-cell">Grade</div><div class="heatmap-cell">Status</div><div class="heatmap-cell">Boundary</div><div class="heatmap-cell">Review</div>
            ${data.results
              .slice(0, 16)
              .map((result) => {
                const color = gradeHeatColor(result.final_percentage);
                const boundary = result.final_percentage === null ? "—" : Math.min(...[45, 50, 55, 60, 65, 70, 75, 80, 100].map((value) => Math.abs(value - result.final_percentage)));
                return `<div class="heatmap-cell name">${escapeHtml(result.name)}</div>
                  <div class="heatmap-cell" style="--cell-color:${color}">${formatNumber(result.final_percentage)}%</div>
                  <div class="heatmap-cell" style="--cell-color:${result.grade?.color_hex || color}">${escapeHtml(result.grade?.grade_label || "—")}</div>
                  <div class="heatmap-cell">${escapeHtml(result.grade?.result_label || "Missing")}</div>
                  <div class="heatmap-cell">${Number.isFinite(boundary) ? `${formatNumber(boundary)} pts` : "—"}</div>
                  <div class="heatmap-cell">${result.final_percentage === null ? "Missing" : boundary <= 2 ? "Near band" : "—"}</div>`;
              })
              .join("")}
          </div>
        </section>
        <aside class="panel span-4"><div class="panel-header"><h3>Observed insights</h3><span>Descriptive</span></div><div class="insight-list">
          ${insightSentences(data)
            .map((sentence) => `<div class="insight-item"><div class="insight-icon">⌁</div><div><strong>Pattern</strong><span>${escapeHtml(sentence)}</span></div></div>`)
            .join("")}
          <div class="insight-item"><div class="insight-icon">!</div><div><strong>Interpret carefully</strong><span>Statistical association does not establish why a pattern occurred.</span></div></div>
        </div></aside>
      </div>
    </div>`;
}

function renderExcel() {
  const excel = state.excel;
  const preview = excel.preview;
  return `
    <div class="page">
      <div class="page-header">
        <div><div class="page-kicker">Exact institutional output</div><h2>Fill the official template, not a recreation.</h2><p>Gradia validates Student ID and Mark columns, previews every match, writes only approved numeric cells, and verifies that non-target workbook parts remain unchanged.</p></div>
      </div>
      <div class="excel-flow">
        <div class="flow-step"><span class="flow-number">1</span><h3>Select context</h3><p>${escapeHtml(activeCourse()?.name || "Choose a course")} · Section ${escapeHtml(activeSection()?.label || "—")} · confirmed term.</p></div>
        <div class="flow-step"><span class="flow-number">2</span><h3>Inspect and match</h3><p>A1 must be Student ID. Gradia matches exact normalized IDs and reports duplicates, missing marks, and unmatched rows.</p></div>
        <div class="flow-step"><span class="flow-number">3</span><h3>Export safely</h3><p>The source is never overwritten. The output is reopened and package fidelity is checked before success is reported.</p></div>
      </div>
      ${
        !excel.preflight
          ? `<div class="file-drop"><div><strong>Choose an institution-provided .xlsx template</strong><span>The original file remains untouched. You will review the selected sheet and every proposed change before export.</span><button class="button primary" id="choose-template" style="margin-top:17px">Choose template</button></div></div>`
          : `<section class="panel" style="margin-top:16px">
              <div class="panel-header"><h3>${escapeHtml(excel.path.split(/[\\/]/).at(-1))}</h3><button class="button small" id="change-template">Change</button></div>
              <div class="inline-fields">
                <div class="form-group"><label>Worksheet</label><select class="form-control" id="excel-sheet">${excel.preflight.sheets.map((sheet) => `<option value="${escapeHtml(sheet.name)}" ${sheet.name === excel.sheet ? "selected" : ""}>${escapeHtml(sheet.name)}</option>`).join("")}</select></div>
                <div class="form-group"><label>Confirmed term</label><select class="form-control" id="excel-term"><option value="mid" ${excel.term === "mid" ? "selected" : ""}>Midterm</option><option value="final" ${excel.term === "final" ? "selected" : ""}>Final term</option><option value="semester" ${excel.term === "semester" ? "selected" : ""}>Semester result</option></select></div>
              </div>
              <div style="display:flex;justify-content:flex-end;margin-top:13px"><button class="button primary" id="preview-export">Analyze matches</button></div>
              ${
                preview
                  ? `<div class="preview-grid">
                      <div class="preview-stat"><strong>${preview.template_rows}</strong><span>Template rows</span></div>
                      <div class="preview-stat"><strong>${preview.matched}</strong><span>Matched IDs</span></div>
                      <div class="preview-stat"><strong>${preview.changed}</strong><span>Marks to write</span></div>
                      <div class="preview-stat"><strong>${preview.template_ids_not_found.length}</strong><span>Template IDs missing</span></div>
                      <div class="preview-stat"><strong>${preview.gradia_students_not_in_template.length}</strong><span>Gradia IDs ignored</span></div>
                      <div class="preview-stat"><strong>${preview.missing_final_marks.length}</strong><span>Final marks missing</span></div>
                    </div>
                    <div class="filename-preview">${escapeHtml(preview.output_filename)}</div>
                    ${
                      preview.duplicate_template_ids.length
                        ? `<p style="color:var(--danger);font-size:11px">Export blocked: duplicate template IDs ${escapeHtml(preview.duplicate_template_ids.join(", "))}</p>`
                        : `<div style="display:flex;justify-content:flex-end;margin-top:13px"><button class="button mint" id="export-workbook" ${preview.changed ? "" : "disabled"}>Export verified workbook</button></div>`
                    }`
                  : ""
              }
            </section>`
      }
    </div>`;
}

function renderSetup() {
  const policies = state.bootstrap.policies;
  const semester = activeSemester();
  const course = activeCourse();
  const section = activeSection();
  return `
    <div class="page">
      <div class="page-header">
        <div><div class="page-kicker">Workspace builder</div><h2>Shape Gradia around the way you teach.</h2><p>Semesters, courses, sections, rosters, assessment fields, and policies remain configurable rather than institution-specific.</p></div>
      </div>
      <section class="setup-context-guide" aria-label="Current setup destination">
        <div>
          <span class="context-guide-kicker">Before you add anything</span>
          <strong>Select its destination in Academic context</strong>
          <p>The left-panel selectors are active: a course is added to the selected semester, a section to the selected course, and a student to the selected section.</p>
        </div>
        <div class="context-guide-path">
          <span>${escapeHtml(semester ? `${semester.season} ${semester.session}` : "Select semester")}</span>
          <b>›</b>
          <span>${escapeHtml(course ? `${course.code} · ${course.name}` : "Select course")}</span>
          <b>›</b>
          <span>${escapeHtml(section ? `Section ${section.label}` : "Select section")}</span>
        </div>
      </section>
      <div class="setup-grid">
        <section class="setup-card"><h3>New semester</h3><p>Store season and session separately for reliable institutional filenames.</p>
          <form id="semester-form"><div class="inline-fields"><div class="form-group"><label>Season</label><input class="form-control" name="season" placeholder="Fall" required /></div><div class="form-group"><label>Session</label><input class="form-control" name="session" placeholder="2025-2026" required /></div></div><button class="button primary" type="submit">Create & activate</button></form>
        </section>
        <section class="setup-card"><h3>New course</h3><p>Official export name controls the institution-ready Excel filename.</p>
          <form id="course-form"><div class="inline-fields"><div class="form-group"><label>Code</label><input class="form-control" name="code" placeholder="CSC2211" required /></div><div class="form-group"><label>Accent</label><input class="form-control" name="color" type="color" value="#8b5cf6" /></div></div><div class="form-group"><label>Course name</label><input class="form-control" name="name" placeholder="Data Structures Lab" required /></div><div class="form-group"><label>Official export name</label><input class="form-control" name="export_name" placeholder="DATA STRUCTURE LAB" /></div><button class="button primary" type="submit" ${state.semesterId ? "" : "disabled"}>Create course</button></form>
        </section>
        <section class="setup-card"><h3>New section</h3><p>Sections ending in digits are automatically bracketed in export filenames.</p>
          <form id="section-form"><div class="form-group"><label>Section label</label><input class="form-control" name="label" placeholder="G or B7" required /></div><button class="button primary" type="submit" ${state.courseId ? "" : "disabled"}>Add section</button></form>
        </section>
        <section class="setup-card"><h3>Add student</h3><p>Student ID is authoritative and remains text. Existing students can be enrolled in another section.</p>
          <form id="student-form"><div class="form-group"><label>Student ID</label><input class="form-control" name="student_id" placeholder="26-10001-1" required /></div><div class="form-group"><label>Name</label><input class="form-control" name="name" placeholder="Student full name" required /></div><div class="form-group"><label>Email <small>optional</small></label><input class="form-control" name="email" type="email" /></div><button class="button primary" type="submit" ${state.sectionId ? "" : "disabled"}>Enroll student</button></form>
          <div class="setup-divider"><span>or</span></div>
          <div class="setup-button-row"><button class="button" id="import-roster" ${state.sectionId ? "" : "disabled"}>↗ Import roster from Excel</button><button class="button" id="edit-student" ${state.gradebook?.enrollments?.length ? "" : "disabled"}>Edit enrolled student</button></div>
        </section>
        <section class="setup-card"><h3>Grading policies</h3><p>Build versioned grade ranges for any institute; no AIUB thresholds are hard-coded.</p>
          <div class="policy-bands">${policies.map((policy) => `<button class="policy-band policy-button" data-policy="${policy.id}"><span class="band-color" style="--band-color:${policy.is_default ? "#62f0bd" : "#8b5cf6"}"></span><strong>v${policy.version}</strong><span>${escapeHtml(policy.name)}</span><span class="tag">${policy.is_default ? "Default" : "Edit"}</span></button>`).join("")}</div>
          <button class="button" id="new-policy" style="margin-top:14px">＋ New policy</button>
        </section>
        <section class="setup-card"><h3>Assessment & calculations</h3><p>Add raw fields or compose totals, averages, best-N, dropped-lowest, scaling, multiplication, and weighted results visually.</p>
          <div class="setup-button-row"><button class="button primary" id="add-field-card" ${state.courseId ? "" : "disabled"}>＋ Add assessment</button><button class="button" id="manage-gradebook-views" ${state.courseId ? "" : "disabled"}>Manage views</button><button class="button" data-route="gradebook">Open gradebook</button></div>
        </section>
        <section class="setup-card manage-context-card"><h3>Edit or permanently delete</h3><p>Edit the currently selected academic context. Permanent deletion shows the exact downstream impact before it can proceed.</p>
          <div class="manage-context-row"><span><strong>Semester</strong><small>${escapeHtml(semester ? `${semester.season} ${semester.session}` : "None selected")}</small></span><div><button class="button compact" id="edit-semester" ${semester ? "" : "disabled"}>Edit</button><button class="button compact danger" id="delete-semester" ${semester ? "" : "disabled"}>Delete</button></div></div>
          <div class="manage-context-row"><span><strong>Course</strong><small>${escapeHtml(course ? `${course.code} · ${course.name}` : "None selected")}</small></span><div><button class="button compact" id="edit-course" ${course ? "" : "disabled"}>Edit</button><button class="button compact danger" id="delete-course" ${course ? "" : "disabled"}>Delete</button></div></div>
          <div class="manage-context-row"><span><strong>Section</strong><small>${escapeHtml(section ? `Section ${section.label}` : "None selected")}</small></span><div><button class="button compact" id="edit-section" ${section ? "" : "disabled"}>Edit</button><button class="button compact danger" id="delete-section" ${section ? "" : "disabled"}>Delete</button></div></div>
        </section>
      </div>
    </div>`;
}

function renderSettings() {
  return `
    <div class="page">
      <div class="page-header"><div><div class="page-kicker">Gradia system</div><h2>Private, portable, recoverable.</h2><p>Move the complete database between devices using one validated Gradia transfer file.</p></div></div>
      <div class="setup-grid">
        <section class="setup-card"><h3>Export database</h3><p>Create one portable <code>.gradia</code> file containing all semesters, courses, rosters, marks, attendance, policies, and settings.</p><p class="transfer-caution">Integrity protected, but not password encrypted—store it securely.</p><button class="button primary" id="export-database" style="margin-top:16px">Export database</button></section>
        <section class="setup-card"><h3>Import database</h3><p>On another device, select the exported <code>.gradia</code> file. Gradia verifies its format, SHA-256 checksum, SQLite integrity, and migrations before replacing local data.</p><p class="transfer-caution">Export this device first if its current data must be retained.</p><button class="button danger" id="import-database" style="margin-top:16px">Import database</button></section>
        <section class="setup-card"><h3>Runtime posture</h3><p>Normal operation uses the local SQLite database and bundled application assets. Telemetry and update checks are disabled.</p><div class="policy-bands"><div class="policy-band"><span class="band-color" style="--band-color:#62f0bd"></span><strong>Local</strong><span>Database</span><span class="tag">gradia.db</span></div><div class="policy-band"><span class="band-color" style="--band-color:#62f0bd"></span><strong>Zero</strong><span>Runtime network calls</span><span class="tag">Required</span></div></div></section>
        <section class="setup-card"><h3>Welcome & semester setup</h3><p>Reopen the guided welcome screen at any time. Existing semesters, courses, rosters, marks, and attendance remain untouched.</p><button class="button primary" id="open-welcome" style="margin-top:16px">Open welcome & setup</button></section>
        <section class="setup-card"><h3>About</h3><p><strong>Gradia</strong><br />Smarter academic assessment.<br /><br />Version ${escapeHtml(APP_VERSION)} · Tauri desktop architecture.</p></section>
      </div>
    </div>`;
}

function renderOnboarding() {
  const returning = state.bootstrap.semesters.length > 0;
  return `
    <main class="content" style="height:100%;overflow:auto">
      <div class="onboarding">
        <section class="onboarding-hero">
          <img src="${gradiaLogoUrl}" alt="" />
          <h1>Meet Gradia.</h1>
          <p>A private academic workspace for flexible gradebooks, fast marks entry, attendance, institute-independent calculations, meaningful insights, and exact institutional Excel output.</p>
          <div class="feature-points">
            <span class="feature-point">Institute-independent policies</span><span class="feature-point">Missing never becomes zero</span><span class="feature-point">Student-ID-safe exports</span><span class="feature-point">Entirely local by default</span>
          </div>
        </section>
        <section class="panel onboarding-form">
          ${returning ? `<button class="button welcome-back" type="button" id="close-welcome">← Back to workspace</button>` : ""}
          <div class="page-kicker">${returning ? "Semester setup" : "First workspace"}</div><h2 style="margin:0">${returning ? "Create another semester" : "Create your active semester"}</h2><p style="color:var(--muted);font-size:11px;line-height:1.5">Gradia stores the season and academic session separately so calculations and filenames remain reliable. Existing academic data is never removed by this guide.</p>
          <form id="onboarding-form" style="display:grid;gap:13px;margin-top:22px">
            <div class="form-group"><label>Season</label><select class="form-control" name="season"><option>Fall</option><option>Spring</option><option>Summer</option></select></div>
            <div class="form-group"><label>Academic session</label><input class="form-control" name="session" placeholder="2025-2026" required /></div>
            <button class="button primary" type="submit">${returning ? "Create & activate semester" : "Create Gradia workspace"}</button>
          </form>
        </section>
      </div>
    </main>`;
}

function render() {
  if (!state.bootstrap) return;
  app.innerHTML =
    !state.bootstrap.semesters.length || state.showWelcome ? renderOnboarding() : renderShell();
}

function modal(content) {
  const backdrop = document.createElement("div");
  backdrop.className = "modal-backdrop";
  backdrop.innerHTML = `<div class="modal">${content}</div>`;
  backdrop.addEventListener("click", (event) => {
    if (event.target === backdrop || event.target.closest("[data-close-modal]")) backdrop.remove();
  });
  document.body.append(backdrop);
  return backdrop;
}

function guideCardMarkup(guide, eyebrow) {
  return `
    <span class="guide-eyebrow">${escapeHtml(eyebrow)}</span>
    <strong>${escapeHtml(guide.label)}</strong>
    <p>${escapeHtml(guide.behavior)}</p>
    <small><span>Example</span>${escapeHtml(guide.example)}</small>`;
}

async function refreshAfterStructureChange({ preserve = true } = {}) {
  await refreshBootstrap({ preserve });
  state.dashboard = null;
  state.gradebook = null;
  state.analytics = null;
  state.attendance = null;
  state.pipeline = null;
  state.quickStudentId = null;
  await loadRoute();
}

function openContextEditor(kind) {
  const record =
    kind === "semester" ? activeSemester() : kind === "course" ? activeCourse() : activeSection();
  if (!record) return toast(`Select a ${kind} first.`, "error");
  const policyOptions = state.bootstrap.policies
    .map(
      (policy) =>
        `<option value="${policy.id}" ${policy.id === Number(record.grading_policy_id) ? "selected" : ""}>${escapeHtml(policy.name)}</option>`
    )
    .join("");
  const fields =
    kind === "semester"
      ? `<div class="inline-fields"><div class="form-group"><label>Season</label><input class="form-control" name="season" value="${escapeHtml(record.season)}" required /></div><div class="form-group"><label>Academic session</label><input class="form-control" name="session" value="${escapeHtml(record.session)}" required /></div></div>`
      : kind === "course"
        ? `<div class="inline-fields"><div class="form-group"><label>Course code</label><input class="form-control" name="code" value="${escapeHtml(record.code)}" required /></div><div class="form-group"><label>Accent color</label><input class="form-control" name="color" type="color" value="${escapeHtml(record.color_hex)}" /></div></div><div class="form-group"><label>Course name</label><input class="form-control" name="name" value="${escapeHtml(record.name)}" required /></div><div class="form-group"><label>Official export name</label><input class="form-control" name="export_name" value="${escapeHtml(record.export_name)}" required /></div><div class="form-group"><label>Grading policy</label><select class="form-control" name="policy">${policyOptions}</select></div>`
        : `<div class="form-group"><label>Section label</label><input class="form-control" name="label" value="${escapeHtml(record.label)}" required /></div>`;
  const box = modal(`
    <h3>Edit ${escapeHtml(kind)}</h3>
    <p>Changes apply everywhere this record is used and are written to Gradia’s audit history.</p>
    <form id="context-edit-form">${fields}
      <div class="modal-actions"><button type="button" class="button" data-close-modal>Cancel</button><button type="submit" class="button primary">Save changes</button></div>
    </form>`);
  box.querySelector("#context-edit-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    try {
      if (kind === "semester") {
        await call("update_semester", {
          id: record.id,
          season: data.get("season"),
          session: data.get("session")
        });
      } else if (kind === "course") {
        await call("update_course", {
          id: record.id,
          code: data.get("code"),
          name: data.get("name"),
          exportName: data.get("export_name"),
          colorHex: data.get("color"),
          gradingPolicyId: data.get("policy") ? Number(data.get("policy")) : null
        });
      } else {
        await call("update_section", { id: record.id, label: data.get("label") });
      }
      box.remove();
      toast(`${kind[0].toUpperCase()}${kind.slice(1)} updated.`, "success");
      await refreshAfterStructureChange();
    } catch (error) {
      toast(errorMessage(error), "error");
    }
  });
}

function openStudentEditor() {
  const students = state.gradebook?.enrollments ?? [];
  if (!students.length) return toast("This section has no enrolled students.", "error");
  const initial =
    students.find((student) => student.enrollment_id === Number(state.quickStudentId)) ?? students[0];
  const box = modal(`
    <h3>Edit enrolled student</h3>
    <p>Identity changes update the shared student record; status and roster position apply to this section.</p>
    <form id="student-edit-form">
      <div class="form-group"><label>Student</label><select class="form-control" name="enrollment">${students.map((student) => `<option value="${student.enrollment_id}" ${student.enrollment_id === initial.enrollment_id ? "selected" : ""}>${escapeHtml(student.student_identifier)} · ${escapeHtml(student.name)}</option>`).join("")}</select></div>
      <div class="inline-fields"><div class="form-group"><label>Student ID</label><input class="form-control" name="student_id" required /></div><div class="form-group"><label>Name</label><input class="form-control" name="name" required /></div></div>
      <div class="inline-fields"><div class="form-group"><label>Email</label><input class="form-control" name="email" type="email" /></div><div class="form-group"><label>Roster position</label><input class="form-control" name="position" type="number" min="1" max="${students.length}" step="1" required /></div></div>
      <div class="form-group"><label>Enrollment status</label><select class="form-control" name="status"><option value="active">Active</option><option value="withdrawn">Withdrawn</option><option value="incomplete">Incomplete</option><option value="archived">Archived</option></select></div>
      <div class="modal-actions"><button type="button" class="button" data-close-modal>Cancel</button><button type="submit" class="button primary">Save student</button></div>
    </form>`);
  const form = box.querySelector("#student-edit-form");
  const sync = () => {
    const student = students.find(
      (item) => item.enrollment_id === Number(form.elements.enrollment.value)
    );
    if (!student) return;
    form.elements.student_id.value = student.student_identifier;
    form.elements.name.value = student.name;
    form.elements.email.value = student.email ?? "";
    form.elements.position.value = student.roll_order + 1;
    form.elements.status.value = student.status;
  };
  form.elements.enrollment.addEventListener("change", sync);
  sync();
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const data = new FormData(form);
    try {
      await call("update_student", {
        enrollmentId: Number(data.get("enrollment")),
        studentIdentifier: data.get("student_id"),
        name: data.get("name"),
        email: data.get("email") || null,
        status: data.get("status"),
        rollOrder: Number(data.get("position")) - 1
      });
      box.remove();
      toast("Student and enrollment updated.", "success");
      await refreshAfterStructureChange();
    } catch (error) {
      toast(errorMessage(error), "error");
    }
  });
}

function openGradebookViewEditor() {
  const views = state.gradebook?.views ?? [];
  const box = modal(`
    <h3>Manage gradebook views</h3>
    <p>Create a new grouping or edit the name and term of an existing one. Fields assigned to an edited view remain attached.</p>
    <form id="view-edit-form">
      <div class="form-group"><label>View to edit</label><select class="form-control" name="view_id"><option value="">＋ Create new view</option>${views.map((view) => `<option value="${view.id}">${escapeHtml(view.name)}</option>`).join("")}</select></div>
      <div class="inline-fields"><div class="form-group"><label>View name</label><input class="form-control" name="name" placeholder="Practical work" required /></div><div class="form-group"><label>Term</label><select class="form-control" name="term"><option value="mid">Midterm</option><option value="final">Final</option><option value="semester">Semester</option><option value="custom">Custom</option></select></div></div>
      <div class="modal-actions"><button type="button" class="button" data-close-modal>Cancel</button><button type="submit" class="button primary">Save view</button></div>
    </form>`);
  const form = box.querySelector("#view-edit-form");
  const sync = () => {
    const view = views.find((item) => item.id === Number(form.elements.view_id.value));
    form.elements.name.value = view?.name ?? "";
    form.elements.term.value = view?.term ?? "custom";
  };
  form.elements.view_id.addEventListener("change", sync);
  sync();
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const data = new FormData(form);
    try {
      if (data.get("view_id")) {
        await call("update_gradebook_view", {
          id: Number(data.get("view_id")),
          name: data.get("name"),
          term: data.get("term")
        });
      } else {
        await call("create_gradebook_view", {
          courseId: Number(state.courseId),
          name: data.get("name"),
          term: data.get("term")
        });
      }
      box.remove();
      toast("Gradebook view saved.", "success");
      await refreshAfterStructureChange();
    } catch (error) {
      toast(errorMessage(error), "error");
    }
  });
}

async function openDeleteModal(kind) {
  const record =
    kind === "semester" ? activeSemester() : kind === "course" ? activeCourse() : activeSection();
  if (!record) return toast(`Select a ${kind} first.`, "error");
  try {
    const impact = await call("get_delete_impact", {
      entityType: kind,
      entityId: record.id
    });
    const box = modal(`
      <h3>Permanently delete ${escapeHtml(impact.label)}?</h3>
      <p>This cannot be undone from inside Gradia. Export the database first if you may need these records again.</p>
      <div class="delete-impact-grid">
        <span><strong>${impact.courses}</strong> courses</span><span><strong>${impact.sections}</strong> sections</span><span><strong>${impact.enrollments}</strong> enrollments</span><span><strong>${impact.assessment_fields}</strong> assessment fields</span><span><strong>${impact.grade_entries}</strong> grade entries</span><span><strong>${impact.attendance_sessions}</strong> attendance sessions</span><span><strong>${impact.result_snapshots}</strong> result snapshots</span>
      </div>
      <div class="form-group"><label>Type <code>${escapeHtml(impact.confirmation)}</code> to confirm</label><input class="form-control" id="delete-confirmation" autocomplete="off" /></div>
      <div class="modal-actions"><button type="button" class="button" data-close-modal>Cancel</button><button type="button" class="button danger" id="confirm-delete" disabled>Permanently delete</button></div>`);
    const input = box.querySelector("#delete-confirmation");
    const confirm = box.querySelector("#confirm-delete");
    input.addEventListener("input", () => {
      confirm.disabled = input.value !== impact.confirmation;
    });
    confirm.addEventListener("click", async () => {
      try {
        await call("delete_academic_entity", {
          entityType: kind,
          entityId: record.id,
          confirmation: input.value
        });
        box.remove();
        toast(`${impact.label} permanently deleted.`, "success");
        await refreshAfterStructureChange({ preserve: false });
      } catch (error) {
        toast(errorMessage(error), "error");
      }
    });
  } catch (error) {
    toast(errorMessage(error), "error");
  }
}

function flattenSubtractionRule(rule) {
  if (rule?.op !== "subtract") return [rule];
  return [...flattenSubtractionRule(rule.left), rule.right];
}

function calculationEditorState(field) {
  const fallback = {
    operation: "sum",
    sourceIds: [],
    count: 1,
    factor: 2,
    from: 100,
    to: 40,
    weights: ""
  };
  if (!field?.rule_json) return fallback;
  try {
    const rule = JSON.parse(field.rule_json);
    if (["sum", "average", "maximum", "best_n", "drop_lowest"].includes(rule.op)) {
      return {
        ...fallback,
        operation: rule.op,
        sourceIds: (rule.inputs ?? []).map((input) => input.field_id),
        count: rule.count ?? 1
      };
    }
    if (["multiply", "scale"].includes(rule.op)) {
      return {
        ...fallback,
        operation: rule.op,
        sourceIds: rule.input?.field_id ? [rule.input.field_id] : [],
        factor: rule.factor ?? 2,
        from: rule.from ?? 100,
        to: rule.to ?? 40
      };
    }
    if (rule.op === "weighted_sum") {
      return {
        ...fallback,
        operation: rule.op,
        sourceIds: (rule.items ?? []).map((item) => item.input?.field_id).filter(Boolean),
        weights: (rule.items ?? []).map((item) => item.weight).join(", ")
      };
    }
    if (rule.op === "subtract") {
      return {
        ...fallback,
        operation: "subtract",
        sourceIds: flattenSubtractionRule(rule)
          .map((input) => input?.field_id)
          .filter(Boolean)
      };
    }
  } catch {
    return fallback;
  }
  return fallback;
}

function openFieldModal(fieldId = null) {
  if (!state.courseId) return toast("Select a course first.", "error");
  const field = (state.gradebook?.fields ?? []).find((item) => item.id === Number(fieldId));
  const calculation = calculationEditorState(field);
  const views = state.gradebook?.views ?? [];
  const sourceFields = (state.gradebook?.fields ?? []).filter(
    (item) => !item.archived && item.id !== field?.id
  );
  const initialTypeGuide = assessmentTypeGuide(field?.field_type ?? "score");
  const selectedView = views.find((view) => view.id === Number(field?.view_id));
  const initialViewGuide = gradebookViewGuide(selectedView?.name, selectedView?.term);
  const initialOperationGuide = calculationOperationGuide(calculation.operation);
  const box = modal(`
    <h3>${field ? "Edit assessment field" : "Add assessment field"}</h3><p>${field ? "Update this field without deleting its existing student entries." : "Create a validated raw column or a reusable calculation."} Missing source marks remain missing.</p>
    <form id="field-form">
      <div class="inline-fields"><div class="form-group"><label>Label</label><input class="form-control" name="label" value="${escapeHtml(field?.label ?? "")}" placeholder="OBE Assessment" required /></div><div class="form-group"><label>Stable key</label><input class="form-control" name="key" value="${escapeHtml(field?.stable_key ?? "")}" placeholder="obe_assessment" required /></div></div>
      <div class="inline-fields"><div class="form-group"><label>Term</label><select class="form-control" name="term"><option value="mid" ${field?.term === "mid" ? "selected" : ""}>Midterm</option><option value="final" ${field?.term === "final" ? "selected" : ""}>Final</option><option value="semester" ${field?.term === "semester" ? "selected" : ""}>Semester</option><option value="custom" ${field?.term === "custom" ? "selected" : ""}>Custom</option></select></div><div class="form-group"><label>Type</label><select class="form-control" name="type"><option value="score" ${!field || field.field_type === "score" ? "selected" : ""}>Score — entered mark</option><option value="calculated" ${field?.field_type === "calculated" ? "selected" : ""}>Calculated — formula result</option><option value="attendance" ${field?.field_type === "attendance" ? "selected" : ""}>Attendance — converted mark</option><option value="bonus" ${field?.field_type === "bonus" ? "selected" : ""}>Bonus — extra credit</option><option value="penalty" ${field?.field_type === "penalty" ? "selected" : ""}>Penalty — deduction amount</option><option value="text" ${field?.field_type === "text" ? "selected" : ""}>Text — short written value</option><option value="note" ${field?.field_type === "note" ? "selected" : ""}>Note — student context</option></select></div></div>
      <div class="guide-card" id="assessment-type-guide" aria-live="polite">${guideCardMarkup(initialTypeGuide, "What this type does")}</div>
      <div class="inline-fields"><div class="form-group"><label>Maximum mark</label><input class="form-control" name="maximum" type="number" min="0.01" step="0.01" value="${field?.max_mark ?? ""}" /></div><div class="form-group"><label>Contribution</label><input class="form-control" name="contribution" type="number" min="0" step="0.01" value="${field?.contribution ?? ""}" /></div></div>
      <div class="form-group"><label>Gradebook view</label><select class="form-control" name="view"><option value="">No specific view</option>${views.map((view) => `<option value="${view.id}" ${view.id === Number(field?.view_id) ? "selected" : ""}>${escapeHtml(view.name)}</option>`).join("")}</select></div>
      <div class="guide-card secondary" id="gradebook-view-guide" aria-live="polite">${guideCardMarkup(initialViewGuide, "What this view means")}</div>
      <div class="calculation-builder" id="calculation-builder" hidden>
        <div class="builder-heading"><strong>Calculation recipe</strong><span>Uses existing fields</span></div>
        <div class="form-group"><label>Operation</label><select class="form-control" name="operation">
          <option value="sum" ${calculation.operation === "sum" ? "selected" : ""}>Sum selected fields</option><option value="average" ${calculation.operation === "average" ? "selected" : ""}>Average selected fields</option>
          <option value="maximum" ${calculation.operation === "maximum" ? "selected" : ""}>Best single field</option><option value="best_n" ${calculation.operation === "best_n" ? "selected" : ""}>Best N (sum)</option>
          <option value="drop_lowest" ${calculation.operation === "drop_lowest" ? "selected" : ""}>Drop lowest (sum)</option><option value="multiply" ${calculation.operation === "multiply" ? "selected" : ""}>Multiply one field</option>
          <option value="scale" ${calculation.operation === "scale" ? "selected" : ""}>Convert mark from one maximum to another</option><option value="weighted_sum" ${calculation.operation === "weighted_sum" ? "selected" : ""}>Weighted combination</option>
          <option value="subtract" ${calculation.operation === "subtract" ? "selected" : ""}>Subtract later fields from the first</option>
        </select></div>
        <div class="guide-card operation" id="calculation-operation-guide" aria-live="polite">${guideCardMarkup(initialOperationGuide, "What this operation does")}</div>
        <div class="source-field-list">
          ${
            sourceFields.length
              ? sourceFields.map((source) => `<label class="source-field"><input type="checkbox" name="source_field" value="${source.id}" ${calculation.sourceIds.includes(source.id) ? "checked" : ""} /><span><strong>${escapeHtml(source.label)}</strong><small>${source.max_mark ? `out of ${formatNumber(source.max_mark)}` : source.term}</small></span></label>`).join("")
              : `<p class="builder-empty">Add at least one raw assessment before creating a calculation.</p>`
          }
        </div>
        <div class="inline-fields calculation-parameters">
          <div class="form-group" data-param="count"><label>N / number to drop</label><input class="form-control" name="count" type="number" min="1" step="1" value="${calculation.count}" /></div>
          <div class="form-group" data-param="factor"><label>Multiplier</label><input class="form-control" name="factor" type="number" min="0" step="0.01" value="${calculation.factor}" /></div>
          <div class="form-group" data-param="from"><label>Convert from</label><input class="form-control" name="from" type="number" min="0.01" step="0.01" value="${calculation.from}" /></div>
          <div class="form-group" data-param="to"><label>Convert to</label><input class="form-control" name="to" type="number" min="0.01" step="0.01" value="${calculation.to}" /></div>
          <div class="form-group full" data-param="weights"><label>Weights in selected-field order</label><input class="form-control" name="weights" value="${escapeHtml(calculation.weights)}" placeholder="0.4, 0.6" /><small class="field-help">Use decimals such as 0.4 and 0.6, or percentages such as 40 and 60.</small></div>
        </div>
      </div>
      <label style="display:flex;gap:8px;align-items:center;font-size:11px"><input type="checkbox" name="is_final" ${field?.is_final ? "checked" : ""} /> This is the final result for its term</label>
      ${field ? `<label style="display:flex;gap:8px;align-items:center;font-size:11px"><input type="checkbox" name="archived" ${field.archived ? "checked" : ""} /> Archive this field from normal gradebook views</label>` : ""}
      <div class="modal-actions"><button type="button" class="button" data-close-modal>Cancel</button><button type="submit" class="button primary">${field ? "Save field" : "Add field"}</button></div>
    </form>`);
  const fieldForm = box.querySelector("#field-form");
  const fieldType = fieldForm.elements.type;
  const viewSelect = fieldForm.elements.view;
  const operation = fieldForm.elements.operation;
  const builder = box.querySelector("#calculation-builder");
  const syncBuilder = () => {
    const calculated = fieldType.value === "calculated";
    const textual = ["text", "note"].includes(fieldType.value);
    builder.hidden = !calculated;
    fieldForm.elements.maximum.disabled = textual;
    fieldForm.elements.contribution.disabled = textual;
    box.querySelector("#assessment-type-guide").innerHTML = guideCardMarkup(
      assessmentTypeGuide(fieldType.value),
      "What this type does"
    );
    const op = operation.value;
    box.querySelector("#calculation-operation-guide").innerHTML = guideCardMarkup(
      calculationOperationGuide(op),
      "What this operation does"
    );
    box.querySelectorAll("[data-param]").forEach((element) => {
      const parameter = element.dataset.param;
      element.hidden =
        !calculated ||
        !(
          (parameter === "count" && ["best_n", "drop_lowest"].includes(op)) ||
          (parameter === "factor" && op === "multiply") ||
          (["from", "to"].includes(parameter) && op === "scale") ||
          (parameter === "weights" && op === "weighted_sum")
        );
    });
  };
  const syncViewGuide = () => {
    const selectedView = views.find((view) => view.id === Number(viewSelect.value));
    box.querySelector("#gradebook-view-guide").innerHTML = guideCardMarkup(
      gradebookViewGuide(selectedView?.name, selectedView?.term),
      "What this view means"
    );
  };
  fieldType.addEventListener("change", syncBuilder);
  viewSelect.addEventListener("change", syncViewGuide);
  operation.addEventListener("change", syncBuilder);
  syncBuilder();
  syncViewGuide();
  fieldForm.addEventListener("submit", async (event) => {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    try {
      let ruleJson = null;
      if (data.get("type") === "calculated") {
        const selected = data.getAll("source_field").map(Number);
        if (!selected.length) throw new Error("Select at least one source field.");
        const inputs = selected.map((field_id) => ({ op: "field", field_id }));
        const op = data.get("operation");
        let rule;
        if (["sum", "average", "maximum"].includes(op)) rule = { op, inputs };
        if (op === "best_n" || op === "drop_lowest") {
          rule = { op, count: Number(data.get("count")), inputs };
        }
        if (op === "multiply") {
          if (inputs.length !== 1) throw new Error("Multiply requires exactly one source field.");
          rule = { op, input: inputs[0], factor: Number(data.get("factor")) };
        }
        if (op === "scale") {
          if (inputs.length !== 1) throw new Error("Mark conversion requires exactly one source field.");
          rule = { op, input: inputs[0], from: Number(data.get("from")), to: Number(data.get("to")) };
        }
        if (op === "weighted_sum") {
          const rawWeights = String(data.get("weights") || "").split(",").map((value) => Number(value.trim()));
          if (rawWeights.length !== inputs.length || rawWeights.some((value) => !Number.isFinite(value) || value < 0)) {
            throw new Error("Provide one valid, non-negative weight for every selected field.");
          }
          const normalized = rawWeights.some((value) => value > 1)
            ? rawWeights.map((value) => value / 100)
            : rawWeights;
          rule = { op, items: inputs.map((input, index) => ({ input, weight: normalized[index] })) };
        }
        if (op === "subtract") {
          if (inputs.length < 2) throw new Error("Subtraction requires at least two source fields.");
          rule = inputs
            .slice(1)
            .reduce((left, right) => ({ op: "subtract", left, right }), inputs[0]);
        }
        ruleJson = JSON.stringify(rule);
      }
      const payload = {
        courseId: Number(state.courseId),
        viewId: data.get("view") ? Number(data.get("view")) : null,
        stableKey: data.get("key"),
        label: data.get("label"),
        term: data.get("term"),
        fieldType: data.get("type"),
        maxMark: data.get("maximum") ? Number(data.get("maximum")) : null,
        contribution: data.get("contribution") ? Number(data.get("contribution")) : null,
        ruleJson,
        isFinal: data.get("is_final") === "on"
      };
      if (field) {
        await call("update_assessment_field", {
          id: field.id,
          viewId: payload.viewId,
          stableKey: payload.stableKey,
          label: payload.label,
          term: payload.term,
          fieldType: payload.fieldType,
          maxMark: payload.maxMark,
          contribution: payload.contribution,
          ruleJson: payload.ruleJson,
          isFinal: payload.isFinal,
          archived: data.get("archived") === "on"
        });
      } else {
        await call("create_assessment_field", payload);
      }
      box.remove();
      toast(field ? "Assessment field updated." : "Assessment field added.", "success");
      await loadRoute();
    } catch (error) {
      toast(errorMessage(error), "error");
    }
  });
}

const starterBands = [
  [80, 100, "A+", 4, "Pass", "#62f0bd"],
  [75, 80, "A", 3.75, "Pass", "#7dd3fc"],
  [70, 75, "B+", 3.5, "Pass", "#a78bfa"],
  [60, 70, "B", 3, "Pass", "#c4b5fd"],
  [50, 60, "C", 2, "Pass", "#fbbf24"],
  [0, 50, "F", 0, "Fail", "#fb7185"]
];

function gradeBandRow(band = {}, index = 0) {
  return `<div class="grade-band-row" data-band-row>
    <input class="form-control" name="min" type="number" step="0.01" min="0" max="100" value="${band.min_percent ?? ""}" aria-label="Minimum percentage" required />
    <input class="form-control" name="max" type="number" step="0.01" min="0" max="100" value="${band.max_percent ?? ""}" aria-label="Maximum percentage" required />
    <input class="form-control" name="grade" value="${escapeHtml(band.grade_label ?? "")}" aria-label="Grade label" required />
    <input class="form-control" name="point" type="number" step="0.01" min="0" value="${band.grade_point ?? ""}" aria-label="Grade point" />
    <select class="form-control" name="result" aria-label="Result"><option ${band.result_label !== "Fail" ? "selected" : ""}>Pass</option><option ${band.result_label === "Fail" ? "selected" : ""}>Fail</option></select>
    <input class="band-color-input" name="color" type="color" value="${escapeHtml(band.color_hex ?? starterBands[index % starterBands.length][5])}" aria-label="Band color" />
    <button class="icon-button" type="button" data-remove-band aria-label="Remove grade band">×</button>
  </div>`;
}

async function openPolicyModal(policyId = null) {
  const policy = state.bootstrap.policies.find((item) => item.id === Number(policyId));
  const bands = policy
    ? await call("get_grade_bands", { policyId: policy.id })
    : starterBands.map(([min, max, grade, point, result, color]) => ({
        min_percent: min, max_percent: max, grade_label: grade, grade_point: point, result_label: result, color_hex: color
      }));
  const box = modal(`
    <h3>${policy ? "Edit grading policy" : "Create grading policy"}</h3>
    <p>Ranges operate on percentages from 0 to 100. Adjacent boundaries are accepted without overlap; the higher band owns the shared boundary.</p>
    <form id="policy-form">
      <div class="form-group"><label>Policy name</label><input class="form-control" name="name" value="${escapeHtml(policy?.name ?? "")}" placeholder="University undergraduate scale" required /></div>
      <div class="form-group"><label>Description</label><textarea class="form-control" name="description" placeholder="Applies to undergraduate courses…">${escapeHtml(policy?.description ?? "")}</textarea></div>
      <div class="grade-band-header"><span>Minimum</span><span>Maximum</span><span>Grade</span><span>Point</span><span>Result</span><span>Color</span><span></span></div>
      <div class="grade-band-editor" id="grade-band-editor">${bands.map(gradeBandRow).join("")}</div>
      <div class="policy-editor-actions"><button class="button" type="button" id="add-grade-band">＋ Add band</button><label><input type="checkbox" name="make_default" ${policy?.is_default || !policy ? "checked" : ""} /> Use as default for new courses</label></div>
      <div class="modal-actions"><button type="button" class="button" data-close-modal>Cancel</button><button type="submit" class="button primary">Save policy</button></div>
    </form>`);
  const editor = box.querySelector("#grade-band-editor");
  box.querySelector("#add-grade-band").addEventListener("click", () => {
    editor.insertAdjacentHTML("beforeend", gradeBandRow({}, editor.children.length));
  });
  editor.addEventListener("click", (event) => {
    const remove = event.target.closest("[data-remove-band]");
    if (remove && editor.children.length > 1) remove.closest("[data-band-row]").remove();
  });
  box.querySelector("#policy-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const form = event.currentTarget;
    const bandsPayload = [...form.querySelectorAll("[data-band-row]")].map((row, index) => ({
      id: 0,
      policy_id: policy?.id ?? 0,
      min_percent: Number(row.querySelector('[name="min"]').value),
      max_percent: Number(row.querySelector('[name="max"]').value),
      min_inclusive: true,
      max_inclusive: index === 0,
      grade_label: row.querySelector('[name="grade"]').value.trim(),
      grade_point: row.querySelector('[name="point"]').value === "" ? null : Number(row.querySelector('[name="point"]').value),
      result_label: row.querySelector('[name="result"]').value,
      color_hex: row.querySelector('[name="color"]').value,
      order_index: index
    }));
    try {
      const data = new FormData(form);
      await call("save_grading_policy", {
        policyId: policy?.id ?? null,
        name: data.get("name"),
        description: data.get("description") || "",
        makeDefault: data.get("make_default") === "on",
        bands: bandsPayload
      });
      box.remove();
      await refreshBootstrap();
      render();
      toast("Grading policy saved as a new version.", "success");
    } catch (error) {
      toast(errorMessage(error), "error");
    }
  });
}

async function importRosterFromWorkbook() {
  const path = await chooseOpenFile([{ name: "Excel roster", extensions: ["xlsx"] }]);
  if (!path) return;
  const preview = await call("preview_roster_import", {
    sectionId: Number(state.sectionId),
    path,
    sheetName: null
  });
  const box = modal(`
    <h3>Review roster import</h3>
    <p>Student ID and Name are required in row 1. Existing students are updated and enrolled without creating duplicates.</p>
    <div class="preview-grid roster-preview-stats">
      <div class="preview-stat"><strong>${preview.rows.length}</strong><span>Workbook rows</span></div>
      <div class="preview-stat"><strong>${preview.new_students}</strong><span>New students</span></div>
      <div class="preview-stat"><strong>${preview.existing_students}</strong><span>Existing, new section</span></div>
      <div class="preview-stat"><strong>${preview.already_enrolled}</strong><span>Already enrolled</span></div>
    </div>
    <div class="roster-preview">
      ${preview.rows.slice(0, 18).map((row) => `<div><span>${escapeHtml(row.student_identifier)}</span><strong>${escapeHtml(row.name)}</strong><span class="tag">${escapeHtml(row.status.replaceAll("_", " "))}</span></div>`).join("")}
      ${preview.rows.length > 18 ? `<p>…and ${preview.rows.length - 18} more rows</p>` : ""}
    </div>
    ${preview.duplicate_ids.length ? `<p class="blocking-message">Import blocked: duplicate Student IDs ${escapeHtml(preview.duplicate_ids.join(", "))}</p>` : ""}
    <div class="modal-actions"><button type="button" class="button" data-close-modal>Cancel</button><button type="button" class="button primary" id="confirm-roster-import" ${preview.duplicate_ids.length ? "disabled" : ""}>Import ${preview.rows.length} students</button></div>`);
  box.querySelector("#confirm-roster-import")?.addEventListener("click", async () => {
    try {
      const result = await call("import_roster", {
        sectionId: Number(state.sectionId),
        path,
        sheetName: preview.sheet
      });
      box.remove();
      state.gradebook = await call("get_gradebook", { sectionId: Number(state.sectionId) });
      toast(
        `${result.enrollments_added} enrolled · ${result.students_created} new student records.`,
        "success"
      );
      render();
    } catch (error) {
      toast(errorMessage(error), "error");
    }
  });
}

function openAttendanceModal(sessionId = null) {
  const existing = (state.attendance?.sessions ?? []).find(
    (session) => session.id === Number(sessionId)
  );
  const today = new Date().toISOString().slice(0, 10);
  const box = modal(`
    <h3>${existing ? "Edit attendance session" : "Create attendance session"}</h3><p>${existing ? "Update the date, title, or note without changing any saved attendance statuses." : "Every active student begins as Present. Record only the exceptions."}</p>
    <form id="attendance-form"><div class="inline-fields"><div class="form-group"><label>Date</label><input class="form-control" name="held_on" type="date" value="${escapeHtml(existing?.held_on ?? today)}" required /></div><div class="form-group"><label>Title</label><input class="form-control" name="title" value="${escapeHtml(existing?.title ?? "Class")}" required /></div></div><div class="form-group"><label>Session note</label><textarea class="form-control" name="note">${escapeHtml(existing?.note ?? "")}</textarea></div><div class="modal-actions"><button type="button" class="button" data-close-modal>Cancel</button><button type="submit" class="button primary">${existing ? "Save session" : "Create with all Present"}</button></div></form>`);
  box.querySelector("#attendance-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    try {
      const session = existing
        ? await call("update_attendance_session", {
            id: existing.id,
            heldOn: data.get("held_on"),
            title: data.get("title"),
            note: data.get("note") || null
          })
        : await call("create_attendance_session", {
            sectionId: Number(state.sectionId),
            heldOn: data.get("held_on"),
            title: data.get("title"),
            note: data.get("note") || null
          });
      state.activeAttendanceId = session.id;
      box.remove();
      toast(
        existing ? "Attendance session updated." : `${session.present} students marked Present.`,
        "success"
      );
      await loadRoute();
    } catch (error) {
      toast(errorMessage(error), "error");
    }
  });
}

async function saveGradeCell(input) {
  const textual = input.dataset.valueKind === "text";
  const maximum = input.dataset.maximum === "" ? null : Number(input.dataset.maximum);
  let numeric = null;
  let textValue = null;
  if (textual) {
    textValue = input.value.trim() ? input.value : null;
  } else {
    const validation = validateMark(input.value, maximum);
    if (!validation.valid) {
      input.classList.add("invalid");
      toast(validation.message, "error");
      return;
    }
    numeric = validation.value;
  }
  input.classList.remove("invalid");
  input.classList.add("pending");
  const update = {
    field_id: Number(input.dataset.field),
    enrollment_id: Number(input.dataset.enrollment),
    numeric_value: numeric,
    text_value: textValue,
    state: numeric === null && textValue === null ? "missing" : "value",
    note: null
  };
  const optimistic = optimisticGradeUpdate(entriesMap(), update);
  state.gradebook.entries = [...optimistic.values()];
  try {
    const saved = await call("save_grade_entry", {
      fieldId: update.field_id,
      enrollmentId: update.enrollment_id,
      numericValue: update.numeric_value,
      textValue: update.text_value,
      entryState: update.state,
      note: null
    });
    state.gradebook.entries = state.gradebook.entries.filter(
      (entry) => !(entry.field_id === saved.field_id && entry.enrollment_id === saved.enrollment_id)
    );
    state.gradebook.entries.push(saved);
    input.classList.remove("pending");
  } catch (error) {
    input.classList.remove("pending");
    input.classList.add("invalid");
    toast(errorMessage(error), "error");
  }
}

async function handleForm(event) {
  const form = event.target;
  if (!form.matches("form")) return;
  event.preventDefault();
  const data = new FormData(form);
  try {
    if (form.id === "onboarding-form" || form.id === "semester-form") {
      await call("create_semester", { season: data.get("season"), session: data.get("session") });
      await refreshBootstrap({ preserve: false });
      state.showWelcome = false;
      toast("Active semester created.", "success");
    } else if (form.id === "course-form") {
      await call("create_course", {
        semesterId: Number(state.semesterId),
        code: data.get("code"),
        name: data.get("name"),
        exportName: data.get("export_name"),
        colorHex: data.get("color"),
        gradingPolicyId: state.bootstrap.policies.find((policy) => policy.is_default)?.id ?? null
      });
      await refreshBootstrap();
      toast("Course created with starter gradebook views.", "success");
    } else if (form.id === "section-form") {
      const section = await call("create_section", { courseId: Number(state.courseId), label: data.get("label") });
      await refreshBootstrap();
      state.sectionId = section.id;
      toast("Section added.", "success");
    } else if (form.id === "student-form") {
      await call("add_student", {
        sectionId: Number(state.sectionId),
        studentIdentifier: data.get("student_id"),
        name: data.get("name"),
        email: data.get("email") || null
      });
      toast("Student enrolled.", "success");
      form.reset();
    } else if (form.id === "quick-entry-form") {
      const student = quickSelectedStudent();
      if (!student) return;
      const note = data.get("entry-note") || null;
      const fields = state.gradebook.fields.filter(
        (field) => !field.archived && !["calculated", "grade"].includes(field.field_type)
      );
      for (const field of fields) {
        const raw = data.get(`field-${field.id}`);
        if (raw === null) continue;
        const textual = ["text", "note"].includes(field.field_type);
        const numeric = textual || raw === "" ? null : Number(raw);
        const textValue = textual && String(raw).trim() ? String(raw) : null;
        if (numeric !== null && field.max_mark !== null && numeric > field.max_mark) {
          throw new Error(`${field.label} cannot exceed ${field.max_mark}.`);
        }
        const saved = await call("save_grade_entry", {
          fieldId: field.id,
          enrollmentId: student.enrollment_id,
          numericValue: numeric,
          textValue,
          entryState: numeric === null && textValue === null ? "missing" : "value",
          note
        });
        state.gradebook.entries = state.gradebook.entries.filter(
          (entry) => !(entry.field_id === saved.field_id && entry.enrollment_id === saved.enrollment_id)
        );
        state.gradebook.entries.push(saved);
      }
      selectNextStudent(true);
      toast("Marks saved. Moved to the next record.", "success");
    }
    render();
  } catch (error) {
    toast(errorMessage(error), "error");
  }
}

function selectNextStudent(preferUnmarked = false) {
  const students = state.gradebook?.enrollments ?? [];
  if (!students.length) return;
  const current = students.findIndex((item) => item.enrollment_id === Number(state.quickStudentId));
  let next = students[(current + 1) % students.length];
  if (preferUnmarked) {
    const map = entriesMap();
    const scorable = state.gradebook.fields.filter((field) => field.field_type === "score" && !field.archived);
    next =
      students
        .slice(current + 1)
        .concat(students.slice(0, current + 1))
        .find((student) =>
          scorable.some((field) => map.get(entryKey(student.enrollment_id, field.id))?.state !== "value")
        ) ?? next;
  }
  state.quickStudentId = next.enrollment_id;
  state.quickQuery = "";
  render();
  setTimeout(() => {
    document.querySelector("#student-search")?.focus();
    document.querySelector(".student-result.active")?.scrollIntoView({ block: "nearest" });
  }, 0);
}

async function changeContext(kind, value) {
  if (kind === "semester") {
    state.semesterId = Number(value);
    const courses = state.bootstrap.courses.filter((item) => item.semester_id === state.semesterId);
    state.courseId = courses[0]?.id ?? null;
    const sections = state.bootstrap.sections.filter((item) => item.course_id === state.courseId && !item.archived);
    state.sectionId = sections[0]?.id ?? null;
  } else if (kind === "course") {
    state.courseId = Number(value);
    state.sectionId =
      state.bootstrap.sections.find((item) => item.course_id === state.courseId && !item.archived)?.id ?? null;
  } else {
    state.sectionId = Number(value);
  }
  state.gradebook = null;
  state.analytics = null;
  state.attendance = null;
  state.pipeline = null;
  state.excel = { path: null, preflight: null, preview: null, sheet: null, term: "mid" };
  await loadRoute();
}

async function selectTemplate() {
  try {
    const path = await chooseOpenFile([{ name: "Excel workbook", extensions: ["xlsx"] }]);
    if (!path) return;
    const preflight = await call("preflight_excel", { path, sheetName: null });
    state.excel = {
      path,
      preflight,
      preview: null,
      sheet: preflight.selected_sheet,
      term: preflight.detected_term ?? "mid"
    };
    render();
  } catch (error) {
    toast(errorMessage(error), "error");
  }
}

async function previewExport() {
  try {
    state.excel.preview = await call("preview_excel_export", {
      templatePath: state.excel.path,
      sheetName: state.excel.sheet,
      sectionId: Number(state.sectionId),
      term: state.excel.term,
      finalFieldId: null
    });
    render();
  } catch (error) {
    toast(errorMessage(error), "error");
  }
}

async function exportWorkbook() {
  try {
    const output = await chooseSaveFile(state.excel.preview.output_filename, [
      { name: "Excel workbook", extensions: ["xlsx"] }
    ]);
    if (!output) return;
    const result = await call("export_excel", {
      templatePath: state.excel.path,
      sheetName: state.excel.sheet,
      sectionId: Number(state.sectionId),
      term: state.excel.term,
      finalFieldId: null,
      outputPath: output
    });
    toast(`Verified workbook exported with ${result.changed} marks.`, "success");
  } catch (error) {
    toast(errorMessage(error), "error");
  }
}

async function transferDatabase(action) {
  try {
    if (action === "export") {
      const date = new Date().toISOString().slice(0, 10);
      const path = await chooseSaveFile(`gradia-database-${date}.gradia`, [
        { name: "Gradia database transfer", extensions: ["gradia"] }
      ]);
      if (!path) return;
      await call("export_backup", { outputPath: path });
      toast("Database exported to a verified .gradia transfer file.", "success");
    } else {
      const path = await chooseOpenFile([
        { name: "Gradia database transfer", extensions: ["gradia"] }
      ]);
      if (!path) return;
      if (
        !window.confirm(
          "Import this Gradia database? After validation, all current local Gradia data on this device will be replaced. Export the current database first if you need to keep it."
        )
      )
        return;
      await call("import_backup", { backupPath: path });
      await refreshBootstrap({ preserve: false });
      state.gradebook = null;
      state.dashboard = null;
      state.analytics = null;
      state.attendance = null;
      state.pipeline = null;
      toast("Database imported, verified, and loaded.", "success");
      await loadRoute();
    }
  } catch (error) {
    toast(errorMessage(error), "error");
  }
}

app.addEventListener("submit", handleForm);

app.addEventListener("input", (event) => {
  if (event.target.id === "student-search") {
    state.quickQuery = event.target.value;
    render();
    const input = document.querySelector("#student-search");
    input?.focus();
    input?.setSelectionRange(input.value.length, input.value.length);
  }
  if (event.target.id === "grade-filter") {
    state.gradeFilter = event.target.value;
    render();
    const input = document.querySelector("#grade-filter");
    input?.focus();
    input?.setSelectionRange(input.value.length, input.value.length);
  }
});

app.addEventListener("change", async (event) => {
  const target = event.target;
  if (target.id === "semester-select") await changeContext("semester", target.value);
  if (target.id === "course-select") await changeContext("course", target.value);
  if (target.id === "section-select") await changeContext("section", target.value);
  if (target.matches("[data-grade-cell]")) await saveGradeCell(target);
  if (target.id === "excel-sheet") {
    state.excel.sheet = target.value;
    state.excel.preview = null;
  }
  if (target.id === "excel-term") {
    state.excel.term = target.value;
    state.excel.preview = null;
  }
});

app.addEventListener("click", async (event) => {
  const button = event.target.closest("button");
  if (!button) return;
  if (button.dataset.route) {
    state.route = button.dataset.route;
    await loadRoute();
    return;
  }
  if (button.dataset.openSection) {
    state.sectionId = Number(button.dataset.openSection);
    state.courseId = activeSection()?.course_id ?? state.courseId;
    state.route = "gradebook";
    await loadRoute();
    return;
  }
  if (button.dataset.term) {
    state.activeTerm = button.dataset.term;
    render();
    return;
  }
  if (button.dataset.student) {
    state.quickStudentId = Number(button.dataset.student);
    render();
    return;
  }
  if (button.dataset.session) {
    state.activeAttendanceId = Number(button.dataset.session);
    render();
    return;
  }
  if (button.dataset.attendanceStatus) {
    try {
      await call("set_attendance_status", {
        sessionId: Number(state.activeAttendanceId),
        enrollmentId: Number(button.dataset.enrollment),
        status: button.dataset.attendanceStatus,
        note: null
      });
      const [sessions, records] = await call("get_attendance", { sectionId: Number(state.sectionId) });
      state.attendance = { sessions, records };
      render();
    } catch (error) {
      toast(errorMessage(error), "error");
    }
    return;
  }
  if (button.dataset.pipelineStage) {
    try {
      await call("toggle_pipeline", {
        fieldId: Number(button.dataset.field),
        sectionId: Number(state.sectionId),
        stage: button.dataset.pipelineStage,
        value: button.dataset.value !== "true"
      });
      state.pipeline = await call("get_pipeline", { sectionId: Number(state.sectionId) });
      render();
    } catch (error) {
      toast(errorMessage(error), "error");
    }
    return;
  }
  if (["add-field", "add-field-card"].includes(button.id)) openFieldModal();
  if (button.dataset.editField) openFieldModal(Number(button.dataset.editField));
  if (button.id === "edit-semester") openContextEditor("semester");
  if (button.id === "edit-course") openContextEditor("course");
  if (button.id === "edit-section") openContextEditor("section");
  if (button.id === "edit-student") openStudentEditor();
  if (button.id === "manage-gradebook-views") openGradebookViewEditor();
  if (button.id === "delete-semester") await openDeleteModal("semester");
  if (button.id === "delete-course") await openDeleteModal("course");
  if (button.id === "delete-section") await openDeleteModal("section");
  if (button.id === "new-policy" || button.dataset.policy) {
    try {
      await openPolicyModal(button.dataset.policy ? Number(button.dataset.policy) : null);
    } catch (error) {
      toast(errorMessage(error), "error");
    }
  }
  if (button.id === "import-roster") {
    try {
      await importRosterFromWorkbook();
    } catch (error) {
      toast(errorMessage(error), "error");
    }
  }
  if (["new-attendance", "new-attendance-empty"].includes(button.id)) openAttendanceModal();
  if (button.id === "edit-attendance-session") {
    openAttendanceModal(Number(state.activeAttendanceId));
  }
  if (button.id === "next-student") selectNextStudent(false);
  if (button.id === "global-search") {
    state.route = "quick";
    await loadRoute();
  }
  if (button.id === "refresh-insights") await loadRoute();
  if (button.id === "choose-template" || button.id === "change-template") await selectTemplate();
  if (button.id === "preview-export") await previewExport();
  if (button.id === "export-workbook") await exportWorkbook();
  if (button.id === "export-database") await transferDatabase("export");
  if (button.id === "import-database") await transferDatabase("import");
  if (button.id === "open-welcome") {
    state.showWelcome = true;
    render();
  }
  if (button.id === "close-welcome") {
    state.showWelcome = false;
    render();
  }
});

document.addEventListener("keydown", async (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    state.route = "quick";
    await loadRoute();
  }
});

async function init() {
  try {
    await refreshBootstrap({ preserve: false });
    render();
    await loadRoute();
  } catch (error) {
    app.innerHTML = `<div class="empty-state"><div><div class="empty-icon">!</div><h3>Gradia could not start</h3><p>${escapeHtml(errorMessage(error))}</p></div></div>`;
  }
}

init();

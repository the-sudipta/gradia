import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { buildOutputFilename, descriptiveStats, entryKey } from "./core.js";

export const isDesktop = Boolean(window.__TAURI_INTERNALS__);

const now = () => new Date().toISOString();
const demo = {
  bootstrap: {
    semesters: [{ id: 1, season: "Fall", session: "2025-2026", is_active: true, created_at: now() }],
    courses: [
      {
        id: 1,
        semester_id: 1,
        grading_policy_id: 1,
        code: "CSC2211",
        name: "Data Structures Lab",
        export_name: "DATA STRUCTURE LAB",
        color_hex: "#9b7bff",
        created_at: now()
      }
    ],
    sections: [
      { id: 1, course_id: 1, label: "G", order_index: 0, archived: false, created_at: now() },
      { id: 2, course_id: 1, label: "B7", order_index: 1, archived: false, created_at: now() }
    ],
    policies: [
      { id: 1, name: "Starter percentage policy", description: "Editable demonstration scale", is_default: true, version: 1 }
    ]
  },
  fields: [
    { id: 1, course_id: 1, view_id: 1, stable_key: "attendance", label: "Attendance", term: "mid", field_type: "score", max_mark: 10, contribution: 10, rule_json: null, is_final: false, order_index: 0, archived: false },
    { id: 2, course_id: 1, view_id: 1, stable_key: "obe", label: "OBE", term: "mid", field_type: "score", max_mark: 15, contribution: 15, rule_json: null, is_final: false, order_index: 1, archived: false },
    { id: 3, course_id: 1, view_id: 1, stable_key: "lab_task", label: "Lab Tasks", term: "mid", field_type: "score", max_mark: 30, contribution: 30, rule_json: null, is_final: false, order_index: 2, archived: false },
    { id: 4, course_id: 1, view_id: 1, stable_key: "mid_lab", label: "Mid-Lab", term: "mid", field_type: "score", max_mark: 25, contribution: 25, rule_json: null, is_final: false, order_index: 3, archived: false },
    { id: 5, course_id: 1, view_id: 1, stable_key: "viva", label: "Viva", term: "mid", field_type: "score", max_mark: 20, contribution: 20, rule_json: null, is_final: false, order_index: 4, archived: false },
    { id: 6, course_id: 1, view_id: 1, stable_key: "mid_total", label: "Midterm Total", term: "mid", field_type: "calculated", max_mark: 100, contribution: 40, rule_json: null, is_final: true, order_index: 5, archived: false }
  ],
  // Fictional records used only when the browser demo runs outside Tauri.
  students: [
    ["26-10001-1", "NILA RAHMAN"],
    ["26-10002-2", "ARIYAN SEN"],
    ["26-10003-3", "MEHRIN ALAM"],
    ["26-10004-1", "TAHSIN NOOR"],
    ["26-10005-2", "ISHRAT RAY"],
    ["26-10006-3", "ZAYAN KARIM"],
    ["26-10007-1", "FARIA HASAN"],
    ["26-10008-2", "RAYHAN DUTTA"],
    ["26-10009-3", "SAMIA NAWAR"],
    ["26-10010-1", "ADNAN SARKER"],
    ["26-10011-2", "MAHIRA ZAMAN"],
    ["26-10012-3", "RAFI AHMED"]
  ].map(([student_identifier, name], index) => ({
    enrollment_id: index + 1,
    section_id: 1,
    student_id: index + 1,
    student_identifier,
    name,
    email: null,
    status: "active",
    roll_order: index
  })),
  entries: new Map(),
  policyBands: [
    { id: 1, policy_id: 1, min_percent: 80, max_percent: 100, min_inclusive: true, max_inclusive: true, grade_label: "A+", grade_point: 4, result_label: "Pass", color_hex: "#62f0bd", order_index: 0 },
    { id: 2, policy_id: 1, min_percent: 75, max_percent: 80, min_inclusive: true, max_inclusive: false, grade_label: "A", grade_point: 3.75, result_label: "Pass", color_hex: "#7dd3fc", order_index: 1 },
    { id: 3, policy_id: 1, min_percent: 70, max_percent: 75, min_inclusive: true, max_inclusive: false, grade_label: "B+", grade_point: 3.5, result_label: "Pass", color_hex: "#a78bfa", order_index: 2 },
    { id: 4, policy_id: 1, min_percent: 60, max_percent: 70, min_inclusive: true, max_inclusive: false, grade_label: "B", grade_point: 3, result_label: "Pass", color_hex: "#c4b5fd", order_index: 3 },
    { id: 5, policy_id: 1, min_percent: 50, max_percent: 60, min_inclusive: true, max_inclusive: false, grade_label: "C", grade_point: 2, result_label: "Pass", color_hex: "#fbbf24", order_index: 4 },
    { id: 6, policy_id: 1, min_percent: 0, max_percent: 50, min_inclusive: true, max_inclusive: false, grade_label: "F", grade_point: 0, result_label: "Fail", color_hex: "#fb7185", order_index: 5 }
  ],
  attendance: { sessions: [], records: {} },
  pipeline: new Map()
};

for (const student of demo.students) {
  const base = 55 + ((student.enrollment_id * 7) % 38);
  const values = [
    Math.min(10, 7 + (student.enrollment_id % 4)),
    Math.min(15, 7 + (student.enrollment_id % 9)),
    Math.min(30, 19 + (student.enrollment_id % 11)),
    Math.min(25, 13 + (student.enrollment_id % 13)),
    Math.min(20, 9 + (student.enrollment_id % 11)),
    base
  ];
  demo.fields.forEach((field, index) => {
    demo.entries.set(entryKey(student.enrollment_id, field.id), {
      id: demo.entries.size + 1,
      field_id: field.id,
      enrollment_id: student.enrollment_id,
      numeric_value: values[index],
      text_value: null,
      state: "value",
      note: null,
      updated_at: now()
    });
  });
}

function demoGradebook(sectionId) {
  const computed = demo.students.map((student) => ({
    enrollment_id: student.enrollment_id,
    student_identifier: student.student_identifier,
    name: student.name,
    values: Object.fromEntries(
      demo.fields.map((field) => [
        field.id,
        demo.entries.get(entryKey(student.enrollment_id, field.id))?.numeric_value ?? null
      ])
    ),
    final_percentage: demo.entries.get(entryKey(student.enrollment_id, 6))?.numeric_value ?? null,
    grade: null
  }));
  return {
    course: demo.bootstrap.courses[0],
    section: demo.bootstrap.sections.find((section) => section.id === Number(sectionId)) ?? demo.bootstrap.sections[0],
    enrollments: demo.students.map((student) => ({ ...student, section_id: Number(sectionId) })),
    views: [
      { id: 1, course_id: 1, name: "Midterm", term: "mid", order_index: 0 },
      { id: 2, course_id: 1, name: "Final", term: "final", order_index: 1 },
      { id: 3, course_id: 1, name: "Semester Result", term: "semester", order_index: 2 },
      { id: 4, course_id: 1, name: "Attendance", term: "custom", order_index: 3 }
    ],
    fields: demo.fields,
    entries: [...demo.entries.values()],
    computed
  };
}

function demoAnalytics(sectionId) {
  const results = demo.students.map((student) => {
    const final = demo.entries.get(entryKey(student.enrollment_id, 6))?.numeric_value ?? null;
    const label = final >= 80 ? "A+" : final >= 75 ? "A" : final >= 70 ? "B+" : final >= 65 ? "B" : final >= 60 ? "C+" : final >= 55 ? "C" : final >= 50 ? "D+" : final >= 45 ? "D" : "F";
    return {
      enrollment_id: student.enrollment_id,
      student_identifier: student.student_identifier,
      name: student.name,
      values: {},
      final_percentage: final,
      grade: final === null ? null : { percentage: final, grade_label: label, grade_point: label === "F" ? 0 : 3, result_label: label === "F" ? "Fail" : "Pass", color_hex: label === "F" ? "#ef4444" : "#8b5cf6" }
    };
  });
  const stats = descriptiveStats(results.map((result) => result.final_percentage));
  const grade_frequency = {};
  for (const result of results) grade_frequency[result.grade.grade_label] = (grade_frequency[result.grade.grade_label] ?? 0) + 1;
  return {
    section_id: Number(sectionId),
    count: results.length,
    completed: results.length,
    missing: 0,
    mean: stats.mean,
    median: stats.median,
    minimum: stats.min,
    maximum: stats.max,
    standard_deviation: stats.standardDeviation,
    pass_count: results.filter((result) => result.grade.result_label === "Pass").length,
    fail_count: results.filter((result) => result.grade.result_label === "Fail").length,
    grade_frequency,
    results
  };
}

async function demoCall(command, args) {
  switch (command) {
    case "get_bootstrap":
      return structuredClone(demo.bootstrap);
    case "get_dashboard":
      return [
        {
          course: demo.bootstrap.courses[0],
          sections: demo.bootstrap.sections.map((section) => ({
            section,
            students: demo.students.length,
            fields: demo.fields.length,
            entered: demo.entries.size,
            possible: demo.students.length * demo.fields.length,
            completion_percent: 100
          }))
        }
      ];
    case "get_gradebook":
      return demoGradebook(args.sectionId);
    case "list_roster":
    case "search_students":
      return structuredClone(demo.students);
    case "save_grade_entry": {
      const entry = {
        id: demo.entries.size + 1,
        field_id: args.fieldId,
        enrollment_id: args.enrollmentId,
        numeric_value: args.numericValue,
        text_value: args.textValue,
        state: args.entryState,
        note: args.note,
        updated_at: now()
      };
      demo.entries.set(entryKey(args.enrollmentId, args.fieldId), entry);
      return entry;
    }
    case "get_section_analytics":
      return demoAnalytics(args.sectionId);
    case "get_pipeline":
      return demo.fields.map((field) =>
        demo.pipeline.get(`${args.sectionId}:${field.id}`) ?? {
          field_id: field.id,
          section_id: args.sectionId,
          evaluated: false,
          evaluated_at: null,
          marks_recorded: false,
          marks_recorded_at: null,
          portal_uploaded: false,
          portal_uploaded_at: null,
          pending_note: null
        }
      );
    case "get_grade_bands":
      return structuredClone(demo.policyBands.filter((band) => band.policy_id === Number(args.policyId)));
    case "save_grading_policy": {
      const id = args.policyId ?? Math.max(0, ...demo.bootstrap.policies.map((policy) => policy.id)) + 1;
      if (args.makeDefault) {
        for (const policy of demo.bootstrap.policies) policy.is_default = false;
      }
      const current = demo.bootstrap.policies.find((policy) => policy.id === id);
      const policy = {
        id,
        name: args.name,
        description: args.description,
        is_default: args.makeDefault,
        version: current ? current.version + 1 : 1
      };
      if (current) Object.assign(current, policy);
      else demo.bootstrap.policies.push(policy);
      demo.policyBands = demo.policyBands.filter((band) => band.policy_id !== id);
      args.bands.forEach((band, index) =>
        demo.policyBands.push({ ...band, id: demo.policyBands.length + 1, policy_id: id, order_index: index })
      );
      return structuredClone(policy);
    }
    case "toggle_pipeline": {
      const key = `${args.sectionId}:${args.fieldId}`;
      const value =
        demo.pipeline.get(key) ?? {
          field_id: args.fieldId,
          section_id: args.sectionId,
          evaluated: false,
          evaluated_at: null,
          marks_recorded: false,
          marks_recorded_at: null,
          portal_uploaded: false,
          portal_uploaded_at: null,
          pending_note: null
        };
      value[args.stage] = args.value;
      value[`${args.stage}_at`] = args.value ? now() : null;
      demo.pipeline.set(key, value);
      return structuredClone(value);
    }
    case "create_attendance_session": {
      const session = { id: demo.attendance.sessions.length + 1, section_id: args.sectionId, held_on: args.heldOn, title: args.title || "Class", note: args.note, present: demo.students.length, absent: 0, late: 0, excused: 0, left_early: 0 };
      demo.attendance.sessions.unshift(session);
      demo.attendance.records[session.id] = demo.students.map((student) => ({ enrollment_id: student.enrollment_id, status: "present", note: null }));
      return session;
    }
    case "get_attendance":
      return [structuredClone(demo.attendance.sessions), structuredClone(demo.attendance.records)];
    case "set_attendance_status": {
      const record = demo.attendance.records[args.sessionId].find((item) => item.enrollment_id === args.enrollmentId);
      record.status = args.status;
      return structuredClone(record);
    }
    case "create_assessment_field": {
      const field = { id: demo.fields.length + 1, course_id: args.courseId, view_id: args.viewId, stable_key: args.stableKey, label: args.label, term: args.term, field_type: args.fieldType, max_mark: args.maxMark, contribution: args.contribution, rule_json: args.ruleJson, is_final: args.isFinal, order_index: demo.fields.length, archived: false };
      demo.fields.push(field);
      return field;
    }
    case "create_semester":
    case "create_course":
    case "create_section":
    case "add_student":
      throw new Error("Setup mutations are available in the installed desktop app. Browser preview uses protected demonstration data.");
    case "preflight_excel":
    case "preview_excel_export":
    case "export_excel":
    case "export_backup":
    case "import_backup":
      throw new Error("This file-system action is available in the installed Gradia desktop app.");
    default:
      return null;
  }
}

export async function call(command, args = {}) {
  return isDesktop ? invoke(command, args) : demoCall(command, args);
}

export async function chooseOpenFile(filters = []) {
  if (!isDesktop) throw new Error("File selection is available in the installed desktop app.");
  return open({ multiple: false, directory: false, filters });
}

export async function chooseSaveFile(defaultPath, filters = []) {
  if (!isDesktop) throw new Error("File saving is available in the installed desktop app.");
  return save({ defaultPath, filters });
}

export function suggestedFilename(context) {
  return buildOutputFilename(context);
}

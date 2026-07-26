export function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export function formatNumber(value, digits = 1) {
  if (value === null || value === undefined || Number.isNaN(Number(value))) return "—";
  return Number(value).toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: digits
  });
}

export function clamp(value, minimum, maximum) {
  return Math.min(maximum, Math.max(minimum, value));
}

export function normalizeStudentId(value) {
  return String(value ?? "").replace(/\s+/g, "").toUpperCase();
}

export function rankStudents(students, query) {
  const needle = String(query ?? "").trim().toLowerCase();
  if (!needle) return students.slice(0, 12);
  return students
    .filter((student) => {
      const id = student.student_identifier.toLowerCase();
      const name = student.name.toLowerCase();
      return id.includes(needle) || name.includes(needle);
    })
    .map((student) => {
      const id = student.student_identifier.toLowerCase();
      const name = student.name.toLowerCase();
      let rank = 4;
      if (id === needle) rank = 0;
      else if (id.startsWith(needle)) rank = 1;
      else if (name.startsWith(needle)) rank = 2;
      else if (name.split(/\s+/).some((word) => word.startsWith(needle))) rank = 3;
      return { student, rank };
    })
    .sort((a, b) => a.rank - b.rank || a.student.name.localeCompare(b.student.name))
    .slice(0, 20)
    .map(({ student }) => student);
}

export function studentWindowDetails(students, selectedEnrollmentId, size = 12) {
  if (!students.length) {
    return { students: [], start: 0, end: 0, total: 0 };
  }
  const windowSize = Math.max(1, Math.floor(Number(size)) || 12);
  const selectedIndex = students.findIndex(
    (student) => student.enrollment_id === Number(selectedEnrollmentId)
  );
  const start = selectedIndex < 0 ? 0 : Math.floor(selectedIndex / windowSize) * windowSize;
  const windowStudents = students.slice(start, start + windowSize);
  return {
    students: windowStudents,
    start: start + 1,
    end: start + windowStudents.length,
    total: students.length
  };
}

export function studentWindow(students, selectedEnrollmentId, size = 12) {
  return studentWindowDetails(students, selectedEnrollmentId, size).students;
}

export const ASSESSMENT_TYPE_GUIDES = Object.freeze({
  score: {
    label: "Score",
    behavior: "A numeric mark entered by the teacher. Gradia validates it against the maximum mark.",
    example: "Quiz 1: 8 out of 10."
  },
  calculated: {
    label: "Calculated",
    behavior:
      "A read-only numeric result produced from existing fields. If a required source mark is missing, the result remains missing.",
    example: "Midterm Total = OBE + Viva + Midterm Exam, or Best 3 of 4 quizzes."
  },
  attendance: {
    label: "Attendance",
    behavior:
      "A numeric assessment column intended for an attendance-derived mark. Attendance sessions remain managed separately on the Attendance screen.",
    example: "Attendance Mark: 9 out of 10 after applying your institution's attendance policy."
  },
  bonus: {
    label: "Bonus",
    behavior:
      "A non-negative extra-credit amount. Include it as a source in a calculated field when it should increase a result.",
    example: "Participation Bonus: 2 points, then Semester Total includes that bonus."
  },
  penalty: {
    label: "Penalty",
    behavior:
      "A non-negative deduction amount. Include it in a calculated field using subtraction when it should reduce a result.",
    example: "Late Submission Penalty: 3 points; Adjusted Project = Project − Penalty."
  },
  text: {
    label: "Text",
    behavior:
      "A short written value rather than a number. It is stored per student and is not used in numeric calculations.",
    example: "Presentation outcome: Satisfactory."
  },
  note: {
    label: "Note",
    behavior:
      "Free-form student-specific context. It documents an exception or decision without changing the mark.",
    example: "Makeup assessment approved for 12 August."
  }
});

export function assessmentTypeGuide(type) {
  return ASSESSMENT_TYPE_GUIDES[type] ?? ASSESSMENT_TYPE_GUIDES.score;
}

const GRADEBOOK_VIEW_GUIDES = Object.freeze({
  midterm: {
    label: "Midterm",
    behavior: "Groups fields used during the midterm part of the course.",
    example: "Midterm OBE, Midterm Exam, and Midterm Total."
  },
  final: {
    label: "Final",
    behavior: "Groups fields used during the final part of the course.",
    example: "Final OBE, Final Exam, Viva, and Final Total."
  },
  "semester result": {
    label: "Semester Result",
    behavior: "Groups fields that combine or summarize the complete course result.",
    example: "Semester Total = 40% Midterm + 60% Final."
  },
  attendance: {
    label: "Attendance",
    behavior:
      "Groups attendance-related assessment fields. The Attendance screen still holds the individual class-session records.",
    example: "Attendance Percentage and converted Attendance Mark."
  }
});

export function gradebookViewGuide(name = "", term = "") {
  const normalized = String(name).trim().toLowerCase();
  if (!normalized) {
    return {
      label: "No specific view",
      behavior:
        "Leaves the field outside a named grouping. Its Term still controls whether it appears under Midterm, Final, Semester, or All fields.",
      example: "Use this for a general reference field that does not belong to one named view."
    };
  }
  return (
    GRADEBOOK_VIEW_GUIDES[normalized] ?? {
      label: String(name).trim(),
      behavior: `Groups the field in the “${String(name).trim()}” view${
        term ? ` for the ${String(term).trim()} term` : ""
      }. The view organizes fields; it does not change their values or calculation rules.`,
      example: `Use it when this field belongs with the other “${String(name).trim()}” columns.`
    }
  );
}

export function entryKey(enrollmentId, fieldId) {
  return `${enrollmentId}:${fieldId}`;
}

export function optimisticGradeUpdate(entries, update) {
  const key = entryKey(update.enrollment_id, update.field_id);
  const next = new Map(entries instanceof Map ? entries : []);
  next.set(key, { ...(next.get(key) ?? {}), ...update, pending: true });
  return next;
}

export function outputSectionLabel(section) {
  const clean = String(section ?? "").trim().replace(/^\[|\]$/g, "");
  return /\d+$/.test(clean) ? `[${clean}]` : clean;
}

export function buildOutputFilename({ exportName, section, term, session, season }) {
  const termLabel =
    term === "mid" ? "Midterm" : term === "final" ? "Finalterm" : term === "semester" ? "Semester Result" : term;
  return `${String(exportName).trim()} ${outputSectionLabel(section)} ${termLabel} for ${String(session).trim()} ${String(season).trim()}.xlsx`;
}

export function descriptiveStats(values) {
  const numbers = values.map(Number).filter(Number.isFinite).sort((a, b) => a - b);
  if (!numbers.length) {
    return { count: 0, mean: null, median: null, min: null, max: null, standardDeviation: null };
  }
  const mean = numbers.reduce((sum, value) => sum + value, 0) / numbers.length;
  const middle = Math.floor(numbers.length / 2);
  const median =
    numbers.length % 2 ? numbers[middle] : (numbers[middle - 1] + numbers[middle]) / 2;
  const variance =
    numbers.reduce((sum, value) => sum + (value - mean) ** 2, 0) / numbers.length;
  return {
    count: numbers.length,
    mean,
    median,
    min: numbers[0],
    max: numbers.at(-1),
    standardDeviation: Math.sqrt(variance)
  };
}

export function initials(name) {
  return String(name)
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((word) => word[0])
    .join("")
    .toUpperCase();
}

export function percentage(value, total) {
  return total > 0 ? (Number(value) / Number(total)) * 100 : 0;
}

export function gradeHeatColor(percent) {
  if (percent === null || percent === undefined) return "var(--muted)";
  if (percent >= 80) return "#22c55e";
  if (percent >= 70) return "#84cc16";
  if (percent >= 60) return "#eab308";
  if (percent >= 50) return "#f97316";
  return "#ef4444";
}

export function validateMark(rawValue, maximum = null) {
  if (rawValue === "" || rawValue === null || rawValue === undefined) {
    return { valid: true, value: null, state: "missing" };
  }
  const value = Number(rawValue);
  if (!Number.isFinite(value)) return { valid: false, message: "Enter a valid number." };
  if (value < 0) return { valid: false, message: "Mark cannot be negative." };
  if (maximum !== null && maximum !== undefined && value > Number(maximum)) {
    return { valid: false, message: `Mark cannot exceed ${maximum}.` };
  }
  return { valid: true, value, state: "value" };
}

export function insightSentences(analytics) {
  if (!analytics || !analytics.count) return ["Add and finalize student results to reveal insights."];
  const insights = [];
  if (analytics.missing) {
    insights.push(
      `${analytics.missing} student${analytics.missing === 1 ? " is" : "s are"} still missing a final result.`
    );
  }
  if (analytics.mean !== null && analytics.standard_deviation !== null) {
    insights.push(
      `The section mean is ${formatNumber(analytics.mean)}% with a spread of ${formatNumber(analytics.standard_deviation)} points.`
    );
  }
  if (analytics.fail_count > 0) {
    insights.push(
      `${analytics.fail_count} finalized result${analytics.fail_count === 1 ? " is" : "s are"} currently below the selected policy's passing range.`
    );
  }
  if (!insights.length) insights.push("All currently finalized results fall within passing bands.");
  return insights;
}

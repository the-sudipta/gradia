import test from "node:test";
import assert from "node:assert/strict";
import { JSDOM } from "jsdom";
import { barChart } from "../../src/charts.js";
import {
  buildOutputFilename,
  descriptiveStats,
  entryKey,
  optimisticGradeUpdate,
  rankStudents,
  validateMark
} from "../../src/core.js";

test("student finder ranks exact ID and prefixes before contains", () => {
  const students = [
    { student_identifier: "25-10000-1", name: "SAMIRA KHAN" },
    { student_identifier: "25-20000-1", name: "HASAN SAMI" },
    { student_identifier: "25-30000-1", name: "TASNIM HASAN" }
  ];
  assert.equal(rankStudents(students, "25-20000-1")[0].name, "HASAN SAMI");
  assert.equal(rankStudents(students, "sami")[0].name, "SAMIRA KHAN");
});

test("optimistic grade update changes local state before confirmation", () => {
  const before = new Map();
  const after = optimisticGradeUpdate(before, {
    enrollment_id: 7,
    field_id: 3,
    numeric_value: 12,
    state: "value"
  });
  assert.equal(before.size, 0);
  assert.equal(after.get(entryKey(7, 3)).numeric_value, 12);
  assert.equal(after.get(entryKey(7, 3)).pending, true);
});

test("mark validation distinguishes missing from zero and rejects limits", () => {
  assert.deepEqual(validateMark("", 15), { valid: true, value: null, state: "missing" });
  assert.deepEqual(validateMark("0", 15), { valid: true, value: 0, state: "value" });
  assert.equal(validateMark("-1", 15).valid, false);
  assert.equal(validateMark("16", 15).valid, false);
});

test("institution filename brackets only sections ending in digits", () => {
  assert.equal(
    buildOutputFilename({
      exportName: "INTRODUCTION TO PROGRAMMING LAB",
      section: "B7",
      term: "mid",
      session: "2025-2026",
      season: "Fall"
    }),
    "INTRODUCTION TO PROGRAMMING LAB [B7] Midterm for 2025-2026 Fall.xlsx"
  );
  assert.equal(
    buildOutputFilename({
      exportName: "DATA STRUCTURE LAB",
      section: "G",
      term: "mid",
      session: "2025-2026",
      season: "Spring"
    }),
    "DATA STRUCTURE LAB G Midterm for 2025-2026 Spring.xlsx"
  );
});

test("descriptive statistics reconcile to a known fixture", () => {
  const stats = descriptiveStats([60, 70, 80, 90]);
  assert.equal(stats.mean, 75);
  assert.equal(stats.median, 75);
  assert.equal(stats.min, 60);
  assert.equal(stats.max, 90);
});

test("chart rendering produces accessible SVG and all categories", () => {
  const dom = new JSDOM(`<main>${barChart([{ label: "A", value: 4 }, { label: "B+", value: 7 }])}</main>`);
  const document = dom.window.document;
  assert.equal(document.querySelectorAll(".chart-bar").length, 2);
  assert.equal(document.querySelector("svg").getAttribute("role"), "img");
  assert.match(document.body.textContent, /B\+/);
});

use crate::models::{CalculationTrace, GradeBand, GradeResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum RuleNode {
    Field {
        field_id: i64,
    },
    Constant {
        value: f64,
    },
    Sum {
        inputs: Vec<RuleNode>,
    },
    Average {
        inputs: Vec<RuleNode>,
    },
    Maximum {
        inputs: Vec<RuleNode>,
    },
    Minimum {
        inputs: Vec<RuleNode>,
    },
    BestN {
        count: usize,
        inputs: Vec<RuleNode>,
    },
    DropLowest {
        count: usize,
        inputs: Vec<RuleNode>,
    },
    Multiply {
        input: Box<RuleNode>,
        factor: f64,
    },
    Scale {
        input: Box<RuleNode>,
        from: f64,
        to: f64,
    },
    WeightedSum {
        items: Vec<WeightedInput>,
    },
    Cap {
        input: Box<RuleNode>,
        minimum: Option<f64>,
        maximum: Option<f64>,
    },
    Add {
        left: Box<RuleNode>,
        right: Box<RuleNode>,
    },
    Subtract {
        left: Box<RuleNode>,
        right: Box<RuleNode>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeightedInput {
    pub input: RuleNode,
    pub weight: f64,
}

pub fn evaluate_rule(
    rule: &RuleNode,
    values: &HashMap<i64, Option<f64>>,
) -> (Option<f64>, CalculationTrace) {
    match rule {
        RuleNode::Field { field_id } => {
            let value = values.get(field_id).copied().flatten();
            (
                value,
                CalculationTrace {
                    op: "field".into(),
                    value,
                    detail: format!("Field {field_id}"),
                    children: vec![],
                },
            )
        }
        RuleNode::Constant { value } => leaf("constant", Some(*value), value.to_string()),
        RuleNode::Sum { inputs } => aggregate("sum", inputs, values, |items| items.iter().sum()),
        RuleNode::Average { inputs } => aggregate("average", inputs, values, |items| {
            items.iter().sum::<f64>() / items.len() as f64
        }),
        RuleNode::Maximum { inputs } => aggregate("maximum", inputs, values, |items| {
            items.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        }),
        RuleNode::Minimum { inputs } => aggregate("minimum", inputs, values, |items| {
            items.iter().copied().fold(f64::INFINITY, f64::min)
        }),
        RuleNode::BestN { count, inputs } => {
            let (mut numbers, children) = collect(inputs, values);
            numbers.sort_by(|a, b| b.total_cmp(a));
            let value = if *count == 0 || numbers.len() < *count {
                None
            } else {
                Some(numbers.iter().take(*count).sum())
            };
            (
                value,
                CalculationTrace {
                    op: "best_n".into(),
                    value,
                    detail: format!("Best {count} of {}", inputs.len()),
                    children,
                },
            )
        }
        RuleNode::DropLowest { count, inputs } => {
            let (mut numbers, children) = collect(inputs, values);
            numbers.sort_by(|a, b| a.total_cmp(b));
            let value = if numbers.len() <= *count {
                None
            } else {
                Some(numbers.iter().skip(*count).sum())
            };
            (
                value,
                CalculationTrace {
                    op: "drop_lowest".into(),
                    value,
                    detail: format!("Drop lowest {count}"),
                    children,
                },
            )
        }
        RuleNode::Multiply { input, factor } => unary("multiply", input, values, |v| v * factor),
        RuleNode::Scale { input, from, to } => {
            if *from == 0.0 {
                return leaf("scale", None, "Source maximum cannot be zero".into());
            }
            unary("scale", input, values, |v| v / from * to)
        }
        RuleNode::WeightedSum { items } => {
            let mut children = Vec::new();
            let mut total = 0.0;
            for item in items {
                let (value, trace) = evaluate_rule(&item.input, values);
                children.push(trace);
                match value {
                    Some(number) => total += number * item.weight,
                    None => {
                        return (
                            None,
                            CalculationTrace {
                                op: "weighted_sum".into(),
                                value: None,
                                detail: "An input is missing".into(),
                                children,
                            },
                        )
                    }
                }
            }
            (
                Some(total),
                CalculationTrace {
                    op: "weighted_sum".into(),
                    value: Some(total),
                    detail: format!("{} weighted inputs", items.len()),
                    children,
                },
            )
        }
        RuleNode::Cap {
            input,
            minimum,
            maximum,
        } => unary("cap", input, values, |mut v| {
            if let Some(minimum) = minimum {
                v = v.max(*minimum);
            }
            if let Some(maximum) = maximum {
                v = v.min(*maximum);
            }
            v
        }),
        RuleNode::Add { left, right } => binary("add", left, right, values, |a, b| a + b),
        RuleNode::Subtract { left, right } => binary("subtract", left, right, values, |a, b| a - b),
    }
}

fn leaf(op: &str, value: Option<f64>, detail: String) -> (Option<f64>, CalculationTrace) {
    (
        value,
        CalculationTrace {
            op: op.into(),
            value,
            detail,
            children: vec![],
        },
    )
}

fn collect(
    inputs: &[RuleNode],
    values: &HashMap<i64, Option<f64>>,
) -> (Vec<f64>, Vec<CalculationTrace>) {
    let mut numbers = Vec::new();
    let mut children = Vec::new();
    for input in inputs {
        let (value, trace) = evaluate_rule(input, values);
        if let Some(value) = value {
            numbers.push(value);
        }
        children.push(trace);
    }
    (numbers, children)
}

fn aggregate<F>(
    op: &str,
    inputs: &[RuleNode],
    values: &HashMap<i64, Option<f64>>,
    calculate: F,
) -> (Option<f64>, CalculationTrace)
where
    F: Fn(&[f64]) -> f64,
{
    let (numbers, children) = collect(inputs, values);
    let value = if inputs.is_empty() || numbers.len() != inputs.len() {
        None
    } else {
        Some(calculate(&numbers))
    };
    (
        value,
        CalculationTrace {
            op: op.into(),
            value,
            detail: if value.is_some() {
                format!("{} inputs", inputs.len())
            } else {
                "One or more inputs are missing".into()
            },
            children,
        },
    )
}

fn unary<F>(
    op: &str,
    input: &RuleNode,
    values: &HashMap<i64, Option<f64>>,
    calculate: F,
) -> (Option<f64>, CalculationTrace)
where
    F: Fn(f64) -> f64,
{
    let (input_value, child) = evaluate_rule(input, values);
    let value = input_value.map(calculate);
    (
        value,
        CalculationTrace {
            op: op.into(),
            value,
            detail: op.into(),
            children: vec![child],
        },
    )
}

fn binary<F>(
    op: &str,
    left: &RuleNode,
    right: &RuleNode,
    values: &HashMap<i64, Option<f64>>,
    calculate: F,
) -> (Option<f64>, CalculationTrace)
where
    F: Fn(f64, f64) -> f64,
{
    let (left_value, left_trace) = evaluate_rule(left, values);
    let (right_value, right_trace) = evaluate_rule(right, values);
    let value = match (left_value, right_value) {
        (Some(left), Some(right)) => Some(calculate(left, right)),
        _ => None,
    };
    (
        value,
        CalculationTrace {
            op: op.into(),
            value,
            detail: op.into(),
            children: vec![left_trace, right_trace],
        },
    )
}

pub fn referenced_fields(rule: &RuleNode) -> HashSet<i64> {
    let mut fields = HashSet::new();
    collect_references(rule, &mut fields);
    fields
}

fn collect_references(rule: &RuleNode, fields: &mut HashSet<i64>) {
    match rule {
        RuleNode::Field { field_id } => {
            fields.insert(*field_id);
        }
        RuleNode::Sum { inputs }
        | RuleNode::Average { inputs }
        | RuleNode::Maximum { inputs }
        | RuleNode::Minimum { inputs }
        | RuleNode::BestN { inputs, .. }
        | RuleNode::DropLowest { inputs, .. } => {
            for input in inputs {
                collect_references(input, fields);
            }
        }
        RuleNode::Multiply { input, .. }
        | RuleNode::Scale { input, .. }
        | RuleNode::Cap { input, .. } => collect_references(input, fields),
        RuleNode::WeightedSum { items } => {
            for item in items {
                collect_references(&item.input, fields);
            }
        }
        RuleNode::Add { left, right } | RuleNode::Subtract { left, right } => {
            collect_references(left, fields);
            collect_references(right, fields);
        }
        RuleNode::Constant { .. } => {}
    }
}

pub fn validate_dependency_graph(rules: &HashMap<i64, RuleNode>) -> Result<(), String> {
    fn visit(
        field: i64,
        rules: &HashMap<i64, RuleNode>,
        visiting: &mut HashSet<i64>,
        visited: &mut HashSet<i64>,
    ) -> Result<(), String> {
        if visited.contains(&field) {
            return Ok(());
        }
        if !visiting.insert(field) {
            return Err(format!("Calculation cycle detected at field {field}"));
        }
        if let Some(rule) = rules.get(&field) {
            for dependency in referenced_fields(rule) {
                if rules.contains_key(&dependency) {
                    visit(dependency, rules, visiting, visited)?;
                }
            }
        }
        visiting.remove(&field);
        visited.insert(field);
        Ok(())
    }

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for field in rules.keys() {
        visit(*field, rules, &mut visiting, &mut visited)?;
    }
    Ok(())
}

pub fn validate_bands(bands: &[GradeBand]) -> Result<(), String> {
    if bands.is_empty() {
        return Err("A grading policy needs at least one grade band.".into());
    }
    for band in bands {
        if !band.min_percent.is_finite() || !band.max_percent.is_finite() {
            return Err(format!(
                "{} contains a non-finite boundary.",
                band.grade_label
            ));
        }
        if band.min_percent < 0.0 || band.max_percent > 100.0 {
            return Err(format!("{} must remain within 0–100%.", band.grade_label));
        }
        if band.min_percent > band.max_percent {
            return Err(format!(
                "{} has minimum greater than maximum.",
                band.grade_label
            ));
        }
    }
    for (index, left) in bands.iter().enumerate() {
        for right in bands.iter().skip(index + 1) {
            let overlaps = left.min_percent < right.max_percent
                && right.min_percent < left.max_percent
                || (left.max_percent == right.min_percent
                    && left.max_inclusive
                    && right.min_inclusive)
                || (right.max_percent == left.min_percent
                    && right.max_inclusive
                    && left.min_inclusive);
            if overlaps {
                return Err(format!(
                    "Grade bands {} and {} overlap.",
                    left.grade_label, right.grade_label
                ));
            }
        }
    }
    Ok(())
}

pub fn lookup_grade(percentage: f64, bands: &[GradeBand]) -> Option<GradeResult> {
    bands.iter().find_map(|band| {
        let above_min =
            percentage > band.min_percent || (band.min_inclusive && percentage == band.min_percent);
        let below_max =
            percentage < band.max_percent || (band.max_inclusive && percentage == band.max_percent);
        (above_min && below_max).then(|| GradeResult {
            percentage,
            grade_label: band.grade_label.clone(),
            grade_point: band.grade_point,
            result_label: band.result_label.clone(),
            color_hex: band.color_hex.clone(),
        })
    })
}

pub fn descriptive_statistics(values: &[f64]) -> (f64, f64, f64, f64, f64) {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let count = sorted.len() as f64;
    let mean = sorted.iter().sum::<f64>() / count;
    let median = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    let variance = sorted
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / count;
    (
        mean,
        median,
        sorted[0],
        sorted[sorted.len() - 1],
        variance.sqrt(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(id: i64) -> RuleNode {
        RuleNode::Field { field_id: id }
    }

    #[test]
    fn evaluates_best_scale_and_weighted_rules() {
        let values = HashMap::from([
            (1, Some(12.0)),
            (2, Some(18.0)),
            (3, Some(16.0)),
            (4, Some(78.0)),
            (5, Some(82.0)),
        ]);
        let lab = RuleNode::Scale {
            input: Box::new(RuleNode::BestN {
                count: 2,
                inputs: vec![field(1), field(2), field(3)],
            }),
            from: 40.0,
            to: 30.0,
        };
        let (lab_value, _) = evaluate_rule(&lab, &values);
        assert_eq!(lab_value, Some(25.5));

        let semester = RuleNode::WeightedSum {
            items: vec![
                WeightedInput {
                    input: field(4),
                    weight: 0.4,
                },
                WeightedInput {
                    input: field(5),
                    weight: 0.6,
                },
            ],
        };
        let (semester_value, _) = evaluate_rule(&semester, &values);
        assert!((semester_value.unwrap() - 80.4).abs() < 1e-9);
    }

    #[test]
    fn missing_input_does_not_become_zero() {
        let values = HashMap::from([(1, Some(0.0)), (2, None)]);
        let (value, _) = evaluate_rule(
            &RuleNode::Sum {
                inputs: vec![field(1), field(2)],
            },
            &values,
        );
        assert_eq!(value, None);
    }

    #[test]
    fn rejects_dependency_cycles() {
        let rules = HashMap::from([(1, field(2)), (2, field(1))]);
        assert!(validate_dependency_graph(&rules).is_err());
    }

    #[test]
    fn calculates_statistics() {
        let (mean, median, min, max, sd) = descriptive_statistics(&[60.0, 70.0, 80.0, 90.0]);
        assert_eq!(mean, 75.0);
        assert_eq!(median, 75.0);
        assert_eq!(min, 60.0);
        assert_eq!(max, 90.0);
        assert!((sd - 11.180_339_887).abs() < 1e-6);
    }
}

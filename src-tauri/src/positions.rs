use crate::app_config::OrderingSection;

pub fn between(ordering: &OrderingSection, left: Option<i64>, right: Option<i64>) -> i64 {
    let step = ordering.step;

    match (left, right) {
        (None, None) => 0,
        (Some(l), None) => l.saturating_add(step),
        (None, Some(r)) => r.saturating_sub(step),
        (Some(l), Some(r)) => {
            if l >= r {
                l.saturating_add(step)
            } else {
                l + ((r - l) / 2)
            }
        }
    }
}

pub fn gap_ok(ordering: &OrderingSection, left: i64, right: i64) -> bool {
    (right - left).abs() > ordering.min_gap
}

pub fn renormalize<T: Copy>(ordering: &OrderingSection, ordered_ids: &[T]) -> Vec<(T, i64)> {
    let mut out = Vec::with_capacity(ordered_ids.len());
    let mut pos = 0_i64;

    for id in ordered_ids {
        out.push((*id, pos));
        pos = pos.saturating_add(ordering.step);
    }

    out
}

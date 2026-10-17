use crate::model::Point;

#[derive(Debug, Clone, Copy)]
pub struct Axis<'a> {
    pub metric: &'a str,
    pub min_gain: f64,
}

impl Axis<'_> {
    fn coords(&self, p: &Point) -> (f64, f64) {
        (p.bitrate_kbps.ln(), p.score(self.metric))
    }

    fn below_or_on(&self, a: &Point, b: &Point, c: &Point) -> bool {
        let (ax, ay) = self.coords(a);
        let (bx, by) = self.coords(b);
        let (cx, cy) = self.coords(c);
        (bx - ax) * (cy - ay) - (by - ay) * (cx - ax) >= 0.0
    }
}

pub fn pareto_front(points: &[Point], axis: Axis) -> Vec<Point> {
    let mut sorted: Vec<Point> = points.to_vec();
    sorted.sort_by(|a, b| {
        a.bitrate_kbps
            .total_cmp(&b.bitrate_kbps)
            .then(b.score(axis.metric).total_cmp(&a.score(axis.metric)))
    });
    let mut front: Vec<Point> = Vec::new();
    for p in sorted {
        let gain = front
            .last()
            .map(|last| p.score(axis.metric) - last.score(axis.metric))
            .unwrap_or(f64::INFINITY);
        if gain > axis.min_gain {
            front.push(p);
        }
    }
    front
}

pub fn convex_hull(points: &[Point], axis: Axis) -> Vec<Point> {
    let mut hull: Vec<Point> = Vec::new();
    for p in pareto_front(points, axis) {
        while hull.len() >= 2 && axis.below_or_on(&hull[hull.len() - 2], &hull[hull.len() - 1], &p) {
            hull.pop();
        }
        hull.push(p);
    }
    hull
}

pub fn bitrate_for(hull: &[Point], axis: Axis, quality: f64) -> Option<f64> {
    let first = hull.first()?;
    let last = hull.last()?;
    if quality <= first.score(axis.metric) {
        return Some(first.bitrate_kbps);
    }
    if quality > last.score(axis.metric) {
        let saturated = quality - last.score(axis.metric) <= axis.min_gain;
        return saturated.then_some(last.bitrate_kbps);
    }
    hull.windows(2).find_map(|w| {
        let (x0, y0) = axis.coords(&w[0]);
        let (x1, y1) = axis.coords(&w[1]);
        (quality > y0 && quality <= y1).then(|| (x0 + (quality - y0) / (y1 - y0) * (x1 - x0)).exp())
    })
}

use super::{N, P2, V2, Curve, FiniteCurve};
use super::primitives::Segment;

pub trait Path {
    fn segments(&self) -> &[Segment];
    fn new(vec: Vec<Segment>) -> Self;

    fn scan_segments<'a>(start_offset: &mut StartOffsetState, segment: &'a Segment) -> Option<(&'a Segment, N)> {
        let pair = (segment, start_offset.0);
        start_offset.0 += segment.length;
        Some(pair)
    }

    fn segments_with_start_offsets<'a>(&'a self)
    -> ::std::iter::Scan<
        ::std::slice::Iter<Segment>,
        StartOffsetState,
        fn(&mut StartOffsetState, &'a Segment) -> Option<(&'a Segment, N)>
    > {
        return self.segments().into_iter().scan(StartOffsetState(0.0), Self::scan_segments);
    }

    fn find_on_segment(&self, distance: N) -> Option<(&Segment, N)> {
        let mut distance_covered = 0.0;
        for segment in self.segments().iter() {
            let new_distance_covered = distance_covered + segment.length();
            if new_distance_covered > distance {
                return Some((segment, distance - distance_covered));
            }
            distance_covered = new_distance_covered;
        }
        None
    }
}

pub struct StartOffsetState(N);

impl<T: Path> FiniteCurve for T {
    fn length(&self) -> N {
        self.segments().into_iter().map(|segment| segment.length()).fold(0.0, ::std::ops::Add::add)
    }

    fn along(&self, distance: N) -> P2 {
        match self.find_on_segment(distance) {
            Some((segment, distance_on_segment)) => segment.along(distance_on_segment),
            None => self.segments()[0].start
        }
    }

    fn direction_along(&self, distance: N) -> V2 {
        match self.find_on_segment(distance) {
            Some((segment, distance_on_segment)) => segment.direction_along(distance_on_segment),
            None => self.segments()[0].direction_along(0.0)
        }
    }

    fn start_direction(&self) -> V2 {
        self.segments()[0].start_direction()
    }

    fn end_direction(&self) -> V2 {
        self.segments().last().unwrap().end_direction()
    }
}

impl<T: Path> Curve for T {
    fn project(&self, point: P2) -> Option<N> {
        self.segments_with_start_offsets().filter_map(|pair: (&Segment, N)| {
            let (segment, start_offset) = pair;
            let offset_on_segment = segment.project(point);
            match offset_on_segment {
                Some(offset) => Some(start_offset + offset),
                None => None
            }
        }).next()
    }

    fn includes(&self, point: P2) -> bool {
        self.segments().into_iter().any(|segment| segment.includes(point))
    }

    fn distance_to(&self, point: P2) -> N {
        self.segments().into_iter().fold(None, |min, segment| {
            let distance = segment.distance_to(point);
            if min.is_some() && distance < min.unwrap() {Some(distance)} else {min}
        }).unwrap()
    }
}
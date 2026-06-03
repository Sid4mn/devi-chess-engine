use crate::types::Move;

// completed root-move results keyed by stable move index, merged on the driver so nothing's shared.
pub struct CompletedMoves {
    inner: Vec<Option<(Move, i32)>>,
}

impl CompletedMoves {
    pub fn new(n: usize) -> Self {
        Self { inner: vec![None; n] }
    }

    pub fn record(&mut self, idx: usize, mv: Move, score: i32) {
        self.inner[idx] = Some((mv, score));
    }

    // first move with the max score - same first-max rule the search drivers reduce with.
    pub fn best(&self) -> Option<(Move, i32)> {
        self.inner
            .iter()
            .flatten()
            .copied()
            .reduce(|best, cur| if cur.1 > best.1 { cur } else { best })
    }
}

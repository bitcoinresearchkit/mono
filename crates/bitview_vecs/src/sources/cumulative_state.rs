/// Writer-owned cumulative values. A length change reloads the persisted checkpoint.
pub struct CumulativeState<R> {
    last: Option<(usize, R)>,
}

impl<R> Default for CumulativeState<R> {
    fn default() -> Self {
        Self { last: None }
    }
}

impl<R: Clone + Default> CumulativeState<R> {
    pub fn accumulate(
        &mut self,
        len: usize,
        load: impl FnOnce() -> Option<R>,
        add: impl FnOnce(&mut R),
    ) -> R {
        let mut values = match self.last.take() {
            Some((cached_len, values)) if cached_len == len => values,
            _ => load().unwrap_or_default(),
        };
        add(&mut values);
        self.last = Some((len + 1, values.clone()));
        values
    }
}

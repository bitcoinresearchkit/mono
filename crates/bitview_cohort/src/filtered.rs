use super::Filter;

pub trait Filtered {
    fn filter(&self) -> &Filter;

    fn is_all(&self) -> bool {
        self.filter().is_all()
    }

    fn includes_first_day(&self) -> bool {
        self.filter().includes_first_day()
    }
}

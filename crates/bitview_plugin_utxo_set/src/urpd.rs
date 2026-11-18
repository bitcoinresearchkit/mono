use bitview_distribution::walk::{Block, Consumer, Origins, Sources};
use bitview_urpd::UrpdHistory;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::AnyVec;

/// The URPD history as a consumer of the walk: each block's moves between ages become its cell
/// changes. It holds the exit lock wherever the cells publish (a reset, a cut, a commit).
pub struct UrpdConsumer<'a> {
    urpd: &'a mut UrpdHistory,
    exit: &'a Exit,
    start: usize,
}

impl<'a> UrpdConsumer<'a> {
    /// Readies `urpd` for a walk over `sources`, from `from` at most.
    pub fn new(
        urpd: &'a mut UrpdHistory,
        sources: Sources<'_>,
        from: Height,
        exit: &'a Exit,
    ) -> Result<Self> {
        let end = sources.inputs(Version::ZERO).len;
        let _lock = exit.lock();
        let start = urpd.begin(
            usize::from(from).min(end),
            sources.prices.version(),
            sources.timestamps.version(),
            sources.history.versions(),
        )?;
        Ok(Self { urpd, exit, start })
    }
}

impl Consumer for UrpdConsumer<'_> {
    fn start(&self) -> usize {
        self.start
    }

    fn kept(&self) -> bool {
        self.urpd.kept()
    }

    fn restore(&mut self, origins: &Origins<'_>) -> Result<()> {
        let _lock = self.exit.lock();
        self.urpd
            .restore(self.start, origins.hash, &origins.prices[..self.start])
    }

    fn apply(&mut self, block: &Block<'_>) -> Result<()> {
        let moves = block.moves().flat_map(|moved| {
            let sats = moved.sats as i64;
            let left = moved.from.map(|age| (age, moved.origin, -sats));
            let arrived = moved.to.map(|age| (age, moved.origin, sats));
            left.into_iter().chain(arrived)
        });
        self.urpd
            .push(block.hash, &block.prices[..=block.height], moves)
    }

    fn save(&mut self) -> Result<()> {
        let _lock = self.exit.lock();
        self.urpd.commit()
    }
}

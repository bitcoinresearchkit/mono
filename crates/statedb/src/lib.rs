//! UTXO supply and count history aggregated by creation height, not individual outpoints.
//! Creation facts and grouped spends reconstruct the state from periodic full snapshots.
//! `CellHistory` keeps any per-block view derived from it as cell changes and checkpoints.
//! No prices, cohorts, analytics, or vecdb storage belong here.
mod amount;
mod block_diff;
mod cells;
mod columns;
mod creations;
mod cursor;
mod diffs;
mod history;
mod journal;
mod journal_reader;
mod reader;
mod snapshot;
mod snapshots;
mod spends;
mod state;
#[cfg(test)]
mod tests;
mod util;
mod view;

pub use amount::Amount;
pub use block_diff::BlockDiff;
pub use cells::CellHistory;
pub use creations::Creations;
pub use cursor::Cursor;
pub use diffs::Diffs;
pub use history::{History, Live, SNAPSHOT_INTERVAL};
pub use reader::Reader;
pub use spends::Spends;
pub use state::State;

pub use view::View;

//! UTXO supply and count history aggregated by creation height, not individual outpoints.
//! Creation facts and grouped spends reconstruct the state from periodic full snapshots.
//! No prices, cohorts, analytics, or vecdb storage belong here.
mod amount;
mod block_diff;
mod creations;
mod cursor;
mod history;
mod journal;
mod journal_reader;
mod reader;
mod snapshot;
mod snapshots;
mod spends;
mod state;
mod util;
mod view;

#[cfg(test)]
mod state_tests;

pub use amount::Amount;
pub use block_diff::BlockDiff;
pub use creations::Creations;
pub use cursor::Cursor;
pub use history::History;
pub use reader::Reader;
pub use spends::Spends;
pub use state::State;

pub use view::View;

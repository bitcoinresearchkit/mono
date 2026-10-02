use super::chain_fixture::{default_first, run_genesis};
#[cfg(feature = "series")]
use brk_types::Index;

#[test]
fn genesis_series_and_broadcast_contracts() {
    run_genesis(default_first(), |fixture| async move {
        #[cfg(feature = "series")]
        {
            assert_eq!(
                fixture
                    .query
                    .sync(|q| q.len(&"timestamp".into(), Index::Height))
                    .unwrap(),
                1
            );
            assert_eq!(
                fixture
                    .query
                    .sync(|q| q.len(&"timestamp_monotonic".into(), Index::Height))
                    .unwrap(),
                0
            );
            super::series_admission::check(&fixture.state).await;
        }
        super::broadcast::check(&fixture.query).await;
    });
}

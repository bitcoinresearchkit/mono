use std::{collections::HashSet, sync::Arc};

use crate::{Database, Error, Region, Result};

mod inner;

pub(crate) use inner::RegionGroupInner;

/// Opaque handle keeping a set of regions in one contiguous allocation.
///
/// Group membership remains active only while this handle is retained. Writes
/// to member regions must be serialized by the caller while the group is active.
#[derive(Debug, Clone)]
#[must_use = "region grouping remains active while this handle is retained"]
pub struct RegionGroup {
    _inner: Arc<RegionGroupInner>,
}

impl Database {
    /// Groups existing regions in the supplied order.
    ///
    /// The group is moved into one contiguous allocation immediately. If any
    /// member later grows, every member's reservation grows and the complete
    /// group moves as a unit. The caller must serialize writes to members while
    /// retaining the returned handle.
    pub fn group_regions(&self, regions: &[Region]) -> Result<RegionGroup> {
        if regions.is_empty() {
            return Err(Error::EmptyRegionGroup);
        }

        let mut indices = HashSet::with_capacity(regions.len());
        for region in regions {
            if !self.ptr_eq(&region.db()) {
                return Err(Error::RegionGroupDatabaseMismatch);
            }
            if !indices.insert(region.index()) {
                return Err(Error::RegionAlreadyGrouped {
                    id: region.meta().id().to_owned(),
                });
            }
        }

        if let Some(group) = regions[0].group()
            && group.matches(regions)
            && regions.iter().all(|region| {
                region
                    .group()
                    .is_some_and(|other| Arc::ptr_eq(&group, &other))
            })
        {
            return Ok(RegionGroup { _inner: group });
        }
        if let Some(region) = regions.iter().find(|region| region.group().is_some()) {
            return Err(Error::RegionAlreadyGrouped {
                id: region.meta().id().to_owned(),
            });
        }

        let group = Arc::new(RegionGroupInner::new(regions)?);
        for region in regions {
            region.set_group(Arc::downgrade(&group));
        }
        Ok(RegionGroup { _inner: group })
    }

    #[inline]
    fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

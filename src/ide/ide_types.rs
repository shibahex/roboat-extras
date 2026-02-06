use bytes::Bytes;

use crate::catalog::AssetType;

#[allow(missing_docs)]
#[derive(Debug, Clone)]
/// An struct to Make a new animation
pub struct NewStudioAsset {
    pub name: String,
    pub description: String,
    pub asset_type: AssetType,
    pub asset_data: Bytes,
    pub group_id: Option<u64>,
    pub place_id: Option<u64>,
}

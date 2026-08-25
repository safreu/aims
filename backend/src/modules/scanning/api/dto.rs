use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::scanning::domain::QrAction;

#[derive(Debug, Deserialize)]
pub struct CreateQrActionRequest {
    pub item_id: Uuid,
    pub kind: String,
    pub amount: u32,
}

#[derive(Debug, Serialize)]
pub struct CreateQrActionResponse {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct QrActionReponse {
    pub id: Uuid,
    pub item_id: Uuid,
    pub kind: String,
    pub amount: u32,
}

impl From<QrAction> for QrActionReponse {
    fn from(value: QrAction) -> Self {
        Self {
            id: value.id().into_uuid(),
            item_id: value.item_id().into_uuid(),
            kind: value.kind().as_str().to_owned(),
            amount: value.amount(),
        }
    }
}

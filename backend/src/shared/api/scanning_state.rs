use std::sync::Arc;

use crate::modules::scanning::application::{
    CreateQrActionService, ExecuteQrActionService, ListQrActionsService, RevokeQrActionService,
};

#[derive(Clone)]
pub struct ScanningState {
    pub create_qr_action: Arc<CreateQrActionService>,
    pub list_qr_actions: Arc<ListQrActionsService>,
    pub revoke_qr_action: Arc<RevokeQrActionService>,
    pub execute_qr_action: Arc<ExecuteQrActionService>,
}

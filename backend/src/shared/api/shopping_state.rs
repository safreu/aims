use std::sync::Arc;

use crate::modules::shopping::application::{
    CreateCustomShoppingEntryService, DeleteCustomShoppingEntryService, DismissShoppingItemService,
    ListShoppingService, SetCheckedService, SetCustomShoppingEntryCheckedService, SetNoteService,
    SetShoppingQuantityService, UpdateCustomShoppingEntryService,
};

#[derive(Clone)]
pub struct ShoppingState {
    pub list_shopping: Arc<ListShoppingService>,
    pub create_custom_shopping_entry: Arc<CreateCustomShoppingEntryService>,
    pub delete_custom_shopping_entry: Arc<DeleteCustomShoppingEntryService>,
    pub set_custom_shopping_entry_checked: Arc<SetCustomShoppingEntryCheckedService>,
    pub update_custom_shopping_entry: Arc<UpdateCustomShoppingEntryService>,

    pub set_shopping_quantity: Arc<SetShoppingQuantityService>,
    pub set_checked: Arc<SetCheckedService>,
    pub set_note: Arc<SetNoteService>,
    pub dismiss_shopping_item: Arc<DismissShoppingItemService>,
}

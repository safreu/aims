use backend::modules::{
    accounts::domain::{DisplayName, Email, PasswordHash, User, UserId},
    devices::domain::{Device, DeviceId, DeviceKind, DeviceName},
    households::domain::{Household, HouseholdId, HouseholdKind, HouseholdName},
    inventory::domain::{
        Category, CategoryId, CategoryName, InventoryItem, InventoryItemId, InventoryItemName,
        InventoryPriority,
    },
};
use chrono::{DateTime, SubsecRound, Utc};

#[allow(unused)]
pub struct UserTestBuilder {
    id: UserId,
    email: String,
    display_name: String,
    password_hash: PasswordHash,
}

#[allow(unused)]
impl UserTestBuilder {
    pub fn new() -> Self {
        Self {
            id: UserId::new(),
            email: "test@email.com".to_owned(),
            display_name: "Test name".to_owned(),
            password_hash: PasswordHash::from_encoded("test-hash")
                .expect("Password hash should be valid"),
        }
    }

    pub fn id(mut self, id: UserId) -> Self {
        self.id = id;
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = email.into();
        self
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    pub fn build(self) -> User {
        User::new(
            self.id,
            Email::parse(&self.email).expect("Email should be valid"),
            DisplayName::parse(&self.display_name).expect("Test name should be valid"),
            self.password_hash,
        )
    }
}

#[allow(unused)]
pub struct HouseholdTestBuilder {
    id: HouseholdId,
    name: String,
    kind: HouseholdKind,
    personal_owner_id: Option<UserId>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[allow(unused)]
impl HouseholdTestBuilder {
    pub fn new() -> Self {
        let now = Utc::now().trunc_subsecs(6);

        Self {
            id: HouseholdId::new(),
            name: "Test household".to_owned(),
            kind: HouseholdKind::Shared,
            personal_owner_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(mut self, id: HouseholdId) -> Self {
        self.id = id;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn shared(mut self) -> Self {
        self.kind = HouseholdKind::Shared;
        self.personal_owner_id = None;
        self
    }

    pub fn personal(mut self, owner_id: UserId) -> Self {
        self.kind = HouseholdKind::Personal;
        self.personal_owner_id = Some(owner_id);
        self
    }

    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
        self.updated_at = updated_at;
        self
    }

    pub fn build(self) -> Household {
        Household::new(
            self.id,
            HouseholdName::parse(&self.name).expect("Test Name should be valid"),
            self.kind,
            self.personal_owner_id,
            self.created_at,
            self.updated_at,
        )
        .expect("Test household should be valid")
    }
}

#[allow(unused)]
pub struct CategoryTestBuilder {
    id: CategoryId,
    household_id: HouseholdId,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[allow(unused)]
impl CategoryTestBuilder {
    pub fn new(household_id: HouseholdId) -> Self {
        let now = Utc::now().trunc_subsecs(6);

        Self {
            id: CategoryId::new(),
            household_id,
            name: "Test category".to_owned(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(mut self, id: CategoryId) -> Self {
        self.id = id;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn build(self) -> Category {
        Category::new(
            self.id,
            self.household_id,
            CategoryName::parse(&self.name).expect("Name should be valid"),
            self.created_at,
            self.updated_at,
        )
    }
}

#[allow(unused)]
pub struct InventoryItemTestBuilder {
    id: InventoryItemId,
    household_id: HouseholdId,
    category_id: Option<CategoryId>,
    name: String,
    current_stock: u32,
    reorder_threshold: u32,
    priority: InventoryPriority,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[allow(unused)]
impl InventoryItemTestBuilder {
    pub fn new(household_id: HouseholdId) -> Self {
        let now = Utc::now().trunc_subsecs(6);

        Self {
            id: InventoryItemId::new(),
            household_id,
            category_id: None,
            name: "Test item".to_owned(),
            current_stock: 0,
            reorder_threshold: 0,
            priority: InventoryPriority::Default,
            created_at: now,
            updated_at: now,
        }
    }
    pub fn id(mut self, id: InventoryItemId) -> Self {
        self.id = id;
        self
    }

    pub fn category(mut self, category_id: CategoryId) -> Self {
        self.category_id = Some(category_id);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn current_stock(mut self, current_stock: u32) -> Self {
        self.current_stock = current_stock;
        self
    }

    pub fn reorder_threshold(mut self, reorder_threshold: u32) -> Self {
        self.reorder_threshold = reorder_threshold;
        self
    }

    pub fn priority(mut self, priority: InventoryPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
        self.updated_at = updated_at;
        self
    }

    pub fn build(self) -> InventoryItem {
        InventoryItem::new(
            self.id,
            self.household_id,
            self.category_id,
            InventoryItemName::parse(&self.name).expect("Inventory item name should be valid"),
            self.current_stock,
            self.reorder_threshold,
            self.priority,
            self.created_at,
            self.updated_at,
        )
    }
}

#[allow(unused)]
pub struct DeviceTestBuilder {
    id: DeviceId,
    household_id: HouseholdId,
    name: String,
    kind: DeviceKind,
    revoked_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[allow(unused)]
impl DeviceTestBuilder {
    pub fn new(household_id: HouseholdId) -> Self {
        let now = Utc::now().trunc_subsecs(6);

        Self {
            id: DeviceId::new(),
            household_id,
            name: "Test device".to_owned(),
            kind: DeviceKind::Scanner,
            revoked_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(mut self, id: DeviceId) -> Self {
        self.id = id;
        self
    }

    pub fn household_id(mut self, household_id: HouseholdId) -> Self {
        self.household_id = household_id;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn kind(mut self, kind: DeviceKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn revoked_at(mut self, revoked_at: Option<DateTime<Utc>>) -> Self {
        self.revoked_at = revoked_at;
        self
    }

    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
        self.updated_at = updated_at;
        self
    }

    pub fn build(self) -> Device {
        Device::new_with_revoked_at(
            self.id,
            self.household_id,
            DeviceName::parse(&self.name).expect("Device name should be valid"),
            self.kind,
            self.revoked_at,
            self.created_at,
            self.updated_at,
        )
    }
}

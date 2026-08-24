# aims Manual API Commands

These commands assume the backend is running at `http://127.0.0.1:3000`.

```bash
export BASE_URL="http://127.0.0.1:3000"
```

## Health check

```bash
curl -i "$BASE_URL/api/v1/health"
```

Expected status: `200 OK`.

## Register a user

```bash
curl -i \
  -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "samuel@example.com",
    "display_name": "Samuel",
    "password": "SuperSecretPassword123!"
  }'
```

Expected status: `201 Created`.

Example response:

```json
{
  "id": "<user-uuid>"
}
```

## Register the same email again

Run the registration command again:

```bash
curl -i \
  -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "samuel@example.com",
    "display_name": "Samuel",
    "password": "SuperSecretPassword123!"
  }'
```

Expected status: `409 Conflict`.

## Register with an invalid email

```bash
curl -i \
  -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "invalid-email",
    "display_name": "Samuel",
    "password": "SuperSecretPassword123!"
  }'
```

Expected status: `400 Bad Request`.

## Register with an empty display name

```bash
curl -i \
  -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "another@example.com",
    "display_name": "",
    "password": "SuperSecretPassword123!"
  }'
```

Expected status: `400 Bad Request`.

## Register with a whitespace-only display name

```bash
curl -i \
  -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "whitespace@example.com",
    "display_name": "   ",
    "password": "SuperSecretPassword123!"
  }'
```

Expected status: `400 Bad Request`.

## Log in

```bash
curl -i \
  -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "samuel@example.com",
    "password": "SuperSecretPassword123!"
  }'
```

Expected status: `200 OK`.

The response should include a `Set-Cookie` header containing the session cookie.

## Log in and save the session cookie

```bash
curl -i \
  -c cookies.txt \
  -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "samuel@example.com",
    "password": "SuperSecretPassword123!"
  }'
```

The cookie is stored in `cookies.txt` and can later be sent with:

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/<protected-route>"
```

Replace `<protected-route>` after adding the next authenticated endpoint.

## Log in with a wrong password

```bash
curl -i \
  -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "samuel@example.com",
    "password": "wrong-password"
  }'
```

Expected status: `401 Unauthorized`.

## Log in with an unknown email

```bash
curl -i \
  -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "unknown@example.com",
    "password": "SuperSecretPassword123!"
  }'
```

Expected status: `401 Unauthorized`.

## Inspect the stored cookie

```bash
cat cookies.txt
```

The raw session token should only appear in the cookie file and HTTP response. The database should contain only its hash.

## Create a personal household

This route requires authentication. Log in and save the session cookie first.

```bash
curl -i \
  -b cookies.txt \
  -X POST "$BASE_URL/api/v1/households" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Household",
    "kind": "personal"
  }'
```

Expected status: `201 Created`.

Example response:

```json
{
  "id": "<household-uuid>"
}
```

## Create a shared household

```bash
curl -i \
  -b cookies.txt \
  -X POST "$BASE_URL/api/v1/households" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Shared Household",
    "kind": "shared"
  }'
```

Expected status: `201 Created`.

Example response:

```json
{
  "id": "<household-uuid>"
}
```

## Create a second personal household

After successfully creating a personal household, run:

```bash
curl -i \
  -b cookies.txt \
  -X POST "$BASE_URL/api/v1/households" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Another Personal Household",
    "kind": "personal"
  }'
```

Expected status: `409 Conflict`.

## Create a household with an invalid name

```bash
curl -i \
  -b cookies.txt \
  -X POST "$BASE_URL/api/v1/households" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "   ",
    "kind": "shared"
  }'
```

Expected status: `400 Bad Request`.

## Create a household with an invalid kind

```bash
curl -i \
  -b cookies.txt \
  -X POST "$BASE_URL/api/v1/households" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Household",
    "kind": "invalid"
  }'
```

Expected status: `400 Bad Request`.

## Create a household without authentication

```bash
curl -i \
  -X POST "$BASE_URL/api/v1/households" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Household",
    "kind": "shared"
  }'
```

Expected status: `401 Unauthorized`.

## List households

Returns all households the authenticated user belongs to.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/households"
```

Expected status: `200 OK`.

Example response:

```json
[
  {
    "id": "<household-uuid>",
    "name": "My Household",
    "kind": "personal"
  },
  {
    "id": "<household-uuid>",
    "name": "Shared Household",
    "kind": "shared"
  }
]
```

## List households without authentication

```bash
curl -i \
  "$BASE_URL/api/v1/households"
```

Expected status: `401 Unauthorized`.

## Get a household by ID

Returns a household if the authenticated user is a member.

Replace `<household-uuid>` with the ID returned when creating or listing households.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/households/<household-uuid>"
```

Expected status: `200 OK`.

Example response:

```json
{
  "id": "<household-uuid>",
  "name": "My Household",
  "kind": "personal"
}
```

## Get an unknown household

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/households/00000000-0000-0000-0000-000000000000"
```

Expected status: `404 Not Found`.

## Get a household without membership

Use the ID of a household belonging to another user.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/households/<other-household-uuid>"
```

Expected status: `403 Forbidden`.

## Get a household without authentication

```bash
curl -i \
  "$BASE_URL/api/v1/households/<household-uuid>"
```

Expected status: `401 Unauthorized`.

## Add a member to a shared household

The authenticated user must be an owner of the household.

Replace `<household-uuid>` with the ID of a shared household.

```bash
curl -i \
  -b cookies.txt \
  -X POST "$BASE_URL/api/v1/households/<household-uuid>/members" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "member@example.com"
  }'
```

Expected status: `204 No Content`.

## Add an unknown user to a household

```bash
curl -i \
  -b cookies.txt \
  -X POST "$BASE_URL/api/v1/households/<household-uuid>/members" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "unknown@example.com"
  }'
```

Expected status: `404 Not Found`.

## Add an existing member again

```bash
curl -i \
  -b cookies.txt \
  -X POST "$BASE_URL/api/v1/households/<household-uuid>/members" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "member@example.com"
  }'
```

Expected status: `409 Conflict`.

## Add a member to a personal household

```bash
curl -i \
  -b cookies.txt \
  -X POST "$BASE_URL/api/v1/households/<personal-household-uuid>/members" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "member@example.com"
  }'
```

Expected status: `409 Conflict`.

## Add a member without owner permission

Use a session belonging to a household member who is not an owner.

```bash
curl -i \
  -b cookies.txt \
  -X POST "$BASE_URL/api/v1/households/<household-uuid>/members" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "another@example.com"
  }'
```

Expected status: `403 Forbidden`.

## Add a member without authentication

```bash
curl -i \
  -X POST "$BASE_URL/api/v1/households/<household-uuid>/members" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "member@example.com"
  }'
```

Expected status: `401 Unauthorized`.

## List household members

Returns all members of a household. The authenticated user must belong to the household.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/households/<household-uuid>/members"
```

Expected status: `200 OK`.

Example response:

```json
[
  {
    "user_id": "<user-uuid>",
    "display_name": "Samuel",
    "role": "owner"
  },
  {
    "user_id": "<user-uuid>",
    "display_name": "Another User",
    "role": "member"
  }
]
```

## List members of an unknown household

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/households/00000000-0000-0000-0000-000000000000/members"
```

Expected status: `404 Not Found`.

## List household members without membership

Use the ID of a household the authenticated user does not belong to.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/households/<other-household-uuid>/members"
```

Expected status: `403 Forbidden`.

## List household members without authentication

```bash
curl -i \
  "$BASE_URL/api/v1/households/<household-uuid>/members"
```

Expected status: `401 Unauthorized`.

## Remove a household member

The authenticated user must either be the member being removed or an owner of the household.

```bash
curl -i \
  -b cookies.txt \
  -X DELETE \
  "$BASE_URL/api/v1/households/<household-uuid>/members/<member-user-uuid>"
```

Expected status: `204 No Content`.

## Leave a household

A normal household member can remove themselves.

```bash
curl -i \
  -b cookies.txt \
  -X DELETE \
  "$BASE_URL/api/v1/households/<household-uuid>/members/<your-user-uuid>"
```

Expected status: `204 No Content`.

## Remove a member without permission

Use a session belonging to a normal member and try to remove another member.

```bash
curl -i \
  -b cookies.txt \
  -X DELETE \
  "$BASE_URL/api/v1/households/<household-uuid>/members/<other-member-user-uuid>"
```

Expected status: `403 Forbidden`.

## Remove an unknown member

```bash
curl -i \
  -b cookies.txt \
  -X DELETE \
  "$BASE_URL/api/v1/households/<household-uuid>/members/00000000-0000-0000-0000-000000000000"
```

Expected status: `404 Not Found`.

## Remove the household owner

```bash
curl -i \
  -b cookies.txt \
  -X DELETE \
  "$BASE_URL/api/v1/households/<household-uuid>/members/<owner-user-uuid>"
```

Expected status: `409 Conflict`.

## Remove a household member without authentication

```bash
curl -i \
  -X DELETE \
  "$BASE_URL/api/v1/households/<household-uuid>/members/<member-user-uuid>"
```

Expected status: `401 Unauthorized`.

## Rename a household

The authenticated user must be allowed to modify the household.

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/households/<household-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Renamed Household"
  }'
```

Expected status: `204 No Content`.

## Rename a household with an invalid name

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/households/<household-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "   "
  }'
```

Expected status: `400 Bad Request`.

## Rename an unknown household

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/households/00000000-0000-0000-0000-000000000000" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Renamed Household"
  }'
```

Expected status: `404 Not Found`.

## Rename a household without permission

Use a session belonging to a normal member of a shared household.

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/households/<household-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Not Allowed"
  }'
```

Expected status: `403 Forbidden`.

## Rename a household without authentication

```bash
curl -i \
  -X PATCH \
  "$BASE_URL/api/v1/households/<household-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Renamed Household"
  }'
```

Expected status: `401 Unauthorized`.

## Create an inventory item

Creates an inventory item in a household. The authenticated user must be a member of the household.

Replace `<household-uuid>` with the ID of the household.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Milk",
    "current_stock": 2,
    "reorder_threshold": 1,
    "priority": "high"
  }'
```

Expected status: `201 Created`.

Example response:

```json
{
  "id": "<inventory-item-uuid>"
}
```

The `priority` field is optional. If omitted, it defaults to `default`.

Valid priority values:

- `default`
- `low`
- `medium`
- `high`

The `category_id` field is optional.

### Create an inventory item with a category

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items" \
  -H "Content-Type: application/json" \
  -d '{
    "category_id": "<category-uuid>",
    "name": "Milk",
    "current_stock": 2,
    "reorder_threshold": 1,
    "priority": "high"
  }'
```

Expected status: `201 Created`.

### Create a duplicate active inventory item

After creating an item called `Milk`, try creating another active item with the same normalized name:

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "milk",
    "current_stock": 1,
    "reorder_threshold": 0
  }'
```

Expected status: `409 Conflict`.

### Create an inventory item with an invalid name

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "   ",
    "current_stock": 1,
    "reorder_threshold": 0
  }'
```

Expected status: `400 Bad Request`.

### Create an inventory item without authentication

```bash
curl -i \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Milk",
    "current_stock": 1,
    "reorder_threshold": 0
  }'
```

Expected status: `401 Unauthorized`.

## List inventory items

Returns all active inventory items for a household. The authenticated user must be a member of the household.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items"
```

Expected status: `200 OK`.

Example response:

```json
[
  {
    "id": "<inventory-item-uuid>",
    "name": "Milk",
    "category": {
      "id": "<category-uuid>",
      "name": "Food"
    },
    "current_stock": 2,
    "reorder_threshold": 1,
    "priority": "high",
    "shopping_quantity": 0
  }
]
```

Inventory items without a category contain:

```json
{
  "category": null
}
```

Archived inventory items are not returned by this endpoint.

If the household has no active inventory items, the endpoint returns:

```json
[]
```

## List inventory items of an unknown household

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/00000000-0000-0000-0000-000000000000/items"
```

Expected status: `404 Not Found`.

## List inventory items without membership

Use the ID of a household the authenticated user does not belong to.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/<other-household-uuid>/items"
```

Expected status: `403 Forbidden`.

## List inventory items without authentication

```bash
curl -i \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items"
```

Expected status: `401 Unauthorized`.

## Create a category

Creates a category for a household. The authenticated user must be a member of the household.

Replace `<household-uuid>` with the ID of the household.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Food"
  }'
```

Expected status: `201 Created`.

Example response:

```json
{
  "id": "<category-uuid>"
}
```

Category names are unique within a household after normalization.

## Create a duplicate category

After creating a category called `Food`, try creating another category with the same normalized name:

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "food"
  }'
```

Expected status: `409 Conflict`.

## Create a category with an invalid name

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "   "
  }'
```

Expected status: `400 Bad Request`.

## Create a category without authentication

```bash
curl -i \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Food"
  }'
```

Expected status: `401 Unauthorized`.## Create a category

Creates a category for a household. The authenticated user must be a member of the household.

Replace `<household-uuid>` with the ID of the household.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Food"
  }'
```

Expected status: `201 Created`.

Example response:

```json
{
  "id": "<category-uuid>"
}
```

Category names are unique within a household after normalization.

## Create a duplicate category

After creating a category called `Food`, try creating another category with the same normalized name:

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "food"
  }'
```

Expected status: `409 Conflict`.

## Create a category with an invalid name

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "   "
  }'
```

Expected status: `400 Bad Request`.

## Create a category without authentication

```bash
curl -i \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Food"
  }'
```

Expected status: `401 Unauthorized`.

## List categories

Returns all categories for a household. The authenticated user must be a member of the household.

Replace `<household-uuid>` with the ID of the household.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories"
```

Expected status: `200 OK`.

Example response:

```json
[
  {
    "id": "<category-uuid>",
    "name": "Food"
  },
  {
    "id": "<category-uuid>",
    "name": "Cleaning"
  }
]
```

If the household has no categories, the endpoint returns an empty list:

```json
[]
```

## List categories of an unknown household

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/00000000-0000-0000-0000-000000000000/categories"
```

Expected status: `404 Not Found`.

## List categories without membership

Use the ID of a household the authenticated user does not belong to.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/<other-household-uuid>/categories"
```

Expected status: `403 Forbidden`.

## List categories without authentication

```bash
curl -i \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories"
```

Expected status: `401 Unauthorized`.

## Delete a category

Deletes a category from a household. The authenticated user must be a member of the household.

Deleting a category does not delete inventory items assigned to it. Their `category_id` is set to `null`.

Replace `<household-uuid>` and `<category-uuid>` with the corresponding IDs.

```bash
curl -i \
  -b cookies.txt \
  -X DELETE \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories/<category-uuid>"
```

Expected status: `204 No Content`.

## Delete an unknown category

```bash
curl -i \
  -b cookies.txt \
  -X DELETE \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories/00000000-0000-0000-0000-000000000000"
```

Expected status: `404 Not Found`.

## Delete a category without membership

Use a category belonging to a household the authenticated user does not belong to.

```bash
curl -i \
  -b cookies.txt \
  -X DELETE \
  "$BASE_URL/api/v1/inventory/<other-household-uuid>/categories/<category-uuid>"
```

Expected status: `403 Forbidden`.

## Delete a category without authentication

```bash
curl -i \
  -X DELETE \
  "$BASE_URL/api/v1/inventory/<household-uuid>/categories/<category-uuid>"
```

Expected status: `401 Unauthorized`.

## Get an inventory item

Returns one active inventory item from a household. The authenticated user must be a member of the household.

Replace `<household-uuid>` and `<inventory-item-uuid>` with the corresponding IDs.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>"
```

Expected status: `200 OK`.

Example response:

```json
{
  "id": "<inventory-item-uuid>",
  "name": "Milk",
  "category": {
    "id": "<category-uuid>",
    "name": "Food"
  },
  "current_stock": 2,
  "reorder_threshold": 1,
  "priority": "high",
  "shopping_quantity": 0
}
```

An inventory item without a category contains:

```json
{
  "category": null
}
```

Archived inventory items are not returned by this endpoint.

## Get an unknown inventory item

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/00000000-0000-0000-0000-000000000000"
```

Expected status: `404 Not Found`.

## Get an inventory item without membership

Use an inventory item belonging to a household the authenticated user does not belong to.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/<other-household-uuid>/items/<inventory-item-uuid>"
```

Expected status: `403 Forbidden`.

## Get an inventory item without authentication

```bash
curl -i \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>"
```

Expected status: `401 Unauthorized`.

## Update an inventory item

Updates the metadata of an active inventory item. The authenticated user must be a member of the household.

At least one field must be provided.

The following fields can be updated:

- `name`
- `category_id`
- `reorder_threshold`
- `priority`

Replace `<household-uuid>` and `<inventory-item-uuid>` with the corresponding IDs.

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Oat Milk",
    "reorder_threshold": 3,
    "priority": "medium"
  }'
```

Expected status: `204 No Content`.

Fields that are omitted remain unchanged.

### Change the category

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "category_id": "<category-uuid>"
  }'
```

Expected status: `204 No Content`.

### Remove the category

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "category_id": null
  }'
```

Expected status: `204 No Content`.

### Update an inventory item without changes

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>" \
  -H "Content-Type: application/json" \
  -d '{}'
```

Expected status: `400 Bad Request`.

### Update an inventory item with an invalid priority

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "priority": "urgent"
  }'
```

Expected status: `400 Bad Request`.

### Update an unknown inventory item

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/00000000-0000-0000-0000-000000000000" \
  -H "Content-Type: application/json" \
  -d '{
    "priority": "high"
  }'
```

Expected status: `404 Not Found`.

### Update an inventory item without authentication

```bash
curl -i \
  -X PATCH \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "priority": "high"
  }'
```

Expected status: `401 Unauthorized`.

## Archive an inventory item

Archives an active inventory item. The authenticated user must be a member of the household.

Replace `<household-uuid>` and `<inventory-item-uuid>` with the corresponding IDs.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/archive"
```

Expected status: `204 No Content`.

Archived inventory items are no longer returned by the active inventory list or active item detail endpoints.

### Archive an already archived inventory item

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/archive"
```

Expected status: `409 Conflict`.

### Archive an unknown inventory item

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/00000000-0000-0000-0000-000000000000/archive"
```

Expected status: `404 Not Found`.

### Archive an inventory item without authentication

```bash
curl -i \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/archive"
```

Expected status: `401 Unauthorized`.

## Restore an inventory item

Restores an archived inventory item. The authenticated user must be a member of the household.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/restore"
```

Expected status: `204 No Content`.

### Restore an active inventory item

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/restore"
```

Expected status: `409 Conflict`.

### Restore an inventory item when an active item with the same name exists

If another active inventory item with the same normalized name exists, restoring the archived item is rejected.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/restore"
```

Expected status: `409 Conflict`.

### Restore an unknown inventory item

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/00000000-0000-0000-0000-000000000000/restore"
```

Expected status: `404 Not Found`.

### Restore an inventory item without authentication

```bash
curl -i \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/restore"
```

Expected status: `401 Unauthorized`.

## Increase inventory stock

Increases the stock of an active inventory item. The authenticated user must be a member of the household.

Replace `<household-uuid>` and `<inventory-item-uuid>` with the corresponding IDs.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/increase" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 2
  }'
```

Expected status: `204 No Content`.

### Increase inventory stock by zero

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/increase" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 0
  }'
```

Expected status: `400 Bad Request`.

### Increase stock of an unknown inventory item

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/00000000-0000-0000-0000-000000000000/increase" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 1
  }'
```

Expected status: `404 Not Found`.

### Increase stock without authentication

```bash
curl -i \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/increase" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 1
  }'
```

Expected status: `401 Unauthorized`.

## Decrease inventory stock

Decreases the stock of an active inventory item. The authenticated user must be a member of the household.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/decrease" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 1
  }'
```

Expected status: `204 No Content`.

### Decrease inventory stock by zero

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/decrease" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 0
  }'
```

Expected status: `400 Bad Request`.

### Decrease inventory stock below zero

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/decrease" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 999999
  }'
```

Expected status: `409 Conflict`.

### Decrease stock without authentication

```bash
curl -i \
  -X POST \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/decrease" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 1
  }'
```

Expected status: `401 Unauthorized`.

## Set inventory stock

Sets the stock of an active inventory item to an absolute value.

Unlike increase and decrease, setting stock to `0` is valid.

```bash
curl -i \
  -b cookies.txt \
  -X PUT \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/stock" \
  -H "Content-Type: application/json" \
  -d '{
    "stock": 5
  }'
```

Expected status: `204 No Content`.

### Set inventory stock to zero

```bash
curl -i \
  -b cookies.txt \
  -X PUT \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/stock" \
  -H "Content-Type: application/json" \
  -d '{
    "stock": 0
  }'
```

Expected status: `204 No Content`.

### Set stock of an unknown inventory item

```bash
curl -i \
  -b cookies.txt \
  -X PUT \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/00000000-0000-0000-0000-000000000000/stock" \
  -H "Content-Type: application/json" \
  -d '{
    "stock": 5
  }'
```

Expected status: `404 Not Found`.

### Set stock without authentication

```bash
curl -i \
  -X PUT \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/stock" \
  -H "Content-Type: application/json" \
  -d '{
    "stock": 5
  }'
```

Expected status: `401 Unauthorized`.

## List inventory stock history

Returns the stock history for a specific inventory item. The authenticated user must be a member of the household.

Replace `<household-uuid>` and `<inventory-item-uuid>` with the corresponding IDs.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/history"
```

Expected status: `200 OK`.

Example response:

```json
[
  {
    "id": "<stock-event-uuid>",
    "sequence_number": 3,
    "item_id": "<inventory-item-uuid>",
    "kind": "set",
    "source": "manual",
    "amount": null,
    "stock_before": 3,
    "stock_after": 0,
    "actor": {
      "type": "user",
      "id": "<user-uuid>",
      "display_name": "Samuel"
    },
    "created_at": "2026-08-18T19:30:00Z"
  },
  {
    "id": "<stock-event-uuid>",
    "sequence_number": 2,
    "item_id": "<inventory-item-uuid>",
    "kind": "decrease",
    "source": "manual",
    "amount": 2,
    "stock_before": 5,
    "stock_after": 3,
    "actor": {
      "type": "user",
      "id": "<user-uuid>",
      "display_name": "Samuel"
    },
    "created_at": "2026-08-18T19:29:00Z"
  },
  {
    "id": "<stock-event-uuid>",
    "sequence_number": 1,
    "item_id": "<inventory-item-uuid>",
    "kind": "increase",
    "source": "manual",
    "amount": 3,
    "stock_before": 2,
    "stock_after": 5,
    "actor": {
      "type": "user",
      "id": "<user-uuid>",
      "display_name": "Samuel"
    },
    "created_at": "2026-08-18T19:28:00Z"
  }
]
```

History is returned newest first.

For `increase` and `decrease` events, `amount` contains the requested stock change.

For `set` events, `amount` is `null`, while `stock_before` and `stock_after` describe the absolute change.

If the inventory item exists but has no stock history, the endpoint returns:

```json
[]
```

Expected status: `200 OK`.

### List history of an unknown inventory item

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/00000000-0000-0000-0000-000000000000/history"
```

Expected status: `404 Not Found`.

### List stock history without membership

Use an inventory item belonging to another household.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/inventory/<other-household-uuid>/items/<inventory-item-uuid>/history"
```

Expected status: `403 Forbidden`.

### List stock history without authentication

```bash
curl -i \
  "$BASE_URL/api/v1/inventory/<household-uuid>/items/<inventory-item-uuid>/history"
```

Expected status: `401 Unauthorized`.

## Register a device

Registers a device for a household. The authenticated user must be a member of the household.

Replace `<household-uuid>` with the corresponding household ID.

Valid device kinds are:

- `scanner`
- `display`
- `other`

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Kitchen Scanner",
    "kind": "scanner"
  }'
```

Expected status: `201 Created`.

Example response:

```json
{
  "id": "<device-uuid>"
}
```

### Register a device with an invalid name

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "   ",
    "kind": "scanner"
  }'
```

Expected status: `400 Bad Request`.

### Register a device with an invalid kind

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Kitchen Scanner",
    "kind": "invalid"
  }'
```

Expected status: `400 Bad Request`.

### Register a device without authentication

```bash
curl -i \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Kitchen Scanner",
    "kind": "scanner"
  }'
```

Expected status: `401 Unauthorized`.

## List devices

Returns all active devices registered for a household. Revoked devices are not returned.

```bash
curl -i \
  -b cookies.txt \
  "$BASE_URL/api/v1/households/<household-uuid>/devices"
```

Expected status: `200 OK`.

Example response:

```json
[
  {
    "id": "<device-uuid>",
    "name": "Kitchen Scanner",
    "kind": "scanner"
  }
]
```

If the household has no active devices, the endpoint returns:

```json
[]
```

## Rename a device

Renames an active device.

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/<device-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Kitchen Raspberry Pi"
  }'
```

Expected status: `204 No Content`.

### Rename an unknown device

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/00000000-0000-0000-0000-000000000000" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Kitchen Raspberry Pi"
  }'
```

Expected status: `404 Not Found`.

### Rename a revoked device

```bash
curl -i \
  -b cookies.txt \
  -X PATCH \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/<revoked-device-uuid>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New Name"
  }'
```

Expected status: `409 Conflict`.

## Revoke a device

Revokes a device. Revoked devices are no longer returned by the active device list and cannot be modified.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/<device-uuid>/revoke"
```

Expected status: `204 No Content`.

### Revoke an already revoked device

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/<device-uuid>/revoke"
```

Expected status: `409 Conflict`.

### Revoke an unknown device

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/00000000-0000-0000-0000-000000000000/revoke"
```

Expected status: `404 Not Found`.

## Issue a device credential

Issues the first active credential for a registered device.

The plaintext token is returned exactly once. Only the token hash is stored by the backend.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/<device-uuid>/credentials"
```

Expected status: `201 Created`.

Example response:

```json
{
  "token": "<device-token>"
}
```

Store this token securely. It cannot be retrieved again later.

### Issue a second active credential

A device may have at most one active credential.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/<device-uuid>/credentials"
```

Expected status: `409 Conflict`.

## Rotate a device credential

Revokes the currently active credential and atomically replaces it with a new one.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/<device-uuid>/credentials/rotate"
```

Expected status: `200 OK`.

Example response:

```json
{
  "token": "<new-device-token>"
}
```

The previous token becomes invalid immediately.

### Rotate a credential when no active credential exists

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/<device-uuid>/credentials/rotate"
```

Expected status: `409 Conflict`.

## Authenticate as a device

Device-authenticated requests use the HTTP `Authorization` header:

```bash
curl -i \
  -H "Authorization: Bearer <device-token>" \
  "$BASE_URL/<device-protected-endpoint>"
```

An invalid, rotated, revoked, or unknown token returns:

```text
401 Unauthorized
```

## Revoke a device

Revoking a device also atomically revokes its active credential.

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/<device-uuid>/revoke"
```

Expected status: `204 No Content`.

After revocation:

- the device is no longer returned by the active device list
- its active credential is revoked
- its previous bearer token can no longer authenticate

### Revoke an already revoked device

```bash
curl -i \
  -b cookies.txt \
  -X POST \
  "$BASE_URL/api/v1/households/<household-uuid>/devices/<device-uuid>/revoke"
```

Expected status: `409 Conflict`.
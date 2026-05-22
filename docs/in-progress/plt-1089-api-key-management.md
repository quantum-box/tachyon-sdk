# PLT-1089 API Key Management

The generated Tachyon OpenAPI document currently exposes service-account API-key create and list:

- `POST /v1/auth/service-accounts/{service_account_id}/api-keys`
- `GET /v1/auth/service-accounts/{service_account_id}/api-keys`

It does not expose an API-key revoke operation. The CLI includes `tachyon api-key revoke` against the required tachyon-apps contract below, so the SDK side is ready once the backend route is available.

## Required tachyon-apps API

`POST /v1/auth/service-accounts/{service_account_id}/api-keys/{api_key_id}/revoke`

Request body:

```json
{
  "operatorId": "op_..."
}
```

Required auth and tenant behavior:

- Authenticate the caller the same way as service-account API-key create/list.
- Authorize against the tenant in `operatorId` and the existing `x-operator-id` header behavior.
- Verify `api_key_id` belongs to `service_account_id`.
- Revoke the key idempotently. A second revoke of the same key should return success.
- Never return the secret key value from revoke.

Response:

- Preferred: `204 No Content`.
- Also acceptable: `200 OK` with the API-key metadata and `revokedAt`, excluding `value`.

OpenAPI should add this operation to the Auth API Keys group so generated SDKs expose it alongside `create_api_key` and `list_api_keys`.

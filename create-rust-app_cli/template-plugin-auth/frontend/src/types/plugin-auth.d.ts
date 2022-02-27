type ID = number

type UTC = Date

interface PaginationParams {
    page: number
    page_size: number
}

interface AccessTokenClaims {
    exp: number
    sub: number
    token_type: string
    roles: Array<string>
    permissions: Array<Permission>
}

interface UserSessionJson {
    id: number
    device: string | undefined
    created_atDate
    updated_atDate
}

interface UserSessionResponse {
    sessions: Array<UserSessionJson>
    num_pages: number
}

interface Permission {
    from_role: string
    permission: string
}

interface RolePermission {
    role: string
    permission: string
    created_atDate
}

interface UserPermission {
    user_id: number
    permission: string
    created_atDate
}

interface UserRole {
    user_id: number
    role: string
    created_atDate
}

interface User {
    id: number
    email: string
    hash_password: string
    activated: boolean
    created_atDate
    updated_atDate
}

interface UserChangeset {
    email: string
    hash_password: string
    activated: boolean
}

interface UserSession {
    id: number
    user_id: number
    refresh_token: string
    device: string | undefined
    created_atDate
    updated_atDate
}

interface UserSessionChangeset {
    user_id: number
    refresh_token: string
    device: string | undefined
}

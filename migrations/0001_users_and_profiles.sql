-- Tomi keeps its tables in their own schema. Supabase's Data API exposes
-- only the schemas listed in its settings (public by default), so nothing
-- here is reachable except through the Tomi service.
create schema if not exists tomi;

create table tomi.users (
    id uuid primary key default gen_random_uuid(),
    -- The Supabase Auth user id (the JWT "sub"). Kept apart from our own id
    -- so that changing the sign-in provider does not change a user's id.
    auth_subject uuid not null unique,
    created_at timestamptz not null default now(),
    deleted_at timestamptz,
    -- Version of the consent text the cook accepted for sending confirmed
    -- ingredients and restrictions. Null until a consent endpoint exists.
    consent_version integer,
    consent_recorded_at timestamptz
);

create table tomi.profiles (
    user_id uuid primary key references tomi.users (id) on delete cascade,
    version bigint not null check (version >= 1),
    schema_version integer not null,
    data jsonb not null,
    updated_at timestamptz not null default now()
);

-- No policies: with row level security on, Supabase's anon and
-- authenticated roles can read nothing even if the schema were exposed.
-- The service connects as the table owner, which RLS does not restrict.
alter table tomi.users enable row level security;
alter table tomi.profiles enable row level security;

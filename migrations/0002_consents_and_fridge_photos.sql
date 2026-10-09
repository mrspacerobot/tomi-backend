-- Each consent on its own row: what the cook agreed to, to which version
-- of the text, and when. Withdrawing deletes the row.
create table tomi.consents (
    user_id uuid not null references tomi.users (id) on delete cascade,
    purpose text not null check (purpose in ('photo-scan', 'photo-dataset')),
    version integer not null check (version >= 1),
    given_at timestamptz not null default now(),
    primary key (user_id, purpose)
);

-- Fridge photos kept for the dataset, only with the photo-dataset consent.
-- The photo itself is in the private storage bucket at object_path, with
-- its metadata removed. seen is what the model said; confirmed is what the
-- cook then said they have.
create table tomi.fridge_photos (
    id uuid primary key,
    user_id uuid not null references tomi.users (id) on delete cascade,
    object_path text not null unique,
    mime_type text not null,
    consent_version integer not null,
    model text not null,
    seen jsonb not null,
    confirmed jsonb,
    created_at timestamptz not null default now()
);

create index fridge_photos_by_user on tomi.fridge_photos (user_id);

alter table tomi.consents enable row level security;
alter table tomi.fridge_photos enable row level security;

-- Fridge photos kept for the dataset. Proof of concept: every scanned photo is
-- kept as it arrived, metadata included, until a consent step exists. The photo
-- itself is in the private storage bucket at object_path. seen is what the model
-- said; confirmed is what the cook then said they have.
create table tomi.fridge_photos (
    id uuid primary key,
    user_id uuid not null references tomi.users (id) on delete cascade,
    object_path text not null unique,
    mime_type text not null,
    model text not null,
    seen jsonb not null,
    confirmed jsonb,
    created_at timestamptz not null default now()
);

create index fridge_photos_by_user on tomi.fridge_photos (user_id);

alter table tomi.fridge_photos enable row level security;

-- The cook log: one row per cook the phone sent, per cook. The phone makes the
-- id when cooking begins, so the same id twice is the same cook: the first
-- accepted copy stays, and payload_sha256 tells a retry from a different body.
-- No totals, XP or quiz result: progress is computed from these rows, and the
-- quiz result is derived from the recorded answers.
create table tomi.cook_entries (
    user_id uuid not null references tomi.users (id) on delete cascade,
    id uuid not null,
    payload_sha256 text not null,
    schema_version integer not null,
    skill text not null,
    level smallint not null check (level >= 1),
    recipe_id text not null,
    graph_version text not null,
    outcome text not null check (outcome in ('finished', 'stoppedEarly')),
    weekly_goal smallint not null,
    cooked_at timestamptz not null,
    -- The IANA zone the cook was made in, so a past week is judged by the
    -- cook's own day there.
    time_zone text not null,
    received_at timestamptz not null default now(),
    primary key (user_id, id)
);

alter table tomi.cook_entries enable row level security;

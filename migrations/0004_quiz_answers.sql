-- The first answer a cook gave to each quiz question of one cook. The phone makes entry_id when cooking begins and
-- later sends it as the cook entry's id, so answers and the entry meet by id in either order. The first insert wins:
-- a later pick of the same question is checked and answered, but only teaches, and is never stored.
-- quiz_question_ids is the whole quiz at the first answer, so scoring never needs an older graph.
-- Numbered 0004 because mrspacerobot/tomi-backend#48 adds 0003_cook_entries.sql.
create table tomi.quiz_answers (
    user_id uuid not null references tomi.users (id) on delete cascade,
    entry_id uuid not null,
    question_id text not null,
    recipe_id text not null,
    graph_version text not null,
    choice integer not null check (choice >= 0),
    correct boolean not null,
    quiz_question_ids text[] not null,
    answered_at timestamptz not null default now(),
    primary key (user_id, entry_id, question_id)
);

alter table tomi.quiz_answers enable row level security;

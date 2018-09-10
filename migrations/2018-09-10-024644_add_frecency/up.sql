alter table characters
  add column frecency float not null default 0.0,
  add column last_update timestamp not null default now()

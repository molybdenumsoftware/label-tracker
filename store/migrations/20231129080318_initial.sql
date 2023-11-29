-- Add migration script here
create table landings (
    github_pr int NOT NULL,
    channel varchar(255)
)

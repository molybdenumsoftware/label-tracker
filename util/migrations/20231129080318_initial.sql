create table github_prs (
    number int PRIMARY KEY,
    commit varchar(40) not null
);

create table landings (
    github_pr int not null references github_prs(number),
    channel int not null references channels(number)
);

create table channels (
    number int PRIMARY KEY,
    name varchar(255) not null
)

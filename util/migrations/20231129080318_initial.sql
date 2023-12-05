create table github_prs (
    number int PRIMARY KEY,
    commit varchar(40) not null
);

create table landings (
    github_pr_number int not null references github_prs(number)
    ,
    channel varchar(255) not null
);

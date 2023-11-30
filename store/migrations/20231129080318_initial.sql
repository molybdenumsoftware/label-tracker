create table github_prs (
    number int PRIMARY KEY
);

create table landings (
    github_pr_number int not null references github_prs(number)
    ,
    channel varchar(255)
);

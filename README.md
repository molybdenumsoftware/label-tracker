# label-tracker

Fork of <https://git.eno.space/label-tracker.git>.

Fetcher TODO:

Nightly polling, fetching:

1. newly merged pull requests {number, merge_commit}
1. HEADs of new/updated tracked branches
    1. for each such HEAD,

where <interesting> means either a release branch or a channel, aka matches regex: "nixos-*"

Future: faster detection of above via webhook
